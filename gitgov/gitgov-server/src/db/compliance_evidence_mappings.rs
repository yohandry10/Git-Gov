use super::*;

fn framework_from_row(row: &PgRow, controls: Vec<ComplianceControl>) -> ComplianceControlFramework {
    ComplianceControlFramework {
        framework_id: row.get("framework_id"),
        org_id: row.get("org_id"),
        name: row.get("name"),
        version: row.get("version"),
        description: row.get("description"),
        is_regulatory: row.get("is_regulatory"),
        is_active: row.get("is_active"),
        owner_type: row.get("owner_type"),
        owner_name: row.get("owner_name"),
        source: row.get("source"),
        is_gitgov_owned: row.get("is_gitgov_owned"),
        official_regulatory_mapping: row.get("official_regulatory_mapping"),
        framework_pack_id: row.get("framework_pack_id"),
        pack_hash: row.get("pack_hash"),
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

fn framework_pack_from_row(row: &PgRow) -> ComplianceFrameworkPackRecord {
    ComplianceFrameworkPackRecord {
        framework_pack_id: row.get("framework_pack_id"),
        org_id: row.get("org_id"),
        framework_id: row.get("framework_id"),
        framework_name: row.get("framework_name"),
        framework_version: row.get("framework_version"),
        description: row.get("description"),
        owner_type: row.get("owner_type"),
        owner_name: row.get("owner_name"),
        source: row.get("source"),
        review_status: row.get("review_status"),
        schema_version: row.get("schema_version"),
        pack_hash: row.get("pack_hash"),
        control_count: row.get("control_count"),
        compliance_claim: row.get("compliance_claim"),
        regulatory_claim: row.get("regulatory_claim"),
        gitgov_certifies: row.get("gitgov_certifies"),
        requires_auditor_review: row.get("requires_auditor_review"),
        official_regulatory_mapping: row.get("official_regulatory_mapping"),
        created_by_user_id: row.get("created_by_user_id"),
        created_at: row.get("created_at_ms"),
        archived_at: row.get("archived_at_ms"),
    }
}

impl Database {
    pub async fn list_compliance_control_frameworks(
        &self,
        org_id: Option<&str>,
    ) -> Result<Vec<ComplianceControlFramework>, DbError> {
        let mut query = QueryBuilder::new(
            r#"
            SELECT
                framework_id,
                org_id::text,
                name,
                version,
                description,
                is_regulatory,
                is_active,
                owner_type,
                owner_name,
                source,
                is_gitgov_owned,
                official_regulatory_mapping,
                framework_pack_id,
                pack_hash
            FROM compliance_control_frameworks
            WHERE is_active = TRUE
              AND (org_id IS NULL
            "#,
        );
        if let Some(org_id) = org_id {
            query.push(" OR org_id = ");
            query.push_bind(org_id);
            query.push("::uuid");
        }
        query.push(") ORDER BY is_gitgov_owned DESC, name ASC, framework_id ASC");

        let rows = query
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|row| framework_from_row(row, Vec::new()))
            .collect())
    }

    pub async fn list_compliance_framework_packs(
        &self,
        org_id: &str,
    ) -> Result<Vec<ComplianceFrameworkPackRecord>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id AS framework_pack_id,
                org_id::text,
                framework_id,
                framework_name,
                framework_version,
                description,
                owner_type,
                owner_name,
                source,
                review_status,
                schema_version,
                pack_hash,
                control_count,
                compliance_claim,
                regulatory_claim,
                gitgov_certifies,
                requires_auditor_review,
                official_regulatory_mapping,
                created_by_user_id,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM archived_at) * 1000)::BIGINT AS archived_at_ms
            FROM compliance_framework_packs
            WHERE org_id = $1::uuid
              AND archived_at IS NULL
            ORDER BY created_at DESC, framework_name ASC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(framework_pack_from_row).collect())
    }

    pub async fn get_compliance_framework_pack(
        &self,
        org_id: &str,
        framework_pack_id: &str,
    ) -> Result<Option<ComplianceFrameworkPackRecord>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT
                id AS framework_pack_id,
                org_id::text,
                framework_id,
                framework_name,
                framework_version,
                description,
                owner_type,
                owner_name,
                source,
                review_status,
                schema_version,
                pack_hash,
                control_count,
                compliance_claim,
                regulatory_claim,
                gitgov_certifies,
                requires_auditor_review,
                official_regulatory_mapping,
                created_by_user_id,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM archived_at) * 1000)::BIGINT AS archived_at_ms
            FROM compliance_framework_packs
            WHERE org_id = $1::uuid
              AND id = $2
              AND archived_at IS NULL
            LIMIT 1
            "#,
        )
        .bind(org_id)
        .bind(framework_pack_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(row.map(|row| framework_pack_from_row(&row)))
    }

    pub async fn get_compliance_control_framework(
        &self,
        org_id: Option<&str>,
        framework_id: &str,
    ) -> Result<Option<ComplianceControlFramework>, DbError> {
        let mut query = QueryBuilder::new(
            r#"
            SELECT
                framework_id,
                org_id::text,
                name,
                version,
                description,
                is_regulatory,
                is_active,
                owner_type,
                owner_name,
                source,
                is_gitgov_owned,
                official_regulatory_mapping,
                framework_pack_id,
                pack_hash
            FROM compliance_control_frameworks
            WHERE framework_id =
            "#,
        );
        query.push_bind(framework_id);
        query.push(
            r#"
              AND is_active = TRUE
              AND (org_id IS NULL
            "#,
        );
        if let Some(org_id) = org_id {
            query.push(" OR org_id = ");
            query.push_bind(org_id);
            query.push("::uuid");
        }
        query.push(") LIMIT 1");

        let row = query
            .build()
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

    pub async fn create_compliance_framework_pack(
        &self,
        input: CreateComplianceFrameworkPackInput,
    ) -> Result<ComplianceFrameworkPackImportResponse, DbError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let pack_row = sqlx::query(
            r#"
            INSERT INTO compliance_framework_packs (
                id,
                org_id,
                framework_id,
                framework_name,
                framework_version,
                description,
                owner_type,
                owner_name,
                source,
                review_status,
                schema_version,
                pack_hash,
                raw_pack_redacted,
                control_count,
                compliance_claim,
                regulatory_claim,
                gitgov_certifies,
                requires_auditor_review,
                official_regulatory_mapping,
                created_by_user_id
            )
            VALUES (
                $1,
                $2::uuid,
                $3,
                $4,
                $5,
                $6,
                'customer',
                $7,
                'customer_provided',
                'customer_review_required',
                $8,
                $9,
                $10::jsonb,
                $11,
                FALSE,
                FALSE,
                FALSE,
                TRUE,
                FALSE,
                $12
            )
            ON CONFLICT (id) DO UPDATE
            SET framework_name = EXCLUDED.framework_name,
                framework_version = EXCLUDED.framework_version,
                description = EXCLUDED.description,
                owner_name = EXCLUDED.owner_name,
                schema_version = EXCLUDED.schema_version,
                pack_hash = EXCLUDED.pack_hash,
                raw_pack_redacted = EXCLUDED.raw_pack_redacted,
                control_count = EXCLUDED.control_count,
                archived_at = NULL
            RETURNING
                id AS framework_pack_id,
                org_id::text,
                framework_id,
                framework_name,
                framework_version,
                description,
                owner_type,
                owner_name,
                source,
                review_status,
                schema_version,
                pack_hash,
                control_count,
                compliance_claim,
                regulatory_claim,
                gitgov_certifies,
                requires_auditor_review,
                official_regulatory_mapping,
                created_by_user_id,
                ROUND(EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT AS created_at_ms,
                ROUND(EXTRACT(EPOCH FROM archived_at) * 1000)::BIGINT AS archived_at_ms
            "#,
        )
        .bind(&input.framework_pack_id)
        .bind(&input.org_id)
        .bind(&input.framework_id)
        .bind(&input.framework_name)
        .bind(&input.framework_version)
        .bind(&input.description)
        .bind(&input.owner_name)
        .bind(&input.schema_version)
        .bind(&input.pack_hash)
        .bind(&input.raw_pack_redacted)
        .bind(input.controls.len() as i32)
        .bind(&input.created_by_user_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO compliance_control_frameworks (
                framework_id,
                org_id,
                name,
                version,
                description,
                is_regulatory,
                is_active,
                owner_type,
                owner_name,
                source,
                is_gitgov_owned,
                official_regulatory_mapping,
                framework_pack_id,
                pack_hash,
                created_by_user_id
            )
            VALUES (
                $1,
                $2::uuid,
                $3,
                $4,
                $5,
                FALSE,
                TRUE,
                'customer',
                $6,
                'customer_provided',
                FALSE,
                FALSE,
                $7,
                $8,
                $9
            )
            ON CONFLICT (framework_id) DO UPDATE
            SET name = EXCLUDED.name,
                version = EXCLUDED.version,
                description = EXCLUDED.description,
                is_regulatory = FALSE,
                is_active = TRUE,
                owner_type = 'customer',
                owner_name = EXCLUDED.owner_name,
                source = 'customer_provided',
                is_gitgov_owned = FALSE,
                official_regulatory_mapping = FALSE,
                framework_pack_id = EXCLUDED.framework_pack_id,
                pack_hash = EXCLUDED.pack_hash,
                created_by_user_id = EXCLUDED.created_by_user_id
            "#,
        )
        .bind(&input.framework_id)
        .bind(&input.org_id)
        .bind(&input.framework_name)
        .bind(&input.framework_version)
        .bind(&input.description)
        .bind(&input.owner_name)
        .bind(&input.framework_pack_id)
        .bind(&input.pack_hash)
        .bind(&input.created_by_user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        sqlx::query("DELETE FROM compliance_controls WHERE framework_id = $1")
            .bind(&input.framework_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let mut created_controls = Vec::with_capacity(input.controls.len());
        for control in &input.controls {
            let required_evidence_types = serde_json::to_value(&control.required_evidence_types)
                .map_err(|e| DbError::SerializationError(e.to_string()))?;
            let control_row = sqlx::query(
                r#"
                INSERT INTO compliance_controls (
                    id,
                    framework_id,
                    control_id,
                    title,
                    description,
                    required_evidence_types,
                    sort_order
                )
                VALUES ($1, $2, $3, $4, $5, $6::jsonb, $7)
                RETURNING control_id, title, description, required_evidence_types, sort_order
                "#,
            )
            .bind(&control.control_row_id)
            .bind(&input.framework_id)
            .bind(&control.control_id)
            .bind(&control.title)
            .bind(&control.description)
            .bind(&required_evidence_types)
            .bind(control.sort_order)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;
            created_controls.push(control_from_row(&control_row));
        }

        tx.commit()
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let framework = self
            .get_compliance_control_framework(Some(&input.org_id), &input.framework_id)
            .await?
            .ok_or_else(|| DbError::NotFound("created compliance framework".to_string()))?;

        Ok(ComplianceFrameworkPackImportResponse {
            framework_pack: framework_pack_from_row(&pack_row),
            framework: ComplianceControlFramework {
                controls: created_controls,
                ..framework
            },
        })
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
