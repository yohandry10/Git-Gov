// ============================================================================
// COMPLIANCE FRAMEWORK REVIEW REPORTS
// ============================================================================

const COMPLIANCE_FRAMEWORK_REVIEW_REPORT_SCHEMA_VERSION: &str =
    "gitgov_framework_review_report.v1";
const COMPLIANCE_FRAMEWORK_REVIEW_REPORT_MANIFEST_SCHEMA_VERSION: &str =
    "gitgov_framework_review_report_provenance_manifest.v1";
const COMPLIANCE_FRAMEWORK_REVIEW_REPORT_MANIFEST_SIGNATURE_ALGORITHM: &str =
    "sha256-provenance-manifest-v1";
const DEFAULT_FRAMEWORK_REVIEW_REPORT_LIST_LIMIT: i64 = 25;
const MAX_FRAMEWORK_REVIEW_REPORT_LIST_LIMIT: i64 = 100;
const FRAMEWORK_REVIEW_REPORT_REVIEW_NEEDS_REVIEW: &str = "needs_review";
const FRAMEWORK_REVIEW_REPORT_REVIEW_REVIEWED: &str = "reviewed";
const FRAMEWORK_REVIEW_REPORT_REVIEW_NEEDS_CHANGES: &str = "needs_changes";
const FRAMEWORK_REVIEW_REPORT_REVIEW_REJECTED: &str = "rejected";
const MAX_FRAMEWORK_REVIEW_REPORT_REVIEW_NOTE_LEN: usize = 1000;

fn normalize_framework_review_report_request(
    payload: &mut ComplianceFrameworkReviewReportRequest,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut payload.org_name);
    payload.mapping_id = payload.mapping_id.trim().to_string();
    payload.review_package_id = payload.review_package_id.trim().to_string();
    payload.format = payload
        .format
        .take()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .or_else(|| Some("json".to_string()));

    if !payload.mapping_id.starts_with("cem_") || payload.mapping_id.len() > 80 {
        errors.push("mapping_id must be a valid cem_ identifier.".to_string());
    }
    if !payload.review_package_id.starts_with("crp_") || payload.review_package_id.len() > 80 {
        errors.push("review_package_id must be a valid crp_ identifier.".to_string());
    }
    if payload.format.as_deref() != Some("json") {
        errors.push("format must be json for the KAN-105 MVP.".to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn normalize_framework_review_report_query(
    query: &mut ComplianceFrameworkReviewReportQuery,
) -> Result<i64, Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut query.org_name);
    normalize_release_approval_optional_text(&mut query.framework_id);
    normalize_release_approval_optional_text(&mut query.mapping_id);
    normalize_release_approval_optional_text(&mut query.review_package_id);

    if let Some(framework_id) = query.framework_id.as_mut() {
        *framework_id = framework_id.trim().to_ascii_lowercase();
        if framework_id.len() > 96
            || framework_id.is_empty()
            || !framework_id
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
        {
            errors.push("framework_id must be a valid lowercase framework identifier.".to_string());
        }
    }
    if let Some(mapping_id) = query.mapping_id.as_mut() {
        *mapping_id = mapping_id.trim().to_string();
        if !mapping_id.starts_with("cem_") || mapping_id.len() > 80 {
            errors.push("mapping_id must be a valid cem_ identifier.".to_string());
        }
    }
    if let Some(review_package_id) = query.review_package_id.as_mut() {
        *review_package_id = review_package_id.trim().to_string();
        if !review_package_id.starts_with("crp_") || review_package_id.len() > 80 {
            errors.push("review_package_id must be a valid crp_ identifier.".to_string());
        }
    }

    let limit = query
        .limit
        .unwrap_or(DEFAULT_FRAMEWORK_REVIEW_REPORT_LIST_LIMIT)
        .clamp(1, MAX_FRAMEWORK_REVIEW_REPORT_LIST_LIMIT);

    if errors.is_empty() {
        Ok(limit)
    } else {
        Err(errors)
    }
}

fn normalize_safe_framework_review_report_review_text(
    value: &mut Option<String>,
) -> Result<(), String> {
    let Some(text) = value.take() else {
        return Ok(());
    };
    let normalized = text.trim().to_string();
    if normalized.is_empty() {
        *value = None;
        return Ok(());
    }
    if normalized.len() > MAX_FRAMEWORK_REVIEW_REPORT_REVIEW_NOTE_LEN {
        return Err(format!(
            "review notes must be {MAX_FRAMEWORK_REVIEW_REPORT_REVIEW_NOTE_LEN} characters or less"
        ));
    }
    let lowered = normalized.to_ascii_lowercase();
    if lowered.contains("<script")
        || lowered.contains("</")
        || lowered.contains("<iframe")
        || lowered.contains("bearer ")
        || lowered.contains("ghp_")
        || lowered.contains("glpat-")
        || lowered.contains("sk-")
    {
        return Err("review notes must be plain text and cannot contain secrets".to_string());
    }
    *value = Some(normalized);
    Ok(())
}

fn normalize_framework_review_report_review_request(
    payload: &mut ComplianceFrameworkReviewReportReviewRequest,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut payload.org_name);
    payload.review_status = payload.review_status.trim().to_ascii_lowercase();
    if ![
        FRAMEWORK_REVIEW_REPORT_REVIEW_NEEDS_REVIEW,
        FRAMEWORK_REVIEW_REPORT_REVIEW_REVIEWED,
        FRAMEWORK_REVIEW_REPORT_REVIEW_NEEDS_CHANGES,
        FRAMEWORK_REVIEW_REPORT_REVIEW_REJECTED,
    ]
    .contains(&payload.review_status.as_str())
    {
        errors.push(
            "review_status must be needs_review, reviewed, needs_changes, or rejected."
                .to_string(),
        );
    }
    if let Err(error) =
        normalize_safe_framework_review_report_review_text(&mut payload.review_notes_safe)
    {
        errors.push(error);
    }
    if payload.review_status == FRAMEWORK_REVIEW_REPORT_REVIEW_NEEDS_CHANGES
        && payload.review_notes_safe.is_none()
    {
        errors.push("review_notes_safe is required when review_status is needs_changes.".to_string());
    }
    if payload.review_status == FRAMEWORK_REVIEW_REPORT_REVIEW_REJECTED
        && payload.review_notes_safe.is_none()
    {
        errors.push("review_notes_safe is required when review_status is rejected.".to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn deterministic_framework_review_report_id(
    org_id: &str,
    mapping_id: &str,
    review_package_id: &str,
    review_package_hash: &str,
) -> String {
    let content = format!(
        "{COMPLIANCE_FRAMEWORK_REVIEW_REPORT_SCHEMA_VERSION}:{org_id}:{mapping_id}:{review_package_id}:{review_package_hash}"
    );
    let digest = format!("{:x}", Sha256::digest(content.as_bytes()));
    format!("frr_{}", &digest[..32])
}

fn evidence_ref_type(ref_value: &str) -> String {
    ref_value
        .split('.')
        .next()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("evidence")
        .to_string()
}

fn framework_review_report_summary(items: &[ComplianceEvidenceMappingItem]) -> serde_json::Value {
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
        "requires_customer_or_auditor_review": true
    })
}

fn build_framework_review_report_artifact(
    report_id: &str,
    org_id: &str,
    created_by: &str,
    mapping: &ComplianceEvidenceMappingResponse,
    review_package: &ComplianceReviewPackageRecord,
    framework: Option<&ComplianceControlFramework>,
) -> serde_json::Value {
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

    json!({
        "schema_version": COMPLIANCE_FRAMEWORK_REVIEW_REPORT_SCHEMA_VERSION,
        "report_id": report_id,
        "generated_at": mapping.mapping.created_at,
        "org_id": org_id,
        "created_by_user_id": created_by,
        "format": "json",
        "positioning": {
            "purpose": "Framework-specific evidence review report for customer/auditor review",
            "compliance_claim": false,
            "regulatory_claim": false,
            "certification": false,
            "official_regulatory_mapping": false,
            "requires_auditor_review": true
        },
        "framework": {
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
        },
        "source_hashes": {
            "evidence_export_id": mapping.mapping.evidence_export_id,
            "evidence_export_hash": mapping.mapping.evidence_export_hash,
            "mapping_id": mapping.mapping.mapping_id,
            "mapping_hash": review_package.mapping_hash,
            "review_package_id": review_package.review_package_id,
            "review_package_hash": review_package.artifact_hash
        },
        "claims": {
            "compliance_claim": false,
            "regulatory_claim": false,
            "certification": false,
            "official_regulatory_mapping": false,
            "requires_auditor_review": true
        },
        "summary": framework_review_report_summary(&mapping.items),
        "controls": mapping.items.iter().map(|item| {
            json!({
                "control_id": item.control_id,
                "title": item.control_title,
                "status": item.status,
                "evidence_refs": item.evidence_refs.iter().map(|evidence_ref| json!({
                    "type": evidence_ref_type(evidence_ref),
                    "ref": evidence_ref,
                    "source_hash": mapping.mapping.evidence_export_hash
                })).collect::<Vec<_>>(),
                "missing_evidence": item.missing_evidence,
                "requires_manual_review": true,
                "notes": item.notes_safe
            })
        }).collect::<Vec<_>>(),
        "missing_evidence": aggregate_missing_evidence(&mapping.items),
        "audit_metadata": {
            "artifact_redacted": true,
            "raw_payload_included": false,
            "agent_governance_required": false,
            "llm_decision": false,
            "policy_mutation": false,
            "provider_mutation": false,
            "report_scope": "framework_specific_review"
        }
    })
}

fn framework_review_report_response(
    record: ComplianceFrameworkReviewReportRecord,
    artifact: Option<serde_json::Value>,
) -> ComplianceFrameworkReviewReportResponse {
    let download_url = format!(
        "/compliance/framework-review-reports/{}/download",
        record.report_id
    );
    ComplianceFrameworkReviewReportResponse {
        report: record,
        download_url,
        artifact,
    }
}

fn deterministic_framework_review_report_manifest_id(
    report_id: &str,
    generated_by: &str,
    generated_at: i64,
    previous_manifest_hash: Option<&str>,
    content_hash: &str,
) -> String {
    let content = format!(
        "{COMPLIANCE_FRAMEWORK_REVIEW_REPORT_MANIFEST_SCHEMA_VERSION}:{report_id}:{generated_by}:{generated_at}:{}:{content_hash}",
        previous_manifest_hash.unwrap_or("root")
    );
    let digest = format!("{:x}", Sha256::digest(content.as_bytes()));
    format!("frrm_{}", &digest[..32])
}

fn normalize_framework_review_report_manifest_id(
    manifest_id: &str,
) -> Result<String, &'static str> {
    let normalized = manifest_id.trim().to_string();
    if normalized.starts_with("frrm_") && normalized.len() == 37 {
        Ok(normalized)
    } else {
        Err("manifest_id must be a valid frrm_ identifier")
    }
}

struct FrameworkReviewReportManifestPayloadInput<'a> {
    manifest_id: &'a str,
    generated_at: i64,
    generated_by: &'a str,
    report: &'a ComplianceFrameworkReviewReportRecord,
    previous_manifest_hash: Option<&'a str>,
    manifest_hash: Option<&'a str>,
    assignments: &'a [ComplianceFrameworkReviewReportAssignmentRecord],
    comments: &'a [ComplianceFrameworkReviewReportCommentRecord],
}

fn build_framework_review_report_manifest_payload(
    input: &FrameworkReviewReportManifestPayloadInput<'_>,
) -> serde_json::Value {
    let active_assignment_count = input
        .assignments
        .iter()
        .filter(|item| item.assignment_status == "active")
        .count();
    let mut reviewers = HashSet::new();
    for assignment in input.assignments {
        if assignment.assignment_status == "active" {
            reviewers.insert(assignment.auditor_client_id.clone());
        }
    }
    for comment in input.comments {
        reviewers.insert(comment.commenter_client_id.clone());
    }
    if let Some(reviewer) = input.report.reviewed_by_user_id.as_ref() {
        reviewers.insert(reviewer.clone());
    }
    let mut reviewers = reviewers.into_iter().collect::<Vec<_>>();
    reviewers.sort();

    json!({
        "schema_version": COMPLIANCE_FRAMEWORK_REVIEW_REPORT_MANIFEST_SCHEMA_VERSION,
        "manifest_id": input.manifest_id,
        "generated_at": input.generated_at,
        "generated_by_user_id": input.generated_by,
        "signature": {
            "algorithm": COMPLIANCE_FRAMEWORK_REVIEW_REPORT_MANIFEST_SIGNATURE_ALGORITHM,
            "signed_by_user_id": input.generated_by,
            "signed_at": input.generated_at,
            "signature_hash": input.manifest_hash
        },
        "hash_chain": {
            "subject_type": "framework_review_report",
            "subject_id": input.report.report_id,
            "previous_manifest_hash": input.previous_manifest_hash,
            "manifest_hash": input.manifest_hash
        },
        "report": {
            "report_id": input.report.report_id,
            "artifact_hash": input.report.artifact_hash,
            "format": input.report.format,
            "created_at": input.report.created_at,
            "created_by_user_id": input.report.created_by_user_id,
            "review_status": input.report.review_status,
            "reviewed_by_user_id": input.report.reviewed_by_user_id,
            "reviewed_at": input.report.reviewed_at,
            "has_review_notes_safe": input.report.review_notes_safe.is_some()
        },
        "framework": {
            "framework_id": input.report.framework_id,
            "framework_version": input.report.framework_version,
            "framework_owner_type": input.report.framework_owner_type,
            "framework_review_status": input.report.framework_review_status,
            "pack_hash": input.report.pack_hash
        },
        "source_hashes": {
            "evidence_export_id": input.report.evidence_export_id,
            "evidence_export_hash": input.report.evidence_export_hash,
            "mapping_id": input.report.mapping_id,
            "mapping_hash": input.report.mapping_hash,
            "review_package_id": input.report.review_package_id,
            "review_package_hash": input.report.review_package_hash,
            "report_artifact_hash": input.report.artifact_hash
        },
        "collaboration": {
            "assignment_count": input.assignments.len(),
            "active_assignment_count": active_assignment_count,
            "comment_count": input.comments.len(),
            "reviewers": reviewers
        },
        "claims": {
            "compliance_claim": input.report.compliance_claim,
            "regulatory_claim": input.report.regulatory_claim,
            "certification": input.report.certification,
            "requires_auditor_review": input.report.requires_auditor_review,
            "official_regulatory_mapping": false
        },
        "audit_metadata": {
            "artifact_redacted": true,
            "raw_payload_included": false,
            "source_report_artifact_mutated": false,
            "agent_governance_required": false,
            "llm_decision": false,
            "policy_mutation": false,
            "provider_mutation": false,
            "report_scope": "reviewed_framework_report_provenance_manifest"
        }
    })
}

fn framework_review_report_manifest_response(
    manifest: ComplianceFrameworkReviewReportProvenanceManifestRecord,
    artifact: serde_json::Value,
) -> ComplianceFrameworkReviewReportProvenanceManifestResponse {
    let download_url = format!(
        "/compliance/framework-review-reports/{}/provenance-manifests/{}",
        manifest.report_id, manifest.manifest_id
    );
    ComplianceFrameworkReviewReportProvenanceManifestResponse {
        manifest,
        download_url,
        artifact,
    }
}

async fn resolve_compliance_framework_review_report_org(
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

pub async fn create_compliance_framework_review_report(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<ComplianceFrameworkReviewReportRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    if let Err(errors) = normalize_framework_review_report_request(&mut payload) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid compliance framework review report request", "details": errors })),
        )
            .into_response();
    }

    let org_id = match resolve_compliance_framework_review_report_org(
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
            tracing::error!(error = %e, org_id = %org_id, mapping_id = %payload.mapping_id, "Failed to load compliance evidence mapping for framework review report");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    let review_package = match state
        .db
        .get_compliance_review_package(&org_id, &payload.review_package_id)
        .await
    {
        Ok(Some(record)) => record,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Compliance review package not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, review_package_id = %payload.review_package_id, "Failed to load compliance review package for framework review report");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    if review_package.mapping_id != mapping.mapping.mapping_id
        || review_package.evidence_export_id != mapping.mapping.evidence_export_id
        || review_package.evidence_export_hash != mapping.mapping.evidence_export_hash
        || review_package.framework_id != mapping.mapping.framework_id
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Review package does not match the requested mapping" })),
        )
            .into_response();
    }

    if mapping.mapping.compliance_claim
        || mapping.mapping.regulatory_claim
        || !mapping.mapping.requires_auditor_review
        || review_package.compliance_claim
        || review_package.regulatory_claim
        || review_package.certification
        || !review_package.requires_auditor_review
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Source artifacts are not eligible for a non-claim framework review report" })),
        )
            .into_response();
    }

    let framework = match state
        .db
        .get_compliance_control_framework(Some(&org_id), &mapping.mapping.framework_id)
        .await
    {
        Ok(framework) => framework,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, framework_id = %mapping.mapping.framework_id, "Failed to load compliance framework for framework review report");
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

    let report_id = deterministic_framework_review_report_id(
        &org_id,
        &mapping.mapping.mapping_id,
        &review_package.review_package_id,
        &review_package.artifact_hash,
    );
    let artifact = build_framework_review_report_artifact(
        &report_id,
        &org_id,
        &auth_user.client_id,
        &mapping,
        &review_package,
        framework.as_ref(),
    );
    let artifact_hash = compliance_review_package_hash(&artifact);
    let framework_owner_type = framework
        .as_ref()
        .map(|framework| framework.owner_type.as_str())
        .unwrap_or("gitgov");
    let framework_review_status = framework
        .as_ref()
        .and_then(|framework| framework.framework_pack_review_status.as_deref());
    let pack_hash = framework
        .as_ref()
        .and_then(|framework| framework.pack_hash.as_deref());

    match state
        .db
        .create_compliance_framework_review_report(&CreateComplianceFrameworkReviewReportInput {
            report_id: &report_id,
            org_id: &org_id,
            created_by_user_id: &auth_user.client_id,
            mapping_id: &mapping.mapping.mapping_id,
            review_package_id: &review_package.review_package_id,
            evidence_export_id: &mapping.mapping.evidence_export_id,
            evidence_export_hash: &mapping.mapping.evidence_export_hash,
            mapping_hash: &review_package.mapping_hash,
            review_package_hash: &review_package.artifact_hash,
            framework_id: &mapping.mapping.framework_id,
            framework_version: &mapping.mapping.framework_version,
            framework_owner_type,
            framework_review_status,
            pack_hash,
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
                action: "compliance_framework_review_report.created".to_string(),
                target_type: Some("compliance_framework_review_report".to_string()),
                target_id: Some(record.report_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "report_id": record.report_id,
                    "mapping_id": record.mapping_id,
                    "review_package_id": record.review_package_id,
                    "artifact_hash": record.artifact_hash,
                    "framework_id": record.framework_id,
                    "framework_owner_type": record.framework_owner_type,
                    "compliance_claim": false,
                    "regulatory_claim": false,
                    "requires_auditor_review": true,
                    "certification": false,
                    "agent_governance_required": false
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write framework review report audit log: {}", e);
            }
            (StatusCode::CREATED, Json(framework_review_report_response(record, Some(artifact))))
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, mapping_id = %payload.mapping_id, "Failed to create compliance framework review report");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn list_compliance_framework_review_reports(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(mut query): Query<ComplianceFrameworkReviewReportQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let limit = match normalize_framework_review_report_query(&mut query) {
        Ok(limit) => limit,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid compliance framework review report query", "details": errors })),
            )
                .into_response();
        }
    };
    let org_id = match resolve_compliance_framework_review_report_org(
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
        .list_compliance_framework_review_reports(&ListComplianceFrameworkReviewReportsInput {
            org_id: &org_id,
            framework_id: query.framework_id.as_deref(),
            mapping_id: query.mapping_id.as_deref(),
            review_package_id: query.review_package_id.as_deref(),
            assigned_auditor_client_id: query
                .assigned_to_me
                .unwrap_or(false)
                .then_some(auth_user.client_id.as_str()),
            limit,
        })
        .await
    {
        Ok(items) => {
            let count = items.len();
            (
                StatusCode::OK,
                Json(ComplianceFrameworkReviewReportListResponse {
                    items,
                    count,
                    limit,
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to list compliance framework review reports");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn get_compliance_framework_review_report(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(report_id): Path<String>,
    Query(mut query): Query<ComplianceFrameworkReviewReportQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_compliance_framework_review_report_org(
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
        .get_compliance_framework_review_report(&org_id, &report_id)
        .await
    {
        Ok(Some(record)) => (
            StatusCode::OK,
            Json(framework_review_report_response(record, None)),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance framework review report not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to load compliance framework review report");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn review_compliance_framework_review_report(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(report_id): Path<String>,
    Json(mut payload): Json<ComplianceFrameworkReviewReportReviewRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    if !report_id.starts_with("frr_") || report_id.len() > 80 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "report_id must be a valid frr_ identifier" })),
        )
            .into_response();
    }
    if let Err(errors) = normalize_framework_review_report_review_request(&mut payload) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid compliance framework review report review request", "details": errors })),
        )
            .into_response();
    }

    let org_id = match resolve_compliance_framework_review_report_org(
        &state,
        &auth_user,
        payload.org_name.as_deref(),
    )
    .await
    {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    if let Some(resp) =
        require_framework_review_report_collaboration_access(&state, &auth_user, &org_id, &report_id)
            .await
    {
        return resp;
    }

    match state
        .db
        .update_compliance_framework_review_report_review(
            &UpdateComplianceFrameworkReviewReportReviewInput {
                org_id: &org_id,
                report_id: &report_id,
                review_status: &payload.review_status,
                reviewed_by_user_id: &auth_user.client_id,
                review_notes_safe: payload.review_notes_safe.as_deref(),
            },
        )
        .await
    {
        Ok(Some(record)) => {
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "compliance_framework_review_report.reviewed".to_string(),
                target_type: Some("compliance_framework_review_report".to_string()),
                target_id: Some(record.report_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "report_id": record.report_id,
                    "review_status": record.review_status,
                    "reviewed_by_user_id": record.reviewed_by_user_id,
                    "reviewed_at": record.reviewed_at,
                    "has_review_notes_safe": record.review_notes_safe.is_some(),
                    "artifact_hash": record.artifact_hash,
                    "hash_changed": false,
                    "compliance_claim": record.compliance_claim,
                    "regulatory_claim": record.regulatory_claim,
                    "requires_auditor_review": record.requires_auditor_review,
                    "certification": record.certification,
                    "agent_governance_required": false
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write framework review report review audit log: {}", e);
            }
            (
                StatusCode::OK,
                Json(framework_review_report_response(record, None)),
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance framework review report not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to review compliance framework review report");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn download_compliance_framework_review_report(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(report_id): Path<String>,
    Query(mut query): Query<ComplianceFrameworkReviewReportQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_compliance_framework_review_report_org(
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
        .get_compliance_framework_review_report_payload(&org_id, &report_id)
        .await
    {
        Ok(Some(payload)) => (StatusCode::OK, Json(payload)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance framework review report not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to download compliance framework review report");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn create_compliance_framework_review_report_provenance_manifest(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(report_id): Path<String>,
    Json(mut payload): Json<ComplianceFrameworkReviewReportProvenanceManifestRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let report_id = match normalize_framework_review_report_id(&report_id) {
        Ok(report_id) => report_id,
        Err(error) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response();
        }
    };
    normalize_release_approval_optional_text(&mut payload.org_name);
    let org_id = match resolve_compliance_framework_review_report_org(
        &state,
        &auth_user,
        payload.org_name.as_deref(),
    )
    .await
    {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    if let Some(resp) =
        require_framework_review_report_collaboration_access(&state, &auth_user, &org_id, &report_id)
            .await
    {
        return resp;
    }

    let report = match state
        .db
        .get_compliance_framework_review_report(&org_id, &report_id)
        .await
    {
        Ok(Some(record)) => record,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Compliance framework review report not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to load framework review report before provenance manifest");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    if report.review_status != FRAMEWORK_REVIEW_REPORT_REVIEW_REVIEWED {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Framework review report must be reviewed before a provenance manifest can be generated",
                "code": "report_not_reviewed"
            })),
        )
            .into_response();
    }

    if report.compliance_claim
        || report.regulatory_claim
        || report.certification
        || !report.requires_auditor_review
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Framework review report claims are not eligible for a non-claim provenance manifest" })),
        )
            .into_response();
    }

    let assignments = match state
        .db
        .list_compliance_framework_review_report_assignments(&org_id, &report_id)
        .await
    {
        Ok(assignments) => assignments,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to load framework review report assignments for provenance manifest");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    let comments = match state
        .db
        .list_compliance_framework_review_report_comments(&org_id, &report_id)
        .await
    {
        Ok(comments) => comments,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to load framework review report comments for provenance manifest");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    let previous_manifest_hash = match state
        .db
        .latest_compliance_framework_review_report_manifest_hash(&org_id, &report_id)
        .await
    {
        Ok(value) => value,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to load previous framework review report manifest hash");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    let generated_at = chrono::Utc::now().timestamp_millis();
    let preimage =
        build_framework_review_report_manifest_payload(&FrameworkReviewReportManifestPayloadInput {
            manifest_id: "pending",
            generated_at,
            generated_by: &auth_user.client_id,
            report: &report,
            previous_manifest_hash: previous_manifest_hash.as_deref(),
            manifest_hash: None,
            assignments: &assignments,
            comments: &comments,
        });
    let content_hash = compliance_review_package_hash(&preimage);
    let manifest_id = deterministic_framework_review_report_manifest_id(
        &report.report_id,
        &auth_user.client_id,
        generated_at,
        previous_manifest_hash.as_deref(),
        &content_hash,
    );
    let mut artifact =
        build_framework_review_report_manifest_payload(&FrameworkReviewReportManifestPayloadInput {
            manifest_id: &manifest_id,
            generated_at,
            generated_by: &auth_user.client_id,
            report: &report,
            previous_manifest_hash: previous_manifest_hash.as_deref(),
            manifest_hash: None,
            assignments: &assignments,
            comments: &comments,
        });
    let manifest_hash = compliance_review_package_hash(&artifact);
    artifact =
        build_framework_review_report_manifest_payload(&FrameworkReviewReportManifestPayloadInput {
            manifest_id: &manifest_id,
            generated_at,
            generated_by: &auth_user.client_id,
            report: &report,
            previous_manifest_hash: previous_manifest_hash.as_deref(),
            manifest_hash: Some(&manifest_hash),
            assignments: &assignments,
            comments: &comments,
        });

    match state
        .db
        .create_compliance_framework_review_report_provenance_manifest(
            &CreateComplianceFrameworkReviewReportProvenanceManifestInput {
                manifest_id: &manifest_id,
                org_id: &org_id,
                report_id: &report.report_id,
                generated_by_user_id: &auth_user.client_id,
                manifest_hash: &manifest_hash,
                previous_manifest_hash: previous_manifest_hash.as_deref(),
                signature_algorithm: COMPLIANCE_FRAMEWORK_REVIEW_REPORT_MANIFEST_SIGNATURE_ALGORITHM,
                payload_json_redacted: &artifact,
            },
        )
        .await
    {
        Ok(manifest) => {
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "compliance_framework_review_report.provenance_manifest_created".to_string(),
                target_type: Some("compliance_framework_review_report".to_string()),
                target_id: Some(report.report_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "report_id": report.report_id,
                    "manifest_id": manifest.manifest_id,
                    "manifest_hash": manifest.manifest_hash,
                    "previous_manifest_hash": manifest.previous_manifest_hash,
                    "signature_algorithm": manifest.signature_algorithm,
                    "source_report_artifact_hash": report.artifact_hash,
                    "source_report_artifact_mutated": false,
                    "compliance_claim": false,
                    "regulatory_claim": false,
                    "certification": false,
                    "agent_governance_required": false
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write framework review report manifest audit log: {}", e);
            }
            (
                StatusCode::CREATED,
                Json(framework_review_report_manifest_response(manifest, artifact)),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to create framework review report provenance manifest");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn download_compliance_framework_review_report_provenance_manifest(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path((report_id, manifest_id)): Path<(String, String)>,
    Query(mut query): Query<ComplianceFrameworkReviewReportQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let report_id = match normalize_framework_review_report_id(&report_id) {
        Ok(report_id) => report_id,
        Err(error) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response();
        }
    };
    let manifest_id = match normalize_framework_review_report_manifest_id(&manifest_id) {
        Ok(manifest_id) => manifest_id,
        Err(error) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response();
        }
    };
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_compliance_framework_review_report_org(
        &state,
        &auth_user,
        query.org_name.as_deref(),
    )
    .await
    {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    if let Some(resp) =
        require_framework_review_report_collaboration_access(&state, &auth_user, &org_id, &report_id)
            .await
    {
        return resp;
    }

    match state
        .db
        .get_compliance_framework_review_report_manifest_payload(&org_id, &report_id, &manifest_id)
        .await
    {
        Ok(Some(payload)) => (StatusCode::OK, Json(payload)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance framework review report provenance manifest not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, manifest_id = %manifest_id, "Failed to download framework review report provenance manifest");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
