// ============================================================================
// COMPLIANCE EVIDENCE MAPPINGS
// ============================================================================

const GITGOV_BASELINE_FRAMEWORK_ID: &str = "gitgov_release_governance_baseline_v1";

fn normalize_evidence_mapping_request(
    payload: &mut ComplianceEvidenceMappingRequest,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut payload.org_name);
    payload.evidence_export_id = payload.evidence_export_id.trim().to_string();
    payload.framework_id = payload.framework_id.trim().to_ascii_lowercase();
    payload.framework_version = payload
        .framework_version
        .take()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    if !payload.evidence_export_id.starts_with("cee_") || payload.evidence_export_id.len() > 80 {
        errors.push("evidence_export_id must be a valid cee_ identifier.".to_string());
    }
    if payload.framework_id.is_empty()
        || payload.framework_id.len() > 96
        || payload.framework_id != payload.framework_id.to_ascii_lowercase()
        || !payload
            .framework_id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    {
        errors.push("framework_id must be a valid lowercase GitGov/customer framework identifier.".to_string());
    }
    if let Some(version) = &payload.framework_version {
        if version.len() > 64 {
            errors.push("framework_version must be 64 characters or less.".to_string());
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn evidence_mapping_status(
    status: &str,
    evidence_refs: Vec<&str>,
    missing_evidence: Vec<&str>,
    notes_safe: &str,
    control: &ComplianceControl,
) -> ComplianceEvidenceMappingItem {
    ComplianceEvidenceMappingItem {
        control_id: control.control_id.clone(),
        control_title: control.title.clone(),
        status: status.to_string(),
        evidence_refs: evidence_refs.into_iter().map(|value| value.to_string()).collect(),
        missing_evidence: missing_evidence
            .into_iter()
            .map(|value| value.to_string())
            .collect(),
        notes_safe: notes_safe.to_string(),
    }
}

fn json_string_exists(artifact: &serde_json::Value, pointer: &str) -> bool {
    artifact
        .pointer(pointer)
        .and_then(|value| value.as_str())
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

fn json_bool(artifact: &serde_json::Value, pointer: &str) -> Option<bool> {
    artifact.pointer(pointer).and_then(|value| value.as_bool())
}

fn json_i64(artifact: &serde_json::Value, pointer: &str) -> i64 {
    artifact
        .pointer(pointer)
        .and_then(|value| value.as_i64())
        .unwrap_or(0)
}

fn json_array_exists(artifact: &serde_json::Value, pointer: &str) -> bool {
    artifact
        .pointer(pointer)
        .and_then(|value| value.as_array())
        .is_some()
}

fn json_array_contains_string(artifact: &serde_json::Value, pointer: &str, needle: &str) -> bool {
    artifact
        .pointer(pointer)
        .and_then(|value| value.as_array())
        .map(|items| items.iter().any(|item| item.as_str() == Some(needle)))
        .unwrap_or(false)
}

fn map_gitgov_baseline_control(
    artifact: &serde_json::Value,
    control: &ComplianceControl,
) -> ComplianceEvidenceMappingItem {
    match control.control_id.as_str() {
        "GG-RG-01" => {
            if json_string_exists(artifact, "/deployment_gate/decision") {
                evidence_mapping_status(
                    "evidence_present",
                    vec!["deployment_gate.decision"],
                    vec![],
                    "Deployment Gate decision found in the evidence export.",
                    control,
                )
            } else {
                evidence_mapping_status(
                    "missing",
                    vec![],
                    vec!["deployment_gate.decision"],
                    "Deployment Gate decision is missing from the evidence export.",
                    control,
                )
            }
        }
        "GG-RG-02" => {
            let has_checksum = json_string_exists(artifact, "/policy/checksum");
            let has_source = artifact.pointer("/policy/source").is_some();
            match (has_checksum, has_source) {
                (true, true) => evidence_mapping_status(
                    "evidence_present",
                    vec!["policy.checksum", "policy.source"],
                    vec![],
                    "Policy checksum and source are present.",
                    control,
                ),
                (true, false) => evidence_mapping_status(
                    "partial",
                    vec!["policy.checksum"],
                    vec!["policy.source"],
                    "Policy checksum is present but source metadata is missing.",
                    control,
                ),
                (false, true) => evidence_mapping_status(
                    "partial",
                    vec!["policy.source"],
                    vec!["policy.checksum"],
                    "Policy source is present but checksum is missing.",
                    control,
                ),
                (false, false) => evidence_mapping_status(
                    "missing",
                    vec![],
                    vec!["policy.checksum", "policy.source"],
                    "Policy checksum and source are missing.",
                    control,
                ),
            }
        }
        "GG-RG-03" => {
            let required = json_i64(artifact, "/readiness/required_approval_count");
            let valid = json_i64(artifact, "/readiness/valid_approval_count");
            if required <= 0 {
                evidence_mapping_status(
                    "not_applicable",
                    vec!["readiness.required_approval_count"],
                    vec![],
                    "The gate did not require human release approval.",
                    control,
                )
            } else if valid >= required {
                evidence_mapping_status(
                    "evidence_present",
                    vec![
                        "readiness.valid_approval_count",
                        "readiness.required_approval_count",
                        "approvals.release_governance_approvals",
                    ],
                    vec![],
                    "Required human approval evidence is satisfied.",
                    control,
                )
            } else if valid > 0 {
                evidence_mapping_status(
                    "partial",
                    vec!["readiness.valid_approval_count"],
                    vec!["release_approval"],
                    "Some human approval evidence exists, but quorum is incomplete.",
                    control,
                )
            } else {
                evidence_mapping_status(
                    "missing",
                    vec!["readiness.required_approval_count"],
                    vec!["release_approval"],
                    "The gate required human approval, but no valid approval evidence was found.",
                    control,
                )
            }
        }
        "GG-RG-04" => {
            if json_i64(artifact, "/evidence/counts/pipeline_events") > 0 {
                evidence_mapping_status(
                    "evidence_present",
                    vec!["evidence.counts.pipeline_events", "evidence.jenkins.pipeline_event_count"],
                    vec![],
                    "CI/build evidence is present.",
                    control,
                )
            } else {
                evidence_mapping_status(
                    "missing",
                    vec![],
                    vec!["ci_build_evidence"],
                    "No CI/build evidence was found in the export.",
                    control,
                )
            }
        }
        "GG-RG-05" => {
            if json_i64(artifact, "/evidence/counts/client_events") > 0 {
                evidence_mapping_status(
                    "partial",
                    vec!["evidence.counts.client_events"],
                    vec!["pr_review_evidence"],
                    "Code activity evidence exists, but PR review evidence is not included in the KAN-99 export.",
                    control,
                )
            } else {
                evidence_mapping_status(
                    "missing",
                    vec![],
                    vec!["code_change_evidence", "pr_review_evidence"],
                    "No code activity or PR review evidence was found in the export.",
                    control,
                )
            }
        }
        "GG-RG-06" => {
            if json_array_contains_string(
                artifact,
                "/gaps/missing_evidence",
                "sonar_quality_gate",
            ) || json_array_contains_string(
                artifact,
                "/readiness/missing_evidence",
                "sonar_quality_gate",
            ) {
                evidence_mapping_status(
                    "missing",
                    vec!["gaps.missing_evidence"],
                    vec!["sonar_quality_gate"],
                    "The export explicitly reports missing security or quality evidence.",
                    control,
                )
            } else if artifact.pointer("/evidence/sonar").is_some() {
                evidence_mapping_status(
                    "manual_review_required",
                    vec!["evidence.sonar"],
                    vec!["quality_gate_result"],
                    "A quality evidence source is referenced, but the actual quality gate result requires review.",
                    control,
                )
            } else {
                evidence_mapping_status(
                    "missing",
                    vec![],
                    vec!["quality_gate_result"],
                    "No security or quality evidence reference was found.",
                    control,
                )
            }
        }
        "GG-RG-07" => {
            let refs = [
                ("/deployment_gate/repository_full_name", "deployment_gate.repository_full_name"),
                ("/deployment_gate/branch", "deployment_gate.branch"),
                ("/deployment_gate/target_sha", "deployment_gate.target_sha"),
                ("/deployment_gate/environment", "deployment_gate.environment"),
            ];
            let evidence_refs: Vec<&str> = refs
                .iter()
                .filter_map(|(pointer, name)| json_string_exists(artifact, pointer).then_some(*name))
                .collect();
            let missing: Vec<&str> = refs
                .iter()
                .filter_map(|(pointer, name)| (!json_string_exists(artifact, pointer)).then_some(*name))
                .collect();
            if missing.is_empty() {
                evidence_mapping_status(
                    "evidence_present",
                    evidence_refs,
                    vec![],
                    "Deployment target, branch, SHA, and environment are recorded.",
                    control,
                )
            } else if evidence_refs.is_empty() {
                evidence_mapping_status(
                    "missing",
                    vec![],
                    missing,
                    "Deployment target metadata is missing.",
                    control,
                )
            } else {
                evidence_mapping_status(
                    "partial",
                    evidence_refs,
                    missing,
                    "Deployment target metadata is partially recorded.",
                    control,
                )
            }
        }
        "GG-RG-08" => {
            if json_array_exists(artifact, "/gaps/missing_evidence")
                || json_array_exists(artifact, "/readiness/missing_evidence")
            {
                evidence_mapping_status(
                    "evidence_present",
                    vec!["gaps.missing_evidence", "readiness.missing_evidence"],
                    vec![],
                    "Missing evidence and gaps are explicit in the export.",
                    control,
                )
            } else {
                evidence_mapping_status(
                    "missing",
                    vec![],
                    vec!["gaps.missing_evidence"],
                    "The export does not expose an explicit missing-evidence section.",
                    control,
                )
            }
        }
        "GG-RG-09" => {
            let has_generated = artifact.pointer("/audit/export_generated_at").is_some();
            let redacted = json_bool(artifact, "/audit/artifact_redacted").unwrap_or(false);
            let raw_payload_included =
                json_bool(artifact, "/audit/raw_payload_included").unwrap_or(true);
            if has_generated && redacted && !raw_payload_included {
                evidence_mapping_status(
                    "evidence_present",
                    vec![
                        "audit.export_generated_at",
                        "audit.artifact_redacted",
                        "audit.raw_payload_included",
                    ],
                    vec![],
                    "Audit timestamps and redaction markers are present.",
                    control,
                )
            } else {
                evidence_mapping_status(
                    "partial",
                    vec!["audit"],
                    vec!["audit.redaction_markers"],
                    "Audit metadata exists but requires review for completeness.",
                    control,
                )
            }
        }
        "GG-RG-10" => match json_bool(artifact, "/deployment_gate/agent_governance_used") {
            Some(false) => evidence_mapping_status(
                "evidence_present",
                vec!["deployment_gate.agent_governance_used"],
                vec![],
                "Deployment Gate evidence confirms Agent Governance was not required.",
                control,
            ),
            Some(true) => evidence_mapping_status(
                "manual_review_required",
                vec!["deployment_gate.agent_governance_used"],
                vec!["manual_agent_governance_review"],
                "Agent Governance appears in the evidence and requires manual review.",
                control,
            ),
            None => evidence_mapping_status(
                "missing",
                vec![],
                vec!["deployment_gate.agent_governance_used"],
                "The export does not state whether Agent Governance was used.",
                control,
            ),
        },
        _ => evidence_mapping_status(
            "manual_review_required",
            vec![],
            vec!["unsupported_control"],
            "Unsupported control id for the KAN-100 deterministic mapper.",
            control,
        ),
    }
}

fn build_gitgov_baseline_mapping_items(
    artifact: &serde_json::Value,
    framework: &ComplianceControlFramework,
) -> Vec<ComplianceEvidenceMappingItem> {
    framework
        .controls
        .iter()
        .map(|control| map_gitgov_baseline_control(artifact, control))
        .collect()
}

async fn resolve_compliance_mapping_org(
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

pub async fn create_compliance_evidence_mapping(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<ComplianceEvidenceMappingRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    if let Err(errors) = normalize_evidence_mapping_request(&mut payload) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid compliance evidence mapping request", "details": errors })),
        )
            .into_response();
    }

    let org_id = match resolve_compliance_mapping_org(&state, &auth_user, payload.org_name.as_deref()).await {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    let framework = match state
        .db
        .get_compliance_control_framework(Some(&org_id), &payload.framework_id)
        .await
    {
        Ok(Some(framework)) => framework,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Compliance control framework not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, framework_id = %payload.framework_id, "Failed to load compliance control framework");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    if framework.is_regulatory {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Regulatory frameworks are not supported for customer evidence mappings" })),
        )
            .into_response();
    }
    if framework.official_regulatory_mapping {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Official regulatory mappings are not supported by this customer-owned review flow" })),
        )
            .into_response();
    }
    if let Some(requested_version) = &payload.framework_version {
        if requested_version != &framework.version {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "framework_version does not match the active framework version",
                    "expected": framework.version,
                    "received": requested_version
                })),
            )
                .into_response();
        }
    }

    let export = match state
        .db
        .get_compliance_evidence_export(&org_id, &payload.evidence_export_id)
        .await
    {
        Ok(Some(record)) => record,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Compliance evidence export not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, export_id = %payload.evidence_export_id, "Failed to load compliance evidence export");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    let artifact = match state
        .db
        .get_compliance_evidence_export_payload(&org_id, &payload.evidence_export_id)
        .await
    {
        Ok(Some(payload)) => payload,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Compliance evidence export artifact not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, export_id = %payload.evidence_export_id, "Failed to load compliance evidence export artifact");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    let items = if framework.framework_id == GITGOV_BASELINE_FRAMEWORK_ID && framework.is_gitgov_owned {
        build_gitgov_baseline_mapping_items(&artifact, &framework)
    } else {
        build_customer_framework_mapping_items(&artifact, &framework)
    };
    let mapping_id = format!("cem_{}", Uuid::new_v4().simple());
    let item_inputs = items
        .into_iter()
        .map(|item| CreateComplianceEvidenceMappingItemInput {
            item_id: format!("cemi_{}", Uuid::new_v4().simple()),
            control_id: item.control_id,
            control_title: item.control_title,
            status: item.status,
            evidence_refs: item.evidence_refs,
            missing_evidence: item.missing_evidence,
            notes_safe: item.notes_safe,
        })
        .collect();

    match state
        .db
        .create_compliance_evidence_mapping(CreateComplianceEvidenceMappingInput {
            mapping_id,
            org_id: org_id.clone(),
            evidence_export_id: export.export_id.clone(),
            evidence_export_hash: export.artifact_hash.clone(),
            framework_id: framework.framework_id.clone(),
            framework_version: framework.version.clone(),
            created_by_user_id: auth_user.client_id.clone(),
            items: item_inputs,
        })
        .await
    {
        Ok(response) => {
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "compliance_evidence_mapping.created".to_string(),
                target_type: Some("compliance_evidence_mapping".to_string()),
                target_id: Some(response.mapping.mapping_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "mapping_id": response.mapping.mapping_id,
                    "evidence_export_id": response.mapping.evidence_export_id,
                    "evidence_export_hash": response.mapping.evidence_export_hash,
                    "framework_id": response.mapping.framework_id,
                    "framework_version": response.mapping.framework_version,
                    "compliance_claim": false,
                    "regulatory_claim": false,
                    "requires_auditor_review": true
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write compliance evidence mapping audit log: {}", e);
            }
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, export_id = %payload.evidence_export_id, "Failed to create compliance evidence mapping");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn get_compliance_evidence_mapping(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(mapping_id): Path<String>,
    Query(mut query): Query<ComplianceEvidenceMappingQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_compliance_mapping_org(&state, &auth_user, query.org_name.as_deref()).await {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    match state
        .db
        .get_compliance_evidence_mapping(&org_id, &mapping_id)
        .await
    {
        Ok(Some(response)) => (StatusCode::OK, Json(response)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance evidence mapping not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, mapping_id = %mapping_id, "Failed to load compliance evidence mapping");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
