use super::*;

impl Database {
    pub async fn get_ticket_coverage(
        &self,
        org_name: Option<&str>,
        repo_full_name: Option<&str>,
        branch: Option<&str>,
        hours: i64,
    ) -> Result<TicketCoverageResponse, DbError> {
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
            return Ok(TicketCoverageResponse {
                org: org_name.unwrap_or_default().to_string(),
                period: format!("last_{}h", hours),
                ..Default::default()
            });
        }
        if repo_full_name.is_some() && repo_id.is_none() {
            return Ok(TicketCoverageResponse {
                org: org_name.unwrap_or_default().to_string(),
                period: format!("last_{}h", hours),
                ..Default::default()
            });
        }

        let total_commits_row = sqlx::query(
            r#"
            WITH commit_universe AS (
                SELECT
                    c.org_id,
                    c.repo_id,
                    c.commit_sha,
                    c.user_login,
                    c.branch,
                    c.created_at
                FROM client_events c
                WHERE c.event_type = 'commit'
                  AND c.commit_sha IS NOT NULL
                  AND c.created_at >= NOW() - make_interval(hours => $1::int)
                  AND ($2::uuid IS NULL OR c.org_id = $2::uuid)
                  AND ($3::uuid IS NULL OR c.repo_id = $3::uuid)
                  AND ($4::text IS NULL OR c.branch = $4)

                UNION ALL

                SELECT
                    prm.org_id,
                    prm.repo_id,
                    COALESCE(
                        NULLIF(prm.payload #>> '{pull_request,merge_commit_sha}', ''),
                        NULLIF(prm.payload #>> '{gitgov,merge_commit_sha}', ''),
                        NULLIF(prm.head_sha, '')
                    ) AS commit_sha,
                    COALESCE(prm.merged_by_login, prm.author_login) AS user_login,
                    prm.base_branch AS branch,
                    prm.created_at
                FROM pull_request_merges prm
                WHERE prm.created_at >= NOW() - make_interval(hours => $1::int)
                  AND ($2::uuid IS NULL OR prm.org_id = $2::uuid)
                  AND ($3::uuid IS NULL OR prm.repo_id = $3::uuid)
                  AND ($4::text IS NULL OR prm.base_branch = $4)
                  AND COALESCE(
                        NULLIF(prm.payload #>> '{pull_request,merge_commit_sha}', ''),
                        NULLIF(prm.payload #>> '{gitgov,merge_commit_sha}', ''),
                        NULLIF(prm.head_sha, '')
                  ) IS NOT NULL
            )
            SELECT COUNT(DISTINCT commit_sha)::bigint AS total
            FROM commit_universe
            "#,
        )
        .bind(hours)
        .bind(&org_id)
        .bind(&repo_id)
        .bind(branch)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let with_ticket_row = sqlx::query(
            r#"
            WITH commit_universe AS (
                SELECT
                    c.org_id,
                    c.repo_id,
                    c.commit_sha,
                    c.branch,
                    c.created_at
                FROM client_events c
                WHERE c.event_type = 'commit'
                  AND c.commit_sha IS NOT NULL
                  AND c.created_at >= NOW() - make_interval(hours => $1::int)
                  AND ($2::uuid IS NULL OR c.org_id = $2::uuid)
                  AND ($3::uuid IS NULL OR c.repo_id = $3::uuid)
                  AND ($4::text IS NULL OR c.branch = $4)

                UNION ALL

                SELECT
                    prm.org_id,
                    prm.repo_id,
                    COALESCE(
                        NULLIF(prm.payload #>> '{pull_request,merge_commit_sha}', ''),
                        NULLIF(prm.payload #>> '{gitgov,merge_commit_sha}', ''),
                        NULLIF(prm.head_sha, '')
                    ) AS commit_sha,
                    prm.base_branch AS branch,
                    prm.created_at
                FROM pull_request_merges prm
                WHERE prm.created_at >= NOW() - make_interval(hours => $1::int)
                  AND ($2::uuid IS NULL OR prm.org_id = $2::uuid)
                  AND ($3::uuid IS NULL OR prm.repo_id = $3::uuid)
                  AND ($4::text IS NULL OR prm.base_branch = $4)
                  AND COALESCE(
                        NULLIF(prm.payload #>> '{pull_request,merge_commit_sha}', ''),
                        NULLIF(prm.payload #>> '{gitgov,merge_commit_sha}', ''),
                        NULLIF(prm.head_sha, '')
                  ) IS NOT NULL
            )
            SELECT COUNT(DISTINCT u.commit_sha)::bigint AS covered
            FROM commit_universe u
            JOIN commit_ticket_correlations ct
              ON ct.commit_sha = u.commit_sha
             AND (u.org_id IS NULL OR ct.org_id IS NULL OR ct.org_id = u.org_id)
            "#,
        )
        .bind(hours)
        .bind(&org_id)
        .bind(&repo_id)
        .bind(branch)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let missing_rows = sqlx::query(
            r#"
            WITH commit_universe AS (
                SELECT
                    c.org_id,
                    c.repo_id,
                    c.commit_sha,
                    c.user_login,
                    c.branch,
                    c.created_at,
                    'client_event'::text AS source
                FROM client_events c
                WHERE c.event_type = 'commit'
                  AND c.commit_sha IS NOT NULL
                  AND c.created_at >= NOW() - make_interval(hours => $1::int)
                  AND ($2::uuid IS NULL OR c.org_id = $2::uuid)
                  AND ($3::uuid IS NULL OR c.repo_id = $3::uuid)
                  AND ($4::text IS NULL OR c.branch = $4)

                UNION ALL

                SELECT
                    prm.org_id,
                    prm.repo_id,
                    COALESCE(
                        NULLIF(prm.payload #>> '{pull_request,merge_commit_sha}', ''),
                        NULLIF(prm.payload #>> '{gitgov,merge_commit_sha}', ''),
                        NULLIF(prm.head_sha, '')
                    ) AS commit_sha,
                    COALESCE(prm.merged_by_login, prm.author_login) AS user_login,
                    prm.base_branch AS branch,
                    prm.created_at,
                    'pull_request_merge'::text AS source
                FROM pull_request_merges prm
                WHERE prm.created_at >= NOW() - make_interval(hours => $1::int)
                  AND ($2::uuid IS NULL OR prm.org_id = $2::uuid)
                  AND ($3::uuid IS NULL OR prm.repo_id = $3::uuid)
                  AND ($4::text IS NULL OR prm.base_branch = $4)
                  AND COALESCE(
                        NULLIF(prm.payload #>> '{pull_request,merge_commit_sha}', ''),
                        NULLIF(prm.payload #>> '{gitgov,merge_commit_sha}', ''),
                        NULLIF(prm.head_sha, '')
                  ) IS NOT NULL
            ),
            distinct_commits AS (
                SELECT DISTINCT ON (commit_sha)
                    org_id,
                    commit_sha,
                    user_login,
                    branch,
                    created_at,
                    source
                FROM commit_universe
                ORDER BY commit_sha, created_at DESC
            )
            SELECT u.commit_sha, u.user_login, u.branch, u.created_at, u.source
            FROM distinct_commits u
            LEFT JOIN commit_ticket_correlations ct
              ON ct.commit_sha = u.commit_sha
             AND (u.org_id IS NULL OR ct.org_id IS NULL OR ct.org_id = u.org_id)
            WHERE u.commit_sha IS NOT NULL
              AND ct.id IS NULL
            ORDER BY u.created_at DESC
            LIMIT 20
            "#,
        )
        .bind(hours)
        .bind(&org_id)
        .bind(&repo_id)
        .bind(branch)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let orphan_ticket_rows = sqlx::query(
            r#"
            SELECT pt.ticket_id, pt.status, pt.updated_at
            FROM project_tickets pt
            LEFT JOIN commit_ticket_correlations ct
              ON ct.ticket_id = pt.ticket_id
             AND (pt.org_id IS NULL OR ct.org_id = pt.org_id)
            WHERE pt.ingested_at >= NOW() - make_interval(hours => $1::int)
              AND ($2::uuid IS NULL OR pt.org_id = $2::uuid)
              AND ct.id IS NULL
            ORDER BY pt.ingested_at DESC
            LIMIT 20
            "#,
        )
        .bind(hours)
        .bind(&org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let total_commits: i64 = total_commits_row.get("total");
        let commits_with_ticket: i64 = with_ticket_row.get("covered");
        let coverage_percentage = if total_commits > 0 {
            (commits_with_ticket as f64 / total_commits as f64) * 100.0
        } else {
            0.0
        };

        Ok(TicketCoverageResponse {
            org: org_name.unwrap_or("all").to_string(),
            period: format!("last_{}h", hours),
            total_commits,
            commits_with_ticket,
            coverage_percentage,
            commits_without_ticket: missing_rows
                .into_iter()
                .map(|row| {
                    serde_json::json!({
                        "commit_sha": row.get::<String, _>("commit_sha"),
                        "user_login": row.get::<Option<String>, _>("user_login"),
                        "branch": row.get::<Option<String>, _>("branch"),
                        "source": row.get::<String, _>("source"),
                        "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").timestamp_millis(),
                    })
                })
                .collect(),
            tickets_without_commits: orphan_ticket_rows
                .into_iter()
                .map(|row| {
                    serde_json::json!({
                        "ticket_id": row.get::<String, _>("ticket_id"),
                        "status": row.get::<Option<String>, _>("status"),
                        "updated_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("updated_at")
                            .map(|dt| dt.timestamp_millis()),
                    })
                })
                .collect(),
        })
    }

    pub async fn get_commit_pipeline_correlations(
        &self,
        filter: &JenkinsCorrelationFilter,
    ) -> Result<Vec<CommitPipelineCorrelation>, DbError> {
        let limit = if filter.limit == 0 { 20 } else { filter.limit } as i64;
        let offset = filter.offset as i64;

        let org_id = if let Some(org_name) = filter.org_name.as_deref() {
            self.get_org_by_login(org_name).await?.map(|o| o.id)
        } else {
            None
        };
        let repo_id = if let Some(repo_full_name) = filter.repo_full_name.as_deref() {
            self.get_repo_by_full_name(repo_full_name)
                .await?
                .map(|r| r.id)
        } else {
            None
        };

        if filter.org_name.is_some() && org_id.is_none() {
            return Ok(vec![]);
        }
        if filter.repo_full_name.is_some() && repo_id.is_none() {
            return Ok(vec![]);
        }

        let rows = sqlx::query(
            r#"
            SELECT
                c.id::text AS commit_event_id,
                c.commit_sha,
                c.created_at AS commit_created_at,
                c.user_login,
                c.branch,
                r.full_name AS repo_name,
                c.metadata AS metadata,
                p.id::text AS pipeline_event_id,
                p.pipeline_id,
                p.job_name,
                p.status AS pipeline_status,
                p.duration_ms AS pipeline_duration_ms,
                p.triggered_by,
                p.ingested_at AS pipeline_ingested_at
            FROM client_events c
            LEFT JOIN repos r ON r.id = c.repo_id
            LEFT JOIN LATERAL (
                SELECT pe.*
                FROM pipeline_events pe
                WHERE c.commit_sha IS NOT NULL
                  AND pe.commit_sha IS NOT NULL
                  AND (
                    pe.commit_sha = c.commit_sha
                    OR pe.commit_sha LIKE c.commit_sha || '%'
                    OR c.commit_sha LIKE pe.commit_sha || '%'
                  )
                ORDER BY pe.ingested_at DESC
                LIMIT 1
            ) p ON TRUE
            WHERE c.event_type = 'commit'
              AND c.commit_sha IS NOT NULL
              AND ($1::uuid IS NULL OR c.org_id = $1::uuid)
              AND ($2::uuid IS NULL OR c.repo_id = $2::uuid)
              AND ($3::text IS NULL OR c.branch = $3)
              AND ($4::text IS NULL OR c.user_login = $4)
            ORDER BY c.created_at DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(&org_id)
        .bind(&repo_id)
        .bind(&filter.branch)
        .bind(&filter.user_login)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let correlations = rows
            .into_iter()
            .map(|row| {
                let commit_created_at: chrono::DateTime<chrono::Utc> = row.get("commit_created_at");
                let metadata: serde_json::Value = row.get("metadata");
                let commit_message = metadata
                    .as_object()
                    .and_then(|m| m.get("commit_message"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let pipeline =
                    row.get::<Option<String>, _>("pipeline_event_id")
                        .map(|pipeline_event_id| {
                            let ingested_at = row
                                .get::<Option<chrono::DateTime<chrono::Utc>>, _>(
                                    "pipeline_ingested_at",
                                )
                                .map(|dt| dt.timestamp_millis())
                                .unwrap_or_default();

                            CommitPipelineRun {
                                pipeline_event_id,
                                pipeline_id: row.get("pipeline_id"),
                                job_name: row.get("job_name"),
                                status: row.get("pipeline_status"),
                                duration_ms: row.get("pipeline_duration_ms"),
                                triggered_by: row.get("triggered_by"),
                                ingested_at,
                            }
                        });

                CommitPipelineCorrelation {
                    commit_event_id: row.get("commit_event_id"),
                    commit_sha: row.get("commit_sha"),
                    commit_message,
                    commit_created_at: commit_created_at.timestamp_millis(),
                    user_login: row.get("user_login"),
                    branch: row.get("branch"),
                    repo_name: row.get("repo_name"),
                    pipeline,
                }
            })
            .collect();

        Ok(correlations)
    }

    pub async fn get_ticket_flow_correlations_v2(
        &self,
        filter: &CorrelationV2Query,
    ) -> Result<(Vec<TicketFlowCorrelation>, i64), DbError> {
        let limit = if filter.limit == 0 {
            50
        } else {
            filter.limit.min(500)
        } as i64;
        let offset = filter.offset as i64;
        let hours = filter.hours.unwrap_or(24 * 7).clamp(1, 24 * 90);

        let org_id = if let Some(org_name) = filter.org_name.as_deref() {
            self.get_org_by_login(org_name).await?.map(|o| o.id)
        } else {
            None
        };
        let repo_id = if let Some(repo_full_name) = filter.repo_full_name.as_deref() {
            self.get_repo_by_full_name(repo_full_name)
                .await?
                .map(|r| r.id)
        } else {
            None
        };
        let ticket_id = filter
            .ticket_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_ascii_uppercase());

        if filter.org_name.is_some() && org_id.is_none() {
            return Ok((vec![], 0));
        }
        if filter.repo_full_name.is_some() && repo_id.is_none() {
            return Ok((vec![], 0));
        }

        let count_row = sqlx::query(
            r#"
            WITH base AS (
                SELECT
                    ct.ticket_id,
                    ct.commit_sha,
                    COALESCE(c.created_at, ct.created_at) AS ordering_ts
                FROM commit_ticket_correlations ct
                LEFT JOIN project_tickets pt
                  ON pt.ticket_id = ct.ticket_id
                 AND (ct.org_id IS NULL OR pt.org_id = ct.org_id)
                LEFT JOIN LATERAL (
                    SELECT c.created_at, c.repo_id
                    FROM client_events c
                    WHERE c.event_type = 'commit'
                      AND c.commit_sha = ct.commit_sha
                    ORDER BY c.created_at DESC
                    LIMIT 1
                ) c ON TRUE
                WHERE ($1::uuid IS NULL OR ct.org_id = $1::uuid OR pt.org_id = $1::uuid)
                  AND ($2::uuid IS NULL OR c.repo_id = $2::uuid)
                  AND ($3::text IS NULL OR ct.ticket_id = $3)
                  AND COALESCE(c.created_at, ct.created_at) >= NOW() - make_interval(hours => $4::int)
            )
            SELECT COUNT(*)::bigint AS total
            FROM base
            "#,
        )
        .bind(&org_id)
        .bind(&repo_id)
        .bind(&ticket_id)
        .bind(hours as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let total: i64 = count_row.get("total");
        if total == 0 {
            return Ok((vec![], 0));
        }

        let rows = sqlx::query(
            r#"
            SELECT
                ct.ticket_id,
                pt.status AS ticket_status,
                ct.correlation_source,
                ct.confidence AS correlation_confidence,
                ct.commit_sha,
                c.branch,
                c.user_login,
                r.full_name AS repo_name,
                CASE
                    WHEN c.created_at IS NULL THEN NULL
                    ELSE EXTRACT(EPOCH FROM c.created_at)::bigint * 1000
                END AS commit_created_at_ms,
                p.id::text AS pipeline_event_id,
                p.pipeline_id,
                p.job_name,
                p.status AS pipeline_status,
                p.duration_ms AS pipeline_duration_ms,
                p.triggered_by,
                CASE
                    WHEN p.ingested_at IS NULL THEN NULL
                    ELSE EXTRACT(EPOCH FROM p.ingested_at)::bigint * 1000
                END AS pipeline_ingested_at_ms
            FROM commit_ticket_correlations ct
            LEFT JOIN project_tickets pt
              ON pt.ticket_id = ct.ticket_id
             AND (ct.org_id IS NULL OR pt.org_id = ct.org_id)
            LEFT JOIN LATERAL (
                SELECT c.branch, c.user_login, c.repo_id, c.created_at
                FROM client_events c
                WHERE c.event_type = 'commit'
                  AND c.commit_sha = ct.commit_sha
                ORDER BY c.created_at DESC
                LIMIT 1
            ) c ON TRUE
            LEFT JOIN repos r ON r.id = c.repo_id
            LEFT JOIN LATERAL (
                SELECT pe.*
                FROM pipeline_events pe
                WHERE pe.commit_sha IS NOT NULL
                  AND (
                    pe.commit_sha = ct.commit_sha
                    OR pe.commit_sha LIKE ct.commit_sha || '%'
                    OR ct.commit_sha LIKE pe.commit_sha || '%'
                  )
                ORDER BY pe.ingested_at DESC
                LIMIT 1
            ) p ON TRUE
            WHERE ($1::uuid IS NULL OR ct.org_id = $1::uuid OR pt.org_id = $1::uuid)
              AND ($2::uuid IS NULL OR c.repo_id = $2::uuid)
              AND ($3::text IS NULL OR ct.ticket_id = $3)
              AND COALESCE(c.created_at, ct.created_at) >= NOW() - make_interval(hours => $4::int)
            ORDER BY COALESCE(c.created_at, ct.created_at) DESC, ct.ticket_id, ct.commit_sha
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(&org_id)
        .bind(&repo_id)
        .bind(&ticket_id)
        .bind(hours as i32)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let items = rows
            .into_iter()
            .map(|row| {
                let pipeline =
                    row.get::<Option<String>, _>("pipeline_event_id")
                        .map(|pipeline_event_id| CommitPipelineRun {
                            pipeline_event_id,
                            pipeline_id: row.get("pipeline_id"),
                            job_name: row.get("job_name"),
                            status: row.get("pipeline_status"),
                            duration_ms: row.get("pipeline_duration_ms"),
                            triggered_by: row.get("triggered_by"),
                            ingested_at: row
                                .get::<Option<i64>, _>("pipeline_ingested_at_ms")
                                .unwrap_or_default(),
                        });

                TicketFlowCorrelation {
                    ticket_id: row.get("ticket_id"),
                    ticket_status: row.get("ticket_status"),
                    correlation_source: row.get("correlation_source"),
                    correlation_confidence: row.get("correlation_confidence"),
                    commit_sha: row.get("commit_sha"),
                    branch: row.get("branch"),
                    user_login: row.get("user_login"),
                    repo_name: row.get("repo_name"),
                    commit_created_at: row.get("commit_created_at_ms"),
                    pipeline,
                }
            })
            .collect();

        Ok((items, total))
    }
}
