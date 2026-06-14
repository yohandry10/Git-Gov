// ============================================================================
// DEPLOYMENT GATE BREAK-GLASS APPROVALS
// ============================================================================

const DEPLOYMENT_GATE_BREAK_GLASS_APPROVAL_ROLES: &[&str] = &[
    "incident_commander",
    "security",
    "release_manager",
    "platform_admin",
];

fn normalize_and_validate_break_glass_approval(
    payload: &mut CreateDeploymentGateBreakGlassApprovalRequest,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    normalize_release_approval_optional_text(&mut payload.org_name);
    normalize_release_approval_optional_text(&mut payload.ticket_id);
    normalize_release_approval_optional_text(&mut payload.evidence_packet_uri);
    normalize_release_approval_optional_text(&mut payload.approver_role);
    payload.release_id = payload.release_id.trim().to_string();
    payload.repository_full_name = payload.repository_full_name.trim().to_string();
    payload.branch = payload.branch.trim().to_string();
    payload.target_sha = payload.target_sha.trim().to_ascii_lowercase();
    payload.environment = payload.environment.trim().to_ascii_lowercase();
    payload.evidence_packet_hash = payload.evidence_packet_hash.trim().to_ascii_lowercase();
    payload.reason = payload.reason.trim().to_string();
    payload.approver = payload.approver.trim().to_string();
    payload.approver_role = payload
        .approver_role
        .take()
        .map(|role| role.trim().to_ascii_lowercase())
        .filter(|role| !role.is_empty())
        .or_else(|| Some("incident_commander".to_string()));

    if payload.metadata.is_null() {
        payload.metadata = json!({});
    }

    let mut gate_payload = DeploymentGateAuthorizationRequest {
        org_name: payload.org_name.clone(),
        release_id: payload.release_id.clone(),
        repository_full_name: payload.repository_full_name.clone(),
        branch: payload.branch.clone(),
        target_sha: payload.target_sha.clone(),
        environment: payload.environment.clone(),
        deployer: "approval-validator".to_string(),
        ticket_id: payload.ticket_id.clone(),
        evidence_packet_hash: payload.evidence_packet_hash.clone(),
        evidence_packet_uri: payload.evidence_packet_uri.clone(),
        requested_by: None,
        deployment_run_id: None,
        metadata: json!({}),
        break_glass: None,
    };
    if let Err(mut gate_errors) = normalize_and_validate_deployment_gate_authorization(&mut gate_payload) {
        errors.append(&mut gate_errors);
    }
    payload.release_id = gate_payload.release_id;
    payload.repository_full_name = gate_payload.repository_full_name;
    payload.branch = gate_payload.branch;
    payload.target_sha = gate_payload.target_sha;
    payload.environment = gate_payload.environment;
    payload.ticket_id = gate_payload.ticket_id;
    payload.evidence_packet_hash = gate_payload.evidence_packet_hash;
    payload.evidence_packet_uri = gate_payload.evidence_packet_uri;

    if payload.reason.len() < DEPLOYMENT_GATE_BREAK_GLASS_REASON_MIN_CHARS
        || payload.reason.len() > DEPLOYMENT_GATE_BREAK_GLASS_REASON_MAX_CHARS
        || has_control_chars(&payload.reason)
    {
        errors.push(format!(
            "reason must be {DEPLOYMENT_GATE_BREAK_GLASS_REASON_MIN_CHARS}-{DEPLOYMENT_GATE_BREAK_GLASS_REASON_MAX_CHARS} characters without control characters."
        ));
    }

    if payload.approver.is_empty() {
        errors.push("approver is required.".to_string());
    } else if payload.approver.len() > 200 || has_control_chars(&payload.approver) {
        errors.push("approver is invalid or too long.".to_string());
    }

    let approver_role = payload.approver_role.as_deref().unwrap_or("incident_commander");
    if !DEPLOYMENT_GATE_BREAK_GLASS_APPROVAL_ROLES.contains(&approver_role) {
        errors.push("approver_role must be incident_commander, security, release_manager, or platform_admin.".to_string());
    }

    let now_ms = chrono::Utc::now().timestamp_millis();
    if payload.expires_at <= now_ms {
        errors.push("expires_at must be in the future.".to_string());
    } else if payload.expires_at > now_ms + 24 * 60 * 60 * 1000 {
        errors.push("expires_at cannot be more than 24 hours in the future.".to_string());
    }

    if !payload.metadata.is_object() {
        errors.push("metadata must be a JSON object.".to_string());
    } else {
        let metadata_len = serde_json::to_vec(&payload.metadata)
            .map(|bytes| bytes.len())
            .unwrap_or(0);
        if metadata_len > DEPLOYMENT_GATE_METADATA_MAX_BYTES {
            errors.push("metadata is too large.".to_string());
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}


fn normalize_break_glass_approval_query(
    query: &mut DeploymentGateBreakGlassApprovalQuery,
) -> Result<(i64, i64), Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut query.org_name);
    normalize_release_approval_optional_text(&mut query.approval_id);
    normalize_release_approval_optional_text(&mut query.repository_full_name);
    normalize_release_approval_optional_text(&mut query.branch);
    normalize_release_approval_optional_text(&mut query.target_sha);
    normalize_release_approval_optional_text(&mut query.release_id);
    normalize_release_approval_optional_text(&mut query.environment);
    normalize_release_approval_optional_text(&mut query.evidence_packet_hash);
    normalize_release_approval_optional_text(&mut query.approver);

    if let Some(approval_id) = query.approval_id.as_deref() {
        if approval_id.len() > 80 || has_control_chars(approval_id) {
            errors.push("approval_id is invalid or too long.".to_string());
        }
    }
    if let Some(repo) = query.repository_full_name.as_deref() {
        if !is_valid_release_approval_repo(repo) {
            errors.push("repository_full_name must look like owner/repo.".to_string());
        }
    }
    if let Some(branch) = query.branch.as_deref() {
        if branch.len() > 200 || has_control_chars(branch) {
            errors.push("branch is invalid or too long.".to_string());
        }
    }
    if let Some(target_sha) = query.target_sha.as_mut() {
        if is_valid_release_approval_sha(target_sha) {
            *target_sha = target_sha.to_ascii_lowercase();
        } else {
            errors.push(
                "target_sha must be a full 40 or 64 character hexadecimal commit SHA.".to_string(),
            );
        }
    }
    if let Some(environment) = query.environment.as_mut() {
        *environment = environment.to_ascii_lowercase();
        if environment.len() > 80 || has_control_chars(environment) {
            errors.push("environment is invalid or too long.".to_string());
        }
    }
    if let Some(hash) = query.evidence_packet_hash.as_mut() {
        if is_valid_release_approval_hex_hash(hash) {
            *hash = hash.to_ascii_lowercase();
        } else {
            errors.push("evidence_packet_hash must be a 64-character hex SHA-256 hash.".to_string());
        }
    }
    if let Some(approver) = query.approver.as_deref() {
        if approver.len() > 200 || has_control_chars(approver) {
            errors.push("approver is invalid or too long.".to_string());
        }
    }

    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);

    if errors.is_empty() {
        Ok((limit, offset))
    } else {
        Err(errors)
    }
}


fn compute_break_glass_approval_hash(
    approval_id: &str,
    org_id: &str,
    created_by: &str,
    payload: &CreateDeploymentGateBreakGlassApprovalRequest,
) -> String {
    let canonical = json!({
        "approval_id": approval_id,
        "org_id": org_id,
        "release_id": &payload.release_id,
        "repository_full_name": &payload.repository_full_name,
        "branch": &payload.branch,
        "target_sha": &payload.target_sha,
        "environment": &payload.environment,
        "ticket_id": &payload.ticket_id,
        "evidence_packet_hash": &payload.evidence_packet_hash,
        "evidence_packet_uri": &payload.evidence_packet_uri,
        "reason": &payload.reason,
        "approver": &payload.approver,
        "approver_role": &payload.approver_role,
        "expires_at": payload.expires_at,
        "metadata": &payload.metadata,
        "created_by": created_by
    });
    let digest = Sha256::digest(
        serde_json::to_vec(&canonical)
            .expect("canonical break-glass approval JSON should serialize"),
    );
    format!("{digest:x}")
}

fn break_glass_approval_matches_authorization(
    approval: &DeploymentGateBreakGlassApprovalRecord,
    payload: &DeploymentGateAuthorizationRequest,
    request: &DeploymentGateBreakGlassRequest,
    now_ms: i64,
) -> Result<(), String> {
    if approval.expires_at <= now_ms {
        return Err("matching break-glass approval is expired.".to_string());
    }
    if let Some(requested_approval_id) = request.approval_id.as_deref() {
        if approval.approval_id != requested_approval_id {
            return Err("break_glass.approval_id does not match the valid approval.".to_string());
        }
    }
    if approval.release_id != payload.release_id
        || approval.repository_full_name != payload.repository_full_name
        || approval.branch != payload.branch
        || approval.target_sha != payload.target_sha
        || !approval.environment.eq_ignore_ascii_case(&payload.environment)
        || approval.evidence_packet_hash != payload.evidence_packet_hash
    {
        return Err("break-glass approval does not match deployment scope.".to_string());
    }
    if payload
        .ticket_id
        .as_deref()
        .is_some_and(|ticket_id| Some(ticket_id) != approval.ticket_id.as_deref())
    {
        return Err("break-glass approval does not match ticket_id.".to_string());
    }
    if approval.approver == payload.deployer
        || payload
            .requested_by
            .as_deref()
            .is_some_and(|requested_by| requested_by == approval.approver)
    {
        return Err("break-glass approver must be separate from deployer/requester.".to_string());
    }
    Ok(())
}


pub async fn create_deployment_gate_break_glass_approval(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<CreateDeploymentGateBreakGlassApprovalRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    if let Err(errors) = normalize_and_validate_break_glass_approval(&mut payload) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid break-glass approval", "details": errors })),
        )
            .into_response();
    }

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
                Json(json!({ "error": release_approval_scope_error_message(err) })),
            )
                .into_response();
        }
    };

    let binding = match state
        .db
        .get_release_evidence_packet_binding(&org_id, &payload.evidence_packet_hash)
        .await
    {
        Ok(Some(binding)) => binding,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Invalid break-glass approval",
                    "details": ["evidence_packet_hash is not a known release evidence packet for this organization."]
                })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to verify break-glass approval evidence packet binding");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    let approval_query = EnterpriseReleaseGovernanceEvaluationQuery {
        org_name: None,
        repository_full_name: payload.repository_full_name.clone(),
        branch: Some(payload.branch.clone()),
        target_sha: Some(payload.target_sha.clone()),
        release_id: payload.release_id.clone(),
        environment: payload.environment.clone(),
        evidence_packet_hash: Some(payload.evidence_packet_hash.clone()),
    };
    let binding_mismatches = release_evidence_binding_query_mismatches(&binding, &approval_query);
    if !binding_mismatches.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid break-glass approval",
                "details": binding_mismatches
                    .into_iter()
                    .map(|field| format!("evidence packet binding does not match {field}."))
                    .collect::<Vec<_>>()
            })),
        )
            .into_response();
    }
    if payload
        .ticket_id
        .as_deref()
        .is_some_and(|ticket_id| ticket_id != binding.ticket_id)
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid break-glass approval",
                "details": ["evidence packet binding does not match ticket_id."]
            })),
        )
            .into_response();
    }
    if payload.ticket_id.is_none() {
        payload.ticket_id = Some(binding.ticket_id);
    }
    if payload.evidence_packet_uri.is_none() {
        payload.evidence_packet_uri = Some(binding.evidence_packet_uri);
    }

    let approval_id = format!("dgbga_{}", Uuid::new_v4().simple());
    let approval_hash =
        compute_break_glass_approval_hash(&approval_id, &org_id, &auth_user.client_id, &payload);

    match state
        .db
        .create_deployment_gate_break_glass_approval(
            &org_id,
            &approval_id,
            &payload,
            &approval_hash,
            &auth_user.client_id,
        )
        .await
    {
        Ok(record) => {
            let audit = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "create_deployment_gate_break_glass_approval".to_string(),
                target_type: Some("deployment_gate_break_glass_approval".to_string()),
                target_id: Some(record.approval_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "approval_id": &record.approval_id,
                    "release_id": &record.release_id,
                    "repository_full_name": &record.repository_full_name,
                    "branch": &record.branch,
                    "target_sha": &record.target_sha,
                    "environment": &record.environment,
                    "ticket_id": &record.ticket_id,
                    "evidence_packet_hash": &record.evidence_packet_hash,
                    "approver": &record.approver,
                    "approver_role": &record.approver_role,
                    "approval_hash": &record.approval_hash,
                    "expires_at": record.expires_at
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
                tracing::warn!(error = %e, "Failed to write admin audit log (break-glass approval)");
            }

            (StatusCode::CREATED, Json(record)).into_response()
        }
        Err(DbError::Duplicate(_)) => (
            StatusCode::CONFLICT,
            Json(json!({ "error": "Duplicate break-glass approval" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to create deployment gate break-glass approval");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn list_deployment_gate_break_glass_approvals(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(mut query): Query<DeploymentGateBreakGlassApprovalQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    let (limit, offset) = match normalize_break_glass_approval_query(&mut query) {
        Ok(values) => values,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid break-glass approval query", "details": errors })),
            )
                .into_response();
        }
    };

    let org_id = match resolve_and_check_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        query.org_name.as_deref(),
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
                Json(json!({ "error": release_approval_scope_error_message(err) })),
            )
                .into_response();
        }
    };

    match state
        .db
        .list_deployment_gate_break_glass_approvals(&org_id, &query, limit, offset)
        .await
    {
        Ok((items, total)) => (
            StatusCode::OK,
            Json(DeploymentGateBreakGlassApprovalListResponse {
                items,
                total,
                limit,
                offset,
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to list deployment gate break-glass approvals");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}


