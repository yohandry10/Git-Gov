use super::*;

impl Database {
    pub async fn save_policy(
        &self,
        repo_id: &str,
        config: &GitGovConfig,
        checksum: &str,
        override_actor: &str,
    ) -> Result<(), DbError> {
        let config_json =
            serde_json::to_value(config).map_err(|e| DbError::SerializationError(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO policies (repo_id, config, checksum, override_actor, updated_at)
            VALUES ($1::uuid, $2, $3, $4, NOW())
            ON CONFLICT (repo_id) DO UPDATE SET
                config = $2,
                checksum = $3,
                override_actor = $4,
                updated_at = NOW()
            "#,
        )
        .bind(repo_id)
        .bind(&config_json)
        .bind(checksum)
        .bind(override_actor)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_policy(&self, repo_id: &str) -> Result<Option<PolicyResponse>, DbError> {
        let result = sqlx::query(
            r#"
            SELECT config, checksum, updated_at
            FROM policies
            WHERE repo_id = $1::uuid
            "#,
        )
        .bind(repo_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        match result {
            Some(row) => {
                let config: serde_json::Value = row.get("config");
                let config: GitGovConfig = serde_json::from_value(config.clone())
                    .map_err(|e| DbError::SerializationError(e.to_string()))?;
                let checksum: String = row.get("checksum");
                let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

                Ok(Some(PolicyResponse {
                    version: "1.0".to_string(),
                    checksum,
                    config,
                    updated_at: updated_at.timestamp_millis(),
                }))
            }
            None => Ok(None),
        }
    }

    // ========================================================================
    // WEBHOOK EVENTS (raw storage for debugging)
    // ========================================================================

    pub async fn get_policy_history(&self, repo_id: &str) -> Result<Vec<PolicyHistory>, DbError> {
        let rows = sqlx::query("SELECT * FROM get_policy_history($1::uuid)")
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let history: Vec<PolicyHistory> = rows
            .iter()
            .map(|row| {
                let config: serde_json::Value = row.get("config");
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

                PolicyHistory {
                    id: row.get("id"),
                    repo_id: repo_id.to_string(),
                    config: serde_json::from_value(config).unwrap_or_default(),
                    checksum: row.get("checksum"),
                    changed_by: row.get("changed_by"),
                    change_type: row.get("change_type"),
                    previous_checksum: row.get("previous_checksum"),
                    created_at: created_at.timestamp_millis(),
                }
            })
            .collect();

        Ok(history)
    }

    pub async fn create_policy_change_request(
        &self,
        input: CreatePolicyChangeRequestInput<'_>,
    ) -> Result<(), DbError> {
        let requested_config_json = serde_json::to_value(input.requested_config)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO policy_change_requests (
                id, org_id, repo_id, repo_name, requested_by,
                requested_config, requested_checksum, reason, created_at
            )
            VALUES (
                $1::uuid, $2::uuid, $3::uuid, $4, $5,
                $6::jsonb, $7, $8, to_timestamp($9::bigint / 1000.0)
            )
            "#,
        )
        .bind(input.request_id)
        .bind(input.org_id)
        .bind(input.repo_id)
        .bind(input.repo_name)
        .bind(input.requested_by)
        .bind(&requested_config_json)
        .bind(input.requested_checksum)
        .bind(input.reason)
        .bind(input.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn list_policy_change_requests(
        &self,
        input: ListPolicyChangeRequestsInput<'_>,
    ) -> Result<(Vec<PolicyChangeRequestRecord>, i64), DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                r.id::text AS id,
                r.org_id::text AS org_id,
                r.repo_id::text AS repo_id,
                r.repo_name AS repo_name,
                r.requested_by AS requested_by,
                r.requested_checksum AS requested_checksum,
                CASE
                  WHEN $7::boolean THEN r.requested_config
                  ELSE '{}'::jsonb
                END AS requested_config,
                r.reason AS reason,
                COALESCE(d.decision, 'pending') AS status,
                d.decided_by AS decided_by,
                d.note AS decision_note,
                EXTRACT(EPOCH FROM r.created_at)::bigint * 1000 AS created_at_ms,
                EXTRACT(EPOCH FROM d.created_at)::bigint * 1000 AS decided_at_ms
            FROM policy_change_requests r
            LEFT JOIN policy_change_request_decisions d
              ON d.request_id = r.id
            WHERE ($1::uuid IS NULL OR r.org_id = $1::uuid)
              AND ($2::text IS NULL OR r.repo_name = $2)
              AND ($3::text IS NULL OR r.requested_by = $3)
              AND ($4::text IS NULL OR COALESCE(d.decision, 'pending') = $4)
            ORDER BY r.created_at DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(input.org_id)
        .bind(input.repo_name)
        .bind(input.requested_by)
        .bind(input.status)
        .bind(input.limit)
        .bind(input.offset)
        .bind(input.include_config)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM policy_change_requests r
            LEFT JOIN policy_change_request_decisions d
              ON d.request_id = r.id
            WHERE ($1::uuid IS NULL OR r.org_id = $1::uuid)
              AND ($2::text IS NULL OR r.repo_name = $2)
              AND ($3::text IS NULL OR r.requested_by = $3)
              AND ($4::text IS NULL OR COALESCE(d.decision, 'pending') = $4)
            "#,
        )
        .bind(input.org_id)
        .bind(input.repo_name)
        .bind(input.requested_by)
        .bind(input.status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let records = rows
            .iter()
            .map(|row| {
                let config: serde_json::Value = row.get("requested_config");
                PolicyChangeRequestRecord {
                    id: row.get("id"),
                    org_id: row.get("org_id"),
                    repo_id: row.get("repo_id"),
                    repo_name: row.get("repo_name"),
                    requested_by: row.get("requested_by"),
                    requested_checksum: row.get("requested_checksum"),
                    requested_config: serde_json::from_value(config).unwrap_or_default(),
                    reason: row.get("reason"),
                    status: row.get("status"),
                    decided_by: row.get("decided_by"),
                    decision_note: row.get("decision_note"),
                    created_at: row.get("created_at_ms"),
                    decided_at: row.get("decided_at_ms"),
                }
            })
            .collect();

        Ok((records, count))
    }

    pub async fn get_policy_change_request_by_id(
        &self,
        request_id: &str,
        org_id: Option<&str>,
    ) -> Result<Option<PolicyChangeRequestRecord>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                r.id::text AS id,
                r.org_id::text AS org_id,
                r.repo_id::text AS repo_id,
                r.repo_name AS repo_name,
                r.requested_by AS requested_by,
                r.requested_checksum AS requested_checksum,
                r.requested_config AS requested_config,
                r.reason AS reason,
                COALESCE(d.decision, 'pending') AS status,
                d.decided_by AS decided_by,
                d.note AS decision_note,
                EXTRACT(EPOCH FROM r.created_at)::bigint * 1000 AS created_at_ms,
                EXTRACT(EPOCH FROM d.created_at)::bigint * 1000 AS decided_at_ms
            FROM policy_change_requests r
            LEFT JOIN policy_change_request_decisions d
              ON d.request_id = r.id
            WHERE r.id = $1::uuid
              AND ($2::uuid IS NULL OR r.org_id = $2::uuid)
            "#,
        )
        .bind(request_id)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let config: serde_json::Value = row.get("requested_config");
        Ok(Some(PolicyChangeRequestRecord {
            id: row.get("id"),
            org_id: row.get("org_id"),
            repo_id: row.get("repo_id"),
            repo_name: row.get("repo_name"),
            requested_by: row.get("requested_by"),
            requested_checksum: row.get("requested_checksum"),
            requested_config: serde_json::from_value(config)
                .map_err(|e| DbError::SerializationError(e.to_string()))?,
            reason: row.get("reason"),
            status: row.get("status"),
            decided_by: row.get("decided_by"),
            decision_note: row.get("decision_note"),
            created_at: row.get("created_at_ms"),
            decided_at: row.get("decided_at_ms"),
        }))
    }

    pub async fn approve_policy_change_request(
        &self,
        request_id: &str,
        org_id: Option<&str>,
        decided_by: &str,
        note: Option<&str>,
        decided_at_ms: i64,
    ) -> Result<PolicyChangeRequestRecord, DbError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let request_row = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                org_id::text AS org_id,
                repo_id::text AS repo_id,
                repo_name AS repo_name,
                requested_by AS requested_by,
                requested_checksum AS requested_checksum,
                requested_config AS requested_config,
                reason AS reason,
                EXTRACT(EPOCH FROM created_at)::bigint * 1000 AS created_at_ms
            FROM policy_change_requests
            WHERE id = $1::uuid
              AND ($2::uuid IS NULL OR org_id = $2::uuid)
            FOR UPDATE
            "#,
        )
        .bind(request_id)
        .bind(org_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let Some(request_row) = request_row else {
            return Err(DbError::NotFound("policy_change_request".to_string()));
        };

        let existing_decision: Option<String> = sqlx::query_scalar(
            r#"
            SELECT decision
            FROM policy_change_request_decisions
            WHERE request_id = $1::uuid
            LIMIT 1
            "#,
        )
        .bind(request_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        if existing_decision.is_some() {
            return Err(DbError::Duplicate(
                "policy_change_request already decided".to_string(),
            ));
        }

        let requested_config_json: serde_json::Value = request_row.get("requested_config");
        let requested_checksum: String = request_row.get("requested_checksum");
        let repo_id: String = request_row.get("repo_id");

        sqlx::query(
            r#"
            INSERT INTO policy_change_request_decisions (
                id, request_id, org_id, decision, decided_by, note, created_at
            )
            VALUES (
                $1::uuid, $2::uuid, $3::uuid, 'approved', $4, $5, to_timestamp($6::bigint / 1000.0)
            )
            "#,
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(request_id)
        .bind(request_row.get::<Option<String>, _>("org_id"))
        .bind(decided_by)
        .bind(note)
        .bind(decided_at_ms)
        .execute(&mut *tx)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO policies (repo_id, config, checksum, override_actor, updated_at)
            VALUES ($1::uuid, $2::jsonb, $3, $4, NOW())
            ON CONFLICT (repo_id) DO UPDATE SET
                config = $2,
                checksum = $3,
                override_actor = $4,
                updated_at = NOW()
            "#,
        )
        .bind(&repo_id)
        .bind(&requested_config_json)
        .bind(&requested_checksum)
        .bind(decided_by)
        .execute(&mut *tx)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let requested_config: GitGovConfig = serde_json::from_value(requested_config_json)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;
        Ok(PolicyChangeRequestRecord {
            id: request_row.get("id"),
            org_id: request_row.get("org_id"),
            repo_id,
            repo_name: request_row.get("repo_name"),
            requested_by: request_row.get("requested_by"),
            requested_checksum,
            requested_config,
            reason: request_row.get("reason"),
            status: "approved".to_string(),
            decided_by: Some(decided_by.to_string()),
            decision_note: note.map(str::to_string),
            created_at: request_row.get("created_at_ms"),
            decided_at: Some(decided_at_ms),
        })
    }

    pub async fn reject_policy_change_request(
        &self,
        request_id: &str,
        org_id: Option<&str>,
        decided_by: &str,
        note: Option<&str>,
        decided_at_ms: i64,
    ) -> Result<PolicyChangeRequestRecord, DbError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let request_row = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                org_id::text AS org_id,
                repo_id::text AS repo_id,
                repo_name AS repo_name,
                requested_by AS requested_by,
                requested_checksum AS requested_checksum,
                requested_config AS requested_config,
                reason AS reason,
                EXTRACT(EPOCH FROM created_at)::bigint * 1000 AS created_at_ms
            FROM policy_change_requests
            WHERE id = $1::uuid
              AND ($2::uuid IS NULL OR org_id = $2::uuid)
            FOR UPDATE
            "#,
        )
        .bind(request_id)
        .bind(org_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let Some(request_row) = request_row else {
            return Err(DbError::NotFound("policy_change_request".to_string()));
        };

        let existing_decision: Option<String> = sqlx::query_scalar(
            r#"
            SELECT decision
            FROM policy_change_request_decisions
            WHERE request_id = $1::uuid
            LIMIT 1
            "#,
        )
        .bind(request_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        if existing_decision.is_some() {
            return Err(DbError::Duplicate(
                "policy_change_request already decided".to_string(),
            ));
        }

        sqlx::query(
            r#"
            INSERT INTO policy_change_request_decisions (
                id, request_id, org_id, decision, decided_by, note, created_at
            )
            VALUES (
                $1::uuid, $2::uuid, $3::uuid, 'rejected', $4, $5, to_timestamp($6::bigint / 1000.0)
            )
            "#,
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(request_id)
        .bind(request_row.get::<Option<String>, _>("org_id"))
        .bind(decided_by)
        .bind(note)
        .bind(decided_at_ms)
        .execute(&mut *tx)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let requested_config_json: serde_json::Value = request_row.get("requested_config");
        let requested_config: GitGovConfig = serde_json::from_value(requested_config_json)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;
        Ok(PolicyChangeRequestRecord {
            id: request_row.get("id"),
            org_id: request_row.get("org_id"),
            repo_id: request_row.get("repo_id"),
            repo_name: request_row.get("repo_name"),
            requested_by: request_row.get("requested_by"),
            requested_checksum: request_row.get("requested_checksum"),
            requested_config,
            reason: request_row.get("reason"),
            status: "rejected".to_string(),
            decided_by: Some(decided_by.to_string()),
            decision_note: note.map(str::to_string),
            created_at: request_row.get("created_at_ms"),
            decided_at: Some(decided_at_ms),
        })
    }

    // ========================================================================
    // EXPORT LOGS
    // ========================================================================
}
