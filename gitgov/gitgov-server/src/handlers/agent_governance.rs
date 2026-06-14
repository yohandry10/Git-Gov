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
    let evaluation = json!({
        "contract_version": "agent-governance-evaluation.v1",
        "policy": {
            "policy_id": AGENT_GOVERNANCE_POLICY_ID,
            "policy_checksum": agent_policy_checksum(),
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
        "required_evidence": required_evidence
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
                Json(json!({ "error": release_approval_scope_error_message(err) })),
            )
                .into_response();
        }
    };

    let agent_type = payload
        .agent_type
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let request_payload = match serde_json::to_value(&payload) {
        Ok(value) => value,
        Err(e) => {
            tracing::error!(error = %e, "Failed to serialize agent governance request");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
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
                action: "evaluate_agent_governance".to_string(),
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

#[cfg(test)]
mod agent_governance_tests {
    use super::*;

    fn base_payload(action: &str) -> AgentGovernanceEvaluationRequest {
        AgentGovernanceEvaluationRequest {
            org_name: Some("example".to_string()),
            agent_id: "codex-agent-1".to_string(),
            agent_type: Some("codex".to_string()),
            actor: "dev@example.com".to_string(),
            action: action.to_string(),
            repository_full_name: "owner/repo".to_string(),
            branch: Some("feature/KAN-90-agent-api".to_string()),
            target_sha: Some("a".repeat(40)),
            environment: Some("production".to_string()),
            ticket_id: Some("KAN-90".to_string()),
            operation_id: Some("op-123".to_string()),
            metadata: json!({}),
        }
    }

    #[test]
    fn agent_governance_allows_ticketed_commit() {
        let payload = base_payload("commit");
        let (decision, allowed, requires_approval, _, _, required, evaluation) =
            decide_agent_governance(&payload);
        assert_eq!(decision, "allowed");
        assert!(allowed);
        assert!(!requires_approval);
        assert!(required.is_empty());
        assert_eq!(evaluation["policy"]["llm_decision"], false);
    }

    #[test]
    fn agent_governance_blocks_deploy_without_context() {
        let mut payload = base_payload("deploy");
        payload.target_sha = None;
        payload.operation_id = None;
        let (decision, allowed, requires_approval, _, reasons, required, _) =
            decide_agent_governance(&payload);
        assert_eq!(decision, "blocked");
        assert!(!allowed);
        assert!(!requires_approval);
        assert!(reasons[0].contains("Deploy requires"));
        assert!(required.contains(&"target_sha".to_string()));
        assert!(required.contains(&"operation_id".to_string()));
    }

    #[test]
    fn agent_governance_requires_approval_for_protected_branch_push() {
        let mut payload = base_payload("push");
        payload.branch = Some("main".to_string());
        let (decision, allowed, requires_approval, _, _, required, evaluation) =
            decide_agent_governance(&payload);
        assert_eq!(decision, "requires_approval");
        assert!(!allowed);
        assert!(requires_approval);
        assert!(required.contains(&"human_approval".to_string()));
        assert_eq!(evaluation["protected_branch"], true);
    }
}
