use super::*;

fn compliance_evidence_export_from_row(row: &PgRow) -> ComplianceEvidenceExportRecord {
    ComplianceEvidenceExportRecord {
        export_id: row.get("export_id"),
        org_id: row.get("org_id"),
        created_by_user_id: row.get("created_by_user_id"),
        scope: row.get("scope"),
        deployment_gate_id: row.get("deployment_gate_id"),
        release_id: row.get("release_id"),
        status: row.get("status"),
        format: row.get("format"),
        artifact_hash: row.get("artifact_hash"),
        policy_checksum: row.get("policy_checksum"),
        gate_decision: row.get("gate_decision"),
        created_at: row.get("created_at_ms"),
        completed_at: row.get("completed_at_ms"),
        error_message_safe: row.get("error_message_safe"),
    }
}

impl Database {
    pub async fn get_deployment_gate_authorization_by_id(
        &self,
        org_id: &str,
        authorization_id: &str,
    ) -> Result<Option<DeploymentGateAuthorizationRecord>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                id::text,
                authorization_id,
                org_id::text,
                release_id,
                repository_full_name,
                branch,
                target_sha,
                environment,
                deployer,
                ticket_id,
                evidence_packet_hash,
                evidence_packet_uri,
                decision,
                approved,
                blocking,
                would_block,
                reason,
                blocked_by,
                warnings,
                policy_checksum,
                break_glass_eligible,
                break_glass_used,
                break_glass_reason,
                break_glass_authorized_by,
                ROUND(EXTRACT(EPOCH FROM break_glass_expires_at) * 1000)::BIGINT AS break_glass_expires_at_ms,
                break_glass_approval_id,
                break_glass_approval_hash,
                evaluation,
                details,
                request_payload,
                requested_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            FROM deployment_gate_authorizations
            WHERE org_id = $1::uuid
              AND authorization_id = $2
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(authorization_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| deployment_gate_authorization_from_row(&row)))
    }

    pub async fn get_compliance_evidence_context(
        &self,
        org_id: &str,
        gate: &DeploymentGateAuthorizationRecord,
    ) -> Result<serde_json::Value, DbError> {
        let client_event_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::bigint
            FROM client_events ce
            LEFT JOIN repos r ON ce.repo_id = r.id
            WHERE ce.org_id = $1::uuid
              AND (r.full_name = $2 OR ce.metadata->>'repo_full_name' = $2)
              AND ($3::text IS NULL OR ce.branch = $3)
              AND ($4::text IS NULL OR ce.commit_sha = $4)
            "#,
        )
        .bind(org_id)
        .bind(&gate.repository_full_name)
        .bind(Some(gate.branch.as_str()))
        .bind(Some(gate.target_sha.as_str()))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let pipeline_event_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::bigint
            FROM pipeline_events
            WHERE org_id = $1::uuid
              AND repo_full_name = $2
              AND branch = $3
              AND commit_sha = $4
            "#,
        )
        .bind(org_id)
        .bind(&gate.repository_full_name)
        .bind(&gate.branch)
        .bind(&gate.target_sha)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let ticket_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::bigint
            FROM project_tickets
            WHERE org_id = $1::uuid
              AND ($2::text IS NOT NULL AND ticket_id = $2)
            "#,
        )
        .bind(org_id)
        .bind(gate.ticket_id.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let release_approval_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::bigint
            FROM enterprise_release_approvals
            WHERE org_id = $1::uuid
              AND release_id = $2
              AND repository_full_name = $3
              AND environment = $4
              AND ($5::text IS NULL OR branch = $5)
              AND ($6::text IS NULL OR target_sha = $6)
              AND ($7::text IS NULL OR evidence_packet_hash = $7)
            "#,
        )
        .bind(org_id)
        .bind(&gate.release_id)
        .bind(&gate.repository_full_name)
        .bind(&gate.environment)
        .bind(Some(gate.branch.as_str()))
        .bind(Some(gate.target_sha.as_str()))
        .bind(Some(gate.evidence_packet_hash.as_str()))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let admin_audit_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::bigint
            FROM admin_audit_log
            WHERE metadata->>'org_id' = $1
              AND (
                target_id = $2
                OR metadata->>'authorization_id' = $2
                OR metadata->>'release_id' = $3
              )
            "#,
        )
        .bind(org_id)
        .bind(&gate.authorization_id)
        .bind(&gate.release_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(serde_json::json!({
            "counts": {
                "client_events": client_event_count,
                "pipeline_events": pipeline_event_count,
                "jira_tickets": ticket_count,
                "release_approvals": release_approval_count,
                "admin_audit_events": admin_audit_count
            }
        }))
    }

    pub async fn create_compliance_evidence_export(
        &self,
        input: &CreateComplianceEvidenceExportInput<'_>,
    ) -> Result<ComplianceEvidenceExportRecord, DbError> {
        let row = sqlx::query(
            r#"
            INSERT INTO compliance_evidence_exports (
                export_id,
                org_id,
                created_by_user_id,
                scope,
                deployment_gate_id,
                release_id,
                status,
                format,
                artifact_hash,
                policy_checksum,
                gate_decision,
                payload_json_redacted,
                completed_at
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
                $12::jsonb,
                NOW()
            )
            RETURNING
                export_id,
                org_id::text,
                created_by_user_id,
                scope,
                deployment_gate_id,
                release_id,
                status,
                format,
                artifact_hash,
                policy_checksum,
                gate_decision,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM completed_at) * 1000)::BIGINT AS completed_at_ms,
                error_message_safe
            "#,
        )
        .bind(input.export_id)
        .bind(input.org_id)
        .bind(input.created_by_user_id)
        .bind(input.scope)
        .bind(input.deployment_gate_id)
        .bind(input.release_id)
        .bind(input.status)
        .bind(input.format)
        .bind(input.artifact_hash)
        .bind(input.policy_checksum)
        .bind(input.gate_decision)
        .bind(input.payload_json_redacted)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(compliance_evidence_export_from_row(&row))
    }

    pub async fn get_compliance_evidence_export(
        &self,
        org_id: &str,
        export_id: &str,
    ) -> Result<Option<ComplianceEvidenceExportRecord>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                export_id,
                org_id::text,
                created_by_user_id,
                scope,
                deployment_gate_id,
                release_id,
                status,
                format,
                artifact_hash,
                policy_checksum,
                gate_decision,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM completed_at) * 1000)::BIGINT AS completed_at_ms,
                error_message_safe
            FROM compliance_evidence_exports
            WHERE org_id = $1::uuid
              AND export_id = $2
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(export_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| compliance_evidence_export_from_row(&row)))
    }

    pub async fn get_compliance_evidence_export_payload(
        &self,
        org_id: &str,
        export_id: &str,
    ) -> Result<Option<serde_json::Value>, DbError> {
        sqlx::query_scalar(
            r#"
            SELECT payload_json_redacted
            FROM compliance_evidence_exports
            WHERE org_id = $1::uuid
              AND export_id = $2
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(export_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))
    }
}
