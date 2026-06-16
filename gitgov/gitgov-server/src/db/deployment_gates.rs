use super::*;

impl Database {
    pub async fn create_deployment_gate_authorization(
        &self,
        org_id: &str,
        input: &CreateDeploymentGateAuthorizationInput,
    ) -> Result<DeploymentGateAuthorizationRecord, DbError> {
        let blocked_by_json = serde_json::to_value(&input.blocked_by)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;
        let warnings_json = serde_json::to_value(&input.warnings)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;
        let evaluation_json = serde_json::to_value(&input.evaluation)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;

        let row = sqlx::query(
            r#"
            INSERT INTO deployment_gate_authorizations (
                authorization_id,
                org_id,
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
                break_glass_expires_at,
                break_glass_approval_id,
                break_glass_approval_hash,
                evaluation,
                details,
                request_payload,
                requested_by
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
                $21,
                $22,
                $23,
                CASE WHEN $24::BIGINT IS NULL THEN NULL ELSE to_timestamp($24::DOUBLE PRECISION / 1000.0) END,
                $25,
                $26,
                $27::jsonb,
                $28::jsonb,
                $29::jsonb,
                $30
            )
            RETURNING
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
            "#,
        )
        .bind(&input.authorization_id)
        .bind(org_id)
        .bind(&input.payload.release_id)
        .bind(&input.payload.repository_full_name)
        .bind(&input.payload.branch)
        .bind(&input.payload.target_sha)
        .bind(&input.payload.environment)
        .bind(&input.payload.deployer)
        .bind(input.payload.ticket_id.as_deref())
        .bind(&input.payload.evidence_packet_hash)
        .bind(input.payload.evidence_packet_uri.as_deref())
        .bind(&input.decision)
        .bind(input.approved)
        .bind(input.blocking)
        .bind(input.would_block)
        .bind(&input.reason)
        .bind(&blocked_by_json)
        .bind(&warnings_json)
        .bind(&input.policy_checksum)
        .bind(input.break_glass_eligible)
        .bind(input.break_glass_used)
        .bind(input.break_glass_reason.as_deref())
        .bind(input.break_glass_authorized_by.as_deref())
        .bind(input.break_glass_expires_at)
        .bind(input.break_glass_approval_id.as_deref())
        .bind(input.break_glass_approval_hash.as_deref())
        .bind(&evaluation_json)
        .bind(&input.details)
        .bind(&input.request_payload)
        .bind(&input.requested_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(deployment_gate_authorization_from_row(&row))
    }

    pub async fn list_deployment_gate_authorizations(
        &self,
        org_id: &str,
        query: &DeploymentGateAuthorizationQuery,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<DeploymentGateAuthorizationRecord>, i64), DbError> {
        let rows = sqlx::query(
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
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                COUNT(*) OVER() AS total_count
            FROM deployment_gate_authorizations
            WHERE org_id = $1::uuid
              AND ($2::TEXT IS NULL OR authorization_id = $2)
              AND ($3::TEXT IS NULL OR repository_full_name = $3)
              AND ($4::TEXT IS NULL OR branch = $4)
              AND ($5::TEXT IS NULL OR target_sha = $5)
              AND ($6::TEXT IS NULL OR release_id = $6)
              AND ($7::TEXT IS NULL OR environment = $7)
              AND ($8::TEXT IS NULL OR decision = $8)
              AND ($9::TEXT IS NULL OR deployer = $9)
            ORDER BY created_at DESC
            LIMIT $10
            OFFSET $11
            "#,
        )
        .bind(org_id)
        .bind(query.authorization_id.as_deref())
        .bind(query.repository_full_name.as_deref())
        .bind(query.branch.as_deref())
        .bind(query.target_sha.as_deref())
        .bind(query.release_id.as_deref())
        .bind(query.environment.as_deref())
        .bind(query.decision.as_deref())
        .bind(query.deployer.as_deref())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let total = rows
            .first()
            .map(|row| row.get::<i64, _>("total_count"))
            .unwrap_or(0);
        let items = rows
            .iter()
            .map(deployment_gate_authorization_from_row)
            .collect();

        Ok((items, total))
    }

    pub async fn get_multi_repo_executive_governance(
        &self,
        org_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<MultiRepoExecutiveGovernanceRepository>, DbError> {
        let rows = sqlx::query(
            r#"
            WITH repo_names AS (
                SELECT repository_full_name
                FROM deployment_gate_authorizations
                WHERE org_id = $1::uuid
                UNION
                SELECT repository_full_name
                FROM change_risk_evaluations
                WHERE org_id = $1::uuid
            ),
            gate_stats AS (
                SELECT
                    repository_full_name,
                    COUNT(*)::BIGINT AS gate_count,
                    COUNT(*) FILTER (WHERE decision = 'blocked' OR blocking = TRUE)::BIGINT AS blocked_gate_count,
                    COUNT(*) FILTER (WHERE decision = 'advisory')::BIGINT AS advisory_gate_count,
                    COUNT(*) FILTER (WHERE break_glass_used = TRUE)::BIGINT AS break_glass_count
                FROM deployment_gate_authorizations
                WHERE org_id = $1::uuid
                GROUP BY repository_full_name
            ),
            latest_gate AS (
                SELECT DISTINCT ON (repository_full_name)
                    repository_full_name,
                    authorization_id,
                    decision,
                    ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
                FROM deployment_gate_authorizations
                WHERE org_id = $1::uuid
                ORDER BY repository_full_name, created_at DESC, authorization_id DESC
            ),
            risk_stats AS (
                SELECT
                    repository_full_name,
                    COUNT(*)::BIGINT AS change_risk_count,
                    COUNT(*) FILTER (WHERE risk_level = 'high')::BIGINT AS high_risk_count,
                    COUNT(*) FILTER (WHERE review_status = 'needs_review')::BIGINT AS needs_review_count
                FROM change_risk_evaluations
                WHERE org_id = $1::uuid
                GROUP BY repository_full_name
            ),
            latest_risk AS (
                SELECT DISTINCT ON (repository_full_name)
                    repository_full_name,
                    risk_level,
                    review_status,
                    ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
                FROM change_risk_evaluations
                WHERE org_id = $1::uuid
                ORDER BY repository_full_name, created_at DESC, evaluation_id DESC
            ),
            packet_repo AS (
                SELECT DISTINCT
                    packet.packet_id,
                    eval.repository_full_name
                FROM change_risk_cab_packets packet
                JOIN LATERAL jsonb_array_elements_text(packet.evaluation_ids_json) AS packet_eval(evaluation_id)
                    ON TRUE
                JOIN change_risk_evaluations eval
                    ON eval.org_id = packet.org_id
                   AND eval.evaluation_id = packet_eval.evaluation_id
                WHERE packet.org_id = $1::uuid
            ),
            packet_stats AS (
                SELECT
                    repository_full_name,
                    COUNT(DISTINCT packet_id)::BIGINT AS cab_packet_count
                FROM packet_repo
                GROUP BY repository_full_name
            ),
            manifest_repo AS (
                SELECT DISTINCT
                    manifest.manifest_id,
                    packet_repo.repository_full_name,
                    manifest.manifest_hash,
                    manifest.status,
                    manifest.created_at
                FROM change_risk_cab_decision_manifests manifest
                JOIN packet_repo
                    ON packet_repo.packet_id = manifest.cab_packet_id
                WHERE manifest.org_id = $1::uuid
            ),
            manifest_stats AS (
                SELECT
                    repository_full_name,
                    COUNT(*)::BIGINT AS cab_manifest_count,
                    COUNT(*) FILTER (WHERE status = 'active')::BIGINT AS active_manifest_count,
                    COUNT(*) FILTER (WHERE status = 'revoked')::BIGINT AS revoked_manifest_count
                FROM manifest_repo
                GROUP BY repository_full_name
            ),
            latest_manifest AS (
                SELECT DISTINCT ON (repository_full_name)
                    repository_full_name,
                    manifest_hash,
                    status,
                    ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
                FROM manifest_repo
                ORDER BY repository_full_name, created_at DESC, manifest_id DESC
            )
            SELECT
                repos.repository_full_name,
                COALESCE(gate_stats.gate_count, 0)::BIGINT AS gate_count,
                COALESCE(gate_stats.blocked_gate_count, 0)::BIGINT AS blocked_gate_count,
                COALESCE(gate_stats.advisory_gate_count, 0)::BIGINT AS advisory_gate_count,
                COALESCE(gate_stats.break_glass_count, 0)::BIGINT AS break_glass_count,
                latest_gate.authorization_id AS latest_gate_id,
                latest_gate.decision AS latest_gate_decision,
                latest_gate.created_at_ms AS latest_gate_created_at_ms,
                COALESCE(risk_stats.change_risk_count, 0)::BIGINT AS change_risk_count,
                COALESCE(risk_stats.high_risk_count, 0)::BIGINT AS high_risk_count,
                COALESCE(risk_stats.needs_review_count, 0)::BIGINT AS needs_review_count,
                latest_risk.risk_level AS latest_risk_level,
                latest_risk.review_status AS latest_review_status,
                latest_risk.created_at_ms AS latest_risk_created_at_ms,
                COALESCE(packet_stats.cab_packet_count, 0)::BIGINT AS cab_packet_count,
                COALESCE(manifest_stats.cab_manifest_count, 0)::BIGINT AS cab_manifest_count,
                COALESCE(manifest_stats.active_manifest_count, 0)::BIGINT AS active_manifest_count,
                COALESCE(manifest_stats.revoked_manifest_count, 0)::BIGINT AS revoked_manifest_count,
                latest_manifest.manifest_hash AS latest_manifest_hash,
                latest_manifest.status AS latest_manifest_status,
                latest_manifest.created_at_ms AS latest_manifest_created_at_ms
            FROM repo_names repos
            LEFT JOIN gate_stats ON gate_stats.repository_full_name = repos.repository_full_name
            LEFT JOIN latest_gate ON latest_gate.repository_full_name = repos.repository_full_name
            LEFT JOIN risk_stats ON risk_stats.repository_full_name = repos.repository_full_name
            LEFT JOIN latest_risk ON latest_risk.repository_full_name = repos.repository_full_name
            LEFT JOIN packet_stats ON packet_stats.repository_full_name = repos.repository_full_name
            LEFT JOIN manifest_stats ON manifest_stats.repository_full_name = repos.repository_full_name
            LEFT JOIN latest_manifest ON latest_manifest.repository_full_name = repos.repository_full_name
            ORDER BY
                COALESCE(gate_stats.blocked_gate_count, 0) DESC,
                COALESCE(risk_stats.high_risk_count, 0) DESC,
                COALESCE(risk_stats.needs_review_count, 0) DESC,
                repos.repository_full_name ASC
            LIMIT $2
            OFFSET $3
            "#,
        )
        .bind(org_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .iter()
            .map(multi_repo_executive_governance_repository_from_row)
            .collect())
    }

    pub async fn create_deployment_gate_break_glass_approval(
        &self,
        org_id: &str,
        approval_id: &str,
        payload: &CreateDeploymentGateBreakGlassApprovalRequest,
        approval_hash: &str,
        created_by: &str,
    ) -> Result<DeploymentGateBreakGlassApprovalRecord, DbError> {
        let row = sqlx::query(
            r#"
            INSERT INTO deployment_gate_break_glass_approvals (
                approval_id,
                org_id,
                release_id,
                repository_full_name,
                branch,
                target_sha,
                environment,
                ticket_id,
                evidence_packet_hash,
                evidence_packet_uri,
                reason,
                approver,
                approver_role,
                expires_at,
                approval_hash,
                metadata,
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
                COALESCE($13, 'incident_commander'),
                to_timestamp($14::DOUBLE PRECISION / 1000.0),
                $15,
                $16::jsonb,
                $17
            )
            RETURNING
                approval_id,
                org_id::text,
                release_id,
                repository_full_name,
                branch,
                target_sha,
                environment,
                ticket_id,
                evidence_packet_hash,
                evidence_packet_uri,
                reason,
                approver,
                approver_role,
                ROUND(EXTRACT(EPOCH FROM expires_at) * 1000)::BIGINT AS expires_at_ms,
                approval_hash,
                metadata,
                created_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            "#,
        )
        .bind(approval_id)
        .bind(org_id)
        .bind(&payload.release_id)
        .bind(&payload.repository_full_name)
        .bind(&payload.branch)
        .bind(&payload.target_sha)
        .bind(&payload.environment)
        .bind(payload.ticket_id.as_deref())
        .bind(&payload.evidence_packet_hash)
        .bind(payload.evidence_packet_uri.as_deref())
        .bind(&payload.reason)
        .bind(&payload.approver)
        .bind(payload.approver_role.as_deref())
        .bind(payload.expires_at)
        .bind(approval_hash)
        .bind(&payload.metadata)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await;

        match row {
            Ok(row) => Ok(deployment_gate_break_glass_approval_from_row(&row)),
            Err(e) if e.to_string().contains("duplicate") => {
                Err(DbError::Duplicate("approval_hash".to_string()))
            }
            Err(e) => Err(DbError::DatabaseError(e.to_string())),
        }
    }

    pub async fn list_deployment_gate_break_glass_approvals(
        &self,
        org_id: &str,
        query: &DeploymentGateBreakGlassApprovalQuery,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<DeploymentGateBreakGlassApprovalRecord>, i64), DbError> {
        let active_only = query.active_only.unwrap_or(false);
        let rows = sqlx::query(
            r#"
            SELECT
                approval_id,
                org_id::text,
                release_id,
                repository_full_name,
                branch,
                target_sha,
                environment,
                ticket_id,
                evidence_packet_hash,
                evidence_packet_uri,
                reason,
                approver,
                approver_role,
                ROUND(EXTRACT(EPOCH FROM expires_at) * 1000)::BIGINT AS expires_at_ms,
                approval_hash,
                metadata,
                created_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                COUNT(*) OVER() AS total_count
            FROM deployment_gate_break_glass_approvals
            WHERE org_id = $1::uuid
              AND ($2::TEXT IS NULL OR approval_id = $2)
              AND ($3::TEXT IS NULL OR repository_full_name = $3)
              AND ($4::TEXT IS NULL OR branch = $4)
              AND ($5::TEXT IS NULL OR target_sha = $5)
              AND ($6::TEXT IS NULL OR release_id = $6)
              AND ($7::TEXT IS NULL OR environment = $7)
              AND ($8::TEXT IS NULL OR evidence_packet_hash = $8)
              AND ($9::TEXT IS NULL OR approver = $9)
              AND ($10::BOOLEAN = FALSE OR expires_at > NOW())
            ORDER BY created_at DESC
            LIMIT $11
            OFFSET $12
            "#,
        )
        .bind(org_id)
        .bind(query.approval_id.as_deref())
        .bind(query.repository_full_name.as_deref())
        .bind(query.branch.as_deref())
        .bind(query.target_sha.as_deref())
        .bind(query.release_id.as_deref())
        .bind(query.environment.as_deref())
        .bind(query.evidence_packet_hash.as_deref())
        .bind(query.approver.as_deref())
        .bind(active_only)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let total = rows
            .first()
            .map(|row| row.get::<i64, _>("total_count"))
            .unwrap_or(0);
        let items = rows
            .iter()
            .map(deployment_gate_break_glass_approval_from_row)
            .collect();

        Ok((items, total))
    }
}
