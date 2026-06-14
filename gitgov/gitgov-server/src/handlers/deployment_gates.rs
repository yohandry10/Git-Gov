// ============================================================================
// DEPLOYMENT GATE AUTHORIZATIONS
// ============================================================================

const DEPLOYMENT_GATE_DECISIONS: &[&str] = &["approved", "advisory", "blocked", "break_glass"];
const DEPLOYMENT_GATE_METADATA_MAX_BYTES: usize = 16 * 1024;
const DEPLOYMENT_GATE_BREAK_GLASS_REASON_MIN_CHARS: usize = 16;
const DEPLOYMENT_GATE_BREAK_GLASS_REASON_MAX_CHARS: usize = 1200;

fn normalize_and_validate_deployment_gate_authorization(
    payload: &mut DeploymentGateAuthorizationRequest,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    normalize_release_approval_optional_text(&mut payload.org_name);
    normalize_release_approval_optional_text(&mut payload.ticket_id);
    normalize_release_approval_optional_text(&mut payload.evidence_packet_uri);
    normalize_release_approval_optional_text(&mut payload.requested_by);
    normalize_release_approval_optional_text(&mut payload.deployment_run_id);
    payload.release_id = payload.release_id.trim().to_string();
    payload.repository_full_name = payload.repository_full_name.trim().to_string();
    payload.branch = payload.branch.trim().to_string();
    payload.target_sha = payload.target_sha.trim().to_ascii_lowercase();
    payload.environment = payload.environment.trim().to_ascii_lowercase();
    payload.deployer = payload.deployer.trim().to_string();
    payload.evidence_packet_hash = payload.evidence_packet_hash.trim().to_ascii_lowercase();

    if payload.metadata.is_null() {
        payload.metadata = json!({});
    }

    let mut evaluation_query = EnterpriseReleaseGovernanceEvaluationQuery {
        org_name: payload.org_name.clone(),
        repository_full_name: payload.repository_full_name.clone(),
        branch: Some(payload.branch.clone()),
        target_sha: Some(payload.target_sha.clone()),
        release_id: payload.release_id.clone(),
        environment: payload.environment.clone(),
        evidence_packet_hash: Some(payload.evidence_packet_hash.clone()),
    };
    if let Err(mut query_errors) = normalize_release_governance_evaluation_query(&mut evaluation_query) {
        errors.append(&mut query_errors);
    }

    if payload.deployer.is_empty() {
        errors.push("deployer is required.".to_string());
    } else if payload.deployer.len() > 160 || has_control_chars(&payload.deployer) {
        errors.push("deployer is invalid or too long.".to_string());
    }

    if let Some(ticket_id) = payload.ticket_id.as_mut() {
        if ticket_id.len() <= 32 && is_valid_release_approval_ticket_id(ticket_id) {
            *ticket_id = ticket_id.to_ascii_uppercase();
        } else {
            errors.push("ticket_id must look like KAN-83.".to_string());
        }
    }

    if let Some(uri) = payload.evidence_packet_uri.as_deref() {
        if uri.len() > 500
            || uri.contains(char::is_whitespace)
            || !is_valid_release_approval_evidence_uri(uri)
        {
            errors
                .push("evidence_packet_uri must be a relative API path or https URL.".to_string());
        }
    }

    for (field, value) in [
        ("requested_by", payload.requested_by.as_deref()),
        ("deployment_run_id", payload.deployment_run_id.as_deref()),
    ] {
        if let Some(value) = value {
            if value.len() > 200 || has_control_chars(value) {
                errors.push(format!("{field} is invalid or too long."));
            }
        }
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

    if let Some(break_glass) = payload.break_glass.as_mut() {
        normalize_release_approval_optional_text(&mut break_glass.approval_id);
        normalize_release_approval_optional_text(&mut break_glass.authorized_by);
        break_glass.reason = break_glass.reason.trim().to_string();
        if !break_glass.requested {
            errors.push("break_glass.requested must be true when break_glass is provided.".to_string());
        }
        if break_glass.reason.len() < DEPLOYMENT_GATE_BREAK_GLASS_REASON_MIN_CHARS
            || break_glass.reason.len() > DEPLOYMENT_GATE_BREAK_GLASS_REASON_MAX_CHARS
            || has_control_chars(&break_glass.reason)
        {
            errors.push(format!(
                "break_glass.reason must be {DEPLOYMENT_GATE_BREAK_GLASS_REASON_MIN_CHARS}-{DEPLOYMENT_GATE_BREAK_GLASS_REASON_MAX_CHARS} characters without control characters."
            ));
        }
        if let Some(authorized_by) = break_glass.authorized_by.as_deref() {
            if authorized_by.len() > 200 || has_control_chars(authorized_by) {
                errors.push("break_glass.authorized_by is invalid or too long.".to_string());
            }
        }
        if let Some(approval_id) = break_glass.approval_id.as_deref() {
            if approval_id.len() > 80
                || has_control_chars(approval_id)
                || !approval_id.starts_with("dgbga_")
            {
                errors.push("break_glass.approval_id is invalid.".to_string());
            }
        }
        if let Some(expires_at) = break_glass.expires_at {
            let now_ms = chrono::Utc::now().timestamp_millis();
            if expires_at <= now_ms {
                errors.push("break_glass.expires_at must be in the future.".to_string());
            } else if expires_at > now_ms + 24 * 60 * 60 * 1000 {
                errors.push("break_glass.expires_at cannot be more than 24 hours in the future.".to_string());
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn normalize_deployment_gate_authorization_query(
    query: &mut DeploymentGateAuthorizationQuery,
) -> Result<(i64, i64), Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut query.org_name);
    normalize_release_approval_optional_text(&mut query.authorization_id);
    normalize_release_approval_optional_text(&mut query.repository_full_name);
    normalize_release_approval_optional_text(&mut query.branch);
    normalize_release_approval_optional_text(&mut query.target_sha);
    normalize_release_approval_optional_text(&mut query.release_id);
    normalize_release_approval_optional_text(&mut query.environment);
    normalize_release_approval_optional_text(&mut query.decision);
    normalize_release_approval_optional_text(&mut query.deployer);

    if let Some(authorization_id) = query.authorization_id.as_deref() {
        if authorization_id.len() > 80 || has_control_chars(authorization_id) {
            errors.push("authorization_id is invalid or too long.".to_string());
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
    if let Some(decision) = query.decision.as_mut() {
        *decision = decision.to_ascii_lowercase();
        if !DEPLOYMENT_GATE_DECISIONS.contains(&decision.as_str()) {
            errors.push("decision must be approved, advisory, or blocked.".to_string());
        }
    }
    if let Some(deployer) = query.deployer.as_deref() {
        if deployer.len() > 160 || has_control_chars(deployer) {
            errors.push("deployer is invalid or too long.".to_string());
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

fn deployment_gate_policy_checksum(evaluation: &EnterpriseReleaseGovernanceEvaluationResponse) -> String {
    let canonical = json!({
        "mode": &evaluation.policy.mode,
        "environment": &evaluation.policy.environment,
        "approval_required": evaluation.policy.approval_required,
        "enforcement": &evaluation.policy.enforcement,
        "policy_applies": evaluation.policy.policy_applies,
        "quorum_enabled": evaluation.policy.quorum_enabled,
        "quorum_rules": &evaluation.policy.quorum_rules
    });
    let digest = Sha256::digest(
        serde_json::to_vec(&canonical).expect("deployment gate policy summary should serialize"),
    );
    format!("{digest:x}")
}

fn deployment_gate_setup_warnings(
    setup: Option<&FirstGovernedRepoSetupRecord>,
    payload: &DeploymentGateAuthorizationRequest,
) -> Vec<String> {
    let mut warnings = Vec::new();
    let Some(setup) = setup else {
        warnings.push("No first governed repo setup was found for this organization.".to_string());
        return warnings;
    };

    if !matches!(setup.status.as_str(), "ready" | "completed") {
        warnings.push(format!(
            "First governed repo setup is '{}' rather than ready or completed.",
            setup.status
        ));
    }
    if setup.repository_full_name != payload.repository_full_name {
        warnings.push(format!(
            "First governed repo setup targets '{}' but deployment requested '{}'.",
            setup.repository_full_name, payload.repository_full_name
        ));
    }
    if setup.default_branch != payload.branch {
        warnings.push(format!(
            "First governed repo setup default branch is '{}' but deployment requested '{}'.",
            setup.default_branch, payload.branch
        ));
    }
    if setup
        .baseline
        .get("gate_readiness")
        .and_then(|value| value.as_str())
        .is_some_and(|value| value != "ready")
    {
        warnings.push("First governed repo baseline is not marked gate-ready.".to_string());
    }

    warnings
}

fn deployment_gate_details(
    binding: &ReleaseEvidencePacketBinding,
    setup: Option<&FirstGovernedRepoSetupRecord>,
    payload: &DeploymentGateAuthorizationRequest,
) -> serde_json::Value {
    json!({
        "contract_version": "deployment-gate-authorization.v1",
        "evidence": {
            "ticket_id": binding.ticket_id,
            "release_id": binding.release_id,
            "repository_full_name": binding.repository_full_name,
            "branch": binding.branch,
            "target_sha": binding.target_sha,
            "environment": binding.environment,
            "evidence_packet_hash": binding.evidence_packet_hash,
            "evidence_packet_uri": payload
                .evidence_packet_uri
                .as_deref()
                .unwrap_or(binding.evidence_packet_uri.as_str())
        },
        "first_governed_repo_setup": setup.map(|setup| json!({
            "found": true,
            "run_id": setup.run_id,
            "status": setup.status,
            "repository_full_name": setup.repository_full_name,
            "default_branch": setup.default_branch,
            "policy_preset": setup.policy_preset,
            "gate_readiness": setup.baseline.get("gate_readiness").cloned().unwrap_or(json!(null))
        })).unwrap_or_else(|| json!({ "found": false })),
        "deployment": {
            "deployer": payload.deployer,
            "requested_by": payload.requested_by,
            "deployment_run_id": payload.deployment_run_id,
            "metadata": payload.metadata
        }
    })
}

fn deployment_gate_reason(
    decision: &str,
    evaluation: &EnterpriseReleaseGovernanceEvaluationResponse,
    warnings: &[String],
    break_glass_reason: Option<&str>,
) -> String {
    match decision {
        "break_glass" => break_glass_reason
            .map(|reason| format!("Break-glass deployment authorized: {reason}"))
            .unwrap_or_else(|| "Break-glass deployment authorized.".to_string()),
        "blocked" => evaluation
            .issues
            .first()
            .cloned()
            .unwrap_or_else(|| "Deployment blocked by release governance policy.".to_string()),
        "advisory" => warnings
            .first()
            .cloned()
            .unwrap_or_else(|| "Deployment approved with advisory governance warnings.".to_string()),
        _ => "Deployment approved by current release governance policy.".to_string(),
    }
}

pub async fn authorize_deployment_gate(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<DeploymentGateAuthorizationRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    if let Err(errors) = normalize_and_validate_deployment_gate_authorization(&mut payload) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid deployment gate authorization", "details": errors })),
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
                    "error": "Invalid deployment gate authorization",
                    "details": ["evidence_packet_hash is not a known release evidence packet for this organization."]
                })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                org_id = %org_id,
                "Failed to verify deployment gate evidence packet binding"
            );
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    let evaluation_query = EnterpriseReleaseGovernanceEvaluationQuery {
        org_name: None,
        repository_full_name: payload.repository_full_name.clone(),
        branch: Some(payload.branch.clone()),
        target_sha: Some(payload.target_sha.clone()),
        release_id: payload.release_id.clone(),
        environment: payload.environment.clone(),
        evidence_packet_hash: Some(payload.evidence_packet_hash.clone()),
    };
    let binding_mismatches = release_evidence_binding_query_mismatches(&binding, &evaluation_query);
    if !binding_mismatches.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid deployment gate authorization",
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
                "error": "Invalid deployment gate authorization",
                "details": ["evidence packet binding does not match ticket_id."]
            })),
        )
            .into_response();
    }
    if payload.evidence_packet_uri.is_none() {
        payload.evidence_packet_uri = Some(binding.evidence_packet_uri.clone());
    }
    if payload.ticket_id.is_none() {
        payload.ticket_id = Some(binding.ticket_id.clone());
    }

    let profile = match state.db.get_enterprise_adoption_profile(&org_id).await {
        Ok(profile) => profile,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to load deployment gate governance profile");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    let first_governed_repo_setup = match state.db.get_first_governed_repo_setup(&org_id).await {
        Ok(setup) => setup,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to load first governed repo setup for deployment gate");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    let approval_query = EnterpriseReleaseApprovalQuery {
        org_name: None,
        repository_full_name: Some(payload.repository_full_name.clone()),
        branch: Some(payload.branch.clone()),
        target_sha: Some(payload.target_sha.clone()),
        release_id: Some(payload.release_id.clone()),
        environment: Some(payload.environment.clone()),
        decision: None,
        evidence_packet_hash: Some(payload.evidence_packet_hash.clone()),
        limit: Some(100),
        offset: Some(0),
    };
    let approvals = match state
        .db
        .list_enterprise_release_approvals(&org_id, &approval_query, 100, 0)
        .await
    {
        Ok((items, _)) => items,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to load deployment gate approvals");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    let policy = release_governance_policy_from_profile(
        profile.as_ref().map(|record| &record.profile),
        &payload.environment,
    );
    let evaluation = evaluate_release_governance_policy(
        &evaluation_query,
        policy,
        &approvals,
        chrono::Utc::now().timestamp_millis(),
    );
    let mut warnings = if evaluation.blocking {
        Vec::new()
    } else {
        evaluation.issues.clone()
    };
    warnings.extend(deployment_gate_setup_warnings(
        first_governed_repo_setup.as_ref(),
        &payload,
    ));
    let break_glass_eligible = evaluation.blocking && evaluation.policy.enforcement == "blocking";
    let break_glass_request = payload.break_glass.clone();
    if break_glass_request.is_some() && !break_glass_eligible {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid deployment gate authorization",
                "details": ["break_glass can only be used when the evaluated policy is blocking and break_glass_eligible is true."]
            })),
        )
            .into_response();
    }
    let now_ms = chrono::Utc::now().timestamp_millis();
    let break_glass_approval = if let Some(request) = break_glass_request.as_ref() {
        let approval_query = DeploymentGateBreakGlassApprovalQuery {
            org_name: None,
            approval_id: request.approval_id.clone(),
            repository_full_name: Some(payload.repository_full_name.clone()),
            branch: Some(payload.branch.clone()),
            target_sha: Some(payload.target_sha.clone()),
            release_id: Some(payload.release_id.clone()),
            environment: Some(payload.environment.clone()),
            evidence_packet_hash: Some(payload.evidence_packet_hash.clone()),
            approver: request.authorized_by.clone(),
            active_only: Some(true),
            limit: Some(10),
            offset: Some(0),
        };
        let approvals = match state
            .db
            .list_deployment_gate_break_glass_approvals(&org_id, &approval_query, 10, 0)
            .await
        {
            Ok((items, _)) => items,
            Err(e) => {
                tracing::error!(error = %e, org_id = %org_id, "Failed to load break-glass approvals");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Internal database error" })),
                )
                    .into_response();
            }
        };
        let valid_approval = approvals
            .into_iter()
            .find(|approval| {
                break_glass_approval_matches_authorization(approval, &payload, request, now_ms)
                    .is_ok()
            });
        match valid_approval {
            Some(approval) => Some(approval),
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Invalid deployment gate authorization",
                        "details": ["break_glass requires a valid unexpired matching break-glass approval."]
                    })),
                )
                    .into_response();
            }
        }
    } else {
        None
    };
    let break_glass_used = break_glass_approval.is_some() && break_glass_eligible;
    let decision = if break_glass_used {
        "break_glass"
    } else if evaluation.blocking {
        "blocked"
    } else if evaluation.would_block || !warnings.is_empty() {
        "advisory"
    } else {
        "approved"
    };
    let approved = decision != "blocked";
    let blocked_by = if evaluation.blocking {
        evaluation.issues.clone()
    } else {
        Vec::new()
    };
    let break_glass_reason = break_glass_approval
        .as_ref()
        .map(|approval| approval.reason.clone())
        .or_else(|| break_glass_request.as_ref().map(|request| request.reason.clone()));
    let break_glass_authorized_by = break_glass_approval
        .as_ref()
        .map(|approval| approval.approver.clone())
        .or_else(|| break_glass_request.as_ref().and_then(|request| request.authorized_by.clone()));
    let break_glass_expires_at = break_glass_approval
        .as_ref()
        .map(|approval| approval.expires_at)
        .or_else(|| break_glass_request.as_ref().and_then(|request| request.expires_at));
    let break_glass_approval_id = break_glass_approval
        .as_ref()
        .map(|approval| approval.approval_id.clone());
    let break_glass_approval_hash = break_glass_approval
        .as_ref()
        .map(|approval| approval.approval_hash.clone());
    let reason = deployment_gate_reason(
        decision,
        &evaluation,
        &warnings,
        break_glass_reason.as_deref(),
    );
    let policy_checksum = deployment_gate_policy_checksum(&evaluation);
    let mut details = deployment_gate_details(&binding, first_governed_repo_setup.as_ref(), &payload);
    if let Some(approval) = break_glass_approval.as_ref() {
        if let Some(object) = details.as_object_mut() {
            object.insert(
                "break_glass_approval".to_string(),
                json!({
                    "approval_id": approval.approval_id,
                    "approval_hash": approval.approval_hash,
                    "approver": approval.approver,
                    "approver_role": approval.approver_role,
                    "expires_at": approval.expires_at
                }),
            );
        }
    }
    let request_payload = match serde_json::to_value(&payload) {
        Ok(value) => value,
        Err(e) => {
            tracing::error!(error = %e, "Failed to serialize deployment gate request");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    let requested_by = payload
        .requested_by
        .as_deref()
        .unwrap_or(auth_user.client_id.as_str())
        .to_string();
    let authorization_id = format!("dga_{}", Uuid::new_v4().simple());
    let create_input = CreateDeploymentGateAuthorizationInput {
        authorization_id,
        payload,
        decision: decision.to_string(),
        approved,
        blocking: evaluation.blocking,
        would_block: evaluation.would_block,
        reason: reason.clone(),
        blocked_by: blocked_by.clone(),
        warnings: warnings.clone(),
        policy_checksum: policy_checksum.clone(),
        break_glass_eligible,
        break_glass_used,
        break_glass_reason,
        break_glass_authorized_by,
        break_glass_expires_at,
        break_glass_approval_id,
        break_glass_approval_hash,
        evaluation,
        details,
        request_payload,
        requested_by,
    };

    match state
        .db
        .create_deployment_gate_authorization(&org_id, &create_input)
        .await
    {
        Ok(record) => {
            let audit = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "authorize_deployment_gate".to_string(),
                target_type: Some("deployment_gate_authorization".to_string()),
                target_id: Some(record.authorization_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "authorization_id": &record.authorization_id,
                    "release_id": &record.release_id,
                    "repository_full_name": &record.repository_full_name,
                    "branch": &record.branch,
                    "target_sha": &record.target_sha,
                    "environment": &record.environment,
                    "deployer": &record.deployer,
                    "decision": &record.decision,
                    "approved": record.approved,
                    "blocking": record.blocking,
                    "would_block": record.would_block,
                    "break_glass_eligible": record.break_glass_eligible,
                    "break_glass_used": record.break_glass_used,
                    "break_glass_authorized_by": &record.break_glass_authorized_by,
                    "break_glass_approval_id": &record.break_glass_approval_id,
                    "policy_checksum": &record.policy_checksum
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
                tracing::warn!(error = %e, "Failed to write admin audit log (deployment gate authorization)");
            }

            (StatusCode::CREATED, Json(record)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to persist deployment gate authorization");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn list_deployment_gate_authorizations(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(mut query): Query<DeploymentGateAuthorizationQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    let (limit, offset) = match normalize_deployment_gate_authorization_query(&mut query) {
        Ok(values) => values,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid deployment gate authorization query", "details": errors })),
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
        .list_deployment_gate_authorizations(&org_id, &query, limit, offset)
        .await
    {
        Ok((items, total)) => (
            StatusCode::OK,
            Json(DeploymentGateAuthorizationListResponse {
                items,
                total,
                limit,
                offset,
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to list deployment gate authorizations");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
