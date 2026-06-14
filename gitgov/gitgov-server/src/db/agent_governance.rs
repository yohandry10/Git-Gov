use super::*;

impl Database {
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
}
