use super::*;

fn string_vec_from_json(value: serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn first_governed_repo_setup_from_row(row: &PgRow) -> FirstGovernedRepoSetupRecord {
    FirstGovernedRepoSetupRecord {
        run_id: row.get("run_id"),
        org_id: row.get("org_id"),
        status: row.get("status"),
        goal: row.get("goal"),
        repository_full_name: row.get("repository_full_name"),
        default_branch: row.get("default_branch"),
        selected_providers: string_vec_from_json(row.get("selected_providers")),
        selected_modules: string_vec_from_json(row.get("selected_modules")),
        policy_preset: row.get("policy_preset"),
        baseline: row.get("baseline"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
        created_at: row.get("created_at_ms"),
        updated_at: row.get("updated_at_ms"),
        completed_at: row.get("completed_at_ms"),
    }
}

impl Database {
    pub async fn get_enterprise_adoption_profile(
        &self,
        org_id: &str,
    ) -> Result<Option<EnterpriseAdoptionProfileRecord>, DbError> {
        let result = sqlx::query(
            r#"
            SELECT
                org_id::text,
                profile,
                updated_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM updated_at) * 1000)::BIGINT AS updated_at_ms
            FROM enterprise_adoption_profiles
            WHERE org_id = $1::uuid
            "#,
        )
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(result.map(|row| EnterpriseAdoptionProfileRecord {
            org_id: row.get("org_id"),
            profile: row.get("profile"),
            updated_by: row.get("updated_by"),
            created_at: row.get("created_at_ms"),
            updated_at: row.get("updated_at_ms"),
        }))
    }

    pub async fn upsert_enterprise_adoption_profile(
        &self,
        org_id: &str,
        profile: &serde_json::Value,
        updated_by: &str,
    ) -> Result<EnterpriseAdoptionProfileRecord, DbError> {
        let row = sqlx::query(
            r#"
            INSERT INTO enterprise_adoption_profiles (org_id, profile, updated_by)
            VALUES ($1::uuid, $2::jsonb, $3)
            ON CONFLICT (org_id) DO UPDATE SET
                profile = EXCLUDED.profile,
                updated_by = EXCLUDED.updated_by,
                updated_at = NOW()
            RETURNING
                org_id::text,
                profile,
                updated_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM updated_at) * 1000)::BIGINT AS updated_at_ms
            "#,
        )
        .bind(org_id)
        .bind(profile)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(EnterpriseAdoptionProfileRecord {
            org_id: row.get("org_id"),
            profile: row.get("profile"),
            updated_by: row.get("updated_by"),
            created_at: row.get("created_at_ms"),
            updated_at: row.get("updated_at_ms"),
        })
    }

    pub async fn get_enterprise_onboarding_checklist_tracking(
        &self,
        org_id: &str,
    ) -> Result<Option<EnterpriseOnboardingChecklistTrackingRecord>, DbError> {
        let result = sqlx::query(
            r#"
            SELECT
                org_id::text,
                tracking,
                updated_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM updated_at) * 1000)::BIGINT AS updated_at_ms
            FROM enterprise_onboarding_checklist_tracking
            WHERE org_id = $1::uuid
            "#,
        )
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(
            result.map(|row| EnterpriseOnboardingChecklistTrackingRecord {
                org_id: row.get("org_id"),
                tracking: row.get("tracking"),
                updated_by: row.get("updated_by"),
                created_at: row.get("created_at_ms"),
                updated_at: row.get("updated_at_ms"),
            }),
        )
    }

    pub async fn upsert_enterprise_onboarding_checklist_tracking(
        &self,
        org_id: &str,
        tracking: &serde_json::Value,
        updated_by: &str,
    ) -> Result<EnterpriseOnboardingChecklistTrackingRecord, DbError> {
        let row = sqlx::query(
            r#"
            INSERT INTO enterprise_onboarding_checklist_tracking (org_id, tracking, updated_by)
            VALUES ($1::uuid, $2::jsonb, $3)
            ON CONFLICT (org_id) DO UPDATE SET
                tracking = EXCLUDED.tracking,
                updated_by = EXCLUDED.updated_by,
                updated_at = NOW()
            RETURNING
                org_id::text,
                tracking,
                updated_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM updated_at) * 1000)::BIGINT AS updated_at_ms
            "#,
        )
        .bind(org_id)
        .bind(tracking)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(EnterpriseOnboardingChecklistTrackingRecord {
            org_id: row.get("org_id"),
            tracking: row.get("tracking"),
            updated_by: row.get("updated_by"),
            created_at: row.get("created_at_ms"),
            updated_at: row.get("updated_at_ms"),
        })
    }

    pub async fn get_first_governed_repo_setup(
        &self,
        org_id: &str,
    ) -> Result<Option<FirstGovernedRepoSetupRecord>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                run_id::text,
                org_id::text,
                status,
                goal,
                repository_full_name,
                default_branch,
                selected_providers,
                selected_modules,
                policy_preset,
                baseline,
                created_by,
                updated_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM updated_at) * 1000)::BIGINT AS updated_at_ms,
                ROUND(EXTRACT(EPOCH FROM completed_at) * 1000)::BIGINT AS completed_at_ms
            FROM enterprise_first_governed_repo_setups
            WHERE org_id = $1::uuid
            "#,
        )
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| first_governed_repo_setup_from_row(&row)))
    }

    pub async fn upsert_first_governed_repo_setup(
        &self,
        org_id: &str,
        payload: &UpsertFirstGovernedRepoSetupRequest,
        updated_by: &str,
    ) -> Result<FirstGovernedRepoSetupRecord, DbError> {
        let selected_providers = serde_json::to_value(&payload.selected_providers)
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;
        let selected_modules = serde_json::to_value(&payload.selected_modules)
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;
        let candidate_run_id = uuid::Uuid::new_v4().to_string();
        let status = payload.status.as_deref().unwrap_or("draft");

        let row = sqlx::query(
            r#"
            WITH existing AS (
                SELECT run_id
                FROM enterprise_first_governed_repo_setups
                WHERE org_id = $1::uuid
            )
            INSERT INTO enterprise_first_governed_repo_setups (
                org_id,
                run_id,
                status,
                goal,
                repository_full_name,
                default_branch,
                selected_providers,
                selected_modules,
                policy_preset,
                baseline,
                created_by,
                updated_by,
                completed_at
            )
            VALUES (
                $1::uuid,
                COALESCE((SELECT run_id FROM existing), $2::uuid),
                $3,
                $4,
                $5,
                $6,
                $7::jsonb,
                $8::jsonb,
                $9,
                $10::jsonb,
                $11,
                $11,
                CASE WHEN $3 = 'completed' THEN NOW() ELSE NULL END
            )
            ON CONFLICT (org_id) DO UPDATE SET
                status = EXCLUDED.status,
                goal = EXCLUDED.goal,
                repository_full_name = EXCLUDED.repository_full_name,
                default_branch = EXCLUDED.default_branch,
                selected_providers = EXCLUDED.selected_providers,
                selected_modules = EXCLUDED.selected_modules,
                policy_preset = EXCLUDED.policy_preset,
                baseline = EXCLUDED.baseline,
                updated_by = EXCLUDED.updated_by,
                updated_at = NOW(),
                completed_at = CASE
                    WHEN EXCLUDED.status = 'completed'
                        THEN COALESCE(enterprise_first_governed_repo_setups.completed_at, NOW())
                    ELSE NULL
                END
            RETURNING
                run_id::text,
                org_id::text,
                status,
                goal,
                repository_full_name,
                default_branch,
                selected_providers,
                selected_modules,
                policy_preset,
                baseline,
                created_by,
                updated_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM updated_at) * 1000)::BIGINT AS updated_at_ms,
                ROUND(EXTRACT(EPOCH FROM completed_at) * 1000)::BIGINT AS completed_at_ms
            "#,
        )
        .bind(org_id)
        .bind(&candidate_run_id)
        .bind(status)
        .bind(&payload.goal)
        .bind(&payload.repository_full_name)
        .bind(&payload.default_branch)
        .bind(&selected_providers)
        .bind(&selected_modules)
        .bind(&payload.policy_preset)
        .bind(&payload.baseline)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(first_governed_repo_setup_from_row(&row))
    }

    pub async fn create_enterprise_release_approval(
        &self,
        org_id: &str,
        approval_id: &str,
        payload: &CreateEnterpriseReleaseApprovalRequest,
        approval_hash: &str,
        created_by: &str,
    ) -> Result<EnterpriseReleaseApprovalRecord, DbError> {
        let result = sqlx::query(
            r#"
            INSERT INTO enterprise_release_approvals (
                id,
                org_id,
                release_id,
                repository_full_name,
                branch,
                target_sha,
                environment,
                decision,
                approver,
                ticket_id,
                evidence_packet_hash,
                evidence_packet_uri,
                evidence_summary,
                risk_severity,
                risk_acceptance_reason,
                expires_at,
                approval_hash,
                created_by
            )
            VALUES (
                $1::uuid,
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
                $14,
                $15,
                CASE
                    WHEN $16::BIGINT IS NULL THEN NULL
                    ELSE to_timestamp(($16::BIGINT)::double precision / 1000.0)
                END,
                $17,
                $18
            )
            RETURNING
                id::text,
                org_id::text,
                release_id,
                repository_full_name,
                branch,
                target_sha,
                environment,
                decision,
                approver,
                ticket_id,
                evidence_packet_hash,
                evidence_packet_uri,
                evidence_summary,
                risk_severity,
                risk_acceptance_reason,
                ROUND(EXTRACT(EPOCH FROM expires_at) * 1000)::BIGINT AS expires_at_ms,
                approval_hash,
                created_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            "#,
        )
        .bind(approval_id)
        .bind(org_id)
        .bind(&payload.release_id)
        .bind(&payload.repository_full_name)
        .bind(payload.branch.as_deref())
        .bind(payload.target_sha.as_deref())
        .bind(&payload.environment)
        .bind(&payload.decision)
        .bind(&payload.approver)
        .bind(payload.ticket_id.as_deref())
        .bind(payload.evidence_packet_hash.as_deref())
        .bind(payload.evidence_packet_uri.as_deref())
        .bind(&payload.evidence_summary)
        .bind(payload.risk_severity.as_deref().unwrap_or("none"))
        .bind(payload.risk_acceptance_reason.as_deref())
        .bind(payload.expires_at)
        .bind(approval_hash)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(row) => Ok(enterprise_release_approval_from_row(&row)),
            Err(e) if e.to_string().contains("duplicate") => {
                Err(DbError::Duplicate("approval_hash".to_string()))
            }
            Err(e) => Err(DbError::DatabaseError(e.to_string())),
        }
    }

    pub async fn list_enterprise_release_approvals(
        &self,
        org_id: &str,
        query: &EnterpriseReleaseApprovalQuery,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<EnterpriseReleaseApprovalRecord>, i64), DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id::text,
                org_id::text,
                release_id,
                repository_full_name,
                branch,
                target_sha,
                environment,
                decision,
                approver,
                ticket_id,
                evidence_packet_hash,
                evidence_packet_uri,
                evidence_summary,
                risk_severity,
                risk_acceptance_reason,
                ROUND(EXTRACT(EPOCH FROM expires_at) * 1000)::BIGINT AS expires_at_ms,
                approval_hash,
                created_by,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                COUNT(*) OVER() AS total_count
            FROM enterprise_release_approvals
            WHERE org_id = $1::uuid
              AND ($2::TEXT IS NULL OR repository_full_name = $2)
              AND ($3::TEXT IS NULL OR release_id = $3)
              AND ($4::TEXT IS NULL OR environment = $4)
              AND ($5::TEXT IS NULL OR decision = $5)
              AND ($8::TEXT IS NULL OR branch = $8)
              AND ($9::TEXT IS NULL OR target_sha = $9)
              AND ($10::TEXT IS NULL OR evidence_packet_hash = $10)
            ORDER BY created_at DESC
            LIMIT $6
            OFFSET $7
            "#,
        )
        .bind(org_id)
        .bind(query.repository_full_name.as_deref())
        .bind(query.release_id.as_deref())
        .bind(query.environment.as_deref())
        .bind(query.decision.as_deref())
        .bind(limit)
        .bind(offset)
        .bind(query.branch.as_deref())
        .bind(query.target_sha.as_deref())
        .bind(query.evidence_packet_hash.as_deref())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let total = rows
            .first()
            .map(|row| row.get::<i64, _>("total_count"))
            .unwrap_or(0);
        let items = rows
            .iter()
            .map(enterprise_release_approval_from_row)
            .collect();

        Ok((items, total))
    }

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
                $21::jsonb,
                $22::jsonb,
                $23::jsonb,
                $24
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

    // ========================================================================
    // REPOSITORIES
    // ========================================================================
}
