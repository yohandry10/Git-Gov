use super::*;

fn compliance_review_package_from_row(row: &PgRow) -> ComplianceReviewPackageRecord {
    ComplianceReviewPackageRecord {
        review_package_id: row.get("review_package_id"),
        org_id: row.get("org_id"),
        created_by_user_id: row.get("created_by_user_id"),
        mapping_id: row.get("mapping_id"),
        evidence_export_id: row.get("evidence_export_id"),
        evidence_export_hash: row.get("evidence_export_hash"),
        mapping_hash: row.get("mapping_hash"),
        framework_id: row.get("framework_id"),
        framework_version: row.get("framework_version"),
        format: row.get("format"),
        artifact_hash: row.get("artifact_hash"),
        compliance_claim: row.get("compliance_claim"),
        regulatory_claim: row.get("regulatory_claim"),
        requires_auditor_review: row.get("requires_auditor_review"),
        certification: row.get("certification"),
        created_at: row.get("created_at_ms"),
        downloaded_at: row.get("downloaded_at_ms"),
        error_message_safe: row.get("error_message_safe"),
    }
}

impl Database {
    pub async fn create_compliance_review_package(
        &self,
        input: &CreateComplianceReviewPackageInput<'_>,
    ) -> Result<ComplianceReviewPackageRecord, DbError> {
        let row = sqlx::query(
            r#"
            INSERT INTO compliance_review_packages (
                review_package_id,
                org_id,
                created_by_user_id,
                mapping_id,
                evidence_export_id,
                evidence_export_hash,
                mapping_hash,
                framework_id,
                framework_version,
                format,
                artifact_hash,
                payload_json_redacted,
                compliance_claim,
                regulatory_claim,
                requires_auditor_review,
                certification
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
                FALSE,
                FALSE,
                TRUE,
                FALSE
            )
            ON CONFLICT (review_package_id) DO UPDATE
            SET payload_json_redacted = EXCLUDED.payload_json_redacted,
                artifact_hash = EXCLUDED.artifact_hash,
                error_message_safe = NULL
            RETURNING
                review_package_id,
                org_id::text,
                created_by_user_id,
                mapping_id,
                evidence_export_id,
                evidence_export_hash,
                mapping_hash,
                framework_id,
                framework_version,
                format,
                artifact_hash,
                compliance_claim,
                regulatory_claim,
                requires_auditor_review,
                certification,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM downloaded_at) * 1000)::BIGINT AS downloaded_at_ms,
                error_message_safe
            "#,
        )
        .bind(input.review_package_id)
        .bind(input.org_id)
        .bind(input.created_by_user_id)
        .bind(input.mapping_id)
        .bind(input.evidence_export_id)
        .bind(input.evidence_export_hash)
        .bind(input.mapping_hash)
        .bind(input.framework_id)
        .bind(input.framework_version)
        .bind(input.format)
        .bind(input.artifact_hash)
        .bind(input.payload_json_redacted)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(compliance_review_package_from_row(&row))
    }

    pub async fn get_compliance_review_package(
        &self,
        org_id: &str,
        review_package_id: &str,
    ) -> Result<Option<ComplianceReviewPackageRecord>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                review_package_id,
                org_id::text,
                created_by_user_id,
                mapping_id,
                evidence_export_id,
                evidence_export_hash,
                mapping_hash,
                framework_id,
                framework_version,
                format,
                artifact_hash,
                compliance_claim,
                regulatory_claim,
                requires_auditor_review,
                certification,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM downloaded_at) * 1000)::BIGINT AS downloaded_at_ms,
                error_message_safe
            FROM compliance_review_packages
            WHERE org_id = $1::uuid
              AND review_package_id = $2
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(review_package_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| compliance_review_package_from_row(&row)))
    }

    pub async fn get_compliance_review_package_payload(
        &self,
        org_id: &str,
        review_package_id: &str,
    ) -> Result<Option<serde_json::Value>, DbError> {
        let row = sqlx::query(
            r#"
            UPDATE compliance_review_packages
            SET downloaded_at = NOW()
            WHERE org_id = $1::uuid
              AND review_package_id = $2
            RETURNING payload_json_redacted
            "#,
        )
        .bind(org_id)
        .bind(review_package_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| row.get("payload_json_redacted")))
    }
}
