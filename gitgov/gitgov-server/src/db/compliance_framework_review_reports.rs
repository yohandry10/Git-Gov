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
        review_status: row.get("review_status"),
        reviewed_by_user_id: row.get("reviewed_by_user_id"),
        reviewed_at: row.get("reviewed_at_ms"),
        review_notes_safe: row.get("review_notes_safe"),
        created_at: row.get("created_at_ms"),
        downloaded_at: row.get("downloaded_at_ms"),
        error_message_safe: row.get("error_message_safe"),
    }
}

fn compliance_framework_review_report_assignment_from_row(
    row: &PgRow,
) -> ComplianceFrameworkReviewReportAssignmentRecord {
    ComplianceFrameworkReviewReportAssignmentRecord {
        id: row.get("id"),
        org_id: row.get("org_id"),
        report_id: row.get("report_id"),
        auditor_client_id: row.get("auditor_client_id"),
        assignment_status: row.get("assignment_status"),
        assigned_by_user_id: row.get("assigned_by_user_id"),
        assignment_notes_safe: row.get("assignment_notes_safe"),
        created_at: row.get("created_at_ms"),
        updated_at: row.get("updated_at_ms"),
    }
}

fn compliance_framework_review_report_comment_from_row(
    row: &PgRow,
) -> ComplianceFrameworkReviewReportCommentRecord {
    ComplianceFrameworkReviewReportCommentRecord {
        id: row.get("id"),
        org_id: row.get("org_id"),
        report_id: row.get("report_id"),
        commenter_client_id: row.get("commenter_client_id"),
        comment_body_safe: row.get("comment_body_safe"),
        review_status_suggestion: row.get("review_status_suggestion"),
        created_at: row.get("created_at_ms"),
    }
}

fn compliance_framework_review_report_manifest_from_row(
    row: &PgRow,
) -> ComplianceFrameworkReviewReportProvenanceManifestRecord {
    ComplianceFrameworkReviewReportProvenanceManifestRecord {
        manifest_id: row.get("manifest_id"),
        org_id: row.get("org_id"),
        report_id: row.get("report_id"),
        generated_by_user_id: row.get("generated_by_user_id"),
        manifest_hash: row.get("manifest_hash"),
        previous_manifest_hash: row.get("previous_manifest_hash"),
        signature_algorithm: row.get("signature_algorithm"),
        created_at: row.get("created_at_ms"),
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
                review_status,
                reviewed_by_user_id,
                ROUND(EXTRACT(EPOCH FROM reviewed_at) * 1000)::BIGINT AS reviewed_at_ms,
                review_notes_safe,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM downloaded_at) * 1000)::BIGINT AS downloaded_at_ms,
                error_message_safe
            FROM compliance_framework_review_reports
            WHERE org_id = $1::uuid
              AND ($2::text IS NULL OR framework_id = $2)
              AND ($3::text IS NULL OR mapping_id = $3)
              AND ($4::text IS NULL OR review_package_id = $4)
              AND (
                $5::text IS NULL
                OR EXISTS (
                    SELECT 1
                    FROM compliance_framework_review_report_assignments a
                    WHERE a.org_id = compliance_framework_review_reports.org_id
                      AND a.report_id = compliance_framework_review_reports.report_id
                      AND a.auditor_client_id = $5
                      AND a.assignment_status = 'active'
                )
              )
            ORDER BY created_at DESC, report_id DESC
            LIMIT $6
            "#,
        )
        .bind(input.org_id)
        .bind(input.framework_id)
        .bind(input.mapping_id)
        .bind(input.review_package_id)
        .bind(input.assigned_auditor_client_id)
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
                review_status,
                reviewed_by_user_id,
                ROUND(EXTRACT(EPOCH FROM reviewed_at) * 1000)::BIGINT AS reviewed_at_ms,
                review_notes_safe,
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
                review_status,
                reviewed_by_user_id,
                ROUND(EXTRACT(EPOCH FROM reviewed_at) * 1000)::BIGINT AS reviewed_at_ms,
                review_notes_safe,
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

    pub async fn update_compliance_framework_review_report_review(
        &self,
        input: &UpdateComplianceFrameworkReviewReportReviewInput<'_>,
    ) -> Result<Option<ComplianceFrameworkReviewReportRecord>, DbError> {
        let row = sqlx::query(
            r#"
            UPDATE compliance_framework_review_reports
            SET review_status = $3,
                reviewed_by_user_id = $4,
                reviewed_at = NOW(),
                review_notes_safe = $5,
                error_message_safe = NULL
            WHERE org_id = $1::uuid
              AND report_id = $2
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
                review_status,
                reviewed_by_user_id,
                ROUND(EXTRACT(EPOCH FROM reviewed_at) * 1000)::BIGINT AS reviewed_at_ms,
                review_notes_safe,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM downloaded_at) * 1000)::BIGINT AS downloaded_at_ms,
                error_message_safe
            "#,
        )
        .bind(input.org_id)
        .bind(input.report_id)
        .bind(input.review_status)
        .bind(input.reviewed_by_user_id)
        .bind(input.review_notes_safe)
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

    pub async fn compliance_framework_review_report_has_active_assignments(
        &self,
        org_id: &str,
        report_id: &str,
    ) -> Result<bool, DbError> {
        let row = sqlx::query(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM compliance_framework_review_report_assignments
                WHERE org_id = $1::uuid
                  AND report_id = $2
                  AND assignment_status = 'active'
            ) AS has_assignments
            "#,
        )
        .bind(org_id)
        .bind(report_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.get("has_assignments"))
    }

    pub async fn compliance_framework_review_report_is_assigned_to(
        &self,
        org_id: &str,
        report_id: &str,
        auditor_client_id: &str,
    ) -> Result<bool, DbError> {
        let row = sqlx::query(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM compliance_framework_review_report_assignments
                WHERE org_id = $1::uuid
                  AND report_id = $2
                  AND auditor_client_id = $3
                  AND assignment_status = 'active'
            ) AS is_assigned
            "#,
        )
        .bind(org_id)
        .bind(report_id)
        .bind(auditor_client_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.get("is_assigned"))
    }

    pub async fn tenant_principal_is_auditor(
        &self,
        org_id: &str,
        client_id: &str,
    ) -> Result<bool, DbError> {
        let row = sqlx::query(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM api_keys
                WHERE org_id = $1::uuid
                  AND client_id = $2
                  AND role = 'Auditor'
                  AND is_active = TRUE
            )
            OR EXISTS (
                SELECT 1
                FROM org_users
                WHERE org_id = $1::uuid
                  AND login = $2
                  AND role = 'Auditor'
                  AND status = 'active'
            ) AS is_auditor
            "#,
        )
        .bind(org_id)
        .bind(client_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.get("is_auditor"))
    }

    pub async fn upsert_compliance_framework_review_report_assignments(
        &self,
        input: &UpsertComplianceFrameworkReviewReportAssignmentsInput<'_>,
    ) -> Result<Vec<ComplianceFrameworkReviewReportAssignmentRecord>, DbError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        sqlx::query(
            r#"
            UPDATE compliance_framework_review_report_assignments
            SET assignment_status = 'revoked',
                updated_at = NOW()
            WHERE org_id = $1::uuid
              AND report_id = $2
              AND assignment_status = 'active'
              AND NOT (auditor_client_id = ANY($3::text[]))
            "#,
        )
        .bind(input.org_id)
        .bind(input.report_id)
        .bind(input.auditor_client_ids)
        .execute(&mut *tx)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        for auditor_client_id in input.auditor_client_ids {
            sqlx::query(
                r#"
                INSERT INTO compliance_framework_review_report_assignments (
                    org_id,
                    report_id,
                    auditor_client_id,
                    assignment_status,
                    assigned_by_user_id,
                    assignment_notes_safe
                )
                VALUES ($1::uuid, $2, $3, 'active', $4, $5)
                ON CONFLICT (org_id, report_id, auditor_client_id) DO UPDATE
                SET assignment_status = 'active',
                    assigned_by_user_id = EXCLUDED.assigned_by_user_id,
                    assignment_notes_safe = EXCLUDED.assignment_notes_safe,
                    updated_at = NOW()
                "#,
            )
            .bind(input.org_id)
            .bind(input.report_id)
            .bind(auditor_client_id)
            .bind(input.assigned_by_user_id)
            .bind(input.assignment_notes_safe)
            .execute(&mut *tx)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        self.list_compliance_framework_review_report_assignments(input.org_id, input.report_id)
            .await
    }

    pub async fn list_compliance_framework_review_report_assignments(
        &self,
        org_id: &str,
        report_id: &str,
    ) -> Result<Vec<ComplianceFrameworkReviewReportAssignmentRecord>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id::text,
                org_id::text,
                report_id,
                auditor_client_id,
                assignment_status,
                assigned_by_user_id,
                assignment_notes_safe,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM updated_at) * 1000)::BIGINT AS updated_at_ms
            FROM compliance_framework_review_report_assignments
            WHERE org_id = $1::uuid
              AND report_id = $2
            ORDER BY assignment_status ASC, updated_at DESC, auditor_client_id ASC
            "#,
        )
        .bind(org_id)
        .bind(report_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| compliance_framework_review_report_assignment_from_row(&row))
            .collect())
    }

    pub async fn create_compliance_framework_review_report_comment(
        &self,
        input: &CreateComplianceFrameworkReviewReportCommentInput<'_>,
    ) -> Result<ComplianceFrameworkReviewReportCommentRecord, DbError> {
        let row = sqlx::query(
            r#"
            INSERT INTO compliance_framework_review_report_comments (
                org_id,
                report_id,
                commenter_client_id,
                comment_body_safe,
                review_status_suggestion
            )
            VALUES ($1::uuid, $2, $3, $4, $5)
            RETURNING
                id::text,
                org_id::text,
                report_id,
                commenter_client_id,
                comment_body_safe,
                review_status_suggestion,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            "#,
        )
        .bind(input.org_id)
        .bind(input.report_id)
        .bind(input.commenter_client_id)
        .bind(input.comment_body_safe)
        .bind(input.review_status_suggestion)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(compliance_framework_review_report_comment_from_row(&row))
    }

    pub async fn list_compliance_framework_review_report_comments(
        &self,
        org_id: &str,
        report_id: &str,
    ) -> Result<Vec<ComplianceFrameworkReviewReportCommentRecord>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id::text,
                org_id::text,
                report_id,
                commenter_client_id,
                comment_body_safe,
                review_status_suggestion,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            FROM compliance_framework_review_report_comments
            WHERE org_id = $1::uuid
              AND report_id = $2
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(org_id)
        .bind(report_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| compliance_framework_review_report_comment_from_row(&row))
            .collect())
    }

    pub async fn latest_compliance_framework_review_report_manifest_hash(
        &self,
        org_id: &str,
        report_id: &str,
    ) -> Result<Option<String>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT manifest_hash
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

        Ok(row.map(|row| row.get("manifest_hash")))
    }

    pub async fn create_compliance_framework_review_report_provenance_manifest(
        &self,
        input: &CreateComplianceFrameworkReviewReportProvenanceManifestInput<'_>,
    ) -> Result<ComplianceFrameworkReviewReportProvenanceManifestRecord, DbError> {
        let row = sqlx::query(
            r#"
            INSERT INTO compliance_framework_review_report_manifests (
                manifest_id,
                org_id,
                report_id,
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
                report_id,
                generated_by_user_id,
                manifest_hash,
                previous_manifest_hash,
                signature_algorithm,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            "#,
        )
        .bind(input.manifest_id)
        .bind(input.org_id)
        .bind(input.report_id)
        .bind(input.generated_by_user_id)
        .bind(input.manifest_hash)
        .bind(input.previous_manifest_hash)
        .bind(input.signature_algorithm)
        .bind(input.payload_json_redacted)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(compliance_framework_review_report_manifest_from_row(&row))
    }

    pub async fn get_compliance_framework_review_report_manifest_payload(
        &self,
        org_id: &str,
        report_id: &str,
        manifest_id: &str,
    ) -> Result<Option<serde_json::Value>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT payload_json_redacted
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

        Ok(row.map(|row| row.get("payload_json_redacted")))
    }
}
