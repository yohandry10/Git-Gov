use super::*;

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

    // ========================================================================
    // REPOSITORIES
    // ========================================================================
}
