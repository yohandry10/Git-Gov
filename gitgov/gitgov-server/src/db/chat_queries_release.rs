use super::*;

impl Database {
    pub async fn chat_query_release_readiness_window_summary(
        &self,
        org_id: Option<&str>,
        hours: i64,
    ) -> Result<serde_json::Value, DbError> {
        let safe_hours = hours.clamp(1, 24 * 30) as i32;
        let row = sqlx::query(
            r#"
            WITH readiness_events AS (
              SELECT
                lower(COALESCE(stage->>'status', 'unknown')) AS readiness_status,
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
                AND lower(COALESCE(stage->>'name', '')) = 'release_readiness'
            )
            SELECT
              COUNT(*)::bigint AS total_runs,
              COUNT(*) FILTER (
                WHERE readiness_status IN ('pass', 'passed', 'ok', 'green', 'success')
              )::bigint AS pass_runs,
              COUNT(*) FILTER (
                WHERE readiness_status IN ('warn', 'warning', 'unstable', 'advisory')
              )::bigint AS warn_runs,
              COUNT(*) FILTER (
                WHERE readiness_status IN ('fail', 'failure', 'blocked', 'deny', 'denied', 'error')
              )::bigint AS fail_runs,
              COUNT(*) FILTER (
                WHERE readiness_status NOT IN (
                  'pass', 'passed', 'ok', 'green', 'success',
                  'warn', 'warning', 'unstable', 'advisory',
                  'fail', 'failure', 'blocked', 'deny', 'denied', 'error'
                )
              )::bigint AS other_runs,
              COUNT(DISTINCT repo_full_name)::bigint AS repos_affected,
              COUNT(DISTINCT commit_sha)::bigint AS commits_affected
            FROM readiness_events
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
            "pass_runs": row.get::<i64, _>("pass_runs"),
            "warn_runs": row.get::<i64, _>("warn_runs"),
            "fail_runs": row.get::<i64, _>("fail_runs"),
            "other_runs": row.get::<i64, _>("other_runs"),
            "repos_affected": row.get::<i64, _>("repos_affected"),
            "commits_affected": row.get::<i64, _>("commits_affected"),
        }))
    }

    /// Q7b: Rank repositories with the highest release-readiness failures.
    pub async fn chat_query_release_readiness_top_failing_repos(
        &self,
        org_id: Option<&str>,
        hours: i64,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, DbError> {
        let safe_hours = hours.clamp(1, 24 * 30) as i32;
        let safe_limit = limit.clamp(1, 20) as i32;

        let rows = sqlx::query(
            r#"
            WITH readiness_events AS (
              SELECT
                COALESCE(pe.repo_full_name, 'unknown') AS repo_full_name,
                lower(COALESCE(stage->>'status', 'unknown')) AS readiness_status,
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
                AND lower(COALESCE(stage->>'name', '')) = 'release_readiness'
            )
            SELECT
              repo_full_name,
              COUNT(*)::bigint AS total_runs,
              COUNT(*) FILTER (
                WHERE readiness_status IN ('fail', 'failure', 'blocked', 'deny', 'denied', 'error')
              )::bigint AS fail_runs,
              ROUND(
                (
                  COUNT(*) FILTER (
                    WHERE readiness_status IN ('fail', 'failure', 'blocked', 'deny', 'denied', 'error')
                  )::numeric * 100.0
                ) / NULLIF(COUNT(*)::numeric, 0),
                1
              )::double precision AS fail_pct,
              (EXTRACT(EPOCH FROM MAX(ingested_at)) * 1000)::bigint AS last_seen_ms
            FROM readiness_events
            GROUP BY repo_full_name
            HAVING COUNT(*) FILTER (
              WHERE readiness_status IN ('fail', 'failure', 'blocked', 'deny', 'denied', 'error')
            ) > 0
            ORDER BY fail_runs DESC, fail_pct DESC, MAX(ingested_at) DESC
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
                    "fail_runs": r.get::<i64, _>("fail_runs"),
                    "fail_pct": r.get::<f64, _>("fail_pct"),
                    "last_seen_ms": r.get::<i64, _>("last_seen_ms"),
                    "window_hours": safe_hours,
                })
            })
            .collect())
    }

    /// Q7c: Rank branches with the highest release-readiness failures.
    pub async fn chat_query_release_readiness_top_failing_branches(
        &self,
        org_id: Option<&str>,
        hours: i64,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, DbError> {
        let safe_hours = hours.clamp(1, 24 * 30) as i32;
        let safe_limit = limit.clamp(1, 20) as i32;

        let rows = sqlx::query(
            r#"
            WITH readiness_events AS (
              SELECT
                COALESCE(NULLIF(trim(pe.branch), ''), 'unknown') AS branch_name,
                lower(COALESCE(stage->>'status', 'unknown')) AS readiness_status,
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
                AND lower(COALESCE(stage->>'name', '')) = 'release_readiness'
            )
            SELECT
              branch_name,
              COUNT(*)::bigint AS total_runs,
              COUNT(*) FILTER (
                WHERE readiness_status IN ('fail', 'failure', 'blocked', 'deny', 'denied', 'error')
              )::bigint AS fail_runs,
              ROUND(
                (
                  COUNT(*) FILTER (
                    WHERE readiness_status IN ('fail', 'failure', 'blocked', 'deny', 'denied', 'error')
                  )::numeric * 100.0
                ) / NULLIF(COUNT(*)::numeric, 0),
                1
              )::double precision AS fail_pct,
              (EXTRACT(EPOCH FROM MAX(ingested_at)) * 1000)::bigint AS last_seen_ms
            FROM readiness_events
            GROUP BY branch_name
            HAVING COUNT(*) FILTER (
              WHERE readiness_status IN ('fail', 'failure', 'blocked', 'deny', 'denied', 'error')
            ) > 0
            ORDER BY fail_runs DESC, fail_pct DESC, MAX(ingested_at) DESC
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
                    "fail_runs": r.get::<i64, _>("fail_runs"),
                    "fail_pct": r.get::<f64, _>("fail_pct"),
                    "last_seen_ms": r.get::<i64, _>("last_seen_ms"),
                    "window_hours": safe_hours,
                })
            })
            .collect())
    }
}
