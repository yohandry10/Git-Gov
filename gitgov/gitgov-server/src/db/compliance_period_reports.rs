use super::*;

fn compliance_period_report_source_ids(row: &PgRow) -> Vec<String> {
    let value: serde_json::Value = row.get("source_report_ids");
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn compliance_period_report_from_row(row: &PgRow) -> CompliancePeriodReportRecord {
    CompliancePeriodReportRecord {
        period_report_id: row.get("period_report_id"),
        org_id: row.get("org_id"),
        created_by_user_id: row.get("created_by_user_id"),
        framework_id: row.get("framework_id"),
        date_range_start: row.get("date_range_start_ms"),
        date_range_end: row.get("date_range_end_ms"),
        report_count: row.get("report_count"),
        source_report_ids: compliance_period_report_source_ids(row),
        format: row.get("format"),
        status: row.get("status"),
        artifact_hash: row.get("artifact_hash"),
        compliance_claim: row.get("compliance_claim"),
        regulatory_claim: row.get("regulatory_claim"),
        requires_auditor_review: row.get("requires_auditor_review"),
        certification: row.get("certification"),
        review_status: row.get("review_status"),
        reviewed_by_user_id: row.get("reviewed_by_user_id"),
        reviewed_at: row.get("reviewed_at_ms"),
        review_notes_safe: row.get("review_notes_safe"),
        created_at: row.get("created_at_ms"),
        retention_status: row.get("retention_status"),
        retention_until: row.get("retention_until_ms"),
        download_count: row.get("download_count"),
        last_downloaded_at: row.get("last_downloaded_at_ms"),
        archived_at: row.get("archived_at_ms"),
        downloaded_at: row.get("downloaded_at_ms"),
        error_message_safe: row.get("error_message_safe"),
    }
}

fn compliance_period_report_access_log_from_row(
    row: &PgRow,
) -> CompliancePeriodReportAccessLogRecord {
    CompliancePeriodReportAccessLogRecord {
        access_log_id: row.get("access_log_id"),
        org_id: row.get("org_id"),
        period_report_id: row.get("period_report_id"),
        actor_client_id: row.get("actor_client_id"),
        action: row.get("action"),
        artifact_type: row.get("artifact_type"),
        artifact_id: row.get("artifact_id"),
        artifact_hash: row.get("artifact_hash"),
        metadata: row.get("metadata"),
        created_at: row.get("created_at_ms"),
    }
}

fn compliance_period_source_report_from_row(row: &PgRow) -> CompliancePeriodSourceReport {
    CompliancePeriodSourceReport {
        report: ComplianceFrameworkReviewReportRecord {
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
            review_status: row.get("review_status"),
            reviewed_by_user_id: row.get("reviewed_by_user_id"),
            reviewed_at: row.get("reviewed_at_ms"),
            review_notes_safe: row.get("review_notes_safe"),
            created_at: row.get("created_at_ms"),
            downloaded_at: row.get("downloaded_at_ms"),
            error_message_safe: row.get("error_message_safe"),
        },
        payload_json_redacted: row.get("payload_json_redacted"),
        latest_manifest_id: row.get("latest_manifest_id"),
        latest_manifest_hash: row.get("latest_manifest_hash"),
        latest_manifest_created_at: row.get("latest_manifest_created_at_ms"),
        manifest_count: row.get("manifest_count"),
    }
}

fn compliance_period_report_pdf_export_from_row(
    row: &PgRow,
) -> CompliancePeriodReportPdfExportRecord {
    CompliancePeriodReportPdfExportRecord {
        pdf_export_id: row.get("pdf_export_id"),
        org_id: row.get("org_id"),
        period_report_id: row.get("period_report_id"),
        created_by_user_id: row.get("created_by_user_id"),
        source_period_report_hash: row.get("source_period_report_hash"),
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

fn compliance_period_report_provenance_manifest_from_row(
    row: &PgRow,
) -> CompliancePeriodReportProvenanceManifestRecord {
    CompliancePeriodReportProvenanceManifestRecord {
        manifest_id: row.get("manifest_id"),
        org_id: row.get("org_id"),
        period_report_id: row.get("period_report_id"),
        generated_by_user_id: row.get("generated_by_user_id"),
        manifest_hash: row.get("manifest_hash"),
        previous_manifest_hash: row.get("previous_manifest_hash"),
        signature_algorithm: row.get("signature_algorithm"),
        created_at: row.get("created_at_ms"),
    }
}

fn compliance_period_report_profile_from_row(row: &PgRow) -> CompliancePeriodReportProfileRecord {
    CompliancePeriodReportProfileRecord {
        profile_id: row.get("profile_id"),
        org_id: row.get("org_id"),
        created_by_user_id: row.get("created_by_user_id"),
        updated_by_user_id: row.get("updated_by_user_id"),
        name: row.get("name"),
        period_type: row.get("period_type"),
        framework_id: row.get("framework_id"),
        framework_owner_type: row.get("framework_owner_type"),
        include_pdf: row.get("include_pdf"),
        include_manifest: row.get("include_manifest"),
        retention_days: row.get("retention_days"),
        filters: row.get("filters"),
        status: row.get("status"),
        run_count: row.get("run_count"),
        last_run_at: row.get("last_run_at_ms"),
        last_period_report_id: row.get("last_period_report_id"),
        last_pdf_export_id: row.get("last_pdf_export_id"),
        last_manifest_id: row.get("last_manifest_id"),
        archived_at: row.get("archived_at_ms"),
        created_at: row.get("created_at_ms"),
        updated_at: row.get("updated_at_ms"),
    }
}

impl Database {
    pub async fn list_reviewed_compliance_framework_review_reports_for_period(
        &self,
        org_id: &str,
        date_range_start: chrono::DateTime<chrono::Utc>,
        date_range_end: chrono::DateTime<chrono::Utc>,
        framework_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<CompliancePeriodSourceReport>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                r.report_id,
                r.org_id::text,
                r.created_by_user_id,
                r.mapping_id,
                r.review_package_id,
                r.evidence_export_id,
                r.evidence_export_hash,
                r.mapping_hash,
                r.review_package_hash,
                r.framework_id,
                r.framework_version,
                r.framework_owner_type,
                r.framework_review_status,
                r.pack_hash,
                r.format,
                r.artifact_hash,
                r.compliance_claim,
                r.regulatory_claim,
                r.requires_auditor_review,
                r.certification,
                r.review_status,
                r.reviewed_by_user_id,
                ROUND(EXTRACT(EPOCH FROM r.reviewed_at) * 1000)::BIGINT AS reviewed_at_ms,
                r.review_notes_safe,
                ROUND(EXTRACT(EPOCH FROM r.created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM r.downloaded_at) * 1000)::BIGINT AS downloaded_at_ms,
                r.error_message_safe,
                r.payload_json_redacted,
                latest_manifest.manifest_id AS latest_manifest_id,
                latest_manifest.manifest_hash AS latest_manifest_hash,
                ROUND(EXTRACT(EPOCH FROM latest_manifest.created_at) * 1000)::BIGINT
                    AS latest_manifest_created_at_ms,
                COALESCE(manifest_counts.manifest_count, 0)::BIGINT AS manifest_count
            FROM compliance_framework_review_reports r
            LEFT JOIN LATERAL (
                SELECT manifest_id, manifest_hash, created_at
                FROM compliance_framework_review_report_manifests m
                WHERE m.org_id = r.org_id
                  AND m.report_id = r.report_id
                ORDER BY m.created_at DESC, m.manifest_id DESC
                LIMIT 1
            ) latest_manifest ON TRUE
            LEFT JOIN LATERAL (
                SELECT COUNT(*)::BIGINT AS manifest_count
                FROM compliance_framework_review_report_manifests m
                WHERE m.org_id = r.org_id
                  AND m.report_id = r.report_id
            ) manifest_counts ON TRUE
            WHERE r.org_id = $1::uuid
              AND r.review_status = 'reviewed'
              AND r.created_at >= $2
              AND r.created_at < $3
              AND ($4::text IS NULL OR r.framework_id = $4)
              AND r.compliance_claim = FALSE
              AND r.regulatory_claim = FALSE
              AND r.certification = FALSE
              AND r.requires_auditor_review = TRUE
            ORDER BY r.created_at ASC, r.report_id ASC
            LIMIT $5
            "#,
        )
        .bind(org_id)
        .bind(date_range_start)
        .bind(date_range_end)
        .bind(framework_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| compliance_period_source_report_from_row(&row))
            .collect())
    }

    pub async fn create_compliance_period_report(
        &self,
        input: &CreateCompliancePeriodReportInput<'_>,
    ) -> Result<CompliancePeriodReportRecord, DbError> {
        let row = sqlx::query(
            r#"
            INSERT INTO compliance_period_reports (
                period_report_id,
                org_id,
                created_by_user_id,
                framework_id,
                date_range_start,
                date_range_end,
                report_count,
                source_report_ids,
                format,
                status,
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
                $8::jsonb,
                $9,
                $10,
                $11,
                $12::jsonb,
                FALSE,
                FALSE,
                TRUE,
                FALSE
            )
            RETURNING
                period_report_id,
                org_id::text,
                created_by_user_id,
                framework_id,
                ROUND(EXTRACT(EPOCH FROM date_range_start) * 1000)::BIGINT AS date_range_start_ms,
                ROUND(EXTRACT(EPOCH FROM date_range_end) * 1000)::BIGINT AS date_range_end_ms,
                report_count,
                source_report_ids,
                format,
                status,
                artifact_hash,
                compliance_claim,
                regulatory_claim,
                requires_auditor_review,
                certification,
                review_status,
                reviewed_by_user_id,
                ROUND(EXTRACT(EPOCH FROM reviewed_at) * 1000)::BIGINT AS reviewed_at_ms,
                review_notes_safe,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                CASE
                    WHEN retention_status = 'active' AND retention_until < NOW() THEN 'retention_expired'
                    ELSE retention_status
                END AS retention_status,
                ROUND(EXTRACT(EPOCH FROM retention_until) * 1000)::BIGINT AS retention_until_ms,
                download_count,
                ROUND(EXTRACT(EPOCH FROM last_downloaded_at) * 1000)::BIGINT AS last_downloaded_at_ms,
                ROUND(EXTRACT(EPOCH FROM archived_at) * 1000)::BIGINT AS archived_at_ms,
                ROUND(EXTRACT(EPOCH FROM downloaded_at) * 1000)::BIGINT AS downloaded_at_ms,
                error_message_safe
            "#,
        )
        .bind(input.period_report_id)
        .bind(input.org_id)
        .bind(input.created_by_user_id)
        .bind(input.framework_id)
        .bind(input.date_range_start)
        .bind(input.date_range_end)
        .bind(input.report_count)
        .bind(input.source_report_ids)
        .bind(input.format)
        .bind(input.status)
        .bind(input.artifact_hash)
        .bind(input.payload_json_redacted)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(compliance_period_report_from_row(&row))
    }

    pub async fn list_compliance_period_reports(
        &self,
        input: &ListCompliancePeriodReportsInput<'_>,
    ) -> Result<Vec<CompliancePeriodReportRecord>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                period_report_id,
                org_id::text,
                created_by_user_id,
                framework_id,
                ROUND(EXTRACT(EPOCH FROM date_range_start) * 1000)::BIGINT AS date_range_start_ms,
                ROUND(EXTRACT(EPOCH FROM date_range_end) * 1000)::BIGINT AS date_range_end_ms,
                report_count,
                source_report_ids,
                format,
                status,
                artifact_hash,
                compliance_claim,
                regulatory_claim,
                requires_auditor_review,
                certification,
                review_status,
                reviewed_by_user_id,
                ROUND(EXTRACT(EPOCH FROM reviewed_at) * 1000)::BIGINT AS reviewed_at_ms,
                review_notes_safe,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                CASE
                    WHEN retention_status = 'active' AND retention_until < NOW() THEN 'retention_expired'
                    ELSE retention_status
                END AS retention_status,
                ROUND(EXTRACT(EPOCH FROM retention_until) * 1000)::BIGINT AS retention_until_ms,
                download_count,
                ROUND(EXTRACT(EPOCH FROM last_downloaded_at) * 1000)::BIGINT AS last_downloaded_at_ms,
                ROUND(EXTRACT(EPOCH FROM archived_at) * 1000)::BIGINT AS archived_at_ms,
                ROUND(EXTRACT(EPOCH FROM downloaded_at) * 1000)::BIGINT AS downloaded_at_ms,
                error_message_safe
            FROM compliance_period_reports p
            WHERE p.org_id = $1::uuid
              AND ($2::text IS NULL OR p.framework_id = $2)
              AND (
                $3::text IS NULL
                OR NOT EXISTS (
                    SELECT 1
                    FROM jsonb_array_elements_text(p.source_report_ids) AS source_report(report_id)
                    WHERE EXISTS (
                        SELECT 1
                        FROM compliance_framework_review_report_assignments a
                        WHERE a.org_id = p.org_id
                          AND a.report_id = source_report.report_id
                          AND a.assignment_status = 'active'
                    )
                    AND NOT EXISTS (
                        SELECT 1
                        FROM compliance_framework_review_report_assignments a
                        WHERE a.org_id = p.org_id
                          AND a.report_id = source_report.report_id
                          AND a.auditor_client_id = $3
                          AND a.assignment_status = 'active'
                    )
                )
              )
            ORDER BY created_at DESC, period_report_id DESC
            LIMIT $4
            "#,
        )
        .bind(input.org_id)
        .bind(input.framework_id)
        .bind(input.auditor_client_id)
        .bind(input.limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| compliance_period_report_from_row(&row))
            .collect())
    }

    pub async fn get_compliance_period_report(
        &self,
        org_id: &str,
        period_report_id: &str,
        auditor_client_id: Option<&str>,
    ) -> Result<Option<CompliancePeriodReportRecord>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                period_report_id,
                org_id::text,
                created_by_user_id,
                framework_id,
                ROUND(EXTRACT(EPOCH FROM date_range_start) * 1000)::BIGINT AS date_range_start_ms,
                ROUND(EXTRACT(EPOCH FROM date_range_end) * 1000)::BIGINT AS date_range_end_ms,
                report_count,
                source_report_ids,
                format,
                status,
                artifact_hash,
                compliance_claim,
                regulatory_claim,
                requires_auditor_review,
                certification,
                review_status,
                reviewed_by_user_id,
                ROUND(EXTRACT(EPOCH FROM reviewed_at) * 1000)::BIGINT AS reviewed_at_ms,
                review_notes_safe,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                CASE
                    WHEN retention_status = 'active' AND retention_until < NOW() THEN 'retention_expired'
                    ELSE retention_status
                END AS retention_status,
                ROUND(EXTRACT(EPOCH FROM retention_until) * 1000)::BIGINT AS retention_until_ms,
                download_count,
                ROUND(EXTRACT(EPOCH FROM last_downloaded_at) * 1000)::BIGINT AS last_downloaded_at_ms,
                ROUND(EXTRACT(EPOCH FROM archived_at) * 1000)::BIGINT AS archived_at_ms,
                ROUND(EXTRACT(EPOCH FROM downloaded_at) * 1000)::BIGINT AS downloaded_at_ms,
                error_message_safe
            FROM compliance_period_reports p
            WHERE p.org_id = $1::uuid
              AND p.period_report_id = $2
              AND (
                $3::text IS NULL
                OR NOT EXISTS (
                    SELECT 1
                    FROM jsonb_array_elements_text(p.source_report_ids) AS source_report(report_id)
                    WHERE EXISTS (
                        SELECT 1
                        FROM compliance_framework_review_report_assignments a
                        WHERE a.org_id = p.org_id
                          AND a.report_id = source_report.report_id
                          AND a.assignment_status = 'active'
                    )
                    AND NOT EXISTS (
                        SELECT 1
                        FROM compliance_framework_review_report_assignments a
                        WHERE a.org_id = p.org_id
                          AND a.report_id = source_report.report_id
                          AND a.auditor_client_id = $3
                          AND a.assignment_status = 'active'
                    )
                )
              )
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(period_report_id)
        .bind(auditor_client_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| compliance_period_report_from_row(&row)))
    }

    pub async fn download_compliance_period_report(
        &self,
        org_id: &str,
        period_report_id: &str,
        auditor_client_id: Option<&str>,
    ) -> Result<Option<(CompliancePeriodReportRecord, serde_json::Value)>, DbError> {
        let row = sqlx::query(
            r#"
            WITH selected AS (
                SELECT period_report_id
                FROM compliance_period_reports p
                WHERE p.org_id = $1::uuid
                  AND p.period_report_id = $2
                  AND (
                    $3::text IS NULL
                    OR NOT EXISTS (
                        SELECT 1
                        FROM jsonb_array_elements_text(p.source_report_ids) AS source_report(report_id)
                        WHERE EXISTS (
                            SELECT 1
                            FROM compliance_framework_review_report_assignments a
                            WHERE a.org_id = p.org_id
                              AND a.report_id = source_report.report_id
                              AND a.assignment_status = 'active'
                        )
                        AND NOT EXISTS (
                            SELECT 1
                            FROM compliance_framework_review_report_assignments a
                            WHERE a.org_id = p.org_id
                              AND a.report_id = source_report.report_id
                              AND a.auditor_client_id = $3
                              AND a.assignment_status = 'active'
                        )
                    )
                  )
                LIMIT 1
            )
            UPDATE compliance_period_reports p
            SET downloaded_at = NOW(),
                last_downloaded_at = NOW(),
                download_count = p.download_count + 1,
                retention_status = CASE
                    WHEN p.retention_status = 'active' AND p.retention_until < NOW() THEN 'retention_expired'
                    ELSE p.retention_status
                END
            FROM selected
            WHERE p.period_report_id = selected.period_report_id
            RETURNING
                p.period_report_id,
                p.org_id::text,
                p.created_by_user_id,
                p.framework_id,
                ROUND(EXTRACT(EPOCH FROM p.date_range_start) * 1000)::BIGINT AS date_range_start_ms,
                ROUND(EXTRACT(EPOCH FROM p.date_range_end) * 1000)::BIGINT AS date_range_end_ms,
                p.report_count,
                p.source_report_ids,
                p.format,
                p.status,
                p.artifact_hash,
                p.compliance_claim,
                p.regulatory_claim,
                p.requires_auditor_review,
                p.certification,
                p.review_status,
                p.reviewed_by_user_id,
                ROUND(EXTRACT(EPOCH FROM p.reviewed_at) * 1000)::BIGINT AS reviewed_at_ms,
                p.review_notes_safe,
                ROUND(EXTRACT(EPOCH FROM p.created_at) * 1000)::BIGINT AS created_at_ms,
                p.retention_status,
                ROUND(EXTRACT(EPOCH FROM p.retention_until) * 1000)::BIGINT AS retention_until_ms,
                p.download_count,
                ROUND(EXTRACT(EPOCH FROM p.last_downloaded_at) * 1000)::BIGINT AS last_downloaded_at_ms,
                ROUND(EXTRACT(EPOCH FROM p.archived_at) * 1000)::BIGINT AS archived_at_ms,
                ROUND(EXTRACT(EPOCH FROM p.downloaded_at) * 1000)::BIGINT AS downloaded_at_ms,
                p.error_message_safe,
                p.payload_json_redacted
            "#,
        )
        .bind(org_id)
        .bind(period_report_id)
        .bind(auditor_client_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| {
            let record = compliance_period_report_from_row(&row);
            let artifact: serde_json::Value = row.get("payload_json_redacted");
            (record, artifact)
        }))
    }

    pub async fn get_compliance_period_report_with_payload(
        &self,
        org_id: &str,
        period_report_id: &str,
        auditor_client_id: Option<&str>,
    ) -> Result<Option<(CompliancePeriodReportRecord, serde_json::Value)>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                p.period_report_id,
                p.org_id::text,
                p.created_by_user_id,
                p.framework_id,
                ROUND(EXTRACT(EPOCH FROM p.date_range_start) * 1000)::BIGINT AS date_range_start_ms,
                ROUND(EXTRACT(EPOCH FROM p.date_range_end) * 1000)::BIGINT AS date_range_end_ms,
                p.report_count,
                p.source_report_ids,
                p.format,
                p.status,
                p.artifact_hash,
                p.compliance_claim,
                p.regulatory_claim,
                p.requires_auditor_review,
                p.certification,
                p.review_status,
                p.reviewed_by_user_id,
                ROUND(EXTRACT(EPOCH FROM p.reviewed_at) * 1000)::BIGINT AS reviewed_at_ms,
                p.review_notes_safe,
                ROUND(EXTRACT(EPOCH FROM p.created_at) * 1000)::BIGINT AS created_at_ms,
                CASE
                    WHEN p.retention_status = 'active' AND p.retention_until < NOW() THEN 'retention_expired'
                    ELSE p.retention_status
                END AS retention_status,
                ROUND(EXTRACT(EPOCH FROM p.retention_until) * 1000)::BIGINT AS retention_until_ms,
                p.download_count,
                ROUND(EXTRACT(EPOCH FROM p.last_downloaded_at) * 1000)::BIGINT AS last_downloaded_at_ms,
                ROUND(EXTRACT(EPOCH FROM p.archived_at) * 1000)::BIGINT AS archived_at_ms,
                ROUND(EXTRACT(EPOCH FROM p.downloaded_at) * 1000)::BIGINT AS downloaded_at_ms,
                p.error_message_safe,
                p.payload_json_redacted
            FROM compliance_period_reports p
            WHERE p.org_id = $1::uuid
              AND p.period_report_id = $2
              AND (
                $3::text IS NULL
                OR NOT EXISTS (
                    SELECT 1
                    FROM jsonb_array_elements_text(p.source_report_ids) AS source_report(report_id)
                    WHERE EXISTS (
                        SELECT 1
                        FROM compliance_framework_review_report_assignments a
                        WHERE a.org_id = p.org_id
                          AND a.report_id = source_report.report_id
                          AND a.assignment_status = 'active'
                    )
                    AND NOT EXISTS (
                        SELECT 1
                        FROM compliance_framework_review_report_assignments a
                        WHERE a.org_id = p.org_id
                          AND a.report_id = source_report.report_id
                          AND a.auditor_client_id = $3
                          AND a.assignment_status = 'active'
                    )
                )
              )
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(period_report_id)
        .bind(auditor_client_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| {
            let record = compliance_period_report_from_row(&row);
            let artifact: serde_json::Value = row.get("payload_json_redacted");
            (record, artifact)
        }))
    }

    pub async fn update_compliance_period_report_retention(
        &self,
        input: &UpdateCompliancePeriodReportRetentionInput<'_>,
    ) -> Result<Option<CompliancePeriodReportRecord>, DbError> {
        let row = sqlx::query(
            r#"
            UPDATE compliance_period_reports p
            SET retention_until = COALESCE($3, p.retention_until),
                retention_status = CASE
                    WHEN $4 THEN 'archived'
                    WHEN p.retention_status = 'archived' THEN 'archived'
                    WHEN COALESCE($3, p.retention_until) < NOW() THEN 'retention_expired'
                    ELSE 'active'
                END,
                archived_at = CASE
                    WHEN $4 THEN COALESCE(p.archived_at, NOW())
                    ELSE p.archived_at
                END
            WHERE p.org_id = $1::uuid
              AND p.period_report_id = $2
            RETURNING
                p.period_report_id,
                p.org_id::text,
                p.created_by_user_id,
                p.framework_id,
                ROUND(EXTRACT(EPOCH FROM p.date_range_start) * 1000)::BIGINT AS date_range_start_ms,
                ROUND(EXTRACT(EPOCH FROM p.date_range_end) * 1000)::BIGINT AS date_range_end_ms,
                p.report_count,
                p.source_report_ids,
                p.format,
                p.status,
                p.artifact_hash,
                p.compliance_claim,
                p.regulatory_claim,
                p.requires_auditor_review,
                p.certification,
                p.review_status,
                p.reviewed_by_user_id,
                ROUND(EXTRACT(EPOCH FROM p.reviewed_at) * 1000)::BIGINT AS reviewed_at_ms,
                p.review_notes_safe,
                ROUND(EXTRACT(EPOCH FROM p.created_at) * 1000)::BIGINT AS created_at_ms,
                p.retention_status,
                ROUND(EXTRACT(EPOCH FROM p.retention_until) * 1000)::BIGINT AS retention_until_ms,
                p.download_count,
                ROUND(EXTRACT(EPOCH FROM p.last_downloaded_at) * 1000)::BIGINT AS last_downloaded_at_ms,
                ROUND(EXTRACT(EPOCH FROM p.archived_at) * 1000)::BIGINT AS archived_at_ms,
                ROUND(EXTRACT(EPOCH FROM p.downloaded_at) * 1000)::BIGINT AS downloaded_at_ms,
                p.error_message_safe
            "#,
        )
        .bind(input.org_id)
        .bind(input.period_report_id)
        .bind(input.retention_until)
        .bind(input.archive)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| compliance_period_report_from_row(&row)))
    }

    pub async fn update_compliance_period_report_review(
        &self,
        input: &UpdateCompliancePeriodReportReviewInput<'_>,
    ) -> Result<Option<CompliancePeriodReportRecord>, DbError> {
        let row = sqlx::query(
            r#"
            UPDATE compliance_period_reports p
            SET review_status = $3,
                reviewed_by_user_id = $4,
                reviewed_at = NOW(),
                review_notes_safe = $5,
                error_message_safe = NULL
            WHERE p.org_id = $1::uuid
              AND p.period_report_id = $2
              AND p.retention_status <> 'archived'
            RETURNING
                p.period_report_id,
                p.org_id::text,
                p.created_by_user_id,
                p.framework_id,
                ROUND(EXTRACT(EPOCH FROM p.date_range_start) * 1000)::BIGINT AS date_range_start_ms,
                ROUND(EXTRACT(EPOCH FROM p.date_range_end) * 1000)::BIGINT AS date_range_end_ms,
                p.report_count,
                p.source_report_ids,
                p.format,
                p.status,
                p.artifact_hash,
                p.compliance_claim,
                p.regulatory_claim,
                p.requires_auditor_review,
                p.certification,
                p.review_status,
                p.reviewed_by_user_id,
                ROUND(EXTRACT(EPOCH FROM p.reviewed_at) * 1000)::BIGINT AS reviewed_at_ms,
                p.review_notes_safe,
                ROUND(EXTRACT(EPOCH FROM p.created_at) * 1000)::BIGINT AS created_at_ms,
                CASE
                    WHEN p.retention_status = 'active' AND p.retention_until < NOW() THEN 'retention_expired'
                    ELSE p.retention_status
                END AS retention_status,
                ROUND(EXTRACT(EPOCH FROM p.retention_until) * 1000)::BIGINT AS retention_until_ms,
                p.download_count,
                ROUND(EXTRACT(EPOCH FROM p.last_downloaded_at) * 1000)::BIGINT AS last_downloaded_at_ms,
                ROUND(EXTRACT(EPOCH FROM p.archived_at) * 1000)::BIGINT AS archived_at_ms,
                ROUND(EXTRACT(EPOCH FROM p.downloaded_at) * 1000)::BIGINT AS downloaded_at_ms,
                p.error_message_safe
            "#,
        )
        .bind(input.org_id)
        .bind(input.period_report_id)
        .bind(input.review_status)
        .bind(input.reviewed_by_user_id)
        .bind(input.review_notes_safe)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| compliance_period_report_from_row(&row)))
    }

    pub async fn create_compliance_period_report_access_log(
        &self,
        input: &CreateCompliancePeriodReportAccessLogInput<'_>,
    ) -> Result<CompliancePeriodReportAccessLogRecord, DbError> {
        let row = sqlx::query(
            r#"
            INSERT INTO compliance_period_report_access_log (
                access_log_id,
                org_id,
                period_report_id,
                actor_client_id,
                action,
                artifact_type,
                artifact_id,
                artifact_hash,
                metadata
            )
            VALUES ($1, $2::uuid, $3, $4, $5, $6, $7, $8, $9::jsonb)
            RETURNING
                access_log_id,
                org_id::text,
                period_report_id,
                actor_client_id,
                action,
                artifact_type,
                artifact_id,
                artifact_hash,
                metadata,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            "#,
        )
        .bind(input.access_log_id)
        .bind(input.org_id)
        .bind(input.period_report_id)
        .bind(input.actor_client_id)
        .bind(input.action)
        .bind(input.artifact_type)
        .bind(input.artifact_id)
        .bind(input.artifact_hash)
        .bind(input.metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(compliance_period_report_access_log_from_row(&row))
    }

    pub async fn list_compliance_period_report_access_logs(
        &self,
        org_id: &str,
        period_report_id: &str,
        limit: i64,
    ) -> Result<Vec<CompliancePeriodReportAccessLogRecord>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                access_log_id,
                org_id::text,
                period_report_id,
                actor_client_id,
                action,
                artifact_type,
                artifact_id,
                artifact_hash,
                metadata,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            FROM compliance_period_report_access_log
            WHERE org_id = $1::uuid
              AND period_report_id = $2
            ORDER BY created_at DESC, access_log_id DESC
            LIMIT $3
            "#,
        )
        .bind(org_id)
        .bind(period_report_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| compliance_period_report_access_log_from_row(&row))
            .collect())
    }

    pub async fn create_compliance_period_report_pdf_export(
        &self,
        input: &CreateCompliancePeriodReportPdfExportInput<'_>,
    ) -> Result<CompliancePeriodReportPdfExportRecord, DbError> {
        sqlx::query(
            r#"
            INSERT INTO compliance_period_report_pdf_exports (
                pdf_export_id,
                org_id,
                period_report_id,
                created_by_user_id,
                source_period_report_hash,
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
        .bind(input.period_report_id)
        .bind(input.created_by_user_id)
        .bind(input.source_period_report_hash)
        .bind(input.pdf_artifact_hash)
        .bind(input.content_type)
        .bind(input.page_count)
        .bind(input.pdf_bytes)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        self.get_compliance_period_report_pdf_export(
            input.org_id,
            input.period_report_id,
            input.pdf_export_id,
        )
        .await?
        .ok_or_else(|| DbError::NotFound("period compliance report PDF export".to_string()))
    }

    pub async fn get_latest_compliance_period_report_pdf_export(
        &self,
        org_id: &str,
        period_report_id: &str,
    ) -> Result<Option<CompliancePeriodReportPdfExportRecord>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                pdf_export_id,
                org_id::text,
                period_report_id,
                created_by_user_id,
                source_period_report_hash,
                pdf_artifact_hash,
                content_type,
                page_count,
                compliance_claim,
                regulatory_claim,
                requires_auditor_review,
                certification,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM downloaded_at) * 1000)::BIGINT AS downloaded_at_ms
            FROM compliance_period_report_pdf_exports
            WHERE org_id = $1::uuid
              AND period_report_id = $2
            ORDER BY created_at DESC, pdf_export_id DESC
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(period_report_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| compliance_period_report_pdf_export_from_row(&row)))
    }

    pub async fn get_compliance_period_report_pdf_export(
        &self,
        org_id: &str,
        period_report_id: &str,
        pdf_export_id: &str,
    ) -> Result<Option<CompliancePeriodReportPdfExportRecord>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                pdf_export_id,
                org_id::text,
                period_report_id,
                created_by_user_id,
                source_period_report_hash,
                pdf_artifact_hash,
                content_type,
                page_count,
                compliance_claim,
                regulatory_claim,
                requires_auditor_review,
                certification,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM downloaded_at) * 1000)::BIGINT AS downloaded_at_ms
            FROM compliance_period_report_pdf_exports
            WHERE org_id = $1::uuid
              AND period_report_id = $2
              AND pdf_export_id = $3
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(period_report_id)
        .bind(pdf_export_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| compliance_period_report_pdf_export_from_row(&row)))
    }

    pub async fn list_compliance_period_report_pdf_exports(
        &self,
        org_id: &str,
        period_report_id: &str,
        limit: i64,
    ) -> Result<Vec<CompliancePeriodReportPdfExportRecord>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                pdf_export_id,
                org_id::text,
                period_report_id,
                created_by_user_id,
                source_period_report_hash,
                pdf_artifact_hash,
                content_type,
                page_count,
                compliance_claim,
                regulatory_claim,
                requires_auditor_review,
                certification,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM downloaded_at) * 1000)::BIGINT AS downloaded_at_ms
            FROM compliance_period_report_pdf_exports
            WHERE org_id = $1::uuid
              AND period_report_id = $2
            ORDER BY created_at DESC, pdf_export_id DESC
            LIMIT $3
            "#,
        )
        .bind(org_id)
        .bind(period_report_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| compliance_period_report_pdf_export_from_row(&row))
            .collect())
    }

    pub async fn latest_compliance_period_report_manifest_hash(
        &self,
        org_id: &str,
        period_report_id: &str,
    ) -> Result<Option<String>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT manifest_hash
            FROM compliance_period_report_manifests
            WHERE org_id = $1::uuid
              AND period_report_id = $2
            ORDER BY created_at DESC, manifest_id DESC
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(period_report_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| row.get("manifest_hash")))
    }

    pub async fn create_compliance_period_report_profile(
        &self,
        input: &CreateCompliancePeriodReportProfileInput<'_>,
    ) -> Result<CompliancePeriodReportProfileRecord, DbError> {
        let row = sqlx::query(
            r#"
            INSERT INTO compliance_period_report_profiles (
                profile_id,
                org_id,
                created_by_user_id,
                updated_by_user_id,
                name,
                period_type,
                framework_id,
                framework_owner_type,
                include_pdf,
                include_manifest,
                retention_days,
                filters,
                status
            )
            VALUES (
                $1,
                $2::uuid,
                $3,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10,
                $11::jsonb,
                'active'
            )
            RETURNING
                profile_id,
                org_id::text,
                created_by_user_id,
                updated_by_user_id,
                name,
                period_type,
                framework_id,
                framework_owner_type,
                include_pdf,
                include_manifest,
                retention_days,
                filters,
                status,
                run_count,
                ROUND(EXTRACT(EPOCH FROM last_run_at) * 1000)::BIGINT AS last_run_at_ms,
                last_period_report_id,
                last_pdf_export_id,
                last_manifest_id,
                ROUND(EXTRACT(EPOCH FROM archived_at) * 1000)::BIGINT AS archived_at_ms,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM updated_at) * 1000)::BIGINT AS updated_at_ms
            "#,
        )
        .bind(input.profile_id)
        .bind(input.org_id)
        .bind(input.created_by_user_id)
        .bind(input.name)
        .bind(input.period_type)
        .bind(input.framework_id)
        .bind(input.framework_owner_type)
        .bind(input.include_pdf)
        .bind(input.include_manifest)
        .bind(input.retention_days)
        .bind(input.filters)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(compliance_period_report_profile_from_row(&row))
    }

    pub async fn list_compliance_period_report_profiles(
        &self,
        input: &ListCompliancePeriodReportProfilesInput<'_>,
    ) -> Result<Vec<CompliancePeriodReportProfileRecord>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                profile_id,
                org_id::text,
                created_by_user_id,
                updated_by_user_id,
                name,
                period_type,
                framework_id,
                framework_owner_type,
                include_pdf,
                include_manifest,
                retention_days,
                filters,
                status,
                run_count,
                ROUND(EXTRACT(EPOCH FROM last_run_at) * 1000)::BIGINT AS last_run_at_ms,
                last_period_report_id,
                last_pdf_export_id,
                last_manifest_id,
                ROUND(EXTRACT(EPOCH FROM archived_at) * 1000)::BIGINT AS archived_at_ms,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM updated_at) * 1000)::BIGINT AS updated_at_ms
            FROM compliance_period_report_profiles
            WHERE org_id = $1::uuid
              AND ($2::text IS NULL OR framework_id = $2)
              AND ($3::text IS NULL OR status = $3)
            ORDER BY updated_at DESC, profile_id DESC
            LIMIT $4
            "#,
        )
        .bind(input.org_id)
        .bind(input.framework_id)
        .bind(input.status)
        .bind(input.limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| compliance_period_report_profile_from_row(&row))
            .collect())
    }

    pub async fn get_compliance_period_report_profile(
        &self,
        org_id: &str,
        profile_id: &str,
    ) -> Result<Option<CompliancePeriodReportProfileRecord>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                profile_id,
                org_id::text,
                created_by_user_id,
                updated_by_user_id,
                name,
                period_type,
                framework_id,
                framework_owner_type,
                include_pdf,
                include_manifest,
                retention_days,
                filters,
                status,
                run_count,
                ROUND(EXTRACT(EPOCH FROM last_run_at) * 1000)::BIGINT AS last_run_at_ms,
                last_period_report_id,
                last_pdf_export_id,
                last_manifest_id,
                ROUND(EXTRACT(EPOCH FROM archived_at) * 1000)::BIGINT AS archived_at_ms,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM updated_at) * 1000)::BIGINT AS updated_at_ms
            FROM compliance_period_report_profiles
            WHERE org_id = $1::uuid
              AND profile_id = $2
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(profile_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| compliance_period_report_profile_from_row(&row)))
    }

    pub async fn update_compliance_period_report_profile(
        &self,
        input: &UpdateCompliancePeriodReportProfileInput<'_>,
    ) -> Result<Option<CompliancePeriodReportProfileRecord>, DbError> {
        let row = sqlx::query(
            r#"
            UPDATE compliance_period_report_profiles
            SET updated_by_user_id = $3,
                name = $4,
                period_type = $5,
                framework_id = $6,
                framework_owner_type = $7,
                include_pdf = $8,
                include_manifest = $9,
                retention_days = $10,
                filters = $11::jsonb,
                updated_at = NOW()
            WHERE org_id = $1::uuid
              AND profile_id = $2
              AND status = 'active'
            RETURNING
                profile_id,
                org_id::text,
                created_by_user_id,
                updated_by_user_id,
                name,
                period_type,
                framework_id,
                framework_owner_type,
                include_pdf,
                include_manifest,
                retention_days,
                filters,
                status,
                run_count,
                ROUND(EXTRACT(EPOCH FROM last_run_at) * 1000)::BIGINT AS last_run_at_ms,
                last_period_report_id,
                last_pdf_export_id,
                last_manifest_id,
                ROUND(EXTRACT(EPOCH FROM archived_at) * 1000)::BIGINT AS archived_at_ms,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM updated_at) * 1000)::BIGINT AS updated_at_ms
            "#,
        )
        .bind(input.org_id)
        .bind(input.profile_id)
        .bind(input.updated_by_user_id)
        .bind(input.name)
        .bind(input.period_type)
        .bind(input.framework_id)
        .bind(input.framework_owner_type)
        .bind(input.include_pdf)
        .bind(input.include_manifest)
        .bind(input.retention_days)
        .bind(input.filters)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| compliance_period_report_profile_from_row(&row)))
    }

    pub async fn archive_compliance_period_report_profile(
        &self,
        input: &ArchiveCompliancePeriodReportProfileInput<'_>,
    ) -> Result<Option<CompliancePeriodReportProfileRecord>, DbError> {
        let row = sqlx::query(
            r#"
            UPDATE compliance_period_report_profiles
            SET status = 'archived',
                archived_at = COALESCE(archived_at, NOW()),
                updated_at = NOW(),
                updated_by_user_id = $3
            WHERE org_id = $1::uuid
              AND profile_id = $2
            RETURNING
                profile_id,
                org_id::text,
                created_by_user_id,
                updated_by_user_id,
                name,
                period_type,
                framework_id,
                framework_owner_type,
                include_pdf,
                include_manifest,
                retention_days,
                filters,
                status,
                run_count,
                ROUND(EXTRACT(EPOCH FROM last_run_at) * 1000)::BIGINT AS last_run_at_ms,
                last_period_report_id,
                last_pdf_export_id,
                last_manifest_id,
                ROUND(EXTRACT(EPOCH FROM archived_at) * 1000)::BIGINT AS archived_at_ms,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM updated_at) * 1000)::BIGINT AS updated_at_ms
            "#,
        )
        .bind(input.org_id)
        .bind(input.profile_id)
        .bind(input.updated_by_user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| compliance_period_report_profile_from_row(&row)))
    }

    pub async fn record_compliance_period_report_profile_run(
        &self,
        input: &RecordCompliancePeriodReportProfileRunInput<'_>,
    ) -> Result<Option<CompliancePeriodReportProfileRecord>, DbError> {
        let row = sqlx::query(
            r#"
            UPDATE compliance_period_report_profiles
            SET run_count = run_count + 1,
                last_run_at = NOW(),
                last_period_report_id = $3,
                last_pdf_export_id = $4,
                last_manifest_id = $5,
                updated_by_user_id = $6,
                updated_at = NOW()
            WHERE org_id = $1::uuid
              AND profile_id = $2
              AND status = 'active'
            RETURNING
                profile_id,
                org_id::text,
                created_by_user_id,
                updated_by_user_id,
                name,
                period_type,
                framework_id,
                framework_owner_type,
                include_pdf,
                include_manifest,
                retention_days,
                filters,
                status,
                run_count,
                ROUND(EXTRACT(EPOCH FROM last_run_at) * 1000)::BIGINT AS last_run_at_ms,
                last_period_report_id,
                last_pdf_export_id,
                last_manifest_id,
                ROUND(EXTRACT(EPOCH FROM archived_at) * 1000)::BIGINT AS archived_at_ms,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM updated_at) * 1000)::BIGINT AS updated_at_ms
            "#,
        )
        .bind(input.org_id)
        .bind(input.profile_id)
        .bind(input.period_report_id)
        .bind(input.pdf_export_id)
        .bind(input.manifest_id)
        .bind(input.updated_by_user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| compliance_period_report_profile_from_row(&row)))
    }

    pub async fn create_compliance_period_report_provenance_manifest(
        &self,
        input: &CreateCompliancePeriodReportProvenanceManifestInput<'_>,
    ) -> Result<CompliancePeriodReportProvenanceManifestRecord, DbError> {
        let row = sqlx::query(
            r#"
            INSERT INTO compliance_period_report_manifests (
                manifest_id,
                org_id,
                period_report_id,
                generated_by_user_id,
                manifest_hash,
                previous_manifest_hash,
                signature_algorithm,
                payload_json_redacted
            )
            VALUES ($1, $2::uuid, $3, $4, $5, $6, $7, $8::jsonb)
            RETURNING
                manifest_id,
                org_id::text,
                period_report_id,
                generated_by_user_id,
                manifest_hash,
                previous_manifest_hash,
                signature_algorithm,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            "#,
        )
        .bind(input.manifest_id)
        .bind(input.org_id)
        .bind(input.period_report_id)
        .bind(input.generated_by_user_id)
        .bind(input.manifest_hash)
        .bind(input.previous_manifest_hash)
        .bind(input.signature_algorithm)
        .bind(input.payload_json_redacted)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(compliance_period_report_provenance_manifest_from_row(&row))
    }

    pub async fn get_compliance_period_report_manifest_payload(
        &self,
        org_id: &str,
        period_report_id: &str,
        manifest_id: &str,
    ) -> Result<Option<serde_json::Value>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT payload_json_redacted
            FROM compliance_period_report_manifests
            WHERE org_id = $1::uuid
              AND period_report_id = $2
              AND manifest_id = $3
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(period_report_id)
        .bind(manifest_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| row.get("payload_json_redacted")))
    }

    pub async fn download_compliance_period_report_pdf_export(
        &self,
        org_id: &str,
        period_report_id: &str,
        pdf_export_id: Option<&str>,
    ) -> Result<Option<(CompliancePeriodReportPdfExportRecord, Vec<u8>)>, DbError> {
        let row = sqlx::query(
            r#"
            WITH selected AS (
                SELECT pdf_export_id
                FROM compliance_period_report_pdf_exports
                WHERE org_id = $1::uuid
                  AND period_report_id = $2
                  AND ($3::text IS NULL OR pdf_export_id = $3)
                ORDER BY created_at DESC, pdf_export_id DESC
                LIMIT 1
            )
            UPDATE compliance_period_report_pdf_exports e
            SET downloaded_at = NOW()
            FROM selected
            WHERE e.pdf_export_id = selected.pdf_export_id
            RETURNING
                e.pdf_export_id,
                e.org_id::text,
                e.period_report_id,
                e.created_by_user_id,
                e.source_period_report_hash,
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
        .bind(period_report_id)
        .bind(pdf_export_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        if row.is_some() {
            sqlx::query(
                r#"
                UPDATE compliance_period_reports
                SET downloaded_at = NOW(),
                    last_downloaded_at = NOW(),
                    download_count = download_count + 1,
                    retention_status = CASE
                        WHEN retention_status = 'active' AND retention_until < NOW() THEN 'retention_expired'
                        ELSE retention_status
                    END
                WHERE org_id = $1::uuid
                  AND period_report_id = $2
                "#,
            )
            .bind(org_id)
            .bind(period_report_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;
        }

        Ok(row.map(|row| {
            let record = compliance_period_report_pdf_export_from_row(&row);
            let bytes: Vec<u8> = row.get("pdf_bytes");
            (record, bytes)
        }))
    }
}
