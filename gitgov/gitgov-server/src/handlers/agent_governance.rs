// ============================================================================
// AGENT GOVERNANCE POLICY API
// ============================================================================

const AGENT_GOVERNANCE_ACTIONS: &[&str] = &[
    "commit",
    "push",
    "open_pr",
    "merge_pr",
    "change_policy",
    "deploy",
];
const AGENT_GOVERNANCE_DECISIONS: &[&str] = &["allowed", "requires_approval", "blocked"];
const AGENT_GOVERNANCE_POLICY_ID: &str = "agent-governance.v1";
const AGENT_GOVERNANCE_METADATA_MAX_BYTES: usize = 16 * 1024;
const AGENT_GOVERNANCE_REASON_MAX_CHARS: usize = 500;
const REDACTED_VALUE: &str = "[REDACTED]";

fn normalize_and_validate_agent_governance_request(
    payload: &mut AgentGovernanceEvaluationRequest,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    normalize_release_approval_optional_text(&mut payload.org_name);
    normalize_release_approval_optional_text(&mut payload.agent_type);
    normalize_release_approval_optional_text(&mut payload.branch);
    normalize_release_approval_optional_text(&mut payload.target_sha);
    normalize_release_approval_optional_text(&mut payload.environment);
    normalize_release_approval_optional_text(&mut payload.ticket_id);
    normalize_release_approval_optional_text(&mut payload.operation_id);

    payload.agent_id = payload.agent_id.trim().to_string();
    payload.actor = payload.actor.trim().to_string();
    payload.action = payload.action.trim().to_ascii_lowercase();
    payload.repository_full_name = payload.repository_full_name.trim().to_string();

    if let Some(agent_type) = payload.agent_type.as_mut() {
        *agent_type = agent_type.to_ascii_lowercase();
    }
    if let Some(target_sha) = payload.target_sha.as_mut() {
        *target_sha = target_sha.to_ascii_lowercase();
    }
    if let Some(environment) = payload.environment.as_mut() {
        *environment = environment.to_ascii_lowercase();
    }
    if let Some(ticket_id) = payload.ticket_id.as_mut() {
        *ticket_id = ticket_id.to_ascii_uppercase();
    }
    if payload.metadata.is_null() {
        payload.metadata = json!({});
    }

    if payload.agent_id.is_empty()
        || payload.agent_id.len() > 160
        || has_control_chars(&payload.agent_id)
    {
        errors.push("agent_id is required and must be 1-160 characters without control characters.".to_string());
    }
    if let Some(agent_type) = payload.agent_type.as_deref() {
        if agent_type.len() > 80 || has_control_chars(agent_type) {
            errors.push("agent_type is invalid or too long.".to_string());
        }
    }
    if payload.actor.is_empty()
        || payload.actor.len() > 160
        || has_control_chars(&payload.actor)
    {
        errors.push("actor is required and must be 1-160 characters without control characters.".to_string());
    }
    if !AGENT_GOVERNANCE_ACTIONS.contains(&payload.action.as_str()) {
        errors.push("action must be one of commit, push, open_pr, merge_pr, change_policy, or deploy.".to_string());
    }
    if !is_valid_release_approval_repo(&payload.repository_full_name) {
        errors.push("repository_full_name must look like owner/repo.".to_string());
    }
    if let Some(branch) = payload.branch.as_deref() {
        if branch.len() > 200 || has_control_chars(branch) {
            errors.push("branch is invalid or too long.".to_string());
        }
    }
    if let Some(target_sha) = payload.target_sha.as_deref() {
        if !is_valid_release_approval_sha(target_sha) {
            errors.push("target_sha must be a full 40 or 64 character hexadecimal commit SHA.".to_string());
        }
    }
    if let Some(environment) = payload.environment.as_deref() {
        if environment.len() > 80 || has_control_chars(environment) {
            errors.push("environment is invalid or too long.".to_string());
        }
    }
    if let Some(ticket_id) = payload.ticket_id.as_deref() {
        if ticket_id.len() > 32 || !is_valid_release_approval_ticket_id(ticket_id) {
            errors.push("ticket_id must look like KAN-90.".to_string());
        }
    }
    if let Some(operation_id) = payload.operation_id.as_deref() {
        if operation_id.len() > 160 || has_control_chars(operation_id) {
            errors.push("operation_id is invalid or too long.".to_string());
        }
    }
    if !payload.metadata.is_object() {
        errors.push("metadata must be a JSON object.".to_string());
    } else {
        let metadata_len = serde_json::to_vec(&payload.metadata)
            .map(|bytes| bytes.len())
            .unwrap_or(0);
        if metadata_len > AGENT_GOVERNANCE_METADATA_MAX_BYTES {
            errors.push("metadata is too large.".to_string());
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn is_protected_agent_branch(branch: Option<&str>) -> bool {
    branch
        .map(|branch| {
            matches!(
                branch.to_ascii_lowercase().as_str(),
                "main" | "master" | "production" | "prod" | "release"
            )
        })
        .unwrap_or(false)
}

fn agent_governance_scope_error_message(error: OrgScopeError) -> &'static str {
    match error {
        OrgScopeError::BadRequest => "org_name is required for global admin keys",
        OrgScopeError::NotFound => "Organization not found",
        OrgScopeError::Forbidden => "Requested org is outside API key scope",
        OrgScopeError::Internal => "Internal database error",
    }
}

fn normalize_agent_governance_reason(reason: Option<String>) -> Option<String> {
    reason
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(|value| {
            value
                .chars()
                .take(AGENT_GOVERNANCE_REASON_MAX_CHARS)
                .collect()
        })
}

fn is_secret_like_metadata_key(key: &str) -> bool {
    let lowered = key.to_ascii_lowercase();
    lowered.contains("token")
        || lowered.contains("secret")
        || lowered.contains("password")
        || lowered.contains("credential")
        || lowered.contains("authorization")
        || lowered.contains("api_key")
        || lowered.contains("apikey")
        || lowered == "key"
}

fn minimize_agent_governance_json(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut sanitized = serde_json::Map::new();
            for (key, value) in map {
                if is_secret_like_metadata_key(key) {
                    sanitized.insert(key.clone(), json!(REDACTED_VALUE));
                } else {
                    sanitized.insert(key.clone(), minimize_agent_governance_json(value));
                }
            }
            serde_json::Value::Object(sanitized)
        }
        serde_json::Value::Array(values) => serde_json::Value::Array(
            values
                .iter()
                .take(25)
                .map(minimize_agent_governance_json)
                .collect(),
        ),
        serde_json::Value::String(value) => {
            let lowered = value.to_ascii_lowercase();
            if lowered.contains("bearer ") || lowered.contains("ghp_") || lowered.contains("gho_")
            {
                json!(REDACTED_VALUE)
            } else {
                json!(value.chars().take(512).collect::<String>())
            }
        }
        _ => value.clone(),
    }
}

fn minimized_agent_governance_request_payload(
    payload: &AgentGovernanceEvaluationRequest,
) -> serde_json::Value {
    json!({
        "org_name": payload.org_name,
        "agent_id": payload.agent_id,
        "agent_type": payload.agent_type,
        "actor": payload.actor,
        "action": payload.action,
        "repository_full_name": payload.repository_full_name,
        "branch": payload.branch,
        "target_sha": payload.target_sha,
        "environment": payload.environment,
        "ticket_id": payload.ticket_id,
        "operation_id": payload.operation_id,
        "metadata": minimize_agent_governance_json(&payload.metadata),
        "payload_mode": "minimized"
    })
}

async fn write_agent_governance_audit(
    state: &Arc<AppState>,
    actor_client_id: &str,
    action: &str,
    target_type: &str,
    target_id: Option<String>,
    metadata: serde_json::Value,
) {
    let audit = AdminAuditLogEntry {
        id: Uuid::new_v4().to_string(),
        actor_client_id: actor_client_id.to_string(),
        action: action.to_string(),
        target_type: Some(target_type.to_string()),
        target_id,
        metadata,
        created_at: chrono::Utc::now().timestamp_millis(),
    };
    if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
        tracing::warn!(error = %e, action, "Failed to write admin audit log (agent governance)");
    }
}

fn agent_policy_checksum() -> String {
    let canonical = json!({
        "policy_id": AGENT_GOVERNANCE_POLICY_ID,
        "actions": AGENT_GOVERNANCE_ACTIONS,
        "decisions": AGENT_GOVERNANCE_DECISIONS,
        "rules": {
            "commit": "allowed with ticket, otherwise requires approval",
            "push": "blocked without ticket or branch; protected branches require approval",
            "open_pr": "allowed with ticket, otherwise requires approval",
            "merge_pr": "requires approval with ticket and branch, blocked without them",
            "change_policy": "requires approval with ticket and operation id, blocked without them",
            "deploy": "requires approval with ticket, branch, target sha, environment, and operation id; blocked without them"
        }
    });
    let digest = Sha256::digest(
        serde_json::to_vec(&canonical).expect("agent governance policy should serialize"),
    );
    format!("{digest:x}")
}

fn decide_agent_governance(
    payload: &AgentGovernanceEvaluationRequest,
) -> (String, bool, bool, String, Vec<String>, Vec<String>, serde_json::Value) {
    let mut reasons = Vec::new();
    let mut required_evidence = Vec::new();
    let has_ticket = payload.ticket_id.is_some();
    let has_branch = payload.branch.is_some();
    let has_target_sha = payload.target_sha.is_some();
    let has_environment = payload.environment.is_some();
    let has_operation_id = payload.operation_id.is_some();
    let protected_branch = is_protected_agent_branch(payload.branch.as_deref());

    let decision = match payload.action.as_str() {
        "commit" => {
            if has_ticket {
                reasons.push("Commit is allowed because the agent supplied ticket traceability.".to_string());
                "allowed"
            } else {
                reasons.push("Commit lacks ticket traceability.".to_string());
                required_evidence.push("ticket_id".to_string());
                "requires_approval"
            }
        }
        "push" => {
            if !has_branch {
                reasons.push("Push requires a target branch.".to_string());
                required_evidence.push("branch".to_string());
                "blocked"
            } else if !has_ticket {
                reasons.push("Push lacks ticket traceability.".to_string());
                required_evidence.push("ticket_id".to_string());
                "blocked"
            } else if protected_branch {
                reasons.push("Push targets a protected branch and requires human approval.".to_string());
                required_evidence.push("human_approval".to_string());
                "requires_approval"
            } else {
                reasons.push("Push is allowed for a non-protected branch with ticket traceability.".to_string());
                "allowed"
            }
        }
        "open_pr" => {
            if has_ticket {
                reasons.push("Pull request creation is allowed with ticket traceability.".to_string());
                "allowed"
            } else {
                reasons.push("Pull request creation lacks ticket traceability.".to_string());
                required_evidence.push("ticket_id".to_string());
                "requires_approval"
            }
        }
        "merge_pr" => {
            if !has_ticket || !has_branch {
                reasons.push("Merge requires ticket traceability and target branch context.".to_string());
                if !has_ticket {
                    required_evidence.push("ticket_id".to_string());
                }
                if !has_branch {
                    required_evidence.push("branch".to_string());
                }
                "blocked"
            } else {
                reasons.push("Merge is high-impact and requires human approval.".to_string());
                required_evidence.push("human_approval".to_string());
                required_evidence.push("pull_request_review".to_string());
                "requires_approval"
            }
        }
        "change_policy" => {
            if !has_ticket || !has_operation_id {
                reasons.push("Policy changes require ticket traceability and an operation id.".to_string());
                if !has_ticket {
                    required_evidence.push("ticket_id".to_string());
                }
                if !has_operation_id {
                    required_evidence.push("operation_id".to_string());
                }
                "blocked"
            } else {
                reasons.push("Policy changes require reviewed policy-change approval.".to_string());
                required_evidence.push("policy_change_request".to_string());
                required_evidence.push("human_approval".to_string());
                "requires_approval"
            }
        }
        "deploy" => {
            if !has_ticket || !has_branch || !has_target_sha || !has_environment || !has_operation_id {
                reasons.push("Deploy requires ticket, branch, target SHA, environment, and operation id context.".to_string());
                if !has_ticket {
                    required_evidence.push("ticket_id".to_string());
                }
                if !has_branch {
                    required_evidence.push("branch".to_string());
                }
                if !has_target_sha {
                    required_evidence.push("target_sha".to_string());
                }
                if !has_environment {
                    required_evidence.push("environment".to_string());
                }
                if !has_operation_id {
                    required_evidence.push("operation_id".to_string());
                }
                "blocked"
            } else {
                reasons.push("Deploy is high-impact and must go through Deployment Gates.".to_string());
                required_evidence.push("release_evidence_packet".to_string());
                required_evidence.push("deployment_gate_authorization".to_string());
                required_evidence.push("human_approval_if_policy_requires".to_string());
                "requires_approval"
            }
        }
        _ => {
            reasons.push("Unknown action.".to_string());
            "blocked"
        }
    };

    required_evidence.sort();
    required_evidence.dedup();

    let allowed = decision == "allowed";
    let requires_approval = decision == "requires_approval";
    let reason = reasons
        .first()
        .cloned()
        .unwrap_or_else(|| "Agent governance evaluated the request.".to_string());
    let policy_checksum = agent_policy_checksum();
    let governance_decision = build_agent_governance_decision(AgentGovernanceDecisionInput {
        payload,
        decision,
        allowed,
        requires_approval,
        reasons: &reasons,
        required_evidence: &required_evidence,
        policy_checksum: &policy_checksum,
        protected_branch,
    });
    let evaluation = json!({
        "contract_version": "agent-governance-evaluation.v1",
        "policy": {
            "policy_id": AGENT_GOVERNANCE_POLICY_ID,
            "policy_checksum": policy_checksum,
            "deterministic": true,
            "llm_decision": false
        },
        "request": {
            "agent_id": payload.agent_id,
            "agent_type": payload.agent_type,
            "actor": payload.actor,
            "action": payload.action,
            "repository_full_name": payload.repository_full_name,
            "branch": payload.branch,
            "target_sha": payload.target_sha,
            "environment": payload.environment,
            "ticket_id": payload.ticket_id,
            "operation_id": payload.operation_id
        },
        "decision": decision,
        "allowed": allowed,
        "requires_approval": requires_approval,
        "protected_branch": protected_branch,
        "reasons": reasons,
        "required_evidence": required_evidence,
        "shared_governance_decision": governance_decision
    });

    (
        decision.to_string(),
        allowed,
        requires_approval,
        reason,
        reasons,
        required_evidence,
        evaluation,
    )
}

pub async fn evaluate_agent_governance(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<AgentGovernanceEvaluationRequest>,
) -> impl IntoResponse {
    if let Err(errors) = normalize_and_validate_agent_governance_request(&mut payload) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid agent governance evaluation", "details": errors })),
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
                Json(json!({ "error": agent_governance_scope_error_message(err) })),
            )
                .into_response();
        }
    };

    let settings = match state.db.get_agent_governance_settings(&org_id).await {
        Ok(settings) => settings,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to load agent governance settings");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    if !settings.enabled {
        write_agent_governance_audit(
            &state,
            &auth_user.client_id,
            "agent_governance.evaluation_denied",
            "agent_governance_settings",
            Some(org_id.clone()),
            json!({
                "org_id": org_id,
                "enabled": false,
                "mode": settings.mode,
                "payload_mode": settings.payload_mode,
                "agent_id": payload.agent_id,
                "agent_type": payload.agent_type,
                "actor": payload.actor,
                "action": payload.action,
                "repository_full_name": payload.repository_full_name,
                "branch": payload.branch,
                "ticket_id": payload.ticket_id,
                "reason": "agent_governance_disabled"
            }),
        )
        .await;
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "Agent Governance is disabled for this organization",
                "code": "agent_governance_disabled",
                "enabled": false,
                "mode": "manual_only",
                "manual_governance_available": true,
                "next_step": "An Admin must explicitly enable Agent Governance before agent evaluations are accepted."
            })),
        )
            .into_response();
    }

    let agent_type = payload
        .agent_type
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let request_payload = minimized_agent_governance_request_payload(&payload);
    let (
        decision,
        allowed,
        requires_approval,
        reason,
        reasons,
        required_evidence,
        evaluation,
    ) = decide_agent_governance(&payload);
    let policy_checksum = agent_policy_checksum();

    let create_input = CreateAgentGovernanceEvaluationInput {
        evaluation_id: format!("agv_{}", Uuid::new_v4().simple()),
        payload,
        agent_type,
        decision,
        allowed,
        requires_approval,
        reason,
        reasons,
        required_evidence,
        policy_id: AGENT_GOVERNANCE_POLICY_ID.to_string(),
        policy_checksum,
        evaluation,
        request_payload,
    };

    match state
        .db
        .create_agent_governance_evaluation(&org_id, &create_input)
        .await
    {
        Ok(record) => {
            let audit = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "agent_governance.evaluation_requested".to_string(),
                target_type: Some("agent_governance_evaluation".to_string()),
                target_id: Some(record.evaluation_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "evaluation_id": &record.evaluation_id,
                    "agent_id": &record.agent_id,
                    "agent_type": &record.agent_type,
                    "actor": &record.actor,
                    "action": &record.action,
                    "repository_full_name": &record.repository_full_name,
                    "branch": &record.branch,
                    "target_sha": &record.target_sha,
                    "environment": &record.environment,
                    "ticket_id": &record.ticket_id,
                    "decision": &record.decision,
                    "allowed": record.allowed,
                    "requires_approval": record.requires_approval,
                    "policy_checksum": &record.policy_checksum
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
                tracing::warn!(error = %e, "Failed to write admin audit log (agent governance evaluation)");
            }

            (StatusCode::CREATED, Json(record)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to persist agent governance evaluation");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn get_agent_governance_settings(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<AgentGovernanceSettingsQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

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
                Json(json!({ "error": agent_governance_scope_error_message(err) })),
            )
                .into_response();
        }
    };

    match state.db.get_agent_governance_settings(&org_id).await {
        Ok(settings) => (StatusCode::OK, Json(settings)).into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to load agent governance settings");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn upsert_agent_governance_settings(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpsertAgentGovernanceSettingsRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
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
                Json(json!({ "error": agent_governance_scope_error_message(err) })),
            )
                .into_response();
        }
    };

    let reason = normalize_agent_governance_reason(payload.reason);
    match state
        .db
        .upsert_agent_governance_settings(
            &org_id,
            payload.enabled,
            reason.as_deref(),
            &auth_user.client_id,
        )
        .await
    {
        Ok(settings) => {
            write_agent_governance_audit(
                &state,
                &auth_user.client_id,
                if settings.enabled {
                    "agent_governance.enabled"
                } else {
                    "agent_governance.disabled"
                },
                "agent_governance_settings",
                Some(org_id.clone()),
                json!({
                    "org_id": org_id,
                    "enabled": settings.enabled,
                    "mode": settings.mode,
                    "payload_mode": settings.payload_mode,
                    "reason": settings.reason
                }),
            )
            .await;
            (StatusCode::OK, Json(settings)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to upsert agent governance settings");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn list_agent_governance_evaluations(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<AgentGovernanceEvaluationQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

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
                Json(json!({ "error": agent_governance_scope_error_message(err) })),
            )
                .into_response();
        }
    };

    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);
    let input = ListAgentGovernanceEvaluationsInput {
        org_id: &org_id,
        evaluation_id: query.evaluation_id.as_deref(),
        repository_full_name: query.repository_full_name.as_deref(),
        action: query.action.as_deref(),
        decision: query.decision.as_deref(),
        agent_id: query.agent_id.as_deref(),
        limit,
        offset,
    };

    match state.db.list_agent_governance_evaluations(&input).await {
        Ok((items, total)) => (
            StatusCode::OK,
            Json(AgentGovernanceEvaluationListResponse {
                items,
                total,
                limit,
                offset,
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to list agent governance evaluations");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
