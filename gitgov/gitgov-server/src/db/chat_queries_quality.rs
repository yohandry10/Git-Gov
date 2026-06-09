use super::*;

impl Database {
    pub async fn chat_query_quality_gate_window_summary(
        &self,
        org_id: Option<&str>,
        hours: i64,
    ) -> Result<serde_json::Value, DbError> {
        let safe_hours = hours.clamp(1, 24 * 30) as i32;
        let row = sqlx::query(
            r#"
            WITH quality_gate_events AS (
              SELECT
                lower(COALESCE(stage->>'status', 'unknown')) AS gate_status,
                COALESCE(pe.repo_full_name, 'unknown') AS repo_full_name,
                pe.commit_sha
              FROM pipeline_events pe
              CROSS JOIN LATERAL jsonb_array_elements(
                CASE
                  WHEN jsonb_typeof(pe.stages) = 'array' THEN pe.stages
                  ELSE '[]'::jsonb
                END
              ) AS stage
              WHERE pe.ingested_at >= NOW() - make_interval(hours => $1::int)
                AND ($2::uuid IS NULL OR pe.org_id = $2::uuid)
                AND lower(COALESCE(stage->>'name', '')) = 'quality_gate'
            ),
            signal_events AS (
              SELECT COUNT(*)::bigint AS policy_violation_signals
              FROM noncompliance_signals ns
              WHERE ns.created_at >= NOW() - make_interval(hours => $1::int)
                AND ($2::uuid IS NULL OR ns.org_id = $2::uuid)
                AND ns.signal_type = 'policy_violation'
                AND COALESCE(ns.evidence->>'rule', '') = 'quality_gate_green'
                AND COALESCE(ns.status, 'open') <> 'resolved'
            )
            SELECT
              COUNT(*)::bigint AS total_runs,
              COUNT(*) FILTER (
                WHERE gate_status IN ('passed', 'ok', 'green', 'success')
              )::bigint AS green_runs,
              COUNT(*) FILTER (
                WHERE gate_status NOT IN ('passed', 'ok', 'green', 'success')
              )::bigint AS non_green_runs,
              COUNT(DISTINCT repo_full_name)::bigint AS repos_affected,
              COUNT(DISTINCT commit_sha)::bigint AS commits_affected,
              COALESCE((SELECT policy_violation_signals FROM signal_events), 0)::bigint AS policy_violation_signals
            FROM quality_gate_events
            "#,
        )
        .bind(safe_hours)
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(serde_json::json!({
            "window_hours": safe_hours,
            "total_runs": row.get::<i64, _>("total_runs"),
            "green_runs": row.get::<i64, _>("green_runs"),
            "non_green_runs": row.get::<i64, _>("non_green_runs"),
            "repos_affected": row.get::<i64, _>("repos_affected"),
            "commits_affected": row.get::<i64, _>("commits_affected"),
            "policy_violation_signals": row.get::<i64, _>("policy_violation_signals"),
        }))
    }

    /// Q6b: Rank repositories with the highest non-green quality gate volume.
    pub async fn chat_query_quality_gate_top_failing_repos(
        &self,
        org_id: Option<&str>,
        hours: i64,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, DbError> {
        let safe_hours = hours.clamp(1, 24 * 30) as i32;
        let safe_limit = limit.clamp(1, 20) as i32;

        let rows = sqlx::query(
            r#"
            WITH quality_gate_events AS (
              SELECT
                COALESCE(pe.repo_full_name, 'unknown') AS repo_full_name,
                lower(COALESCE(stage->>'status', 'unknown')) AS gate_status,
                pe.ingested_at
              FROM pipeline_events pe
              CROSS JOIN LATERAL jsonb_array_elements(
                CASE
                  WHEN jsonb_typeof(pe.stages) = 'array' THEN pe.stages
                  ELSE '[]'::jsonb
                END
              ) AS stage
              WHERE pe.ingested_at >= NOW() - make_interval(hours => $1::int)
                AND ($2::uuid IS NULL OR pe.org_id = $2::uuid)
                AND lower(COALESCE(stage->>'name', '')) = 'quality_gate'
            )
            SELECT
              repo_full_name,
              COUNT(*)::bigint AS total_runs,
              COUNT(*) FILTER (
                WHERE gate_status NOT IN ('passed', 'ok', 'green', 'success')
              )::bigint AS non_green_runs,
              ROUND(
                (
                  COUNT(*) FILTER (
                    WHERE gate_status NOT IN ('passed', 'ok', 'green', 'success')
                  )::numeric * 100.0
                ) / NULLIF(COUNT(*)::numeric, 0),
                1
              )::double precision AS non_green_pct,
              (EXTRACT(EPOCH FROM MAX(ingested_at)) * 1000)::bigint AS last_seen_ms
            FROM quality_gate_events
            GROUP BY repo_full_name
            HAVING COUNT(*) FILTER (
              WHERE gate_status NOT IN ('passed', 'ok', 'green', 'success')
            ) > 0
            ORDER BY non_green_runs DESC, non_green_pct DESC, MAX(ingested_at) DESC
            LIMIT $3::int
            "#,
        )
        .bind(safe_hours)
        .bind(org_id)
        .bind(safe_limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "repo_full_name": r.get::<String, _>("repo_full_name"),
                    "total_runs": r.get::<i64, _>("total_runs"),
                    "non_green_runs": r.get::<i64, _>("non_green_runs"),
                    "non_green_pct": r.get::<f64, _>("non_green_pct"),
                    "last_seen_ms": r.get::<i64, _>("last_seen_ms"),
                    "window_hours": safe_hours,
                })
            })
            .collect())
    }

    /// Q6c: Rank branches with the highest non-green quality gate volume.
    pub async fn chat_query_quality_gate_top_failing_branches(
        &self,
        org_id: Option<&str>,
        hours: i64,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, DbError> {
        let safe_hours = hours.clamp(1, 24 * 30) as i32;
        let safe_limit = limit.clamp(1, 20) as i32;

        let rows = sqlx::query(
            r#"
            WITH quality_gate_events AS (
              SELECT
                COALESCE(NULLIF(trim(pe.branch), ''), 'unknown') AS branch_name,
                lower(COALESCE(stage->>'status', 'unknown')) AS gate_status,
                pe.ingested_at
              FROM pipeline_events pe
              CROSS JOIN LATERAL jsonb_array_elements(
                CASE
                  WHEN jsonb_typeof(pe.stages) = 'array' THEN pe.stages
                  ELSE '[]'::jsonb
                END
              ) AS stage
              WHERE pe.ingested_at >= NOW() - make_interval(hours => $1::int)
                AND ($2::uuid IS NULL OR pe.org_id = $2::uuid)
                AND lower(COALESCE(stage->>'name', '')) = 'quality_gate'
            )
            SELECT
              branch_name,
              COUNT(*)::bigint AS total_runs,
              COUNT(*) FILTER (
                WHERE gate_status NOT IN ('passed', 'ok', 'green', 'success')
              )::bigint AS non_green_runs,
              ROUND(
                (
                  COUNT(*) FILTER (
                    WHERE gate_status NOT IN ('passed', 'ok', 'green', 'success')
                  )::numeric * 100.0
                ) / NULLIF(COUNT(*)::numeric, 0),
                1
              )::double precision AS non_green_pct,
              (EXTRACT(EPOCH FROM MAX(ingested_at)) * 1000)::bigint AS last_seen_ms
            FROM quality_gate_events
            GROUP BY branch_name
            HAVING COUNT(*) FILTER (
              WHERE gate_status NOT IN ('passed', 'ok', 'green', 'success')
            ) > 0
            ORDER BY non_green_runs DESC, non_green_pct DESC, MAX(ingested_at) DESC
            LIMIT $3::int
            "#,
        )
        .bind(safe_hours)
        .bind(org_id)
        .bind(safe_limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "branch_name": r.get::<String, _>("branch_name"),
                    "total_runs": r.get::<i64, _>("total_runs"),
                    "non_green_runs": r.get::<i64, _>("non_green_runs"),
                    "non_green_pct": r.get::<f64, _>("non_green_pct"),
                    "last_seen_ms": r.get::<i64, _>("last_seen_ms"),
                    "window_hours": safe_hours,
                })
            })
            .collect())
    }

    /// Q6d: Rank Jira tickets linked to commits with non-green quality gate outcomes.
    pub async fn chat_query_tickets_with_non_green_quality_gate(
        &self,
        org_id: Option<&str>,
        hours: i64,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, DbError> {
        let safe_hours = hours.clamp(1, 24 * 30) as i32;
        let safe_limit = limit.clamp(1, 20) as i32;

        let rows = sqlx::query(
            r#"
            WITH quality_gate_non_green AS (
              SELECT
                pe.commit_sha,
                COALESCE(pe.repo_full_name, 'unknown') AS repo_full_name,
                pe.ingested_at
              FROM pipeline_events pe
              CROSS JOIN LATERAL jsonb_array_elements(
                CASE
                  WHEN jsonb_typeof(pe.stages) = 'array' THEN pe.stages
                  ELSE '[]'::jsonb
                END
              ) AS stage
              WHERE pe.ingested_at >= NOW() - make_interval(hours => $1::int)
                AND pe.commit_sha IS NOT NULL
                AND ($2::uuid IS NULL OR pe.org_id = $2::uuid)
                AND lower(COALESCE(stage->>'name', '')) = 'quality_gate'
                AND lower(COALESCE(stage->>'status', 'unknown')) NOT IN ('passed', 'ok', 'green', 'success')
            ),
            ticket_hits AS (
              SELECT
                ctc.ticket_id,
                q.repo_full_name,
                q.commit_sha,
                q.ingested_at
              FROM quality_gate_non_green q
              JOIN commit_ticket_correlations ctc
                ON ctc.commit_sha IS NOT NULL
               AND (
                    ctc.commit_sha = q.commit_sha
                    OR ctc.commit_sha LIKE q.commit_sha || '%'
                    OR q.commit_sha LIKE ctc.commit_sha || '%'
               )
              WHERE ($2::uuid IS NULL OR ctc.org_id = $2::uuid)
            )
            SELECT
              ticket_id,
              COUNT(*)::bigint AS non_green_runs,
              COUNT(DISTINCT repo_full_name)::bigint AS repos_affected,
              COUNT(DISTINCT commit_sha)::bigint AS commits_affected,
              (EXTRACT(EPOCH FROM MAX(ingested_at)) * 1000)::bigint AS last_seen_ms
            FROM ticket_hits
            GROUP BY ticket_id
            ORDER BY non_green_runs DESC, MAX(ingested_at) DESC
            LIMIT $3::int
            "#,
        )
        .bind(safe_hours)
        .bind(org_id)
        .bind(safe_limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "ticket_id": r.get::<String, _>("ticket_id"),
                    "non_green_runs": r.get::<i64, _>("non_green_runs"),
                    "repos_affected": r.get::<i64, _>("repos_affected"),
                    "commits_affected": r.get::<i64, _>("commits_affected"),
                    "last_seen_ms": r.get::<i64, _>("last_seen_ms"),
                    "window_hours": safe_hours,
                })
            })
            .collect())
    }

    /// Q6e: Rank Jira tickets that had non-green quality gate and a successful deploy/release run.
    pub async fn chat_query_tickets_released_with_non_green_quality_gate(
        &self,
        org_id: Option<&str>,
        hours: i64,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, DbError> {
        let safe_hours = hours.clamp(1, 24 * 30) as i32;
        let safe_limit = limit.clamp(1, 20) as i32;

        let rows = sqlx::query(
            r#"
            WITH quality_gate_non_green AS (
              SELECT
                pe.commit_sha,
                COALESCE(pe.repo_full_name, 'unknown') AS repo_full_name,
                pe.ingested_at AS quality_gate_at
              FROM pipeline_events pe
              CROSS JOIN LATERAL jsonb_array_elements(
                CASE
                  WHEN jsonb_typeof(pe.stages) = 'array' THEN pe.stages
                  ELSE '[]'::jsonb
                END
              ) AS stage
              WHERE pe.ingested_at >= NOW() - make_interval(hours => $1::int)
                AND pe.commit_sha IS NOT NULL
                AND ($2::uuid IS NULL OR pe.org_id = $2::uuid)
                AND lower(COALESCE(stage->>'name', '')) = 'quality_gate'
                AND lower(COALESCE(stage->>'status', 'unknown')) NOT IN ('passed', 'ok', 'green', 'success')
            ),
            release_success AS (
              SELECT
                pe.pipeline_id,
                pe.commit_sha,
                pe.ingested_at AS release_at
              FROM pipeline_events pe
              WHERE pe.ingested_at >= NOW() - make_interval(hours => $1::int)
                AND pe.commit_sha IS NOT NULL
                AND ($2::uuid IS NULL OR pe.org_id = $2::uuid)
                AND lower(COALESCE(pe.status, '')) = 'success'
                AND (
                  lower(COALESCE(pe.job_name, '')) LIKE '%deploy%'
                  OR lower(COALESCE(pe.job_name, '')) LIKE '%release%'
                  OR lower(COALESCE(pe.job_name, '')) LIKE '%prod%'
                  OR lower(COALESCE(pe.job_name, '')) LIKE '%promote%'
                  OR lower(COALESCE(pe.branch, '')) = 'main'
                )
            ),
            ticket_release_hits AS (
              SELECT
                ctc.ticket_id,
                q.repo_full_name,
                q.commit_sha,
                rs.pipeline_id,
                rs.release_at
              FROM quality_gate_non_green q
              JOIN release_success rs
                ON (
                     rs.commit_sha = q.commit_sha
                     OR rs.commit_sha LIKE q.commit_sha || '%'
                     OR q.commit_sha LIKE rs.commit_sha || '%'
                   )
               AND rs.release_at >= q.quality_gate_at
              JOIN commit_ticket_correlations ctc
                ON ctc.commit_sha IS NOT NULL
               AND (
                    ctc.commit_sha = q.commit_sha
                    OR ctc.commit_sha LIKE q.commit_sha || '%'
                    OR q.commit_sha LIKE ctc.commit_sha || '%'
               )
              WHERE ($2::uuid IS NULL OR ctc.org_id = $2::uuid)
            )
            SELECT
              ticket_id,
              COUNT(*)::bigint AS non_green_runs,
              COUNT(DISTINCT pipeline_id)::bigint AS successful_release_runs,
              COUNT(DISTINCT repo_full_name)::bigint AS repos_affected,
              COUNT(DISTINCT commit_sha)::bigint AS commits_affected,
              (EXTRACT(EPOCH FROM MAX(release_at)) * 1000)::bigint AS last_release_ms
            FROM ticket_release_hits
            GROUP BY ticket_id
            ORDER BY successful_release_runs DESC, non_green_runs DESC, MAX(release_at) DESC
            LIMIT $3::int
            "#,
        )
        .bind(safe_hours)
        .bind(org_id)
        .bind(safe_limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "ticket_id": r.get::<String, _>("ticket_id"),
                    "non_green_runs": r.get::<i64, _>("non_green_runs"),
                    "successful_release_runs": r.get::<i64, _>("successful_release_runs"),
                    "repos_affected": r.get::<i64, _>("repos_affected"),
                    "commits_affected": r.get::<i64, _>("commits_affected"),
                    "last_release_ms": r.get::<i64, _>("last_release_ms"),
                    "window_hours": safe_hours,
                })
            })
            .collect())
    }

    /// Q6d: Rank developers (triggered_by) linked to non-green quality gate outcomes.
    pub async fn chat_query_developers_with_non_green_quality_gate(
        &self,
        org_id: Option<&str>,
        hours: i64,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, DbError> {
        let safe_hours = hours.clamp(1, 24 * 30) as i32;
        let safe_limit = limit.clamp(1, 20) as i32;

        let rows = sqlx::query(
            r#"
            WITH quality_gate_non_green AS (
              SELECT
                COALESCE(NULLIF(trim(pe.triggered_by), ''), 'unknown') AS actor_login,
                COALESCE(pe.repo_full_name, 'unknown') AS repo_full_name,
                pe.commit_sha,
                pe.ingested_at
              FROM pipeline_events pe
              CROSS JOIN LATERAL jsonb_array_elements(
                CASE
                  WHEN jsonb_typeof(pe.stages) = 'array' THEN pe.stages
                  ELSE '[]'::jsonb
                END
              ) AS stage
              WHERE pe.ingested_at >= NOW() - make_interval(hours => $1::int)
                AND ($2::uuid IS NULL OR pe.org_id = $2::uuid)
                AND lower(COALESCE(stage->>'name', '')) = 'quality_gate'
                AND lower(COALESCE(stage->>'status', 'unknown')) NOT IN ('passed', 'ok', 'green', 'success')
            )
            SELECT
              actor_login,
              COUNT(*)::bigint AS non_green_runs,
              COUNT(DISTINCT repo_full_name)::bigint AS repos_affected,
              COUNT(DISTINCT commit_sha)::bigint AS commits_affected,
              (EXTRACT(EPOCH FROM MAX(ingested_at)) * 1000)::bigint AS last_seen_ms
            FROM quality_gate_non_green
            GROUP BY actor_login
            ORDER BY non_green_runs DESC, MAX(ingested_at) DESC
            LIMIT $3::int
            "#,
        )
        .bind(safe_hours)
        .bind(org_id)
        .bind(safe_limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "actor_login": r.get::<String, _>("actor_login"),
                    "non_green_runs": r.get::<i64, _>("non_green_runs"),
                    "repos_affected": r.get::<i64, _>("repos_affected"),
                    "commits_affected": r.get::<i64, _>("commits_affected"),
                    "last_seen_ms": r.get::<i64, _>("last_seen_ms"),
                    "window_hours": safe_hours,
                })
            })
            .collect())
    }
}
