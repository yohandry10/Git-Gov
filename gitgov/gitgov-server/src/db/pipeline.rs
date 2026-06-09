use super::*;

impl Database {
    pub async fn insert_pipeline_event(&self, event: &PipelineEvent) -> Result<String, DbError> {
        let stages_json = serde_json::to_value(&event.stages)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;
        let artifacts_json = serde_json::to_value(&event.artifacts)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;

        let ingested_at =
            chrono::DateTime::from_timestamp_millis(event.ingested_at).ok_or_else(|| {
                DbError::SerializationError("Invalid ingested_at timestamp".to_string())
            })?;

        let result = sqlx::query(
            r#"
            INSERT INTO pipeline_events (
                id, org_id, pipeline_id, job_name, status, commit_sha, branch, repo_full_name,
                duration_ms, triggered_by, stages, artifacts, payload, ingested_at
            )
            VALUES (
                $1::uuid, $2::uuid, $3, $4, $5, $6, $7, $8,
                $9, $10, $11::jsonb, $12::jsonb, $13::jsonb, $14
            )
            ON CONFLICT (pipeline_id, job_name, (COALESCE(commit_sha, '')), ingested_at) DO NOTHING
            RETURNING id::text
            "#,
        )
        .bind(&event.id)
        .bind(&event.org_id)
        .bind(&event.pipeline_id)
        .bind(&event.job_name)
        .bind(event.status.as_str())
        .bind(&event.commit_sha)
        .bind(&event.branch)
        .bind(&event.repo_full_name)
        .bind(event.duration_ms)
        .bind(&event.triggered_by)
        .bind(&stages_json)
        .bind(&artifacts_json)
        .bind(&event.payload)
        .bind(ingested_at)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        match result {
            Some(row) => Ok(row.get("id")),
            None => Err(DbError::Duplicate(format!(
                "pipeline_id={}, job_name={}, commit_sha={:?}, ingested_at={}",
                event.pipeline_id, event.job_name, event.commit_sha, event.ingested_at
            ))),
        }
    }

    pub async fn get_jenkins_integration_status(
        &self,
    ) -> Result<JenkinsIntegrationStatusResponse, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                MAX(ingested_at) AS last_ingest_at,
                COUNT(*) FILTER (WHERE ingested_at >= NOW() - INTERVAL '24 hours')::bigint AS recent_events_24h
            FROM pipeline_events
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let last_ingest_at = row
            .get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_ingest_at")
            .map(|dt| dt.timestamp_millis());
        let recent_events_24h: i64 = row.get("recent_events_24h");

        Ok(JenkinsIntegrationStatusResponse {
            ok: true,
            last_ingest_at,
            recent_events_24h,
        })
    }

    pub async fn get_latest_sonar_run_for_commit(
        &self,
        repo_full_name: &str,
        commit_sha: &str,
    ) -> Result<Option<CommitPipelineRun>, DbError> {
        let repo_full_name = repo_full_name.trim();
        let commit_sha = commit_sha.trim();
        if repo_full_name.is_empty() || commit_sha.is_empty() {
            return Ok(None);
        }

        let row = sqlx::query(
            r#"
            SELECT
                pe.id::text AS pipeline_event_id,
                pe.pipeline_id,
                pe.job_name,
                pe.status AS pipeline_status,
                pe.duration_ms,
                pe.triggered_by,
                pe.ingested_at
            FROM pipeline_events pe
            WHERE pe.repo_full_name = $1
              AND pe.commit_sha IS NOT NULL
              AND (
                pe.commit_sha = $2
                OR pe.commit_sha LIKE $2 || '%'
                OR $2 LIKE pe.commit_sha || '%'
              )
              AND lower(pe.job_name) LIKE '%sonar%'
            ORDER BY pe.ingested_at DESC, pe.id DESC
            LIMIT 1
            "#,
        )
        .bind(repo_full_name)
        .bind(commit_sha)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| {
            let ingested_at = row
                .get::<chrono::DateTime<chrono::Utc>, _>("ingested_at")
                .timestamp_millis();
            CommitPipelineRun {
                pipeline_event_id: row.get("pipeline_event_id"),
                pipeline_id: row.get("pipeline_id"),
                job_name: row.get("job_name"),
                status: row.get("pipeline_status"),
                duration_ms: row.get("duration_ms"),
                triggered_by: row.get("triggered_by"),
                ingested_at,
            }
        }))
    }

    pub async fn get_pipeline_health_stats(
        &self,
        org_id: Option<&str>,
    ) -> Result<PipelineHealthStats, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE ingested_at >= NOW() - INTERVAL '7 days' AND ($1::uuid IS NULL OR org_id = $1::uuid))::bigint AS total_7d,
                COUNT(*) FILTER (WHERE ingested_at >= NOW() - INTERVAL '7 days' AND status = 'success' AND ($1::uuid IS NULL OR org_id = $1::uuid))::bigint AS success_7d,
                COUNT(*) FILTER (WHERE ingested_at >= NOW() - INTERVAL '7 days' AND status = 'failure' AND ($1::uuid IS NULL OR org_id = $1::uuid))::bigint AS failure_7d,
                COUNT(*) FILTER (WHERE ingested_at >= NOW() - INTERVAL '7 days' AND status = 'aborted' AND ($1::uuid IS NULL OR org_id = $1::uuid))::bigint AS aborted_7d,
                COUNT(*) FILTER (WHERE ingested_at >= NOW() - INTERVAL '7 days' AND status = 'unstable' AND ($1::uuid IS NULL OR org_id = $1::uuid))::bigint AS unstable_7d,
                COALESCE(AVG(duration_ms) FILTER (WHERE ingested_at >= NOW() - INTERVAL '7 days' AND duration_ms IS NOT NULL AND ($1::uuid IS NULL OR org_id = $1::uuid)), 0)::bigint AS avg_duration_ms_7d,
                COUNT(DISTINCT repo_full_name) FILTER (WHERE ingested_at >= NOW() - INTERVAL '7 days' AND status IN ('failure','unstable') AND repo_full_name IS NOT NULL AND ($1::uuid IS NULL OR org_id = $1::uuid))::bigint AS repos_with_failures_7d
            FROM pipeline_events
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await;

        match row {
            Ok(row) => Ok(PipelineHealthStats {
                total_7d: row.get("total_7d"),
                success_7d: row.get("success_7d"),
                failure_7d: row.get("failure_7d"),
                aborted_7d: row.get("aborted_7d"),
                unstable_7d: row.get("unstable_7d"),
                avg_duration_ms_7d: row.get("avg_duration_ms_7d"),
                repos_with_failures_7d: row.get("repos_with_failures_7d"),
            }),
            Err(e) => {
                // Keep compatibility if migration v5 was not applied yet.
                if e.to_string().contains("pipeline_events") {
                    Ok(PipelineHealthStats::default())
                } else {
                    Err(DbError::DatabaseError(e.to_string()))
                }
            }
        }
    }

    // ========================================================================
    // STATS
    // ========================================================================

    pub async fn get_stats(&self, org_id: Option<&str>) -> Result<AuditStats, DbError> {
        let result = sqlx::query("SELECT get_audit_stats($1::uuid) as stats")
            .bind(org_id)
            .fetch_one(&self.pool)
            .await;

        match result {
            Ok(row) => {
                let stats_json: Option<sqlx::types::Json<AuditStats>> = row.get("stats");
                Ok(stats_json.map(|j| j.0).unwrap_or_default())
            }
            Err(_) => {
                // Function might not exist or return null, return default stats
                Ok(AuditStats::default())
            }
        }
    }

    pub async fn get_desktop_pushes_today(&self, org_id: Option<&str>) -> Result<i64, DbError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)::BIGINT
            FROM client_events
            WHERE event_type = 'successful_push'
              AND ($1::uuid IS NULL OR org_id = $1::uuid)
              AND created_at >= DATE_TRUNC('day', NOW())
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(count)
    }

    pub async fn get_daily_activity(
        &self,
        org_id: Option<&str>,
        days: i64,
    ) -> Result<Vec<DailyActivityPoint>, DbError> {
        let rows = sqlx::query(
            r#"
            WITH series AS (
              SELECT generate_series(
                (date_trunc('day', NOW() AT TIME ZONE 'UTC') - (($1::int - 1) * INTERVAL '1 day')),
                date_trunc('day', NOW() AT TIME ZONE 'UTC'),
                INTERVAL '1 day'
              )::date AS day_utc
            )
            SELECT
              to_char(s.day_utc, 'YYYY-MM-DD') AS day,
              COALESCE(SUM(CASE WHEN ce.event_type = 'commit' THEN 1 ELSE 0 END), 0)::bigint AS commits,
              COALESCE(SUM(CASE WHEN ce.event_type = 'successful_push' THEN 1 ELSE 0 END), 0)::bigint AS pushes
            FROM series s
            LEFT JOIN client_events ce
              ON ce.created_at >= s.day_utc::timestamp
             AND ce.created_at < (s.day_utc::timestamp + INTERVAL '1 day')
             AND ($2::uuid IS NULL OR ce.org_id = $2::uuid)
            GROUP BY s.day_utc
            ORDER BY s.day_utc DESC
            "#,
        )
        .bind(days as i32)
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let points = rows
            .into_iter()
            .map(|row| DailyActivityPoint {
                day: row.get("day"),
                commits: row.get("commits"),
                pushes: row.get("pushes"),
            })
            .collect();

        Ok(points)
    }

    // ========================================================================
    // POLICIES
    // ========================================================================
}
