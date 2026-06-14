use super::*;

impl Database {
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
                metadata
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
                $23::jsonb
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
