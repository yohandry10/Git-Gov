// ============================================================================
// COMPLIANCE EVIDENCE EXPORTS
// ============================================================================

const COMPLIANCE_EXPORT_DEFAULT_SECTIONS: &[&str] = &[
    "gate_decision",
    "policy",
    "readiness",
    "approvals",
    "evidence",
    "gaps",
    "audit",
];

fn normalize_compliance_export_sections(sections: &[String]) -> Result<Vec<String>, Vec<String>> {
    let mut errors = Vec::new();
    let mut normalized = Vec::new();
    let source: Vec<String> = if sections.is_empty() {
        COMPLIANCE_EXPORT_DEFAULT_SECTIONS
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
        if !COMPLIANCE_EXPORT_DEFAULT_SECTIONS.contains(&section.as_str()) {
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

fn normalize_compliance_export_request(
    payload: &mut ComplianceEvidenceExportRequest,
) -> Result<Vec<String>, Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut payload.org_name);
    normalize_release_approval_optional_text(&mut payload.deployment_gate_id);
    payload.scope = payload.scope.trim().to_ascii_lowercase();
    payload.format = payload
        .format
        .take()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .or_else(|| Some("json".to_string()));

    if payload.scope != "deployment_gate" {
        errors.push("scope must be deployment_gate for the KAN-99 MVP.".to_string());
    }
    if !payload
        .deployment_gate_id
        .as_deref()
        .is_some_and(|value| value.starts_with("dga_") && value.len() <= 80)
    {
        errors.push("deployment_gate_id must be a valid dga_ identifier.".to_string());
    }
    if payload.format.as_deref() != Some("json") {
        errors.push("format must be json for the KAN-99 MVP.".to_string());
    }

    match normalize_compliance_export_sections(&payload.include_sections) {
        Ok(sections) if errors.is_empty() => Ok(sections),
        Ok(_) => Err(errors),
        Err(mut section_errors) => {
            errors.append(&mut section_errors);
            Err(errors)
        }
    }
}

fn section_enabled(sections: &[String], section: &str) -> bool {
    sections.iter().any(|value| value == section)
}

fn compliance_export_hash(artifact: &serde_json::Value) -> String {
    let content = serde_json::to_string(artifact).unwrap_or_else(|_| "{}".to_string());
    format!("sha256:{:x}", Sha256::digest(content.as_bytes()))
}

fn deployment_gate_missing_evidence(gate: &DeploymentGateAuthorizationRecord) -> Vec<String> {
    let mut missing = Vec::new();
    if let Some(values) = gate
        .governance_decision
        .get("evidence")
        .and_then(|value| value.get("missing_evidence"))
        .and_then(|value| value.as_array())
    {
        for value in values {
            if let Some(text) = value.as_str() {
                missing.push(text.to_string());
            }
        }
    }
    for item in &gate.blocked_by {
        if !missing.contains(item) {
            missing.push(item.clone());
        }
    }
    missing
}

fn build_compliance_evidence_artifact(
    export_id: &str,
    org_id: &str,
    created_by: &str,
    gate: &DeploymentGateAuthorizationRecord,
    sections: &[String],
    evidence_context: serde_json::Value,
    generated_at: i64,
) -> serde_json::Value {
    let agent_governance_used = gate
        .governance_decision
        .get("agent_governance_used")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let missing_evidence = deployment_gate_missing_evidence(gate);
    let mut artifact = serde_json::Map::new();
    artifact.insert("export_id".to_string(), json!(export_id));
    artifact.insert("generated_at".to_string(), json!(generated_at));
    artifact.insert("org_id".to_string(), json!(org_id));
    artifact.insert("created_by_user_id".to_string(), json!(created_by));
    artifact.insert("scope".to_string(), json!("deployment_gate"));
    artifact.insert(
        "positioning".to_string(),
        json!({
            "purpose": "Evidence package for audit/review",
            "compliance_claim": false,
            "framework_mapping": false
        }),
    );
    artifact.insert("include_sections".to_string(), json!(sections));

    if section_enabled(sections, "gate_decision") {
        artifact.insert(
            "deployment_gate".to_string(),
            json!({
                "id": gate.authorization_id,
                "release_id": gate.release_id,
                "repository_full_name": gate.repository_full_name,
                "branch": gate.branch,
                "target_sha": gate.target_sha,
                "environment": gate.environment,
                "deployer": gate.deployer,
                "ticket_id": gate.ticket_id,
                "decision": gate.decision,
                "approved": gate.approved,
                "blocking": gate.blocking,
                "would_block": gate.would_block,
                "reason": gate.reason,
                "consumer_type": gate.governance_decision.get("consumer_type").cloned().unwrap_or(json!("deployment_gate")),
                "agent_governance_used": agent_governance_used,
                "created_at": gate.created_at
            }),
        );
    }

    if section_enabled(sections, "policy") {
        artifact.insert(
            "policy".to_string(),
            json!({
                "checksum": gate.policy_checksum,
                "source": gate.details.get("policy").cloned().unwrap_or(json!({"source": "gitgov_policy"})),
                "shared_decision_version": gate.governance_decision.get("version").cloned().unwrap_or(json!("shared-governance-decision.v1")),
                "llm_decision": false
            }),
        );
    }

    if section_enabled(sections, "readiness") {
        artifact.insert(
            "readiness".to_string(),
            json!({
                "status": gate.evaluation.status,
                "policy_satisfied": gate.evaluation.policy_satisfied,
                "blocking": gate.evaluation.blocking,
                "would_block": gate.evaluation.would_block,
                "valid_approval_count": gate.evaluation.valid_approval_count,
                "required_approval_count": gate.evaluation.required_approval_count,
                "issues": gate.evaluation.issues,
                "next_steps": gate.evaluation.next_steps,
                "missing_evidence": missing_evidence
            }),
        );
    }

    if section_enabled(sections, "approvals") {
        artifact.insert(
            "approvals".to_string(),
            json!({
                "release_governance_approvals": gate.evaluation.approvals,
                "break_glass": {
                    "eligible": gate.break_glass_eligible,
                    "used": gate.break_glass_used,
                    "approval_id": gate.break_glass_approval_id,
                    "approval_hash": gate.break_glass_approval_hash,
                    "expires_at": gate.break_glass_expires_at
                }
            }),
        );
    }

    if section_enabled(sections, "evidence") {
        artifact.insert(
            "evidence".to_string(),
            json!({
                "evidence_packet_hash": gate.evidence_packet_hash,
                "evidence_packet_uri": gate.evidence_packet_uri,
                "counts": evidence_context.get("counts").cloned().unwrap_or(json!({})),
                "github": {
                    "client_event_count": evidence_context["counts"]["client_events"]
                },
                "jira": {
                    "ticket_id": gate.ticket_id,
                    "ticket_count": evidence_context["counts"]["jira_tickets"]
                },
                "jenkins": {
                    "pipeline_event_count": evidence_context["counts"]["pipeline_events"]
                },
                "sonar": {
                    "quality_gate_source": "pipeline_or_quality_events"
                }
            }),
        );
    }

    if section_enabled(sections, "gaps") {
        artifact.insert(
            "gaps".to_string(),
            json!({
                "missing_evidence": missing_evidence,
                "blocked_by": gate.blocked_by,
                "warnings": gate.warnings
            }),
        );
    }

    if section_enabled(sections, "audit") {
        artifact.insert(
            "audit".to_string(),
            json!({
                "requested_by": gate.requested_by,
                "deployment_gate_created_at": gate.created_at,
                "export_generated_at": generated_at,
                "admin_audit_event_count": evidence_context["counts"]["admin_audit_events"],
                "artifact_redacted": true,
                "raw_payload_included": false
            }),
        );
    }

    serde_json::Value::Object(artifact)
}

pub async fn create_compliance_evidence_export(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<ComplianceEvidenceExportRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    let sections = match normalize_compliance_export_request(&mut payload) {
        Ok(sections) => sections,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid compliance evidence export request", "details": errors })),
            )
                .into_response();
        }
    };

    let org_id = match resolve_and_check_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        payload.org_name.as_deref(),
        true,
    )
    .await
    {
        Ok(Some(org_id)) => org_id,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "org_name is required for global admin keys" })),
            )
                .into_response();
        }
        Err(err) => {
            return (
                org_scope_status(err),
                Json(json!({ "error": agent_governance_scope_error_message(err) })),
            )
                .into_response();
        }
    };

    let deployment_gate_id = payload.deployment_gate_id.clone().unwrap_or_default();
    let gate = match state
        .db
        .get_deployment_gate_authorization_by_id(&org_id, &deployment_gate_id)
        .await
    {
        Ok(Some(gate)) => gate,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Deployment gate authorization not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, deployment_gate_id = %deployment_gate_id, "Failed to load deployment gate for compliance export");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    let evidence_context = match state
        .db
        .get_compliance_evidence_context(&org_id, &gate)
        .await
    {
        Ok(context) => context,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, deployment_gate_id = %deployment_gate_id, "Failed to load compliance evidence context");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    let export_id = format!("cee_{}", Uuid::new_v4().simple());
    let generated_at = chrono::Utc::now().timestamp_millis();
    let artifact = build_compliance_evidence_artifact(
        &export_id,
        &org_id,
        &auth_user.client_id,
        &gate,
        &sections,
        evidence_context,
        generated_at,
    );
    let artifact_hash = compliance_export_hash(&artifact);

    match state
        .db
        .create_compliance_evidence_export(&CreateComplianceEvidenceExportInput {
            export_id: &export_id,
            org_id: &org_id,
            created_by_user_id: &auth_user.client_id,
            scope: "deployment_gate",
            deployment_gate_id: Some(&gate.authorization_id),
            release_id: Some(&gate.release_id),
            status: "completed",
            format: "json",
            artifact_hash: &artifact_hash,
            policy_checksum: Some(&gate.policy_checksum),
            gate_decision: Some(&gate.decision),
            payload_json_redacted: &artifact,
        })
        .await
    {
        Ok(record) => {
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "compliance_evidence_export.created".to_string(),
                target_type: Some("compliance_evidence_export".to_string()),
                target_id: Some(record.export_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "export_id": record.export_id,
                    "deployment_gate_id": gate.authorization_id,
                    "artifact_hash": record.artifact_hash,
                    "scope": record.scope,
                    "format": record.format,
                    "agent_governance_used": gate.governance_decision.get("agent_governance_used").and_then(|value| value.as_bool()).unwrap_or(false)
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write compliance evidence export audit log: {}", e);
            }
            (
                StatusCode::CREATED,
                Json(ComplianceEvidenceExportResponse {
                    export: record,
                    artifact: Some(artifact),
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, deployment_gate_id = %deployment_gate_id, "Failed to create compliance evidence export");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

async fn resolve_compliance_export_org(
    state: &Arc<AppState>,
    auth_user: &AuthUser,
    query: &ComplianceEvidenceExportQuery,
) -> Result<String, axum::response::Response> {
    match resolve_and_check_org_scope(
        state,
        auth_user.org_id.as_deref(),
        query.org_name.as_deref(),
        true,
    )
    .await
    {
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

pub async fn get_compliance_evidence_export(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(export_id): Path<String>,
    Query(mut query): Query<ComplianceEvidenceExportQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_compliance_export_org(&state, &auth_user, &query).await {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    match state
        .db
        .get_compliance_evidence_export(&org_id, &export_id)
        .await
    {
        Ok(Some(record)) => (
            StatusCode::OK,
            Json(ComplianceEvidenceExportResponse {
                export: record,
                artifact: None,
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance evidence export not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, export_id = %export_id, "Failed to load compliance evidence export");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn download_compliance_evidence_export(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(export_id): Path<String>,
    Query(mut query): Query<ComplianceEvidenceExportQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_compliance_export_org(&state, &auth_user, &query).await {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    match state
        .db
        .get_compliance_evidence_export_payload(&org_id, &export_id)
        .await
    {
        Ok(Some(payload)) => (StatusCode::OK, Json(payload)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance evidence export not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, export_id = %export_id, "Failed to download compliance evidence export");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
