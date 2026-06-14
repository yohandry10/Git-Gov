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
            Some("revoked_key".to_string())
        } else if expires_at
            .map(|expires_at| expires_at <= chrono::Utc::now())
            .unwrap_or(false)
        {
            Some("expired_key".to_string())
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
                agent_display_name
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
                $26
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
            "#,
        )
        .bind(input.org_id)
        .bind(input.evaluation_id)
        .bind(input.repository_full_name)
        .bind(input.action)
        .bind(input.decision)
        .bind(input.agent_id)
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
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            FROM agent_governance_evaluations
            WHERE org_id = $1::uuid
              AND ($2::text IS NULL OR evaluation_id = $2)
              AND ($3::text IS NULL OR repository_full_name = $3)
              AND ($4::text IS NULL OR action = $4)
              AND ($5::text IS NULL OR decision = $5)
              AND ($6::text IS NULL OR agent_id = $6)
            ORDER BY created_at DESC
            LIMIT $7 OFFSET $8
            "#,
        )
        .bind(input.org_id)
        .bind(input.evaluation_id)
        .bind(input.repository_full_name)
        .bind(input.action)
        .bind(input.decision)
        .bind(input.agent_id)
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
}
