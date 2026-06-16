use super::*;

impl Database {
    pub async fn create_change_risk_evaluation(
        &self,
        org_id: &str,
        input: &CreateChangeRiskEvaluationInput,
    ) -> Result<ChangeRiskEvaluationRecord, DbError> {
        let risk_reasons_json = serde_json::to_value(&input.risk_reasons)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;
        let missing_evidence_json = serde_json::to_value(&input.missing_evidence)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;
        let blocking_gaps_json = serde_json::to_value(&input.blocking_gaps)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;
        let recommended_manual_actions_json =
            serde_json::to_value(&input.recommended_manual_actions)
                .map_err(|e| DbError::SerializationError(e.to_string()))?;
        let triggered_rules_json = serde_json::to_value(&input.triggered_rules)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;
        let non_triggered_rules_json = serde_json::to_value(&input.non_triggered_rules)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;

        let row = sqlx::query(
            r#"
            INSERT INTO change_risk_evaluations (
                evaluation_id,
                org_id,
                repository_full_name,
                branch,
                environment,
                change_id,
                deployment_gate_id,
                release_id,
                commit_sha,
                evidence_packet_hash,
                risk_level,
                ruleset_version,
                risk_reasons,
                missing_evidence,
                blocking_gaps,
                recommended_manual_actions,
                triggered_rules,
                non_triggered_rules,
                evaluation_trace,
                trace_hash,
                advisory_only,
                llm_used,
                agent_governance_used,
                compliance_claim,
                certification,
                evaluation,
                request_payload,
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
                $10,
                $11,
                $12,
                $13::jsonb,
                $14::jsonb,
                $15::jsonb,
                $16::jsonb,
                $17::jsonb,
                $18::jsonb,
                $19::jsonb,
                $20,
                TRUE,
                FALSE,
                FALSE,
                FALSE,
                FALSE,
                $21::jsonb,
                $22::jsonb,
                $23
            )
            RETURNING
                evaluation_id,
                org_id::text,
                repository_full_name,
                branch,
                environment,
                change_id,
                deployment_gate_id,
                release_id,
                commit_sha,
                evidence_packet_hash,
                risk_level,
                ruleset_version,
                risk_reasons,
                missing_evidence,
                blocking_gaps,
                recommended_manual_actions,
                triggered_rules,
                non_triggered_rules,
                evaluation_trace,
                trace_hash,
                advisory_only,
                llm_used,
                agent_governance_used,
                compliance_claim,
                certification,
                evaluation,
                request_payload,
                created_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            "#,
        )
        .bind(&input.evaluation_id)
        .bind(org_id)
        .bind(&input.payload.repository_full_name)
        .bind(&input.payload.branch)
        .bind(&input.payload.environment)
        .bind(input.payload.change_id.as_deref())
        .bind(input.payload.deployment_gate_id.as_deref())
        .bind(input.payload.release_id.as_deref())
        .bind(input.payload.commit_sha.as_deref())
        .bind(input.payload.evidence_packet_hash.as_deref())
        .bind(&input.risk_level)
        .bind(&input.ruleset_version)
        .bind(&risk_reasons_json)
        .bind(&missing_evidence_json)
        .bind(&blocking_gaps_json)
        .bind(&recommended_manual_actions_json)
        .bind(&triggered_rules_json)
        .bind(&non_triggered_rules_json)
        .bind(&input.evaluation_trace)
        .bind(&input.trace_hash)
        .bind(&input.evaluation)
        .bind(&input.request_payload)
        .bind(&input.created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(change_risk_evaluation_from_row(&row))
    }

    pub async fn get_change_risk_evaluation(
        &self,
        org_id: &str,
        evaluation_id: &str,
    ) -> Result<Option<ChangeRiskEvaluationRecord>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                evaluation_id,
                org_id::text,
                repository_full_name,
                branch,
                environment,
                change_id,
                deployment_gate_id,
                release_id,
                commit_sha,
                evidence_packet_hash,
                risk_level,
                ruleset_version,
                risk_reasons,
                missing_evidence,
                blocking_gaps,
                recommended_manual_actions,
                triggered_rules,
                non_triggered_rules,
                evaluation_trace,
                trace_hash,
                advisory_only,
                llm_used,
                agent_governance_used,
                compliance_claim,
                certification,
                evaluation,
                request_payload,
                created_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            FROM change_risk_evaluations
            WHERE org_id = $1::uuid
              AND evaluation_id = $2
            "#,
        )
        .bind(org_id)
        .bind(evaluation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| change_risk_evaluation_from_row(&row)))
    }

    pub async fn list_change_risk_evaluations(
        &self,
        org_id: &str,
        query: &ChangeRiskEvaluationQuery,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<ChangeRiskEvaluationRecord>, i64), DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                evaluation_id,
                org_id::text,
                repository_full_name,
                branch,
                environment,
                change_id,
                deployment_gate_id,
                release_id,
                commit_sha,
                evidence_packet_hash,
                risk_level,
                ruleset_version,
                risk_reasons,
                missing_evidence,
                blocking_gaps,
                recommended_manual_actions,
                triggered_rules,
                non_triggered_rules,
                evaluation_trace,
                trace_hash,
                advisory_only,
                llm_used,
                agent_governance_used,
                compliance_claim,
                certification,
                evaluation,
                request_payload,
                created_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                COUNT(*) OVER() AS total_count
            FROM change_risk_evaluations
            WHERE org_id = $1::uuid
              AND ($2::TEXT IS NULL OR evaluation_id = $2)
              AND ($3::TEXT IS NULL OR repository_full_name = $3)
              AND ($4::TEXT IS NULL OR branch = $4)
              AND ($5::TEXT IS NULL OR environment = $5)
              AND ($6::TEXT IS NULL OR change_id = $6)
              AND ($7::TEXT IS NULL OR deployment_gate_id = $7)
              AND ($8::TEXT IS NULL OR release_id = $8)
              AND ($9::TEXT IS NULL OR commit_sha = $9)
            ORDER BY created_at DESC
            LIMIT $10
            OFFSET $11
            "#,
        )
        .bind(org_id)
        .bind(query.evaluation_id.as_deref())
        .bind(query.repository_full_name.as_deref())
        .bind(query.branch.as_deref())
        .bind(query.environment.as_deref())
        .bind(query.change_id.as_deref())
        .bind(query.deployment_gate_id.as_deref())
        .bind(query.release_id.as_deref())
        .bind(query.commit_sha.as_deref())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let total = rows
            .first()
            .map(|row| row.get::<i64, _>("total_count"))
            .unwrap_or(0);
        let items = rows.iter().map(change_risk_evaluation_from_row).collect();

        Ok((items, total))
    }
}
