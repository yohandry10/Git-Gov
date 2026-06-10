use super::*;

impl Database {
    pub async fn upsert_project_ticket(&self, ticket: &ProjectTicket) -> Result<(), DbError> {
        let ingested_at =
            chrono::DateTime::from_timestamp_millis(ticket.ingested_at).ok_or_else(|| {
                DbError::SerializationError("Invalid ingested_at timestamp".to_string())
            })?;
        let created_at = ticket
            .created_at
            .and_then(chrono::DateTime::from_timestamp_millis);
        let updated_at = ticket
            .updated_at
            .and_then(chrono::DateTime::from_timestamp_millis);

        sqlx::query(
            r#"
            INSERT INTO project_tickets (
                id, org_id, ticket_id, ticket_url, title, status, assignee, reporter, priority, ticket_type,
                related_commits, related_prs, related_branches, created_at, updated_at, ingested_at
            )
            VALUES (
                $1::uuid, $2::uuid, $3, $4, $5, $6, $7, $8, $9, $10,
                $11::text[], $12::text[], $13::text[], $14, $15, $16
            )
            ON CONFLICT (org_id, ticket_id) DO UPDATE SET
                ticket_url = EXCLUDED.ticket_url,
                title = EXCLUDED.title,
                status = EXCLUDED.status,
                assignee = EXCLUDED.assignee,
                reporter = EXCLUDED.reporter,
                priority = EXCLUDED.priority,
                ticket_type = EXCLUDED.ticket_type,
                related_commits = EXCLUDED.related_commits,
                related_prs = EXCLUDED.related_prs,
                related_branches = EXCLUDED.related_branches,
                created_at = COALESCE(project_tickets.created_at, EXCLUDED.created_at),
                updated_at = EXCLUDED.updated_at,
                ingested_at = EXCLUDED.ingested_at
            "#,
        )
        .bind(&ticket.id)
        .bind(&ticket.org_id)
        .bind(&ticket.ticket_id)
        .bind(&ticket.ticket_url)
        .bind(&ticket.title)
        .bind(&ticket.status)
        .bind(&ticket.assignee)
        .bind(&ticket.reporter)
        .bind(&ticket.priority)
        .bind(&ticket.ticket_type)
        .bind(&ticket.related_commits)
        .bind(&ticket.related_prs)
        .bind(&ticket.related_branches)
        .bind(created_at)
        .bind(updated_at)
        .bind(ingested_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_jira_integration_status(
        &self,
        org_id: Option<&str>,
    ) -> Result<JiraIntegrationStatusResponse, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                MAX(ingested_at) AS last_ingest_at,
                COUNT(*) FILTER (WHERE ingested_at >= NOW() - INTERVAL '24 hours')::bigint AS recent_tickets_24h
            FROM project_tickets
            WHERE ($1::uuid IS NULL OR org_id = $1::uuid)
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(JiraIntegrationStatusResponse {
            ok: true,
            last_ingest_at: row
                .get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_ingest_at")
                .map(|dt| dt.timestamp_millis()),
            recent_tickets_24h: row.get("recent_tickets_24h"),
        })
    }

    pub async fn get_project_ticket_by_ticket_id(
        &self,
        ticket_id: &str,
        org_id: Option<&str>,
    ) -> Result<Option<ProjectTicket>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                id::text,
                org_id::text AS org_id,
                ticket_id,
                ticket_url,
                title,
                status,
                assignee,
                reporter,
                priority,
                ticket_type,
                related_commits,
                related_prs,
                related_branches,
                created_at,
                updated_at,
                ingested_at
            FROM project_tickets
            WHERE ticket_id = $1
              AND ($2::uuid IS NULL OR org_id = $2::uuid)
            ORDER BY ingested_at DESC
            LIMIT 1
            "#,
        )
        .bind(ticket_id)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| {
            let created_at = row
                .get::<Option<chrono::DateTime<chrono::Utc>>, _>("created_at")
                .map(|dt| dt.timestamp_millis());
            let updated_at = row
                .get::<Option<chrono::DateTime<chrono::Utc>>, _>("updated_at")
                .map(|dt| dt.timestamp_millis());
            let ingested_at = row
                .get::<chrono::DateTime<chrono::Utc>, _>("ingested_at")
                .timestamp_millis();

            ProjectTicket {
                id: row.get("id"),
                org_id: row.get("org_id"),
                ticket_id: row.get("ticket_id"),
                ticket_url: row.get("ticket_url"),
                title: row.get("title"),
                status: row.get("status"),
                assignee: row.get("assignee"),
                reporter: row.get("reporter"),
                priority: row.get("priority"),
                ticket_type: row.get("ticket_type"),
                related_commits: row.get("related_commits"),
                related_prs: row.get("related_prs"),
                related_branches: row.get("related_branches"),
                created_at,
                updated_at,
                ingested_at,
            }
        }))
    }

    pub async fn insert_commit_ticket_correlation(
        &self,
        correlation: &CommitTicketCorrelation,
    ) -> Result<bool, DbError> {
        let created_at = chrono::DateTime::from_timestamp_millis(correlation.created_at)
            .ok_or_else(|| {
                DbError::SerializationError("Invalid created_at timestamp".to_string())
            })?;

        let result = sqlx::query(
            r#"
            INSERT INTO commit_ticket_correlations (
                id, org_id, commit_sha, ticket_id, correlation_source, confidence, created_at
            )
            VALUES ($1::uuid, $2::uuid, $3, $4, $5, $6, $7)
            ON CONFLICT (
                COALESCE(org_id, '00000000-0000-0000-0000-000000000000'::uuid),
                commit_sha,
                ticket_id
            ) DO NOTHING
            RETURNING id::text
            "#,
        )
        .bind(&correlation.id)
        .bind(&correlation.org_id)
        .bind(&correlation.commit_sha)
        .bind(&correlation.ticket_id)
        .bind(&correlation.correlation_source)
        .bind(correlation.confidence)
        .bind(created_at)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(result.is_some())
    }

    pub async fn append_project_ticket_relations(
        &self,
        ticket_id: &str,
        org_id: Option<&str>,
        commit_sha: Option<&str>,
        branch: Option<&str>,
    ) -> Result<bool, DbError> {
        self.append_project_ticket_relations_full(ticket_id, org_id, commit_sha, branch, None)
            .await
    }

    pub async fn append_project_ticket_relations_full(
        &self,
        ticket_id: &str,
        org_id: Option<&str>,
        commit_sha: Option<&str>,
        branch: Option<&str>,
        pr_ref: Option<&str>,
    ) -> Result<bool, DbError> {
        let commit_sha = commit_sha.map(str::trim).filter(|s| !s.is_empty());
        let branch = branch.map(str::trim).filter(|s| !s.is_empty());
        let pr_ref = pr_ref.map(str::trim).filter(|s| !s.is_empty());

        let result = sqlx::query(
            r#"
            UPDATE project_tickets
            SET
              related_commits = CASE
                WHEN $2::text IS NULL THEN related_commits
                ELSE (
                  SELECT COALESCE(array_agg(DISTINCT x), '{}'::text[])
                  FROM unnest(COALESCE(related_commits, '{}'::text[]) || ARRAY[$2::text]) AS x
                  WHERE x IS NOT NULL AND x <> ''
                )
              END,
              related_branches = CASE
                WHEN $3::text IS NULL THEN related_branches
                ELSE (
                  SELECT COALESCE(array_agg(DISTINCT x), '{}'::text[])
                  FROM unnest(COALESCE(related_branches, '{}'::text[]) || ARRAY[$3::text]) AS x
                  WHERE x IS NOT NULL AND x <> ''
                )
              END,
              related_prs = CASE
                WHEN $4::text IS NULL THEN related_prs
                ELSE (
                  SELECT COALESCE(array_agg(DISTINCT x), '{}'::text[])
                  FROM unnest(COALESCE(related_prs, '{}'::text[]) || ARRAY[$4::text]) AS x
                  WHERE x IS NOT NULL AND x <> ''
                )
              END,
              updated_at = NOW()
            WHERE ticket_id = $1
              AND ($5::uuid IS NULL OR org_id = $5::uuid)
            "#,
        )
        .bind(ticket_id)
        .bind(commit_sha)
        .bind(branch)
        .bind(pr_ref)
        .bind(org_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Find PRs whose head_sha matches any of the given commit SHAs,
    /// or whose pr_title contains any of the given ticket IDs.
    /// Returns Vec<(pr_number, pr_title, head_sha, repo_full_name)>.
    pub async fn find_prs_related_to_tickets(
        &self,
        commit_shas: &[String],
        ticket_ids: &[String],
        org_id: Option<&str>,
        repo_full_name: Option<&str>,
        hours: i64,
    ) -> Result<Vec<(i32, Option<String>, Option<String>, Option<String>)>, DbError> {
        if commit_shas.is_empty() && ticket_ids.is_empty() {
            return Ok(vec![]);
        }

        let rows = sqlx::query(
            r#"
            SELECT DISTINCT
                prm.pr_number,
                prm.pr_title,
                prm.head_sha,
                r.full_name AS repo_full_name
            FROM pull_request_merges prm
            LEFT JOIN repos r ON r.id = prm.repo_id
            WHERE prm.created_at >= NOW() - make_interval(hours => $3::int)
              AND ($4::uuid IS NULL OR prm.org_id = $4::uuid)
              AND ($5::text IS NULL OR r.full_name = $5)
              AND (
                ($1::text[] IS NOT NULL AND prm.head_sha = ANY($1::text[]))
                OR ($2::text[] IS NOT NULL AND EXISTS (
                  SELECT 1 FROM unnest($2::text[]) AS tid
                  WHERE UPPER(prm.pr_title) LIKE '%' || UPPER(tid) || '%'
                ))
              )
            ORDER BY prm.pr_number
            LIMIT 200
            "#,
        )
        .bind(commit_shas)
        .bind(ticket_ids)
        .bind(hours as i32)
        .bind(org_id)
        .bind(repo_full_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|row| {
                (
                    row.get::<i32, _>("pr_number"),
                    row.get::<Option<String>, _>("pr_title"),
                    row.get::<Option<String>, _>("head_sha"),
                    row.get::<Option<String>, _>("repo_full_name"),
                )
            })
            .collect())
    }

    pub async fn get_recent_pr_merges_for_ticket_correlation(
        &self,
        org_name: Option<&str>,
        repo_full_name: Option<&str>,
        hours: i64,
        limit: i64,
    ) -> Result<
        Vec<(
            Option<String>,
            i32,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        )>,
        DbError,
    > {
        let org_id = if let Some(name) = org_name {
            self.get_org_by_login(name).await?.map(|o| o.id)
        } else {
            None
        };
        let repo_id = if let Some(name) = repo_full_name {
            self.get_repo_by_full_name(name).await?.map(|r| r.id)
        } else {
            None
        };
        if org_name.is_some() && org_id.is_none() {
            return Ok(vec![]);
        }
        if repo_full_name.is_some() && repo_id.is_none() {
            return Ok(vec![]);
        }

        let rows = sqlx::query(
            r#"
            SELECT
                prm.org_id::text AS org_id,
                prm.pr_number,
                prm.pr_title,
                prm.head_sha,
                COALESCE(
                    prm.payload #>> '{pull_request,merge_commit_sha}',
                    prm.payload #>> '{gitgov,merge_commit_sha}'
                ) AS merge_commit_sha,
                prm.base_branch,
                r.full_name AS repo_full_name
            FROM pull_request_merges prm
            LEFT JOIN repos r ON r.id = prm.repo_id
            WHERE prm.created_at >= NOW() - make_interval(hours => $1::int)
              AND ($2::uuid IS NULL OR prm.org_id = $2::uuid)
              AND ($3::uuid IS NULL OR prm.repo_id = $3::uuid)
            ORDER BY prm.created_at DESC
            LIMIT $4
            "#,
        )
        .bind(hours as i32)
        .bind(&org_id)
        .bind(&repo_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|row| {
                (
                    row.get::<Option<String>, _>("org_id"),
                    row.get::<i32, _>("pr_number"),
                    row.get::<Option<String>, _>("pr_title"),
                    row.get::<Option<String>, _>("head_sha"),
                    row.get::<Option<String>, _>("merge_commit_sha"),
                    row.get::<Option<String>, _>("base_branch"),
                    row.get::<Option<String>, _>("repo_full_name"),
                )
            })
            .collect())
    }

    pub async fn get_recent_commit_events_for_ticket_correlation(
        &self,
        org_name: Option<&str>,
        repo_full_name: Option<&str>,
        hours: i64,
        limit: i64,
    ) -> Result<
        Vec<(
            String,
            Option<String>,
            Option<String>,
            serde_json::Value,
            Option<String>,
        )>,
        DbError,
    > {
        let org_id = if let Some(name) = org_name {
            self.get_org_by_login(name).await?.map(|o| o.id)
        } else {
            None
        };
        let repo_id = if let Some(name) = repo_full_name {
            self.get_repo_by_full_name(name).await?.map(|r| r.id)
        } else {
            None
        };

        if org_name.is_some() && org_id.is_none() {
            return Ok(vec![]);
        }
        if repo_full_name.is_some() && repo_id.is_none() {
            return Ok(vec![]);
        }

        let rows = sqlx::query(
            r#"
            SELECT
                c.commit_sha,
                c.branch,
                c.org_id::text AS org_id,
                c.metadata,
                r.full_name AS repo_name
            FROM client_events c
            LEFT JOIN repos r ON r.id = c.repo_id
            WHERE c.event_type = 'commit'
              AND c.commit_sha IS NOT NULL
              AND c.created_at >= NOW() - make_interval(hours => $1::int)
              AND ($2::uuid IS NULL OR c.org_id = $2::uuid)
              AND ($3::uuid IS NULL OR c.repo_id = $3::uuid)
            ORDER BY c.created_at DESC
            LIMIT $4
            "#,
        )
        .bind(hours)
        .bind(&org_id)
        .bind(&repo_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| {
                (
                    row.get("commit_sha"),
                    row.get("branch"),
                    row.get("org_id"),
                    row.get("metadata"),
                    row.get("repo_name"),
                )
            })
            .collect())
    }
}
