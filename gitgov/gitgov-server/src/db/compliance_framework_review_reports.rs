use super::*;

fn compliance_framework_review_report_from_row(
    row: &PgRow,
) -> ComplianceFrameworkReviewReportRecord {
    ComplianceFrameworkReviewReportRecord {
        report_id: row.get("report_id"),
        org_id: row.get("org_id"),
        created_by_user_id: row.get("created_by_user_id"),
        mapping_id: row.get("mapping_id"),
        review_package_id: row.get("review_package_id"),
        evidence_export_id: row.get("evidence_export_id"),
        evidence_export_hash: row.get("evidence_export_hash"),
        mapping_hash: row.get("mapping_hash"),
        review_package_hash: row.get("review_package_hash"),
        framework_id: row.get("framework_id"),
        framework_version: row.get("framework_version"),
        framework_owner_type: row.get("framework_owner_type"),
        framework_review_status: row.get("framework_review_status"),
        pack_hash: row.get("pack_hash"),
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
    pub async fn list_compliance_framework_review_reports(
        &self,
        input: &ListComplianceFrameworkReviewReportsInput<'_>,
    ) -> Result<Vec<ComplianceFrameworkReviewReportRecord>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                report_id,
                org_id::text,
                created_by_user_id,
                mapping_id,
                review_package_id,
                evidence_export_id,
                evidence_export_hash,
                mapping_hash,
                review_package_hash,
                framework_id,
                framework_version,
                framework_owner_type,
                framework_review_status,
                pack_hash,
                format,
                artifact_hash,
                compliance_claim,
                regulatory_claim,
                requires_auditor_review,
                certification,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM downloaded_at) * 1000)::BIGINT AS downloaded_at_ms,
                error_message_safe
            FROM compliance_framework_review_reports
            WHERE org_id = $1::uuid
              AND ($2::text IS NULL OR framework_id = $2)
              AND ($3::text IS NULL OR mapping_id = $3)
              AND ($4::text IS NULL OR review_package_id = $4)
            ORDER BY created_at DESC, report_id DESC
            LIMIT $5
            "#,
        )
        .bind(input.org_id)
        .bind(input.framework_id)
        .bind(input.mapping_id)
        .bind(input.review_package_id)
        .bind(input.limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| compliance_framework_review_report_from_row(&row))
            .collect())
    }

    pub async fn create_compliance_framework_review_report(
        &self,
        input: &CreateComplianceFrameworkReviewReportInput<'_>,
    ) -> Result<ComplianceFrameworkReviewReportRecord, DbError> {
        let row = sqlx::query(
            r#"
            INSERT INTO compliance_framework_review_reports (
                report_id,
                org_id,
                created_by_user_id,
                mapping_id,
                review_package_id,
                evidence_export_id,
                evidence_export_hash,
                mapping_hash,
                review_package_hash,
                framework_id,
                framework_version,
                framework_owner_type,
                framework_review_status,
                pack_hash,
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
                $12,
                $13,
                $14,
                $15,
                $16,
                $17::jsonb,
                FALSE,
                FALSE,
                TRUE,
                FALSE
            )
            ON CONFLICT (report_id) DO UPDATE
            SET payload_json_redacted = EXCLUDED.payload_json_redacted,
                artifact_hash = EXCLUDED.artifact_hash,
                framework_owner_type = EXCLUDED.framework_owner_type,
                framework_review_status = EXCLUDED.framework_review_status,
                pack_hash = EXCLUDED.pack_hash,
                error_message_safe = NULL
            RETURNING
                report_id,
                org_id::text,
                created_by_user_id,
                mapping_id,
                review_package_id,
                evidence_export_id,
                evidence_export_hash,
                mapping_hash,
                review_package_hash,
                framework_id,
                framework_version,
                framework_owner_type,
                framework_review_status,
                pack_hash,
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
        .bind(input.report_id)
        .bind(input.org_id)
        .bind(input.created_by_user_id)
        .bind(input.mapping_id)
        .bind(input.review_package_id)
        .bind(input.evidence_export_id)
        .bind(input.evidence_export_hash)
        .bind(input.mapping_hash)
        .bind(input.review_package_hash)
        .bind(input.framework_id)
        .bind(input.framework_version)
        .bind(input.framework_owner_type)
        .bind(input.framework_review_status)
        .bind(input.pack_hash)
        .bind(input.format)
        .bind(input.artifact_hash)
        .bind(input.payload_json_redacted)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(compliance_framework_review_report_from_row(&row))
    }

    pub async fn get_compliance_framework_review_report(
        &self,
        org_id: &str,
        report_id: &str,
    ) -> Result<Option<ComplianceFrameworkReviewReportRecord>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                report_id,
                org_id::text,
                created_by_user_id,
                mapping_id,
                review_package_id,
                evidence_export_id,
                evidence_export_hash,
                mapping_hash,
                review_package_hash,
                framework_id,
                framework_version,
                framework_owner_type,
                framework_review_status,
                pack_hash,
                format,
                artifact_hash,
                compliance_claim,
                regulatory_claim,
                requires_auditor_review,
                certification,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM downloaded_at) * 1000)::BIGINT AS downloaded_at_ms,
                error_message_safe
            FROM compliance_framework_review_reports
            WHERE org_id = $1::uuid
              AND report_id = $2
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(report_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| compliance_framework_review_report_from_row(&row)))
    }

    pub async fn get_compliance_framework_review_report_payload(
        &self,
        org_id: &str,
        report_id: &str,
    ) -> Result<Option<serde_json::Value>, DbError> {
        let row = sqlx::query(
            r#"
            UPDATE compliance_framework_review_reports
            SET downloaded_at = NOW()
            WHERE org_id = $1::uuid
              AND report_id = $2
            RETURNING payload_json_redacted
            "#,
        )
        .bind(org_id)
        .bind(report_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| row.get("payload_json_redacted")))
    }
}
