use super::*;

impl Database {
    pub async fn validate_agent_governance_agent_key(
        &self,
        token_hash: &str,
    ) -> Result<Option<AgentKeyAuthContext>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                key_id,
                org_id::text,
                display_name,
                scopes,
                allowed_actions,
                revoked_at,
                expires_at
            FROM agent_governance_agent_keys
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let key_id: String = row.get("key_id");
        let org_id: String = row.get("org_id");
        let display_name: String = row.get("display_name");
        let scopes = string_vec_from_json(row.get("scopes"));
        let allowed_actions = string_vec_from_json(row.get("allowed_actions"));
        let revoked_at: Option<chrono::DateTime<chrono::Utc>> = row.get("revoked_at");
        let expires_at: Option<chrono::DateTime<chrono::Utc>> = row.get("expires_at");
        let denied_reason = if revoked_at.is_some() {
            Some("agent_key_revoked".to_string())
        } else if expires_at
            .map(|expires_at| expires_at <= chrono::Utc::now())
            .unwrap_or(false)
        {
            Some("agent_key_expired".to_string())
        } else {
            None
        };

        Ok(Some(AgentKeyAuthContext {
            client_id: format!("agent:{key_id}"),
            org_id,
            agent_key_id: key_id,
            display_name,
            scopes,
            allowed_actions,
            denied_reason,
        }))
    }

    pub async fn mark_agent_governance_agent_key_used(&self, key_id: &str) -> Result<(), DbError> {
        sqlx::query(
            r#"
            UPDATE agent_governance_agent_keys
            SET last_used_at = NOW()
            WHERE key_id = $1
              AND revoked_at IS NULL
            "#,
        )
        .bind(key_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn create_agent_governance_agent_key(
        &self,
        input: &CreateAgentGovernanceAgentKeyInput<'_>,
    ) -> Result<AgentGovernanceAgentKeyRecord, DbError> {
        let scopes_json = serde_json::to_value(input.scopes)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;
        let allowed_actions_json = serde_json::to_value(input.allowed_actions)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;
        let token_preview = format!("{}****{}", input.token_prefix, input.token_last4);

        let row = sqlx::query(
            r#"
            INSERT INTO agent_governance_agent_keys (
                key_id,
                org_id,
                token_hash,
                token_prefix,
                token_last4,
                token_preview,
                display_name,
                description,
                environment,
                scopes,
                allowed_actions,
                expires_at,
                created_by
            )
            VALUES (
                $1,
                $2::uuid,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10::jsonb,
                $11::jsonb,
                $12,
                $13
            )
            RETURNING
                id::text,
                key_id,
                org_id::text,
                display_name,
                description,
                environment,
                scopes,
                allowed_actions,
                token_preview,
                ROUND(EXTRACT(EPOCH FROM expires_at) * 1000)::BIGINT AS expires_at_ms,
                ROUND(EXTRACT(EPOCH FROM last_used_at) * 1000)::BIGINT AS last_used_at_ms,
                ROUND(EXTRACT(EPOCH FROM revoked_at) * 1000)::BIGINT AS revoked_at_ms,
                ROUND(EXTRACT(EPOCH FROM rotated_at) * 1000)::BIGINT AS rotated_at_ms,
                rotated_from_key_id,
                replaced_by_key_id,
                rotation_reason,
                created_by,
                revoked_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            "#,
        )
        .bind(input.key_id)
        .bind(input.org_id)
        .bind(input.token_hash)
        .bind(input.token_prefix)
        .bind(input.token_last4)
        .bind(&token_preview)
        .bind(input.display_name)
        .bind(input.description)
        .bind(input.environment)
        .bind(&scopes_json)
        .bind(&allowed_actions_json)
        .bind(input.expires_at)
        .bind(input.created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(agent_governance_agent_key_from_row(&row))
    }

    pub async fn list_agent_governance_agent_keys(
        &self,
        org_id: &str,
    ) -> Result<Vec<AgentGovernanceAgentKeyRecord>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id::text,
                key_id,
                org_id::text,
                display_name,
                description,
                environment,
                scopes,
                allowed_actions,
                token_preview,
                ROUND(EXTRACT(EPOCH FROM expires_at) * 1000)::BIGINT AS expires_at_ms,
                ROUND(EXTRACT(EPOCH FROM last_used_at) * 1000)::BIGINT AS last_used_at_ms,
                ROUND(EXTRACT(EPOCH FROM revoked_at) * 1000)::BIGINT AS revoked_at_ms,
                ROUND(EXTRACT(EPOCH FROM rotated_at) * 1000)::BIGINT AS rotated_at_ms,
                rotated_from_key_id,
                replaced_by_key_id,
                rotation_reason,
                created_by,
                revoked_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            FROM agent_governance_agent_keys
            WHERE org_id = $1::uuid
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .iter()
            .map(agent_governance_agent_key_from_row)
            .collect())
    }

    pub async fn revoke_agent_governance_agent_key(
        &self,
        org_id: &str,
        key_id: &str,
        revoked_by: &str,
    ) -> Result<Option<AgentGovernanceAgentKeyRecord>, DbError> {
        let row = sqlx::query(
            r#"
            UPDATE agent_governance_agent_keys
            SET revoked_at = COALESCE(revoked_at, NOW()),
                revoked_by = COALESCE(revoked_by, $3)
            WHERE org_id = $1::uuid
              AND key_id = $2
            RETURNING
                id::text,
                key_id,
                org_id::text,
                display_name,
                description,
                environment,
                scopes,
                allowed_actions,
                token_preview,
                ROUND(EXTRACT(EPOCH FROM expires_at) * 1000)::BIGINT AS expires_at_ms,
                ROUND(EXTRACT(EPOCH FROM last_used_at) * 1000)::BIGINT AS last_used_at_ms,
                ROUND(EXTRACT(EPOCH FROM revoked_at) * 1000)::BIGINT AS revoked_at_ms,
                ROUND(EXTRACT(EPOCH FROM rotated_at) * 1000)::BIGINT AS rotated_at_ms,
                rotated_from_key_id,
                replaced_by_key_id,
                rotation_reason,
                created_by,
                revoked_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            "#,
        )
        .bind(org_id)
        .bind(key_id)
        .bind(revoked_by)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.as_ref().map(agent_governance_agent_key_from_row))
    }

    pub async fn rotate_agent_governance_agent_key(
        &self,
        input: &RotateAgentGovernanceAgentKeyInput<'_>,
    ) -> Result<RotateAgentGovernanceAgentKeyOutcome, DbError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let old_row = sqlx::query(
            r#"
            SELECT
                id::text,
                key_id,
                org_id::text,
                display_name,
                description,
                environment,
                scopes,
                allowed_actions,
                token_preview,
                ROUND(EXTRACT(EPOCH FROM expires_at) * 1000)::BIGINT AS expires_at_ms,
                ROUND(EXTRACT(EPOCH FROM last_used_at) * 1000)::BIGINT AS last_used_at_ms,
                ROUND(EXTRACT(EPOCH FROM revoked_at) * 1000)::BIGINT AS revoked_at_ms,
                ROUND(EXTRACT(EPOCH FROM rotated_at) * 1000)::BIGINT AS rotated_at_ms,
                rotated_from_key_id,
                replaced_by_key_id,
                rotation_reason,
                created_by,
                revoked_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            FROM agent_governance_agent_keys
            WHERE org_id = $1::uuid
              AND key_id = $2
            FOR UPDATE
            "#,
        )
        .bind(input.org_id)
        .bind(input.key_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let Some(old_row) = old_row else {
            tx.rollback()
                .await
                .map_err(|e| DbError::DatabaseError(e.to_string()))?;
            return Ok(RotateAgentGovernanceAgentKeyOutcome::NotFound);
        };

        let old_revoked_at: Option<i64> = old_row.get("revoked_at_ms");
        if old_revoked_at.is_some() {
            tx.rollback()
                .await
                .map_err(|e| DbError::DatabaseError(e.to_string()))?;
            return Ok(RotateAgentGovernanceAgentKeyOutcome::Revoked);
        }

        let old_expires_at: Option<i64> = old_row.get("expires_at_ms");
        if old_expires_at
            .map(|value| value <= chrono::Utc::now().timestamp_millis())
            .unwrap_or(false)
        {
            tx.rollback()
                .await
                .map_err(|e| DbError::DatabaseError(e.to_string()))?;
            return Ok(RotateAgentGovernanceAgentKeyOutcome::Expired);
        }

        let scopes_value: serde_json::Value = old_row.get("scopes");
        let allowed_actions_value: serde_json::Value = old_row.get("allowed_actions");
        let replacement_token_preview = format!(
            "{}****{}",
            input.replacement_token_prefix, input.replacement_token_last4
        );

        let replacement_row = sqlx::query(
            r#"
            INSERT INTO agent_governance_agent_keys (
                key_id,
                org_id,
                token_hash,
                token_prefix,
                token_last4,
                token_preview,
                display_name,
                description,
                environment,
                scopes,
                allowed_actions,
                expires_at,
                rotated_at,
                rotated_from_key_id,
                rotation_reason,
                created_by
            )
            VALUES (
                $1,
                $2::uuid,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10::jsonb,
                $11::jsonb,
                $12,
                NOW(),
                $13,
                $14,
                $15
            )
            RETURNING
                id::text,
                key_id,
                org_id::text,
                display_name,
                description,
                environment,
                scopes,
                allowed_actions,
                token_preview,
                ROUND(EXTRACT(EPOCH FROM expires_at) * 1000)::BIGINT AS expires_at_ms,
                ROUND(EXTRACT(EPOCH FROM last_used_at) * 1000)::BIGINT AS last_used_at_ms,
                ROUND(EXTRACT(EPOCH FROM revoked_at) * 1000)::BIGINT AS revoked_at_ms,
                ROUND(EXTRACT(EPOCH FROM rotated_at) * 1000)::BIGINT AS rotated_at_ms,
                rotated_from_key_id,
                replaced_by_key_id,
                rotation_reason,
                created_by,
                revoked_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            "#,
        )
        .bind(input.replacement_key_id)
        .bind(input.org_id)
        .bind(input.replacement_token_hash)
        .bind(input.replacement_token_prefix)
        .bind(input.replacement_token_last4)
        .bind(&replacement_token_preview)
        .bind(old_row.get::<String, _>("display_name"))
        .bind(old_row.get::<Option<String>, _>("description"))
        .bind(old_row.get::<Option<String>, _>("environment"))
        .bind(scopes_value)
        .bind(allowed_actions_value)
        .bind(input.replacement_expires_at)
        .bind(input.key_id)
        .bind(input.rotation_reason)
        .bind(input.rotated_by)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let replaced_row = sqlx::query(
            r#"
            UPDATE agent_governance_agent_keys
            SET expires_at = LEAST(COALESCE(expires_at, $3), $3),
                replaced_by_key_id = $4,
                rotated_at = NOW(),
                rotation_reason = $5
            WHERE org_id = $1::uuid
              AND key_id = $2
            RETURNING
                id::text,
                key_id,
                org_id::text,
                display_name,
                description,
                environment,
                scopes,
                allowed_actions,
                token_preview,
                ROUND(EXTRACT(EPOCH FROM expires_at) * 1000)::BIGINT AS expires_at_ms,
                ROUND(EXTRACT(EPOCH FROM last_used_at) * 1000)::BIGINT AS last_used_at_ms,
                ROUND(EXTRACT(EPOCH FROM revoked_at) * 1000)::BIGINT AS revoked_at_ms,
                ROUND(EXTRACT(EPOCH FROM rotated_at) * 1000)::BIGINT AS rotated_at_ms,
                rotated_from_key_id,
                replaced_by_key_id,
                rotation_reason,
                created_by,
                revoked_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            "#,
        )
        .bind(input.org_id)
        .bind(input.key_id)
        .bind(input.grace_expires_at)
        .bind(input.replacement_key_id)
        .bind(input.rotation_reason)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(RotateAgentGovernanceAgentKeyOutcome::Rotated(Box::new(
            RotateAgentGovernanceAgentKeyRecords {
                replacement: agent_governance_agent_key_from_row(&replacement_row),
                replaced: agent_governance_agent_key_from_row(&replaced_row),
            },
        )))
    }

    pub async fn get_agent_governance_settings(
        &self,
        org_id: &str,
    ) -> Result<AgentGovernanceSettingsRecord, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                $1::uuid::text AS org_id,
                COALESCE(settings.enabled, FALSE) AS enabled,
                COALESCE(settings.mode, 'manual_only') AS mode,
                COALESCE(settings.payload_mode, 'minimized') AS payload_mode,
                settings.reason,
                COALESCE(settings.updated_by, 'system') AS updated_by,
                ROUND(EXTRACT(EPOCH FROM COALESCE(settings.created_at, NOW())) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM COALESCE(settings.updated_at, NOW())) * 1000)::BIGINT AS updated_at_ms
            FROM (SELECT $1::uuid AS org_id) scope
            LEFT JOIN agent_governance_settings settings
              ON settings.org_id = scope.org_id
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(agent_governance_settings_from_row(&row))
    }

    pub async fn upsert_agent_governance_settings(
        &self,
        org_id: &str,
        enabled: bool,
        reason: Option<&str>,
        updated_by: &str,
    ) -> Result<AgentGovernanceSettingsRecord, DbError> {
        let mode = if enabled {
            "opt_in_enabled"
        } else {
            "manual_only"
        };
        let row = sqlx::query(
            r#"
            INSERT INTO agent_governance_settings (
                org_id,
                enabled,
                mode,
                payload_mode,
                reason,
                updated_by
            )
            VALUES ($1::uuid, $2, $3, 'minimized', $4, $5)
            ON CONFLICT (org_id) DO UPDATE SET
                enabled = EXCLUDED.enabled,
                mode = EXCLUDED.mode,
                payload_mode = 'minimized',
                reason = EXCLUDED.reason,
                updated_by = EXCLUDED.updated_by,
                updated_at = NOW()
            RETURNING
                org_id::text,
                enabled,
                mode,
                payload_mode,
                reason,
                updated_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM updated_at) * 1000)::BIGINT AS updated_at_ms
            "#,
        )
        .bind(org_id)
        .bind(enabled)
        .bind(mode)
        .bind(reason)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(agent_governance_settings_from_row(&row))
    }

    pub async fn create_agent_governance_evaluation(
        &self,
        org_id: &str,
        input: &CreateAgentGovernanceEvaluationInput,
    ) -> Result<AgentGovernanceEvaluationRecord, DbError> {
        let reasons_json = serde_json::to_value(&input.reasons)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;
        let required_evidence_json = serde_json::to_value(&input.required_evidence)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;

        let row = sqlx::query(
            r#"
            INSERT INTO agent_governance_evaluations (
                evaluation_id,
                org_id,
                agent_id,
                agent_type,
                actor,
                action,
                repository_full_name,
                branch,
                target_sha,
                environment,
                ticket_id,
                operation_id,
                decision,
                allowed,
                requires_approval,
                reason,
                reasons,
                required_evidence,
                policy_id,
                policy_checksum,
                evaluation,
                request_payload,
                metadata,
                principal_type,
                agent_key_id,
                agent_display_name,
                attribution_id,
                correlation_id,
                parent_correlation_id,
                session_id,
                tool_name,
                tool_version,
                agent_name,
                external_run_id,
                consumer_type
            )
            VALUES (
                $1,
                $2::uuid,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10,
                $11,
                $12,
                $13,
                $14,
                $15,
                $16,
                $17::jsonb,
                $18::jsonb,
                $19,
                $20,
                $21::jsonb,
                $22::jsonb,
                $23::jsonb,
                $24,
                $25,
                $26,
                $27,
                $28,
                $29,
                $30,
                $31,
                $32,
                $33,
                $34,
                $35
            )
            RETURNING
                id::text,
                evaluation_id,
                org_id::text,
                agent_id,
                agent_type,
                actor,
                action,
                repository_full_name,
                branch,
                target_sha,
                environment,
                ticket_id,
                operation_id,
                decision,
                allowed,
                requires_approval,
                reason,
                reasons,
                required_evidence,
                policy_id,
                policy_checksum,
                evaluation,
                request_payload,
                metadata,
                principal_type,
                agent_key_id,
                agent_display_name,
                attribution_id,
                correlation_id,
                parent_correlation_id,
                session_id,
                tool_name,
                tool_version,
                agent_name,
                external_run_id,
                consumer_type,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            "#,
        )
        .bind(&input.evaluation_id)
        .bind(org_id)
        .bind(&input.payload.agent_id)
        .bind(&input.agent_type)
        .bind(&input.payload.actor)
        .bind(&input.payload.action)
        .bind(&input.payload.repository_full_name)
        .bind(input.payload.branch.as_deref())
        .bind(input.payload.target_sha.as_deref())
        .bind(input.payload.environment.as_deref())
        .bind(input.payload.ticket_id.as_deref())
        .bind(input.payload.operation_id.as_deref())
        .bind(&input.decision)
        .bind(input.allowed)
        .bind(input.requires_approval)
        .bind(&input.reason)
        .bind(&reasons_json)
        .bind(&required_evidence_json)
        .bind(&input.policy_id)
        .bind(&input.policy_checksum)
        .bind(&input.evaluation)
        .bind(&input.request_payload)
        .bind(&input.payload.metadata)
        .bind(input.principal_type.as_deref())
        .bind(input.agent_key_id.as_deref())
        .bind(input.agent_display_name.as_deref())
        .bind(&input.attribution.attribution_id)
        .bind(&input.attribution.correlation_id)
        .bind(input.attribution.parent_correlation_id.as_deref())
        .bind(input.attribution.session_id.as_deref())
        .bind(input.attribution.tool_name.as_deref())
        .bind(input.attribution.tool_version.as_deref())
        .bind(input.attribution.agent_name.as_deref())
        .bind(input.attribution.external_run_id.as_deref())
        .bind(&input.attribution.consumer_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(agent_governance_evaluation_from_row(&row))
    }

    pub async fn list_agent_governance_evaluations(
        &self,
        input: &ListAgentGovernanceEvaluationsInput<'_>,
    ) -> Result<(Vec<AgentGovernanceEvaluationRecord>, i64), DbError> {
        let limit = input.limit.clamp(1, 100);
        let offset = input.offset.max(0);

        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::bigint
            FROM agent_governance_evaluations
            WHERE org_id = $1::uuid
              AND ($2::text IS NULL OR evaluation_id = $2)
              AND ($3::text IS NULL OR repository_full_name = $3)
              AND ($4::text IS NULL OR action = $4)
              AND ($5::text IS NULL OR decision = $5)
              AND ($6::text IS NULL OR agent_id = $6)
              AND ($7::text IS NULL OR correlation_id = $7)
            "#,
        )
        .bind(input.org_id)
        .bind(input.evaluation_id)
        .bind(input.repository_full_name)
        .bind(input.action)
        .bind(input.decision)
        .bind(input.agent_id)
        .bind(input.correlation_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let rows = sqlx::query(
            r#"
            SELECT
                id::text,
                evaluation_id,
                org_id::text,
                agent_id,
                agent_type,
                actor,
                action,
                repository_full_name,
                branch,
                target_sha,
                environment,
                ticket_id,
                operation_id,
                decision,
                allowed,
                requires_approval,
                reason,
                reasons,
                required_evidence,
                policy_id,
                policy_checksum,
                evaluation,
                request_payload,
                metadata,
                principal_type,
                agent_key_id,
                agent_display_name,
                attribution_id,
                correlation_id,
                parent_correlation_id,
                session_id,
                tool_name,
                tool_version,
                agent_name,
                external_run_id,
                consumer_type,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            FROM agent_governance_evaluations
            WHERE org_id = $1::uuid
              AND ($2::text IS NULL OR evaluation_id = $2)
              AND ($3::text IS NULL OR repository_full_name = $3)
              AND ($4::text IS NULL OR action = $4)
              AND ($5::text IS NULL OR decision = $5)
              AND ($6::text IS NULL OR agent_id = $6)
              AND ($7::text IS NULL OR correlation_id = $7)
            ORDER BY created_at DESC
            LIMIT $8 OFFSET $9
            "#,
        )
        .bind(input.org_id)
        .bind(input.evaluation_id)
        .bind(input.repository_full_name)
        .bind(input.action)
        .bind(input.decision)
        .bind(input.agent_id)
        .bind(input.correlation_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok((
            rows.iter()
                .map(agent_governance_evaluation_from_row)
                .collect(),
            total,
        ))
    }

    pub async fn get_agent_governance_read_context(
        &self,
        input: &AgentGovernanceReadContextInput<'_>,
    ) -> Result<serde_json::Value, DbError> {
        let commit_events_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::bigint
            FROM client_events ce
            LEFT JOIN repos r ON ce.repo_id = r.id
            WHERE ce.org_id = $1::uuid
              AND ce.event_type = 'commit'
              AND (r.full_name = $2 OR ce.metadata->>'repo_full_name' = $2)
              AND ($3::text IS NULL OR ce.branch = $3)
              AND ($4::text IS NULL OR ce.commit_sha = $4)
            "#,
        )
        .bind(input.org_id)
        .bind(input.repository_full_name)
        .bind(input.branch)
        .bind(input.target_sha)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let latest_pipeline = sqlx::query(
            r#"
            SELECT
                pipeline_id,
                job_name,
                status,
                commit_sha,
                duration_ms,
                ROUND(EXTRACT(EPOCH FROM ingested_at) * 1000)::BIGINT AS ingested_at_ms
            FROM pipeline_events
            WHERE org_id = $1::uuid
              AND repo_full_name = $2
              AND ($3::text IS NULL OR branch = $3)
              AND ($4::text IS NULL OR commit_sha = $4)
            ORDER BY ingested_at DESC
            LIMIT 1
            "#,
        )
        .bind(input.org_id)
        .bind(input.repository_full_name)
        .bind(input.branch)
        .bind(input.target_sha)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let latest_deployment_gate = sqlx::query(
            r#"
            SELECT
                authorization_id,
                decision,
                approved,
                blocking,
                would_block,
                reason,
                policy_checksum,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            FROM deployment_gate_authorizations
            WHERE org_id = $1::uuid
              AND repository_full_name = $2
              AND ($3::text IS NULL OR branch = $3)
              AND ($4::text IS NULL OR target_sha = $4)
              AND ($5::text IS NULL OR environment = $5)
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(input.org_id)
        .bind(input.repository_full_name)
        .bind(input.branch)
        .bind(input.target_sha)
        .bind(input.environment)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let policy = sqlx::query(
            r#"
            SELECT p.checksum, p.config, p.source_metadata
            FROM policies p
            LEFT JOIN repos r ON p.repo_id = r.id
            WHERE p.org_id = $1::uuid
              AND (r.full_name = $2 OR r.full_name IS NULL)
            ORDER BY p.updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(input.org_id)
        .bind(input.repository_full_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let agent_evaluation_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::bigint
            FROM agent_governance_evaluations
            WHERE org_id = $1::uuid
              AND repository_full_name = $2
              AND ($3::text IS NULL OR branch = $3)
            "#,
        )
        .bind(input.org_id)
        .bind(input.repository_full_name)
        .bind(input.branch)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let latest_agent_evaluation = sqlx::query(
            r#"
            SELECT evaluation_id, action, decision, agent_id, agent_key_id,
                   ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            FROM agent_governance_evaluations
            WHERE org_id = $1::uuid
              AND repository_full_name = $2
              AND ($3::text IS NULL OR branch = $3)
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(input.org_id)
        .bind(input.repository_full_name)
        .bind(input.branch)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let agent_key_audit_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::bigint
            FROM admin_audit_log
            WHERE action LIKE 'agent_key.%'
              AND metadata->>'org_id' = $1
            "#,
        )
        .bind(input.org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let branch = input.branch.unwrap_or_default().to_ascii_lowercase();
        let protected_branch = matches!(
            branch.as_str(),
            "main" | "master" | "production" | "prod" | "release"
        );
        let latest_pipeline_status = latest_pipeline
            .as_ref()
            .and_then(|row| row.try_get::<Option<String>, _>("status").ok().flatten());
        let latest_gate_decision = latest_deployment_gate
            .as_ref()
            .and_then(|row| row.try_get::<Option<String>, _>("decision").ok().flatten());
        let policy_found = policy.is_some();
        let risk_score = if latest_gate_decision.as_deref() == Some("blocked") {
            85
        } else if latest_pipeline_status
            .as_deref()
            .is_some_and(|status| !matches!(status, "success" | "passed" | "ok"))
        {
            70
        } else if !policy_found {
            60
        } else if protected_branch && latest_deployment_gate.is_none() {
            45
        } else {
            15
        };
        let risk_level = if risk_score >= 80 {
            "high"
        } else if risk_score >= 50 {
            "medium"
        } else {
            "low"
        };

        Ok(serde_json::json!({
            "branch_status": {
                "protected_branch": protected_branch,
                "commit_events_count": commit_events_count,
                "latest_target_sha_observed": input.target_sha.is_some() && commit_events_count > 0,
                "deployment_gate_evidence_present": latest_deployment_gate.is_some(),
                "agent_governance_read_only": true
            },
            "policy_compliance": {
                "policy_found": policy_found,
                "policy_checksum": policy.as_ref().and_then(|row| row.try_get::<Option<String>, _>("checksum").ok().flatten()),
                "source_metadata": policy.as_ref().and_then(|row| row.try_get::<Option<serde_json::Value>, _>("source_metadata").ok().flatten()),
                "change_policy_requires_human": true,
                "llm_decision": false
            },
            "pipeline_state": {
                "latest": latest_pipeline.as_ref().map(|row| serde_json::json!({
                    "pipeline_id": row.try_get::<Option<String>, _>("pipeline_id").ok().flatten(),
                    "job_name": row.try_get::<Option<String>, _>("job_name").ok().flatten(),
                    "status": row.try_get::<Option<String>, _>("status").ok().flatten(),
                    "commit_sha": row.try_get::<Option<String>, _>("commit_sha").ok().flatten(),
                    "duration_ms": row.try_get::<Option<i64>, _>("duration_ms").ok().flatten(),
                    "created_at": row.try_get::<Option<i64>, _>("ingested_at_ms").ok().flatten()
                })),
                "source": "gitgov_evidence"
            },
            "risk_score": {
                "score": risk_score,
                "level": risk_level,
                "inputs": {
                    "policy_found": policy_found,
                    "protected_branch": protected_branch,
                    "latest_pipeline_status": latest_pipeline_status,
                    "latest_deployment_gate_decision": latest_gate_decision
                }
            },
            "recent_activity": {
                "agent_evaluation_count": agent_evaluation_count,
                "latest_agent_evaluation": latest_agent_evaluation.as_ref().map(|row| serde_json::json!({
                    "evaluation_id": row.try_get::<Option<String>, _>("evaluation_id").ok().flatten(),
                    "action": row.try_get::<Option<String>, _>("action").ok().flatten(),
                    "decision": row.try_get::<Option<String>, _>("decision").ok().flatten(),
                    "agent_id": row.try_get::<Option<String>, _>("agent_id").ok().flatten(),
                    "agent_key_id": row.try_get::<Option<String>, _>("agent_key_id").ok().flatten(),
                    "created_at": row.try_get::<Option<i64>, _>("created_at_ms").ok().flatten()
                })),
                "agent_key_audit_count": agent_key_audit_count,
                "latest_deployment_gate": latest_deployment_gate.as_ref().map(|row| serde_json::json!({
                    "authorization_id": row.try_get::<Option<String>, _>("authorization_id").ok().flatten(),
                    "decision": row.try_get::<Option<String>, _>("decision").ok().flatten(),
                    "approved": row.try_get::<Option<bool>, _>("approved").ok().flatten(),
                    "blocking": row.try_get::<Option<bool>, _>("blocking").ok().flatten(),
                    "would_block": row.try_get::<Option<bool>, _>("would_block").ok().flatten(),
                    "reason": row.try_get::<Option<String>, _>("reason").ok().flatten(),
                    "policy_checksum": row.try_get::<Option<String>, _>("policy_checksum").ok().flatten(),
                    "created_at": row.try_get::<Option<i64>, _>("created_at_ms").ok().flatten()
                }))
            }
        }))
    }
}
