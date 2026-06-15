use super::*;

fn compliance_framework_review_report_pdf_export_from_row(
    row: &PgRow,
) -> ComplianceFrameworkReviewReportPdfExportRecord {
    ComplianceFrameworkReviewReportPdfExportRecord {
        pdf_export_id: row.get("pdf_export_id"),
        org_id: row.get("org_id"),
        report_id: row.get("report_id"),
        manifest_id: row.get("manifest_id"),
        created_by_user_id: row.get("created_by_user_id"),
        source_report_hash: row.get("source_report_hash"),
        manifest_hash: row.get("manifest_hash"),
        pdf_artifact_hash: row.get("pdf_artifact_hash"),
        content_type: row.get("content_type"),
        page_count: row.get("page_count"),
        compliance_claim: row.get("compliance_claim"),
        regulatory_claim: row.get("regulatory_claim"),
        requires_auditor_review: row.get("requires_auditor_review"),
        certification: row.get("certification"),
        created_at: row.get("created_at_ms"),
        downloaded_at: row.get("downloaded_at_ms"),
    }
}

impl Database {
    pub async fn get_compliance_framework_review_report_payload_redacted(
        &self,
        org_id: &str,
        report_id: &str,
    ) -> Result<Option<serde_json::Value>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT payload_json_redacted
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

        Ok(row.map(|row| row.get("payload_json_redacted")))
    }

    pub async fn get_latest_compliance_framework_review_report_manifest(
        &self,
        org_id: &str,
        report_id: &str,
    ) -> Result<Option<ComplianceFrameworkReviewReportProvenanceManifestRecord>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                manifest_id,
                org_id::text,
                report_id,
                generated_by_user_id,
                manifest_hash,
                previous_manifest_hash,
                signature_algorithm,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            FROM compliance_framework_review_report_manifests
            WHERE org_id = $1::uuid
              AND report_id = $2
            ORDER BY created_at DESC, manifest_id DESC
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(report_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(
            |row| ComplianceFrameworkReviewReportProvenanceManifestRecord {
                manifest_id: row.get("manifest_id"),
                org_id: row.get("org_id"),
                report_id: row.get("report_id"),
                generated_by_user_id: row.get("generated_by_user_id"),
                manifest_hash: row.get("manifest_hash"),
                previous_manifest_hash: row.get("previous_manifest_hash"),
                signature_algorithm: row.get("signature_algorithm"),
                created_at: row.get("created_at_ms"),
            },
        ))
    }

    pub async fn get_compliance_framework_review_report_manifest(
        &self,
        org_id: &str,
        report_id: &str,
        manifest_id: &str,
    ) -> Result<Option<ComplianceFrameworkReviewReportProvenanceManifestRecord>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                manifest_id,
                org_id::text,
                report_id,
                generated_by_user_id,
                manifest_hash,
                previous_manifest_hash,
                signature_algorithm,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            FROM compliance_framework_review_report_manifests
            WHERE org_id = $1::uuid
              AND report_id = $2
              AND manifest_id = $3
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(report_id)
        .bind(manifest_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(
            |row| ComplianceFrameworkReviewReportProvenanceManifestRecord {
                manifest_id: row.get("manifest_id"),
                org_id: row.get("org_id"),
                report_id: row.get("report_id"),
                generated_by_user_id: row.get("generated_by_user_id"),
                manifest_hash: row.get("manifest_hash"),
                previous_manifest_hash: row.get("previous_manifest_hash"),
                signature_algorithm: row.get("signature_algorithm"),
                created_at: row.get("created_at_ms"),
            },
        ))
    }

    pub async fn create_compliance_framework_review_report_pdf_export(
        &self,
        input: &CreateComplianceFrameworkReviewReportPdfExportInput<'_>,
    ) -> Result<ComplianceFrameworkReviewReportPdfExportRecord, DbError> {
        sqlx::query(
            r#"
            INSERT INTO compliance_framework_review_report_pdf_exports (
                pdf_export_id,
                org_id,
                report_id,
                manifest_id,
                created_by_user_id,
                source_report_hash,
                manifest_hash,
                pdf_artifact_hash,
                content_type,
                page_count,
                pdf_bytes,
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
                FALSE,
                FALSE,
                TRUE,
                FALSE
            )
            ON CONFLICT (pdf_export_id) DO NOTHING
            "#,
        )
        .bind(input.pdf_export_id)
        .bind(input.org_id)
        .bind(input.report_id)
        .bind(input.manifest_id)
        .bind(input.created_by_user_id)
        .bind(input.source_report_hash)
        .bind(input.manifest_hash)
        .bind(input.pdf_artifact_hash)
        .bind(input.content_type)
        .bind(input.page_count)
        .bind(input.pdf_bytes)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        self.get_compliance_framework_review_report_pdf_export(
            input.org_id,
            input.report_id,
            input.pdf_export_id,
        )
        .await?
        .ok_or_else(|| DbError::NotFound("framework review report PDF export".to_string()))
    }

    pub async fn get_latest_compliance_framework_review_report_pdf_export(
        &self,
        org_id: &str,
        report_id: &str,
    ) -> Result<Option<ComplianceFrameworkReviewReportPdfExportRecord>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                pdf_export_id,
                org_id::text,
                report_id,
                manifest_id,
                created_by_user_id,
                source_report_hash,
                manifest_hash,
                pdf_artifact_hash,
                content_type,
                page_count,
                compliance_claim,
                regulatory_claim,
                requires_auditor_review,
                certification,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM downloaded_at) * 1000)::BIGINT AS downloaded_at_ms
            FROM compliance_framework_review_report_pdf_exports
            WHERE org_id = $1::uuid
              AND report_id = $2
            ORDER BY created_at DESC, pdf_export_id DESC
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(report_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| compliance_framework_review_report_pdf_export_from_row(&row)))
    }

    pub async fn get_compliance_framework_review_report_pdf_export(
        &self,
        org_id: &str,
        report_id: &str,
        pdf_export_id: &str,
    ) -> Result<Option<ComplianceFrameworkReviewReportPdfExportRecord>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                pdf_export_id,
                org_id::text,
                report_id,
                manifest_id,
                created_by_user_id,
                source_report_hash,
                manifest_hash,
                pdf_artifact_hash,
                content_type,
                page_count,
                compliance_claim,
                regulatory_claim,
                requires_auditor_review,
                certification,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM downloaded_at) * 1000)::BIGINT AS downloaded_at_ms
            FROM compliance_framework_review_report_pdf_exports
            WHERE org_id = $1::uuid
              AND report_id = $2
              AND pdf_export_id = $3
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(report_id)
        .bind(pdf_export_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| compliance_framework_review_report_pdf_export_from_row(&row)))
    }

    pub async fn download_compliance_framework_review_report_pdf_export(
        &self,
        org_id: &str,
        report_id: &str,
        pdf_export_id: Option<&str>,
    ) -> Result<Option<(ComplianceFrameworkReviewReportPdfExportRecord, Vec<u8>)>, DbError> {
        let row = sqlx::query(
            r#"
            WITH selected AS (
                SELECT pdf_export_id
                FROM compliance_framework_review_report_pdf_exports
                WHERE org_id = $1::uuid
                  AND report_id = $2
                  AND ($3::text IS NULL OR pdf_export_id = $3)
                ORDER BY created_at DESC, pdf_export_id DESC
                LIMIT 1
            )
            UPDATE compliance_framework_review_report_pdf_exports e
            SET downloaded_at = NOW()
            FROM selected
            WHERE e.pdf_export_id = selected.pdf_export_id
            RETURNING
                e.pdf_export_id,
                e.org_id::text,
                e.report_id,
                e.manifest_id,
                e.created_by_user_id,
                e.source_report_hash,
                e.manifest_hash,
                e.pdf_artifact_hash,
                e.content_type,
                e.page_count,
                e.compliance_claim,
                e.regulatory_claim,
                e.requires_auditor_review,
                e.certification,
                ROUND(EXTRACT(EPOCH FROM e.created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM e.downloaded_at) * 1000)::BIGINT AS downloaded_at_ms,
                e.pdf_bytes
            "#,
        )
        .bind(org_id)
        .bind(report_id)
        .bind(pdf_export_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| {
            let record = compliance_framework_review_report_pdf_export_from_row(&row);
            let bytes: Vec<u8> = row.get("pdf_bytes");
            (record, bytes)
        }))
    }
}
