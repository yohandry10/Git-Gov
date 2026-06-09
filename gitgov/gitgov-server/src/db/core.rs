use super::*;

impl Database {
    /// Create a Database from an existing PgPool (used by integration tests).
    #[cfg(test)]
    pub fn from_pool(pool: PgPool) -> Self {
        Self {
            pool,
            auth_cache: Arc::new(Mutex::new(HashMap::new())),
            auth_cache_ttl: Duration::from_secs(20),
            auth_cache_stale_max: Duration::from_secs(120),
            auth_cache_max_entries: 4096,
            auth_db_failure_streak: Arc::new(AtomicU32::new(0)),
            auth_stale_fail_closed_after: 0,
        }
    }

    pub async fn new(database_url: &str) -> Result<Self, DbError> {
        let runtime_env = std::env::var("GITGOV_ENV")
            .unwrap_or_else(|_| "dev".to_string())
            .trim()
            .to_ascii_lowercase();
        let is_dev_env = matches!(
            runtime_env.as_str(),
            "dev" | "development" | "local" | "test"
        );
        let max_connections = std::env::var("GITGOV_DB_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(20)
            .max(1);
        let min_connections = std::env::var("GITGOV_DB_MIN_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(2)
            .min(max_connections);
        let acquire_timeout_secs = std::env::var("GITGOV_DB_ACQUIRE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(8)
            .max(1);
        let idle_timeout_secs = std::env::var("GITGOV_DB_IDLE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(300)
            .max(10);
        let max_lifetime_secs = std::env::var("GITGOV_DB_MAX_LIFETIME_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(1800)
            .max(60);
        let auth_cache_ttl_secs = std::env::var("GITGOV_AUTH_CACHE_TTL_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(20)
            .clamp(1, 300);
        let auth_cache_max_entries = std::env::var("GITGOV_AUTH_CACHE_MAX_ENTRIES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(4096)
            .max(64);
        let auth_cache_stale_max_secs = std::env::var("GITGOV_AUTH_CACHE_STALE_MAX_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(if is_dev_env { 120 } else { 30 })
            .clamp(auth_cache_ttl_secs, 900);
        let auth_stale_fail_closed_after =
            std::env::var("GITGOV_AUTH_STALE_FAIL_CLOSED_AFTER_DB_ERRORS")
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(if is_dev_env { 0 } else { 3 })
                .min(10_000);

        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(min_connections)
            .acquire_timeout(Duration::from_secs(acquire_timeout_secs))
            .idle_timeout(Some(Duration::from_secs(idle_timeout_secs)))
            .max_lifetime(Some(Duration::from_secs(max_lifetime_secs)))
            .connect(database_url)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(Self {
            pool,
            auth_cache: Arc::new(Mutex::new(HashMap::new())),
            auth_cache_ttl: Duration::from_secs(auth_cache_ttl_secs),
            auth_cache_stale_max: Duration::from_secs(auth_cache_stale_max_secs),
            auth_cache_max_entries,
            auth_db_failure_streak: Arc::new(AtomicU32::new(0)),
            auth_stale_fail_closed_after,
        })
    }

    fn get_cached_api_key_auth_with_max_age(
        &self,
        key_hash: &str,
        max_age: Duration,
    ) -> Option<Option<ApiKeyAuthCacheValue>> {
        let cache = self.auth_cache.lock().ok()?;
        let entry = cache.get(key_hash).cloned()?;
        if entry.cached_at.elapsed() <= max_age {
            return Some(entry.value);
        }
        None
    }

    pub(super) fn get_cached_api_key_auth(
        &self,
        key_hash: &str,
    ) -> Option<Option<ApiKeyAuthCacheValue>> {
        self.get_cached_api_key_auth_with_max_age(key_hash, self.auth_cache_ttl)
    }

    pub(super) fn get_stale_cached_api_key_auth(
        &self,
        key_hash: &str,
    ) -> Option<StaleApiKeyAuthCacheValue> {
        let mut cache = self.auth_cache.lock().ok()?;
        let entry = cache.get(key_hash).cloned()?;
        let age = entry.cached_at.elapsed();
        if age <= self.auth_cache_stale_max {
            return entry.value.map(|auth| (auth, age.as_secs()));
        }
        cache.remove(key_hash);
        None
    }

    pub(super) fn put_cached_api_key_auth(
        &self,
        key_hash: &str,
        value: Option<ApiKeyAuthCacheValue>,
    ) {
        if let Ok(mut cache) = self.auth_cache.lock() {
            if cache.len() >= self.auth_cache_max_entries && !cache.contains_key(key_hash) {
                if let Some(stale_key) = cache.iter().find_map(|(k, v)| {
                    (v.cached_at.elapsed() > self.auth_cache_ttl).then(|| k.clone())
                }) {
                    cache.remove(&stale_key);
                } else if let Some(first_key) = cache.keys().next().cloned() {
                    cache.remove(&first_key);
                }
            }

            cache.insert(
                key_hash.to_string(),
                CachedApiKeyAuth {
                    value,
                    cached_at: Instant::now(),
                },
            );
        }
    }

    pub(super) fn invalidate_auth_cache_key(&self, key_hash: &str) {
        if let Ok(mut cache) = self.auth_cache.lock() {
            cache.remove(key_hash);
        }
    }

    pub(super) fn invalidate_auth_cache_all(&self) {
        if let Ok(mut cache) = self.auth_cache.lock() {
            cache.clear();
        }
    }

    pub(super) fn note_auth_db_failure(&self) -> (u32, bool) {
        let streak = self
            .auth_db_failure_streak
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);
        let should_fail_closed =
            self.auth_stale_fail_closed_after > 0 && streak >= self.auth_stale_fail_closed_after;
        (streak, should_fail_closed)
    }

    pub(super) fn reset_auth_db_failure_streak(&self) {
        self.auth_db_failure_streak.store(0, Ordering::Relaxed);
    }

    pub async fn ensure_rate_limit_storage(&self) -> Result<(), DbError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS rate_limit_counters (
                limiter_name TEXT NOT NULL,
                scope_key TEXT NOT NULL,
                window_start TIMESTAMPTZ NOT NULL,
                count INTEGER NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (limiter_name, scope_key, window_start)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_rate_limit_counters_updated_at
            ON rate_limit_counters (updated_at)
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn ensure_outbox_lease_storage(&self) -> Result<(), DbError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS outbox_flush_leases (
                lease_key TEXT PRIMARY KEY,
                holder TEXT NOT NULL,
                lease_until TIMESTAMPTZ NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_outbox_flush_leases_updated_at
            ON outbox_flush_leases (updated_at)
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn try_acquire_outbox_flush_lease(
        &self,
        lease_key: &str,
        holder: &str,
        lease_ttl: Duration,
    ) -> Result<OutboxLeaseDecision, DbError> {
        let ttl_ms = lease_ttl.as_millis().clamp(1, i64::MAX as u128) as i64;
        let row = sqlx::query(
            r#"
            INSERT INTO outbox_flush_leases (lease_key, holder, lease_until, updated_at)
            VALUES (
                $1::text,
                $2::text,
                NOW() + ($3::bigint * INTERVAL '1 millisecond'),
                NOW()
            )
            ON CONFLICT (lease_key) DO UPDATE
            SET
                holder = CASE
                    WHEN outbox_flush_leases.lease_until <= NOW()
                        OR outbox_flush_leases.holder = EXCLUDED.holder
                    THEN EXCLUDED.holder
                    ELSE outbox_flush_leases.holder
                END,
                lease_until = CASE
                    WHEN outbox_flush_leases.lease_until <= NOW()
                        OR outbox_flush_leases.holder = EXCLUDED.holder
                    THEN EXCLUDED.lease_until
                    ELSE outbox_flush_leases.lease_until
                END,
                updated_at = NOW()
            RETURNING holder, lease_until, NOW() AS now_ts
            "#,
        )
        .bind(lease_key)
        .bind(holder)
        .bind(ttl_ms)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let granted_holder: String = row
            .try_get("holder")
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;
        let lease_until: chrono::DateTime<chrono::Utc> = row
            .try_get("lease_until")
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;
        let now_ts: chrono::DateTime<chrono::Utc> = row
            .try_get("now_ts")
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let granted = granted_holder == holder;
        let wait_ms = if granted {
            0
        } else {
            lease_until
                .signed_duration_since(now_ts)
                .num_milliseconds()
                .max(1) as u64
        };

        Ok(OutboxLeaseDecision { granted, wait_ms })
    }

    pub async fn prune_rate_limit_counters(&self, retention: Duration) -> Result<u64, DbError> {
        if retention.is_zero() {
            return Ok(0);
        }
        let retention_secs = retention.as_secs().min(i64::MAX as u64) as i64;
        let result = sqlx::query(
            r#"
            DELETE FROM rate_limit_counters
            WHERE updated_at < NOW() - make_interval(secs => $1)
            "#,
        )
        .bind(retention_secs)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;
        Ok(result.rows_affected())
    }

    pub async fn publish_sse_notification(
        &self,
        channel: &str,
        payload: &str,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"
            SELECT pg_notify($1::text, $2::text)
            "#,
        )
        .bind(channel)
        .bind(payload)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn check_distributed_rate_limit(
        &self,
        limiter_name: &str,
        scope_key: &str,
        limit: u32,
        window: Duration,
    ) -> Result<DistributedRateLimitCheck, DbError> {
        if limit == 0 {
            return Ok(DistributedRateLimitCheck {
                allowed: true,
                retry_after_secs: 0,
            });
        }

        let window_secs = window.as_secs().max(1).min(i64::MAX as u64) as i64;
        let limit_i64 = limit as i64;

        let row = sqlx::query(
            r#"
            WITH params AS (
                SELECT
                    $1::text AS limiter_name,
                    $2::text AS scope_key,
                    $3::bigint AS limit_count,
                    $4::bigint AS window_secs,
                    NOW() AS now_ts
            ),
            bucket AS (
                SELECT
                    limiter_name,
                    scope_key,
                    limit_count,
                    window_secs,
                    now_ts,
                    to_timestamp(floor(extract(epoch FROM now_ts) / window_secs) * window_secs) AS window_start
                FROM params
            ),
            upsert AS (
                INSERT INTO rate_limit_counters (limiter_name, scope_key, window_start, count, updated_at)
                SELECT limiter_name, scope_key, window_start, 1, now_ts
                FROM bucket
                ON CONFLICT (limiter_name, scope_key, window_start)
                DO UPDATE
                SET count = rate_limit_counters.count + 1,
                    updated_at = EXCLUDED.updated_at
                RETURNING count, window_start
            )
            SELECT
                upsert.count::bigint AS current_count,
                upsert.window_start,
                bucket.now_ts,
                bucket.window_secs,
                bucket.limit_count
            FROM upsert
            CROSS JOIN bucket
            "#,
        )
        .bind(limiter_name)
        .bind(scope_key)
        .bind(limit_i64)
        .bind(window_secs)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let current_count: i64 = row
            .try_get("current_count")
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;
        let window_start: chrono::DateTime<chrono::Utc> = row
            .try_get("window_start")
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;
        let now_ts: chrono::DateTime<chrono::Utc> = row
            .try_get("now_ts")
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;
        let window_secs_row: i64 = row
            .try_get("window_secs")
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;
        let limit_count: i64 = row
            .try_get("limit_count")
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        if current_count <= limit_count {
            return Ok(DistributedRateLimitCheck {
                allowed: true,
                retry_after_secs: 0,
            });
        }

        let elapsed_secs = now_ts
            .signed_duration_since(window_start)
            .num_seconds()
            .max(0);
        let retry_after_secs = (window_secs_row - elapsed_secs).max(1) as u64;

        Ok(DistributedRateLimitCheck {
            allowed: false,
            retry_after_secs,
        })
    }

    // ========================================================================
    // ORGANIZATIONS
    // ========================================================================
}
