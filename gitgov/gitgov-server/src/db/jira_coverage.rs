use super::*;

impl Database {
    pub async fn get_ticket_coverage(
        &self,
        org_name: Option<&str>,
        org_id_override: Option<&str>,
        repo_full_name: Option<&str>,
        branch: Option<&str>,
        hours: i64,
    ) -> Result<TicketCoverageResponse, DbError> {
        // Prefer the server-side scope binding (org_id_override) over the
        // client-supplied org_name so a scoped key cannot read another org.
        let org_id = if let Some(id) = org_id_override {
            Some(id.to_string())
        } else if let Some(name) = org_name {
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
            ),
            correlated_commits AS (
                SELECT
                    u.commit_sha,
                    BOOL_OR(pt.ticket_id IS NOT NULL) AS has_verified_ticket
                FROM commit_universe u
                JOIN commit_ticket_correlations ct
                  ON ct.commit_sha = u.commit_sha
                 AND (
                    (u.org_id IS NULL AND ct.org_id IS NULL)
                    OR ct.org_id = u.org_id
                 )
                LEFT JOIN project_tickets pt
                  ON pt.ticket_id = ct.ticket_id
                 AND (
                    (ct.org_id IS NULL AND pt.org_id IS NULL)
                    OR pt.org_id = ct.org_id
                 )
                WHERE u.commit_sha IS NOT NULL
                GROUP BY u.commit_sha
            )
            SELECT
                COUNT(*) FILTER (WHERE has_verified_ticket)::bigint AS verified_covered,
                COUNT(*) FILTER (WHERE NOT has_verified_ticket)::bigint AS unverified_covered
            FROM correlated_commits
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
            SELECT
                u.commit_sha,
                u.user_login,
                u.branch,
                u.created_at,
                u.source,
                EXISTS (
                    SELECT 1
                    FROM commit_ticket_correlations ct
                    WHERE ct.commit_sha = u.commit_sha
                      AND (
                        (u.org_id IS NULL AND ct.org_id IS NULL)
                        OR ct.org_id = u.org_id
                      )
                ) AS has_unverified_link
            FROM distinct_commits u
            WHERE u.commit_sha IS NOT NULL
              AND NOT EXISTS (
                  SELECT 1
                  FROM commit_ticket_correlations ct
                  JOIN project_tickets pt
                    ON pt.ticket_id = ct.ticket_id
                   AND (
                      (ct.org_id IS NULL AND pt.org_id IS NULL)
                      OR pt.org_id = ct.org_id
                   )
                  WHERE ct.commit_sha = u.commit_sha
                    AND (
                      (u.org_id IS NULL AND ct.org_id IS NULL)
                      OR ct.org_id = u.org_id
                    )
              )
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
        let commits_with_ticket: i64 = with_ticket_row.get("verified_covered");
        let detected_unverified_commits: i64 = with_ticket_row.get("unverified_covered");
        let coverage_percentage = if total_commits > 0 {
            (commits_with_ticket as f64 / total_commits as f64) * 100.0
        } else {
            0.0
        };
        let unverified_coverage_percentage = if total_commits > 0 {
            (detected_unverified_commits as f64 / total_commits as f64) * 100.0
        } else {
            0.0
        };

        Ok(TicketCoverageResponse {
            org: org_name.unwrap_or("all").to_string(),
            period: format!("last_{}h", hours),
            total_commits,
            commits_with_ticket,
            coverage_percentage,
            detected_unverified_commits,
            unverified_coverage_percentage,
            commits_without_ticket: missing_rows
                .into_iter()
                .map(|row| {
                    serde_json::json!({
                        "commit_sha": row.get::<String, _>("commit_sha"),
                        "user_login": row.get::<Option<String>, _>("user_login"),
                        "branch": row.get::<Option<String>, _>("branch"),
                        "source": row.get::<String, _>("source"),
                        "unverified_link": row.get::<bool, _>("has_unverified_link"),
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

        // Prefer the server-side scope binding (filter.org_id) over the
        // client-supplied org_name so a scoped key cannot read another org.
        let org_id = if let Some(id) = filter.org_id.clone() {
            Some(id)
        } else if let Some(org_name) = filter.org_name.as_deref() {
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

        if filter.org_id.is_none() && filter.org_name.is_some() && org_id.is_none() {
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
                p.branch AS pipeline_branch,
                p.repo_full_name AS pipeline_repo_full_name,
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
                  AND (c.org_id IS NULL OR pe.org_id = c.org_id)
                  AND (
                    pe.repo_full_name IS NULL
                    OR r.full_name IS NULL
                    OR pe.repo_full_name = r.full_name
                  )
                  AND (
                    pe.branch IS NULL
                    OR c.branch IS NULL
                    OR pe.branch = c.branch
                  )
                  AND (
                    LOWER(pe.commit_sha) = LOWER(c.commit_sha)
                    OR (
                      length(pe.commit_sha) < 40
                      AND LOWER(c.commit_sha) LIKE LOWER(pe.commit_sha) || '%'
                    )
                    OR (
                      length(c.commit_sha) < 40
                      AND LOWER(pe.commit_sha) LIKE LOWER(c.commit_sha) || '%'
                    )
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
                                branch: row.get("pipeline_branch"),
                                repo_full_name: row.get("pipeline_repo_full_name"),
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

        // Prefer the server-side scope binding (filter.org_id) over the
        // client-supplied org_name so a scoped key cannot read another org.
        let org_id = if let Some(id) = filter.org_id.clone() {
            Some(id)
        } else if let Some(org_name) = filter.org_name.as_deref() {
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
        let branch = filter
            .branch
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let target_sha = filter
            .target_sha
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_ascii_lowercase());

        if filter.org_id.is_none() && filter.org_name.is_some() && org_id.is_none() {
            return Ok((vec![], 0));
        }
        if filter.repo_full_name.is_some() && repo_id.is_none() {
            return Ok((vec![], 0));
        }

        let count_row = sqlx::query(
            r#"
            WITH correlated AS (
                SELECT
                    ct.ticket_id,
                    ct.commit_sha,
                    ct.org_id,
                    ct.created_at
                FROM commit_ticket_correlations ct
                LEFT JOIN project_tickets pt
                  ON pt.ticket_id = ct.ticket_id
                 AND (
                    (ct.org_id IS NULL AND pt.org_id IS NULL)
                    OR pt.org_id = ct.org_id
                 )
                WHERE (($1::uuid IS NULL AND ct.org_id IS NULL) OR ct.org_id = $1::uuid)
                  AND ($3::text IS NULL OR ct.ticket_id = $3)
                  AND (
                    $6::text IS NULL
                    OR LOWER(ct.commit_sha) = LOWER($6)
                    OR (
                      length(ct.commit_sha) < 40
                      AND LOWER($6) LIKE LOWER(ct.commit_sha) || '%'
                    )
                    OR (
                      length($6) < 40
                      AND LOWER(ct.commit_sha) LIKE LOWER($6) || '%'
                    )
                  )
            ),
            base AS (
                SELECT
                    ct.ticket_id,
                    ct.commit_sha,
                    COALESCE(ev.created_at, ct.created_at) AS ordering_ts
                FROM correlated ct
                LEFT JOIN LATERAL (
                    SELECT candidate.repo_id, candidate.branch, candidate.created_at
                    FROM (
                        SELECT c.repo_id, c.branch, c.created_at, 1 AS source_priority
                        FROM client_events c
                        WHERE c.event_type = 'commit'
                          AND c.commit_sha = ct.commit_sha
                          AND (
                            ($1::uuid IS NULL AND ct.org_id IS NULL)
                            OR c.org_id = COALESCE(ct.org_id, $1::uuid)
                          )

                        UNION ALL

                        SELECT prm.repo_id, prm.base_branch AS branch, prm.created_at, 2 AS source_priority
                        FROM pull_request_merges prm
                        WHERE (
                            ($1::uuid IS NULL AND ct.org_id IS NULL)
                            OR prm.org_id = COALESCE(ct.org_id, $1::uuid)
                          )
                          AND (
                            LOWER(COALESCE(NULLIF(prm.head_sha, ''), '')) = LOWER(ct.commit_sha)
                            OR LOWER(COALESCE(
                              NULLIF(prm.payload #>> '{pull_request,merge_commit_sha}', ''),
                              NULLIF(prm.payload #>> '{gitgov,merge_commit_sha}', ''),
                              ''
                            )) = LOWER(ct.commit_sha)
                          )
                    ) candidate
                    WHERE ($2::uuid IS NULL OR candidate.repo_id = $2::uuid)
                      AND ($5::text IS NULL OR candidate.branch = $5)
                    ORDER BY candidate.created_at DESC, candidate.source_priority
                    LIMIT 1
                ) ev ON TRUE
                WHERE ev.created_at IS NOT NULL
                  AND ev.created_at >= NOW() - make_interval(hours => $4::int)
                  AND ($2::uuid IS NULL OR ev.repo_id = $2::uuid)
                  AND ($5::text IS NULL OR ev.branch = $5)
            )
            SELECT COUNT(*)::bigint AS total
            FROM base
            "#,
        )
        .bind(&org_id)
        .bind(&repo_id)
        .bind(&ticket_id)
        .bind(hours as i32)
        .bind(&branch)
        .bind(&target_sha)
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
                ev.evidence_source,
                ev.branch,
                ev.user_login,
                r.full_name AS repo_name,
                CASE
                    WHEN ev.created_at IS NULL THEN NULL
                    ELSE EXTRACT(EPOCH FROM ev.created_at)::bigint * 1000
                END AS commit_created_at_ms,
                p.id::text AS pipeline_event_id,
                p.pipeline_id,
                p.job_name,
                p.status AS pipeline_status,
                p.branch AS pipeline_branch,
                p.repo_full_name AS pipeline_repo_full_name,
                p.duration_ms AS pipeline_duration_ms,
                p.triggered_by,
                CASE
                    WHEN p.ingested_at IS NULL THEN NULL
                    ELSE EXTRACT(EPOCH FROM p.ingested_at)::bigint * 1000
                END AS pipeline_ingested_at_ms
            FROM commit_ticket_correlations ct
            LEFT JOIN project_tickets pt
              ON pt.ticket_id = ct.ticket_id
             AND (
                (ct.org_id IS NULL AND pt.org_id IS NULL)
                OR pt.org_id = ct.org_id
             )
            LEFT JOIN LATERAL (
                SELECT candidate.*
                FROM (
                    SELECT
                        c.org_id,
                        c.repo_id,
                        c.branch,
                        c.user_login,
                        c.created_at,
                        'client_event'::text AS evidence_source,
                        1 AS source_priority
                    FROM client_events c
                    WHERE c.event_type = 'commit'
                      AND c.commit_sha = ct.commit_sha
                      AND (
                        ($1::uuid IS NULL AND ct.org_id IS NULL)
                        OR c.org_id = COALESCE(ct.org_id, $1::uuid)
                      )

                    UNION ALL

                    SELECT
                        prm.org_id,
                        prm.repo_id,
                        prm.base_branch AS branch,
                        COALESCE(prm.merged_by_login, prm.author_login) AS user_login,
                        prm.created_at,
                        'pull_request_merge'::text AS evidence_source,
                        2 AS source_priority
                    FROM pull_request_merges prm
                    WHERE (
                        ($1::uuid IS NULL AND ct.org_id IS NULL)
                        OR prm.org_id = COALESCE(ct.org_id, $1::uuid)
                      )
                      AND (
                        LOWER(COALESCE(NULLIF(prm.head_sha, ''), '')) = LOWER(ct.commit_sha)
                        OR LOWER(COALESCE(
                          NULLIF(prm.payload #>> '{pull_request,merge_commit_sha}', ''),
                          NULLIF(prm.payload #>> '{gitgov,merge_commit_sha}', ''),
                          ''
                        )) = LOWER(ct.commit_sha)
                      )
                ) candidate
                WHERE ($2::uuid IS NULL OR candidate.repo_id = $2::uuid)
                  AND ($5::text IS NULL OR candidate.branch = $5)
                ORDER BY candidate.created_at DESC, candidate.source_priority
                LIMIT 1
            ) ev ON TRUE
            LEFT JOIN repos r ON r.id = ev.repo_id
            LEFT JOIN LATERAL (
                SELECT pe.*
                FROM pipeline_events pe
                WHERE pe.commit_sha IS NOT NULL
                  AND (
                    COALESCE(ev.org_id, ct.org_id, $1::uuid) IS NULL
                    OR pe.org_id = COALESCE(ev.org_id, ct.org_id, $1::uuid)
                  )
                  AND (
                    pe.repo_full_name IS NULL
                    OR r.full_name IS NULL
                    OR pe.repo_full_name = r.full_name
                  )
                  AND (
                    pe.branch IS NULL
                    OR ev.branch IS NULL
                    OR pe.branch = ev.branch
                  )
                  AND (
                    LOWER(pe.commit_sha) = LOWER(ct.commit_sha)
                    OR (
                      length(pe.commit_sha) < 40
                      AND LOWER(ct.commit_sha) LIKE LOWER(pe.commit_sha) || '%'
                    )
                    OR (
                      length(ct.commit_sha) < 40
                      AND LOWER(pe.commit_sha) LIKE LOWER(ct.commit_sha) || '%'
                    )
                  )
                ORDER BY pe.ingested_at DESC
                LIMIT 1
            ) p ON TRUE
            WHERE (($1::uuid IS NULL AND ct.org_id IS NULL) OR ct.org_id = $1::uuid)
              AND ($2::uuid IS NULL OR ev.repo_id = $2::uuid)
              AND ($3::text IS NULL OR ct.ticket_id = $3)
              AND ev.created_at IS NOT NULL
              AND ev.created_at >= NOW() - make_interval(hours => $4::int)
              AND ($5::text IS NULL OR ev.branch = $5)
              AND (
                $6::text IS NULL
                OR LOWER(ct.commit_sha) = LOWER($6)
                OR (
                  length(ct.commit_sha) < 40
                  AND LOWER($6) LIKE LOWER(ct.commit_sha) || '%'
                )
                OR (
                  length($6) < 40
                  AND LOWER(ct.commit_sha) LIKE LOWER($6) || '%'
                )
              )
            ORDER BY ev.created_at DESC, ct.ticket_id, ct.commit_sha
            LIMIT $7 OFFSET $8
            "#,
        )
        .bind(&org_id)
        .bind(&repo_id)
        .bind(&ticket_id)
        .bind(hours as i32)
        .bind(&branch)
        .bind(&target_sha)
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
                            branch: row.get("pipeline_branch"),
                            repo_full_name: row.get("pipeline_repo_full_name"),
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
                    evidence_source: row.get("evidence_source"),
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
