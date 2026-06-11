use super::*;

impl Database {
    /// Stores a raw inbound webhook and enforces content-bound idempotency.
    /// `payload_sha256` is a hash of the signed material (event type + raw body),
    /// so a replay with a fresh `delivery_id` but the same signed body collides
    /// here. A duplicate is only skipped when the prior occurrence was already
    /// processed successfully; a prior occurrence whose processing failed is
    /// returned for reprocessing so transient failures are not silently lost.
    pub async fn store_webhook_event(
        &self,
        delivery_id: &str,
        event_type: &str,
        signature: Option<&str>,
        payload: &serde_json::Value,
        payload_sha256: &str,
    ) -> Result<WebhookIngestDecision, DbError> {
        let inserted: Option<String> = sqlx::query_scalar(
            r#"
            INSERT INTO webhook_events (delivery_id, event_type, signature, payload, payload_sha256)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (payload_sha256) DO NOTHING
            RETURNING id::text
            "#,
        )
        .bind(delivery_id)
        .bind(event_type)
        .bind(signature)
        .bind(payload)
        .bind(payload_sha256)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        if let Some(id) = inserted {
            return Ok(WebhookIngestDecision::Process(Some(id)));
        }

        // Conflict: a row with this content hash already exists. Only treat it as
        // a duplicate to skip when it was processed successfully; otherwise allow
        // reprocessing of a previously-failed delivery (retry-safety).
        let existing = sqlx::query(
            r#"
            SELECT id::text AS id, COALESCE(processed, FALSE) AS processed
            FROM webhook_events
            WHERE payload_sha256 = $1
            "#,
        )
        .bind(payload_sha256)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        match existing {
            Some(row) if row.get::<bool, _>("processed") => {
                Ok(WebhookIngestDecision::SkipDuplicate)
            }
            Some(row) => Ok(WebhookIngestDecision::Process(Some(row.get("id")))),
            // The conflicting row vanished between INSERT and SELECT (no normal
            // delete path exists). Be conservative and process; the per-table
            // dedup (e.g. github_events.delivery_id) remains a backstop.
            None => Ok(WebhookIngestDecision::Process(None)),
        }
    }

    pub async fn mark_webhook_processed(
        &self,
        id: &str,
        error: Option<&str>,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"
            UPDATE webhook_events
            SET processed = TRUE, processed_at = NOW(), error = $2
            WHERE id = $1::uuid
            "#,
        )
        .bind(id)
        .bind(error)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // ========================================================================
    // API KEYS
    // ========================================================================

    pub async fn validate_api_key(&self, key_hash: &str) -> Result<ApiKeyAuthValidation, DbError> {
        if let Some(cached) = self.get_cached_api_key_auth(key_hash) {
            return Ok(ApiKeyAuthValidation {
                auth: cached,
                used_stale_cache: false,
            });
        }

        let simulated_auth_db_failure = auth_db_failure_simulation_enabled();
        if simulated_auth_db_failure {
            tracing::warn!(
                "Simulating auth DB query failure via debug failpoint (validate_api_key)"
            );
        }

        let result = if simulated_auth_db_failure {
            Err("Simulated auth DB failure (debug failpoint)".to_string())
        } else {
            sqlx::query(
                r#"
                SELECT client_id, role, org_id::text, last_used
                FROM api_keys
                WHERE key_hash = $1 AND is_active = TRUE
                "#,
            )
            .bind(key_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())
        };

        let result = match result {
            Ok(row) => row,
            Err(error_msg) => {
                let (failure_streak, should_fail_closed) = self.note_auth_db_failure();
                if should_fail_closed {
                    tracing::warn!(
                        error = %error_msg,
                        failure_streak,
                        fail_closed_threshold = self.auth_stale_fail_closed_after,
                        "Auth DB failure threshold reached; stale auth fallback disabled"
                    );
                    return Err(DbError::DatabaseError(error_msg));
                }
                if let Some((stale_auth, stale_age_secs)) =
                    self.get_stale_cached_api_key_auth(key_hash)
                {
                    tracing::warn!(
                        error = %error_msg,
                        client_id = %stale_auth.0,
                        failure_streak,
                        stale_age_secs,
                        "Using stale API key auth cache due transient database error"
                    );
                    return Ok(ApiKeyAuthValidation {
                        auth: Some(stale_auth),
                        used_stale_cache: true,
                    });
                }
                return Err(DbError::DatabaseError(error_msg));
            }
        };

        self.reset_auth_db_failure_streak();

        match result {
            Some(row) => {
                let role: String = row.get("role");
                let role = UserRole::from_str(&role);
                let client_id: String = row.get("client_id");
                let org_id: Option<String> = row.get("org_id");
                let auth_tuple = Some((client_id.clone(), role.clone(), org_id.clone()));

                // Reduce write amplification on high-traffic endpoints.
                // `last_used` is observability metadata, so update only every ~5 minutes.
                let last_used: Option<chrono::DateTime<chrono::Utc>> = row.get("last_used");
                let should_update_last_used = last_used
                    .map(|ts| chrono::Utc::now().signed_duration_since(ts).num_minutes() >= 5)
                    .unwrap_or(true);

                if should_update_last_used {
                    if let Err(e) = sqlx::query(
                        r#"
                        UPDATE api_keys
                        SET last_used = NOW()
                        WHERE key_hash = $1
                          AND is_active = TRUE
                          AND (last_used IS NULL OR last_used < NOW() - INTERVAL '5 minutes')
                        "#,
                    )
                    .bind(key_hash)
                    .execute(&self.pool)
                    .await
                    {
                        tracing::warn!(
                            error = %e,
                            client_id = %client_id,
                            "Failed to update api_keys.last_used (non-fatal)"
                        );
                    }
                }

                self.put_cached_api_key_auth(key_hash, auth_tuple.clone());
                Ok(ApiKeyAuthValidation {
                    auth: auth_tuple,
                    used_stale_cache: false,
                })
            }
            None => {
                // Cache negative lookups briefly to reduce repeated DB hits on invalid tokens.
                self.put_cached_api_key_auth(key_hash, None);
                Ok(ApiKeyAuthValidation {
                    auth: None,
                    used_stale_cache: false,
                })
            }
        }
    }

    pub async fn create_api_key(
        &self,
        key_hash: &str,
        client_id: &str,
        org_id: Option<&str>,
        role: &UserRole,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO api_keys (key_hash, client_id, org_id, role) VALUES ($1, $2, $3::uuid, $4)
            "#,
        )
        .bind(key_hash)
        .bind(client_id)
        .bind(org_id)
        .bind(role.as_str())
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        self.invalidate_auth_cache_key(key_hash);

        Ok(())
    }

    pub async fn ensure_admin_api_key(
        &self,
        key_hash: &str,
        client_id: &str,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO api_keys (key_hash, client_id, org_id, role, is_active)
            VALUES ($1, $2, NULL, $3, TRUE)
            ON CONFLICT (key_hash) DO UPDATE
            SET
                client_id = EXCLUDED.client_id,
                org_id = NULL,
                role = EXCLUDED.role,
                is_active = TRUE
            "#,
        )
        .bind(key_hash)
        .bind(client_id)
        .bind(UserRole::Admin.as_str())
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        self.invalidate_auth_cache_key(key_hash);

        Ok(())
    }

    pub async fn count_api_keys(&self) -> Result<i64, DbError> {
        let result = sqlx::query("SELECT COUNT(*) as count FROM api_keys WHERE is_active = TRUE")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let count: i64 = result.get("count");
        Ok(count)
    }

    pub async fn list_api_keys(&self, org_id: Option<&str>) -> Result<Vec<ApiKeyInfo>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                ak.id::text,
                ak.client_id,
                ak.role,
                ak.org_id::text,
                o.login AS org_name,
                EXTRACT(EPOCH FROM ak.created_at)::bigint * 1000 AS created_at_ms,
                EXTRACT(EPOCH FROM ak.last_used)::bigint * 1000 AS last_used_ms,
                ak.is_active
            FROM api_keys ak
            LEFT JOIN orgs o ON o.id = ak.org_id
            WHERE ($1::uuid IS NULL OR ak.org_id = $1::uuid)
            ORDER BY ak.created_at DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let keys = rows
            .iter()
            .map(|row| ApiKeyInfo {
                id: row.get("id"),
                client_id: row.get("client_id"),
                role: row.get("role"),
                org_id: row.get("org_id"),
                org_name: row.get("org_name"),
                created_at: row.get::<i64, _>("created_at_ms"),
                last_used: row.get("last_used_ms"),
                is_active: row.get("is_active"),
            })
            .collect();

        Ok(keys)
    }

    pub async fn revoke_api_key(&self, id: &str, org_id: Option<&str>) -> Result<bool, DbError> {
        let result = sqlx::query(
            r#"
            UPDATE api_keys
            SET is_active = FALSE
            WHERE
                id = $1::uuid
                AND is_active = TRUE
                AND ($2::uuid IS NULL OR org_id = $2::uuid)
            "#,
        )
        .bind(id)
        .bind(org_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let revoked = result.rows_affected() > 0;
        if revoked {
            // Revoke works by key id; safest is clearing auth cache to avoid stale entries.
            self.invalidate_auth_cache_all();
        }

        Ok(revoked)
    }

    // ========================================================================
    // PULL REQUEST MERGES
    // ========================================================================
}
