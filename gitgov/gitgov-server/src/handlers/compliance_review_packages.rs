// ============================================================================
// COMPLIANCE REVIEW PACKAGES
// ============================================================================

const COMPLIANCE_REVIEW_PACKAGE_SCHEMA_VERSION: &str = "gitgov_control_review_package.v1";
const COMPLIANCE_REVIEW_PACKAGE_DEFAULT_SECTIONS: &[&str] = &[
    "summary",
    "source_hashes",
    "framework",
    "control_matrix",
    "missing_evidence",
    "no_claims",
    "audit_metadata",
];

fn normalize_review_package_sections(sections: &[String]) -> Result<Vec<String>, Vec<String>> {
    let mut errors = Vec::new();
    let mut normalized = Vec::new();
    let source: Vec<String> = if sections.is_empty() {
        COMPLIANCE_REVIEW_PACKAGE_DEFAULT_SECTIONS
            .iter()
            .map(|section| section.to_string())
            .collect()
    } else {
        sections
            .iter()
            .map(|section| section.trim().to_ascii_lowercase())
            .filter(|section| !section.is_empty())
            .collect()
    };

    for section in source {
        if !COMPLIANCE_REVIEW_PACKAGE_DEFAULT_SECTIONS.contains(&section.as_str()) {
            errors.push(format!("unsupported include section: {section}"));
        } else if !normalized.contains(&section) {
            normalized.push(section);
        }
    }

    if errors.is_empty() {
        Ok(normalized)
    } else {
        Err(errors)
    }
}

fn normalize_review_package_request(
    payload: &mut ComplianceReviewPackageRequest,
) -> Result<Vec<String>, Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut payload.org_name);
    payload.mapping_id = payload.mapping_id.trim().to_string();
    payload.format = payload
        .format
        .take()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .or_else(|| Some("json".to_string()));

    if !payload.mapping_id.starts_with("cem_") || payload.mapping_id.len() > 80 {
        errors.push("mapping_id must be a valid cem_ identifier.".to_string());
    }
    if payload.format.as_deref() != Some("json") {
        errors.push("format must be json for the KAN-101 MVP.".to_string());
    }

    match normalize_review_package_sections(&payload.include_sections) {
        Ok(sections) if errors.is_empty() => Ok(sections),
        Ok(_) => Err(errors),
        Err(mut section_errors) => {
            errors.append(&mut section_errors);
            Err(errors)
        }
    }
}

fn review_section_enabled(sections: &[String], section: &str) -> bool {
    sections.iter().any(|value| value == section)
}

fn compliance_review_package_hash(artifact: &serde_json::Value) -> String {
    let content = serde_json::to_string(artifact).unwrap_or_else(|_| "{}".to_string());
    format!("sha256:{:x}", Sha256::digest(content.as_bytes()))
}

fn compliance_mapping_hash(mapping: &ComplianceEvidenceMappingResponse) -> String {
    let content = json!({
        "schema_version": "gitgov_evidence_mapping_hash.v1",
        "mapping": mapping.mapping,
        "items": mapping.items
    });
    compliance_review_package_hash(&content)
}

fn deterministic_review_package_id(org_id: &str, mapping_id: &str, mapping_hash: &str) -> String {
    let content =
        format!("{COMPLIANCE_REVIEW_PACKAGE_SCHEMA_VERSION}:{org_id}:{mapping_id}:{mapping_hash}");
    let digest = format!("{:x}", Sha256::digest(content.as_bytes()));
    format!("crp_{}", &digest[..32])
}

fn review_package_summary(items: &[ComplianceEvidenceMappingItem]) -> serde_json::Value {
    let mut evidence_present = 0;
    let mut partial = 0;
    let mut missing = 0;
    let mut not_applicable = 0;
    let mut manual_review_required = 0;

    for item in items {
        match item.status.as_str() {
            "evidence_present" => evidence_present += 1,
            "partial" => partial += 1,
            "missing" => missing += 1,
            "not_applicable" => not_applicable += 1,
            "manual_review_required" => manual_review_required += 1,
            _ => {}
        }
    }

    json!({
        "total_controls": items.len(),
        "evidence_present": evidence_present,
        "partial": partial,
        "missing": missing,
        "not_applicable": not_applicable,
        "manual_review_required": manual_review_required,
        "controls_requiring_customer_or_auditor_review": items.len()
    })
}

fn aggregate_missing_evidence(items: &[ComplianceEvidenceMappingItem]) -> Vec<String> {
    let mut missing = Vec::new();
    for item in items {
        for evidence in &item.missing_evidence {
            if !missing.contains(evidence) {
                missing.push(evidence.clone());
            }
        }
    }
    missing.sort();
    missing
}

fn build_compliance_review_package_artifact(
    review_package_id: &str,
    org_id: &str,
    created_by: &str,
    mapping: &ComplianceEvidenceMappingResponse,
    framework: Option<&ComplianceControlFramework>,
    mapping_hash: &str,
    sections: &[String],
) -> serde_json::Value {
    let mut artifact = serde_json::Map::new();
    artifact.insert(
        "schema_version".to_string(),
        json!(COMPLIANCE_REVIEW_PACKAGE_SCHEMA_VERSION),
    );
    artifact.insert("review_package_id".to_string(), json!(review_package_id));
    artifact.insert("generated_at".to_string(), json!(mapping.mapping.created_at));
    artifact.insert("org_id".to_string(), json!(org_id));
    artifact.insert("created_by_user_id".to_string(), json!(created_by));
    artifact.insert("format".to_string(), json!("json"));
    artifact.insert("include_sections".to_string(), json!(sections));

    artifact.insert(
        "positioning".to_string(),
        json!({
            "purpose": "Control evidence review package for customer/auditor review",
            "compliance_claim": false,
            "regulatory_claim": false,
            "requires_auditor_review": true,
            "certification": false,
            "official_regulatory_mapping": false
        }),
    );

    if review_section_enabled(sections, "source_hashes") {
        artifact.insert(
            "source".to_string(),
            json!({
                "evidence_export_id": mapping.mapping.evidence_export_id,
                "evidence_export_hash": mapping.mapping.evidence_export_hash,
                "mapping_id": mapping.mapping.mapping_id,
                "mapping_hash": mapping_hash
            }),
        );
    }

    if review_section_enabled(sections, "framework") {
        let owner_type = framework
            .map(|framework| framework.owner_type.as_str())
            .unwrap_or("gitgov");
        let owner_name = framework
            .and_then(|framework| framework.owner_name.as_deref())
            .unwrap_or("GitGov");
        let source = framework
            .map(|framework| framework.source.as_str())
            .unwrap_or("gitgov_owned");
        let is_gitgov_owned = framework
            .map(|framework| framework.is_gitgov_owned)
            .unwrap_or(true);
        let official_regulatory_mapping = framework
            .map(|framework| framework.official_regulatory_mapping)
            .unwrap_or(false);
        let framework_pack_id = framework.and_then(|framework| framework.framework_pack_id.clone());
        let pack_hash = framework.and_then(|framework| framework.pack_hash.clone());
        let review_status = framework
            .and_then(|framework| framework.framework_pack_review_status.clone());
        let reviewed_by_user_id = framework
            .and_then(|framework| framework.framework_pack_reviewed_by_user_id.clone());
        let reviewed_at = framework.and_then(|framework| framework.framework_pack_reviewed_at);
        let review_notes_safe = framework
            .and_then(|framework| framework.framework_pack_review_notes_safe.clone());

        artifact.insert(
            "framework".to_string(),
            json!({
                "id": mapping.mapping.framework_id,
                "version": mapping.mapping.framework_version,
                "owner": owner_name,
                "owner_type": owner_type,
                "source": source,
                "is_gitgov_owned": is_gitgov_owned,
                "is_regulatory": false,
                "official_regulatory_mapping": official_regulatory_mapping,
                "framework_pack_id": framework_pack_id,
                "pack_hash": pack_hash,
                "review_status": review_status,
                "reviewed_by_user_id": reviewed_by_user_id,
                "reviewed_at": reviewed_at,
                "review_notes_safe": review_notes_safe,
                "customer_provided": owner_type == "customer"
            }),
        );
    }

    if review_section_enabled(sections, "no_claims") {
        artifact.insert(
            "claims".to_string(),
            json!({
                "compliance_claim": false,
                "regulatory_claim": false,
                "requires_auditor_review": true,
                "certification": false,
                "official_regulatory_mapping": false
            }),
        );
    }

    if review_section_enabled(sections, "summary") {
        artifact.insert("summary".to_string(), review_package_summary(&mapping.items));
    }

    if review_section_enabled(sections, "control_matrix") {
        artifact.insert(
            "controls".to_string(),
            json!(
                mapping
                    .items
                    .iter()
                    .map(|item| json!({
                        "control_id": item.control_id,
                        "title": item.control_title,
                        "status": item.status,
                        "evidence_refs": item.evidence_refs,
                        "missing_evidence": item.missing_evidence,
                        "notes": item.notes_safe
                    }))
                    .collect::<Vec<_>>()
            ),
        );
    }

    if review_section_enabled(sections, "missing_evidence") {
        artifact.insert(
            "missing_evidence".to_string(),
            json!(aggregate_missing_evidence(&mapping.items)),
        );
    }

    if review_section_enabled(sections, "audit_metadata") {
        artifact.insert(
            "audit_metadata".to_string(),
            json!({
                "mapping_created_at": mapping.mapping.created_at,
                "artifact_redacted": true,
                "raw_payload_included": false,
                "agent_governance_required": false,
                "llm_decision": false,
                "policy_mutation": false,
                "provider_mutation": false
            }),
        );
    }

    serde_json::Value::Object(artifact)
}

fn review_package_response(
    record: ComplianceReviewPackageRecord,
    artifact: Option<serde_json::Value>,
) -> ComplianceReviewPackageResponse {
    let download_url = format!(
        "/compliance/review-packages/{}/download",
        record.review_package_id
    );
    ComplianceReviewPackageResponse {
        review_package: record,
        download_url,
        artifact,
    }
}

async fn resolve_compliance_review_package_org(
    state: &Arc<AppState>,
    auth_user: &AuthUser,
    org_name: Option<&str>,
) -> Result<String, axum::response::Response> {
    match resolve_and_check_org_scope(state, auth_user.org_id.as_deref(), org_name, true).await {
        Ok(Some(org_id)) => Ok(org_id),
        Ok(None) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "org_name is required for global admin keys" })),
        )
            .into_response()),
        Err(err) => Err((
            org_scope_status(err),
            Json(json!({ "error": agent_governance_scope_error_message(err) })),
        )
            .into_response()),
    }
}

pub async fn create_compliance_review_package(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<ComplianceReviewPackageRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    let sections = match normalize_review_package_request(&mut payload) {
        Ok(sections) => sections,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid compliance review package request", "details": errors })),
            )
                .into_response();
        }
    };

    let org_id = match resolve_compliance_review_package_org(
        &state,
        &auth_user,
        payload.org_name.as_deref(),
    )
    .await
    {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    let mapping = match state
        .db
        .get_compliance_evidence_mapping(&org_id, &payload.mapping_id)
        .await
    {
        Ok(Some(mapping)) => mapping,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Compliance evidence mapping not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, mapping_id = %payload.mapping_id, "Failed to load compliance evidence mapping for review package");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    if mapping.mapping.compliance_claim
        || mapping.mapping.regulatory_claim
        || !mapping.mapping.requires_auditor_review
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Mapping is not eligible for a non-claim review package" })),
        )
            .into_response();
    }

    let mapping_hash = compliance_mapping_hash(&mapping);
    let review_package_id =
        deterministic_review_package_id(&org_id, &mapping.mapping.mapping_id, &mapping_hash);
    let framework = match state
        .db
        .get_compliance_control_framework(Some(&org_id), &mapping.mapping.framework_id)
        .await
    {
        Ok(framework) => framework,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, framework_id = %mapping.mapping.framework_id, "Failed to load compliance framework for review package");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    if let Some(framework) = framework.as_ref() {
        if let Some(resp) = customer_framework_review_block_response(framework) {
            return resp;
        }
    }
    let artifact = build_compliance_review_package_artifact(
        &review_package_id,
        &org_id,
        &auth_user.client_id,
        &mapping,
        framework.as_ref(),
        &mapping_hash,
        &sections,
    );
    let artifact_hash = compliance_review_package_hash(&artifact);

    match state
        .db
        .create_compliance_review_package(&CreateComplianceReviewPackageInput {
            review_package_id: &review_package_id,
            org_id: &org_id,
            created_by_user_id: &auth_user.client_id,
            mapping_id: &mapping.mapping.mapping_id,
            evidence_export_id: &mapping.mapping.evidence_export_id,
            evidence_export_hash: &mapping.mapping.evidence_export_hash,
            mapping_hash: &mapping_hash,
            framework_id: &mapping.mapping.framework_id,
            framework_version: &mapping.mapping.framework_version,
            format: "json",
            artifact_hash: &artifact_hash,
            payload_json_redacted: &artifact,
        })
        .await
    {
        Ok(record) => {
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "compliance_review_package.created".to_string(),
                target_type: Some("compliance_review_package".to_string()),
                target_id: Some(record.review_package_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "review_package_id": record.review_package_id,
                    "mapping_id": record.mapping_id,
                    "mapping_hash": record.mapping_hash,
                    "evidence_export_id": record.evidence_export_id,
                    "evidence_export_hash": record.evidence_export_hash,
                    "artifact_hash": record.artifact_hash,
                    "framework_id": record.framework_id,
                    "framework_version": record.framework_version,
                    "compliance_claim": false,
                    "regulatory_claim": false,
                    "requires_auditor_review": true,
                    "certification": false,
                    "agent_governance_required": false
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write compliance review package audit log: {}", e);
            }
            (StatusCode::CREATED, Json(review_package_response(record, Some(artifact))))
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, mapping_id = %payload.mapping_id, "Failed to create compliance review package");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn get_compliance_review_package(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(review_package_id): Path<String>,
    Query(mut query): Query<ComplianceReviewPackageQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_compliance_review_package_org(
        &state,
        &auth_user,
        query.org_name.as_deref(),
    )
    .await
    {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    match state
        .db
        .get_compliance_review_package(&org_id, &review_package_id)
        .await
    {
        Ok(Some(record)) => (
            StatusCode::OK,
            Json(review_package_response(record, None)),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance review package not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, review_package_id = %review_package_id, "Failed to load compliance review package");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn download_compliance_review_package(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(review_package_id): Path<String>,
    Query(mut query): Query<ComplianceReviewPackageQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_compliance_review_package_org(
        &state,
        &auth_user,
        query.org_name.as_deref(),
    )
    .await
    {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    match state
        .db
        .get_compliance_review_package_payload(&org_id, &review_package_id)
        .await
    {
        Ok(Some(payload)) => (StatusCode::OK, Json(payload)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance review package not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, review_package_id = %review_package_id, "Failed to download compliance review package");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
