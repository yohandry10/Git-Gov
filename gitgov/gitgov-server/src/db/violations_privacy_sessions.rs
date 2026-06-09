use super::*;

impl Database {
    // ========================================================================
    // VIOLATION DECISIONS (v3 schema)
    // ========================================================================

    /// Add a decision to a violation (append-only audit trail).
    /// Decision types: acknowledged, false_positive, resolved, escalated, dismissed, wont_fix
    pub async fn add_violation_decision(
        &self,
        violation_id: &str,
        decision_type: &str,
        decided_by: &str,
        notes: Option<&str>,
        evidence: Option<serde_json::Value>,
    ) -> Result<String, DbError> {
        let function_call = sqlx::query(
            r#"
            SELECT add_violation_decision(
                $1::uuid,
                $2,
                $3,
                $4,
                $5
            ) as decision_id
            "#,
        )
        .bind(violation_id)
        .bind(decision_type)
        .bind(decided_by)
        .bind(notes)
        .bind(evidence.clone().unwrap_or(serde_json::Value::Null))
        .fetch_one(&self.pool)
        .await;

        let decision_id: String = match function_call {
            Ok(result) => result.get("decision_id"),
            Err(function_err) => {
                tracing::warn!(
                    violation_id = %violation_id,
                    decision_type = %decision_type,
                    error = %function_err,
                    "Falling back to direct violation_decisions insert"
                );

                let insert_result = sqlx::query(
                    r#"
                    INSERT INTO violation_decisions (
                        violation_id, decision_type, decided_by, notes, evidence
                    ) VALUES (
                        $1::uuid, $2, $3, $4, $5
                    )
                    RETURNING id::text as decision_id
                    "#,
                )
                .bind(violation_id)
                .bind(decision_type)
                .bind(decided_by)
                .bind(notes)
                .bind(evidence.unwrap_or(serde_json::Value::Null))
                .fetch_one(&self.pool)
                .await;

                match insert_result {
                    Ok(row) => row.get("decision_id"),
                    Err(insert_err) => {
                        let err_msg = insert_err.to_string();
                        if err_msg.contains("duplicate key value")
                            || err_msg.contains("violations_once_per_type")
                            || err_msg.contains("violation_decisions_once_per_type")
                        {
                            let existing_id: Option<String> = sqlx::query_scalar(
                                r#"
                                SELECT id::text
                                FROM violation_decisions
                                WHERE violation_id = $1::uuid
                                  AND decision_type = $2
                                ORDER BY decided_at DESC, created_at DESC
                                LIMIT 1
                                "#,
                            )
                            .bind(violation_id)
                            .bind(decision_type)
                            .fetch_optional(&self.pool)
                            .await
                            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

                            existing_id.ok_or_else(|| {
                                DbError::DatabaseError(
                                    "duplicate violation decision detected but existing row not found"
                                        .to_string(),
                                )
                            })?
                        } else {
                            return Err(DbError::DatabaseError(err_msg));
                        }
                    }
                }
            }
        };

        tracing::info!(
            violation_id = %violation_id,
            decision_type = %decision_type,
            decided_by = %decided_by,
            "Violation decision recorded"
        );

        Ok(decision_id)
    }

    /// Get decision history for a violation.
    pub async fn get_violation_decisions(
        &self,
        violation_id: &str,
    ) -> Result<Vec<ViolationDecision>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT id::text, violation_id::text, decision_type, decided_by,
                   decided_at, notes, evidence, created_at
            FROM violation_decisions
            WHERE violation_id = $1::uuid
            ORDER BY decided_at DESC
            "#,
        )
        .bind(violation_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let decisions: Vec<ViolationDecision> = rows
            .iter()
            .map(|r| {
                let decided_at: chrono::DateTime<chrono::Utc> = r.get("decided_at");
                let created_at: chrono::DateTime<chrono::Utc> = r.get("created_at");
                ViolationDecision {
                    id: r.get("id"),
                    violation_id: r.get("violation_id"),
                    decision_type: r.get("decision_type"),
                    decided_by: r.get("decided_by"),
                    decided_at: decided_at.timestamp_millis(),
                    notes: r.get("notes"),
                    evidence: r.get("evidence"),
                    created_at: created_at.timestamp_millis(),
                }
            })
            .collect();

        Ok(decisions)
    }

    pub async fn get_violation_scope(
        &self,
        violation_id: &str,
    ) -> Result<Option<(Option<String>, Option<String>)>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT org_id::text, user_login
            FROM violations
            WHERE id = $1::uuid
            "#,
        )
        .bind(violation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| (r.get("org_id"), r.get("user_login"))))
    }

    // ========================================================================
    // GDPR — T2 (art. 17 erasure, art. 20 export, TTL cleanup)
    // ========================================================================

    /// Register GDPR erasure request for a user.
    /// Audit tables are append-only, so this records intent and returns scoped counts only.
    /// Returns (client_events_matched, github_events_matched).
    pub async fn erase_user_data(
        &self,
        user_login: &str,
        org_id: Option<&str>,
    ) -> Result<(i64, i64), DbError> {
        let client_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)::bigint
            FROM client_events
            WHERE user_login = $1
              AND ($2::uuid IS NULL OR org_id = $2::uuid)
            "#,
        )
        .bind(user_login)
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let github_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)::bigint
            FROM github_events
            WHERE actor_login = $1
              AND ($2::uuid IS NULL OR org_id = $2::uuid)
            "#,
        )
        .bind(user_login)
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        // Only record erasure intent if there is data in the visible scope.
        if client_count > 0 || github_count > 0 {
            sqlx::query(
                r#"
                INSERT INTO user_pseudonyms (user_login, erased_at)
                VALUES ($1, NOW())
                ON CONFLICT (user_login) DO UPDATE SET erased_at = NOW()
                "#,
            )
            .bind(user_login)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;
        }

        Ok((client_count, github_count))
    }

    /// Export all events for a user (GDPR art. 20 data portability).
    pub async fn export_user_data(
        &self,
        user_login: &str,
        org_id: Option<&str>,
    ) -> Result<Vec<CombinedEvent>, DbError> {
        let filter = EventFilter {
            user_login: Some(user_login.to_string()),
            org_id: org_id.map(str::to_string),
            limit: 50_000,
            ..Default::default()
        };
        self.get_combined_events(&filter).await
    }

    /// Delete client session rows older than `retention_days` days.
    /// Audit events remain append-only by design.
    /// Returns number of rows deleted.
    pub async fn delete_old_events(&self, retention_days: i64) -> Result<i64, DbError> {
        let result = sqlx::query(
            r#"
            DELETE FROM client_sessions
            WHERE last_seen_at < NOW() - ($1::bigint * INTERVAL '1 day')
            "#,
        )
        .bind(retention_days)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() as i64)
    }

    // ========================================================================
    // CLIENT SESSIONS — T3.A (heartbeat / last_seen)
    // ========================================================================

    /// Upsert client session — called on every inbound event + heartbeat.
    pub async fn upsert_client_session(
        &self,
        client_id: &str,
        org_id: Option<&str>,
        device_metadata: &serde_json::Value,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO client_sessions (client_id, org_id, last_seen_at, device_metadata)
            VALUES ($1, $2::uuid, NOW(), $3::jsonb)
            ON CONFLICT (client_id) DO UPDATE SET
                last_seen_at    = NOW(),
                device_metadata = EXCLUDED.device_metadata,
                org_id          = COALESCE(EXCLUDED.org_id, client_sessions.org_id)
            "#,
        )
        .bind(client_id)
        .bind(org_id)
        .bind(device_metadata.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// List client sessions (for GET /clients), scoped by org.
    pub async fn get_client_sessions(
        &self,
        org_id: Option<&str>,
    ) -> Result<Vec<ClientSession>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                client_id,
                org_id::text,
                EXTRACT(EPOCH FROM last_seen_at)::bigint * 1000 AS last_seen_ms,
                COALESCE(device_metadata, '{}')::text            AS device_metadata,
                EXTRACT(EPOCH FROM created_at)::bigint  * 1000  AS created_at_ms
            FROM client_sessions
            WHERE ($1::uuid IS NULL OR org_id = $1::uuid)
            ORDER BY last_seen_at DESC
            LIMIT 500
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let now_ms = chrono::Utc::now().timestamp_millis();
        let sessions = rows
            .iter()
            .map(|r| {
                let last_seen_ms: i64 = r.get("last_seen_ms");
                ClientSession {
                    client_id: r.get("client_id"),
                    org_id: r.get("org_id"),
                    last_seen_at: last_seen_ms,
                    device_metadata: serde_json::from_str(r.get::<&str, _>("device_metadata"))
                        .unwrap_or_default(),
                    created_at: r.get("created_at_ms"),
                    is_active: last_seen_ms > (now_ms - 86_400_000), // active = seen in last 24h
                }
            })
            .collect();

        Ok(sessions)
    }

    // ========================================================================
    // IDENTITY ALIASES — T3.B
    // ========================================================================

    /// Map alias_login → canonical_login (idempotent on alias conflict).
    /// Returns true if newly created, false if alias already mapped.
    pub async fn create_identity_alias(
        &self,
        canonical: &str,
        alias: &str,
        org_id: Option<&str>,
    ) -> Result<bool, DbError> {
        let result = sqlx::query(
            r#"
            INSERT INTO identity_aliases (canonical_login, alias_login, org_id)
            VALUES ($1, $2, $3::uuid)
            ON CONFLICT (alias_login) DO NOTHING
            "#,
        )
        .bind(canonical)
        .bind(alias)
        .bind(org_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// List identity aliases, optionally scoped by org.
    pub async fn list_identity_aliases(
        &self,
        org_id: Option<&str>,
    ) -> Result<Vec<IdentityAlias>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT canonical_login, alias_login, org_id::text,
                   EXTRACT(EPOCH FROM created_at)::bigint * 1000 AS created_at_ms
            FROM identity_aliases
            WHERE ($1::uuid IS NULL OR org_id = $1::uuid)
            ORDER BY canonical_login, alias_login
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|r| IdentityAlias {
                canonical_login: r.get("canonical_login"),
                alias_login: r.get("alias_login"),
                org_id: r.get("org_id"),
                created_at: r.get("created_at_ms"),
            })
            .collect())
    }

    // ========================================================================
    // ORG USERS — V1.4-A
    // ========================================================================
}
