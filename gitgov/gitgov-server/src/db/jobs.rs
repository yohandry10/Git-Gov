use super::*;

impl Database {
    // ========================================================================
    // JOB QUEUE (Production-hardened with backpressure control)
    // ========================================================================
    // Features:
    // - Atomic claim with FOR UPDATE SKIP LOCKED (no race conditions)
    // - Dedupe: only 1 pending/running job per (org_id, job_type)
    // - Exponential backoff: 30s * 2^attempts, capped at 1 hour
    // - Dead-letter: jobs exceeding max_attempts marked as 'dead'
    // - Structured logging with job_id, org_id, duration_ms
    // - Safe stale reset with backoff scheduling

    /// Enqueue a job (idempotent - one pending/running job per org+type)
    /// Uses partial unique index to prevent duplicate jobs.
    /// FIX: On conflict, returns the existing job's id instead of a fake UUID.
    pub async fn enqueue_job(
        &self,
        org_id: &str,
        job_type: &str,
        payload: Option<serde_json::Value>,
    ) -> Result<String, DbError> {
        let job_id = uuid::Uuid::new_v4().to_string();
        let payload = payload.unwrap_or(serde_json::Value::Null);
        let start = std::time::Instant::now();

        let result = sqlx::query(
            r#"
            INSERT INTO jobs (id, org_id, job_type, status, payload, max_attempts, created_at)
            VALUES ($1::uuid, $2::uuid, $3, 'pending', $4, 10, NOW())
            ON CONFLICT (org_id, job_type) WHERE status IN ('pending', 'running') DO NOTHING
            RETURNING id::text
            "#,
        )
        .bind(&job_id)
        .bind(org_id)
        .bind(job_type)
        .bind(&payload)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        match result {
            Some(row) => {
                let returned_id: String = row.get("id");
                tracing::info!(
                    job_id = %returned_id,
                    org_id = %org_id,
                    job_type = %job_type,
                    duration_ms = start.elapsed().as_millis() as u64,
                    "Job enqueued"
                );
                Ok(returned_id)
            }
            None => {
                tracing::debug!(
                    org_id = %org_id,
                    job_type = %job_type,
                    "Job already pending/running, fetching existing id"
                );
                let existing = sqlx::query(
                    r#"
                    SELECT id::text FROM jobs
                    WHERE org_id = $1::uuid
                      AND job_type = $2
                      AND status IN ('pending', 'running')
                    LIMIT 1
                    "#,
                )
                .bind(org_id)
                .bind(job_type)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| DbError::DatabaseError(e.to_string()))?;

                match existing {
                    Some(row) => Ok(row.get("id")),
                    None => {
                        tracing::warn!(
                            org_id = %org_id,
                            job_type = %job_type,
                            "Job not found after conflict - returning new id"
                        );
                        Ok(job_id)
                    }
                }
            }
        }
    }

    /// Claim next pending job atomically.
    /// Uses FOR UPDATE SKIP LOCKED to prevent race conditions.
    /// Records start time for duration tracking.
    pub async fn claim_job(&self, worker_id: &str) -> Result<Option<Job>, DbError> {
        let start = std::time::Instant::now();

        let row = sqlx::query(
            r#"
            UPDATE jobs
            SET status = 'running',
                locked_at = NOW(),
                locked_by = $1,
                attempts = attempts + 1,
                started_at = NOW()
            WHERE id = (
                SELECT id FROM jobs
                WHERE status = 'pending' AND next_run_at <= NOW()
                ORDER BY priority DESC, created_at ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING id::text, org_id::text, job_type, status, priority, payload,
                      attempts, max_attempts, created_at, locked_at, started_at
            "#,
        )
        .bind(worker_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => {
                let job_id: String = r.get("id");
                let org_id: String = r.get("org_id");
                let job_type: String = r.get("job_type");
                let attempts: i32 = r.get("attempts");

                tracing::info!(
                    job_id = %job_id,
                    org_id = %org_id,
                    job_type = %job_type,
                    attempt = attempts,
                    worker_id = %worker_id,
                    claim_duration_ms = start.elapsed().as_millis() as u64,
                    "Job claimed"
                );

                let created_at: chrono::DateTime<chrono::Utc> = r.get("created_at");
                let locked_at: Option<chrono::DateTime<chrono::Utc>> = r.get("locked_at");
                let started_at: Option<chrono::DateTime<chrono::Utc>> = r.get("started_at");

                Ok(Some(Job {
                    id: job_id,
                    org_id,
                    job_type,
                    status: r.get("status"),
                    priority: r.get("priority"),
                    payload: r.get("payload"),
                    attempts,
                    max_attempts: r.get("max_attempts"),
                    created_at: created_at.timestamp_millis(),
                    locked_at: locked_at.map(|t| t.timestamp_millis()),
                    locked_by: Some(worker_id.to_string()),
                    started_at: started_at.map(|t| t.timestamp_millis()),
                    duration_ms: None,
                }))
            }
            None => Ok(None),
        }
    }

    /// Complete a job successfully.
    /// Records duration for metrics.
    pub async fn complete_job(&self, job_id: &str) -> Result<(), DbError> {
        let start = std::time::Instant::now();

        let result = sqlx::query(
            r#"
            UPDATE jobs
            SET status = 'completed',
                completed_at = NOW(),
                duration_ms = (EXTRACT(EPOCH FROM (NOW() - started_at)) * 1000)::BIGINT
            WHERE id = $1::uuid
            "#,
        )
        .bind(job_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("Job not found: {}", job_id)));
        }

        tracing::info!(
            job_id = %job_id,
            total_duration_ms = start.elapsed().as_millis() as u64,
            "Job completed"
        );
        Ok(())
    }

    /// Fail a job with exponential backoff retry scheduling.
    /// If attempts >= max_attempts, marks as 'dead' (dead-letter queue).
    /// Backoff: 30s * 2^attempts, capped at 1 hour.
    pub async fn fail_job(&self, job_id: &str, error: &str) -> Result<(), DbError> {
        let start = std::time::Instant::now();

        // Calculate backoff using PostgreSQL function
        let row = sqlx::query(
            r#"
            UPDATE jobs
            SET status = CASE
                WHEN attempts >= max_attempts THEN 'dead'
                ELSE 'pending'
            END,
            last_error = $1,
            next_run_at = CASE
                WHEN attempts < max_attempts THEN NOW() + (job_backoff_seconds(attempts) || ' seconds')::INTERVAL
                ELSE NULL
            END,
            locked_at = NULL,
            locked_by = NULL,
            duration_ms = (EXTRACT(EPOCH FROM (NOW() - started_at)) * 1000)::BIGINT
            WHERE id = $2::uuid
            RETURNING status, attempts, max_attempts
            "#,
        )
        .bind(error)
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => {
                let status: String = r.get("status");
                let attempts: i32 = r.get("attempts");
                let max_attempts: i32 = r.get("max_attempts");

                if status == "dead" {
                    tracing::warn!(
                        job_id = %job_id,
                        attempts = attempts,
                        max_attempts = max_attempts,
                        error = %error,
                        "Job dead (exceeded max attempts) - moved to dead-letter"
                    );
                } else {
                    let backoff_secs = Self::calculate_backoff(attempts);
                    tracing::warn!(
                        job_id = %job_id,
                        attempt = attempts,
                        max_attempts = max_attempts,
                        backoff_secs = backoff_secs,
                        error = %error,
                        "Job failed, scheduled retry"
                    );
                }
            }
            None => {
                tracing::error!(job_id = %job_id, "Job not found for failure update");
                return Err(DbError::NotFound(format!("Job not found: {}", job_id)));
            }
        }

        let _ = start;
        Ok(())
    }

    /// Calculate exponential backoff in seconds.
    /// Formula: 30 * 2^attempts, capped at 3600 (1 hour).
    fn calculate_backoff(attempts: i32) -> u64 {
        let base: u64 = 30;
        let max: u64 = 3600;
        let backoff = base.saturating_mul(1u64 << attempts.min(7));
        backoff.min(max)
    }

    /// Safely reset stale jobs (locked > TTL minutes).
    /// Uses FOR UPDATE SKIP LOCKED to prevent race conditions.
    /// FIX: Uses attempts+1 for backoff, marks dead if max_attempts exceeded.
    pub async fn reset_stale_jobs(&self) -> Result<i64, DbError> {
        let result = sqlx::query(
            r#"
            WITH stale_jobs AS (
                SELECT id, attempts, max_attempts FROM jobs
                WHERE status = 'running'
                  AND locked_at < NOW() - INTERVAL '5 minutes'
                FOR UPDATE SKIP LOCKED
            )
            UPDATE jobs
            SET status = CASE
                WHEN (attempts + 1) >= max_attempts THEN 'dead'
                ELSE 'pending'
            END,
            locked_at = NULL,
            locked_by = NULL,
            started_at = NULL,
            attempts = attempts + 1,
            last_error = CASE
                WHEN (attempts + 1) >= max_attempts THEN 'Job exceeded max_attempts after timeout'
                ELSE 'Job timed out after 5 minutes'
            END,
            next_run_at = CASE
                WHEN (attempts + 1) < max_attempts THEN NOW() + (job_backoff_seconds(attempts + 1) || ' seconds')::INTERVAL
                ELSE NULL
            END,
            completed_at = CASE
                WHEN (attempts + 1) >= max_attempts THEN NOW()
                ELSE completed_at
            END
            WHERE id IN (SELECT id FROM stale_jobs)
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let count = result.rows_affected() as i64;
        if count > 0 {
            tracing::warn!(
                stale_count = count,
                ttl_minutes = 5,
                "Reset stale jobs with backoff scheduling (dead-letter aware)"
            );
        }
        Ok(count)
    }

    /// Reset stale jobs using the SQL function (single source of truth).
    /// This calls reset_stale_jobs_safe() defined in supabase_schema_v2.sql.
    pub async fn reset_stale_jobs_safe(&self) -> Result<i64, DbError> {
        let result = sqlx::query("SELECT reset_stale_jobs_safe(5) as count")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let count: i64 = result.get("count");
        if count > 0 {
            tracing::warn!(
                stale_count = count,
                ttl_minutes = 5,
                "Reset stale jobs via SQL function"
            );
        }
        Ok(count)
    }

    /// Get job queue metrics for observability.
    pub async fn get_job_metrics(&self) -> Result<JobMetrics, DbError> {
        let row = sqlx::query("SELECT get_job_metrics() as metrics")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let metrics: sqlx::types::Json<JobMetrics> = row.get("metrics");
        Ok(metrics.0)
    }

    /// Execute detect_noncompliance_signals via job.
    /// This is idempotent - uses ingested_at cursor.
    pub async fn execute_detect_signals(&self, org_id: &str) -> Result<i64, DbError> {
        let start = std::time::Instant::now();
        let count = self.detect_noncompliance_signals(org_id).await?;

        tracing::info!(
            org_id = %org_id,
            signals_created = count,
            duration_ms = start.elapsed().as_millis() as u64,
            "Signal detection completed"
        );

        Ok(count)
    }

    /// Get dead-letter jobs for inspection.
    pub async fn get_dead_jobs(&self, limit: i64) -> Result<Vec<Job>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT id::text, org_id::text, job_type, status, priority, payload,
                   attempts, max_attempts, last_error, created_at, locked_at
            FROM jobs
            WHERE status = 'dead'
            ORDER BY created_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let jobs: Vec<Job> = rows
            .iter()
            .map(|r| {
                let created_at: chrono::DateTime<chrono::Utc> = r.get("created_at");
                Job {
                    id: r.get("id"),
                    org_id: r.get("org_id"),
                    job_type: r.get("job_type"),
                    status: r.get("status"),
                    priority: r.get("priority"),
                    payload: r.get("payload"),
                    attempts: r.get("attempts"),
                    max_attempts: r.get("max_attempts"),
                    created_at: created_at.timestamp_millis(),
                    locked_at: None,
                    locked_by: None,
                    started_at: None,
                    duration_ms: None,
                }
            })
            .collect();

        Ok(jobs)
    }

    /// Retry a dead job (manual intervention).
    pub async fn retry_dead_job(&self, job_id: &str) -> Result<(), DbError> {
        let result = sqlx::query(
            r#"
            UPDATE jobs
            SET status = 'pending',
                attempts = 0,
                next_run_at = NOW(),
                last_error = NULL
            WHERE id = $1::uuid AND status = 'dead'
            "#,
        )
        .bind(job_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("Dead job not found: {}", job_id)));
        }

        tracing::info!(job_id = %job_id, "Dead job queued for retry");
        Ok(())
    }
}
