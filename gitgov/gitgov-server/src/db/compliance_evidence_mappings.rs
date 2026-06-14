use super::*;

fn framework_from_row(row: &PgRow, controls: Vec<ComplianceControl>) -> ComplianceControlFramework {
    ComplianceControlFramework {
        framework_id: row.get("framework_id"),
        name: row.get("name"),
        version: row.get("version"),
        description: row.get("description"),
        is_regulatory: row.get("is_regulatory"),
        is_active: row.get("is_active"),
        controls,
    }
}

fn control_from_row(row: &PgRow) -> ComplianceControl {
    let evidence_types: serde_json::Value = row.get("required_evidence_types");
    ComplianceControl {
        control_id: row.get("control_id"),
        title: row.get("title"),
        description: row.get("description"),
        required_evidence_types: evidence_types
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(|value| value.to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        sort_order: row.get("sort_order"),
    }
}

fn mapping_from_row(row: &PgRow) -> ComplianceEvidenceMappingRecord {
    ComplianceEvidenceMappingRecord {
        mapping_id: row.get("mapping_id"),
        org_id: row.get("org_id"),
        evidence_export_id: row.get("evidence_export_id"),
        evidence_export_hash: row.get("evidence_export_hash"),
        framework_id: row.get("framework_id"),
        framework_version: row.get("framework_version"),
        created_by_user_id: row.get("created_by_user_id"),
        compliance_claim: row.get("compliance_claim"),
        regulatory_claim: row.get("regulatory_claim"),
        requires_auditor_review: row.get("requires_auditor_review"),
        created_at: row.get("created_at_ms"),
    }
}

fn mapping_item_from_row(row: &PgRow) -> ComplianceEvidenceMappingItem {
    let evidence_refs: serde_json::Value = row.get("evidence_refs");
    let missing_evidence: serde_json::Value = row.get("missing_evidence");
    ComplianceEvidenceMappingItem {
        control_id: row.get("control_id"),
        control_title: row.get("control_title"),
        status: row.get("status"),
        evidence_refs: evidence_refs
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(|value| value.to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        missing_evidence: missing_evidence
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(|value| value.to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        notes_safe: row.get("notes_safe"),
    }
}

impl Database {
    pub async fn list_compliance_control_frameworks(
        &self,
    ) -> Result<Vec<ComplianceControlFramework>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT framework_id, name, version, description, is_regulatory, is_active
            FROM compliance_control_frameworks
            WHERE is_active = TRUE
            ORDER BY framework_id
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|row| framework_from_row(row, Vec::new()))
            .collect())
    }

    pub async fn get_compliance_control_framework(
        &self,
        framework_id: &str,
    ) -> Result<Option<ComplianceControlFramework>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT framework_id, name, version, description, is_regulatory, is_active
            FROM compliance_control_frameworks
            WHERE framework_id = $1
              AND is_active = TRUE
            LIMIT 1
            "#,
        )
        .bind(framework_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let control_rows = sqlx::query(
            r#"
            SELECT control_id, title, description, required_evidence_types, sort_order
            FROM compliance_controls
            WHERE framework_id = $1
            ORDER BY sort_order ASC, control_id ASC
            "#,
        )
        .bind(framework_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;
        let controls = control_rows.iter().map(control_from_row).collect();

        Ok(Some(framework_from_row(&row, controls)))
    }

    pub async fn create_compliance_evidence_mapping(
        &self,
        input: CreateComplianceEvidenceMappingInput,
    ) -> Result<ComplianceEvidenceMappingResponse, DbError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let row = sqlx::query(
            r#"
            INSERT INTO compliance_evidence_mappings (
                mapping_id,
                org_id,
                evidence_export_id,
                evidence_export_hash,
                framework_id,
                framework_version,
                created_by_user_id,
                compliance_claim,
                regulatory_claim,
                requires_auditor_review
            )
            VALUES ($1, $2::uuid, $3, $4, $5, $6, $7, FALSE, FALSE, TRUE)
            RETURNING
                mapping_id,
                org_id::text,
                evidence_export_id,
                evidence_export_hash,
                framework_id,
                framework_version,
                created_by_user_id,
                compliance_claim,
                regulatory_claim,
                requires_auditor_review,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            "#,
        )
        .bind(&input.mapping_id)
        .bind(&input.org_id)
        .bind(&input.evidence_export_id)
        .bind(&input.evidence_export_hash)
        .bind(&input.framework_id)
        .bind(&input.framework_version)
        .bind(&input.created_by_user_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let mut created_items = Vec::with_capacity(input.items.len());
        for item in input.items {
            let evidence_refs = serde_json::to_value(&item.evidence_refs)
                .map_err(|e| DbError::SerializationError(e.to_string()))?;
            let missing_evidence = serde_json::to_value(&item.missing_evidence)
                .map_err(|e| DbError::SerializationError(e.to_string()))?;
            let item_row = sqlx::query(
                r#"
                INSERT INTO compliance_evidence_mapping_items (
                    id,
                    mapping_id,
                    control_id,
                    control_title,
                    status,
                    evidence_refs,
                    missing_evidence,
                    notes_safe
                )
                VALUES ($1, $2, $3, $4, $5, $6::jsonb, $7::jsonb, $8)
                RETURNING
                    control_id,
                    control_title,
                    status,
                    evidence_refs,
                    missing_evidence,
                    notes_safe
                "#,
            )
            .bind(&item.item_id)
            .bind(&input.mapping_id)
            .bind(&item.control_id)
            .bind(&item.control_title)
            .bind(&item.status)
            .bind(&evidence_refs)
            .bind(&missing_evidence)
            .bind(&item.notes_safe)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;
            created_items.push(mapping_item_from_row(&item_row));
        }

        tx.commit()
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(ComplianceEvidenceMappingResponse {
            mapping: mapping_from_row(&row),
            items: created_items,
        })
    }

    pub async fn get_compliance_evidence_mapping(
        &self,
        org_id: &str,
        mapping_id: &str,
    ) -> Result<Option<ComplianceEvidenceMappingResponse>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                mapping_id,
                org_id::text,
                evidence_export_id,
                evidence_export_hash,
                framework_id,
                framework_version,
                created_by_user_id,
                compliance_claim,
                regulatory_claim,
                requires_auditor_review,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms
            FROM compliance_evidence_mappings
            WHERE org_id = $1::uuid
              AND mapping_id = $2
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(mapping_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let item_rows = sqlx::query(
            r#"
            SELECT control_id, control_title, status, evidence_refs, missing_evidence, notes_safe
            FROM compliance_evidence_mapping_items
            WHERE mapping_id = $1
            ORDER BY control_id ASC
            "#,
        )
        .bind(mapping_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(Some(ComplianceEvidenceMappingResponse {
            mapping: mapping_from_row(&row),
            items: item_rows.iter().map(mapping_item_from_row).collect(),
        }))
    }
}
