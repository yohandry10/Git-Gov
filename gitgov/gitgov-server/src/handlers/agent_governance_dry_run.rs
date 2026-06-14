// ============================================================================
// AGENT GOVERNANCE DRY-RUN PREVIEW
// ============================================================================

pub async fn dry_run_agent_governance(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<AgentGovernanceEvaluationRequest>,
) -> impl IntoResponse {
    if let Err(errors) = normalize_and_validate_agent_governance_request(&mut payload) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid agent governance dry-run", "details": errors })),
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
            tracing::error!(error = %e, org_id = %org_id, "Failed to load agent governance settings for dry-run");
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
            "agent_governance.dry_run_denied",
            "agent_governance_settings",
            Some(org_id.clone()),
            json!({
                "org_id": org_id,
                "enabled": false,
                "mode": settings.mode,
                "payload_mode": settings.payload_mode,
                "dry_run": true,
                "would_persist_evaluation": false,
                "agent_id": payload.agent_id,
                "agent_type": payload.agent_type,
                "actor": payload.actor,
                "action": payload.action,
                "repository_full_name": payload.repository_full_name,
                "branch": payload.branch,
                "ticket_id": payload.ticket_id,
                "principal_type": auth_user.principal_type,
                "agent_key_id": auth_user.agent_key_id,
                "agent_display_name": auth_user.agent_display_name,
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
                "dry_run": true,
                "would_persist_evaluation": false,
                "manual_governance_available": true,
                "next_step": "An Admin must explicitly enable Agent Governance before dry-run previews are accepted."
            })),
        )
            .into_response();
    }

    if let Some(response) = enforce_agent_governance_agent_permissions(
        &state,
        &auth_user,
        &org_id,
        payload.action.as_str(),
    )
    .await
    {
        return response;
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
        mut evaluation,
    ) = decide_agent_governance(&payload);
    if let Some(object) = evaluation.as_object_mut() {
        object.insert(
            "principal".to_string(),
            json!({
                "principal_type": auth_user.principal_type,
                "agent_key_id": auth_user.agent_key_id,
                "agent_display_name": auth_user.agent_display_name,
                "scope": if auth_user.principal_type == "agent" {
                    Some(AGENT_GOVERNANCE_EVALUATE_SCOPE)
                } else {
                    None
                }
            }),
        );
        object.insert(
            "dry_run".to_string(),
            json!({
                "dry_run": true,
                "would_persist_evaluation": false,
                "would_authorize_execution": false
            }),
        );
    }
    let policy_checksum = agent_policy_checksum();
    let previewed_at = chrono::Utc::now().timestamp_millis();

    write_agent_governance_audit(
        &state,
        &auth_user.client_id,
        "agent_governance.dry_run_requested",
        "agent_governance_dry_run",
        None,
        json!({
            "org_id": org_id,
            "dry_run": true,
            "would_persist_evaluation": false,
            "would_authorize_execution": false,
            "agent_id": payload.agent_id,
            "agent_type": agent_type,
            "actor": payload.actor,
            "action": payload.action,
            "repository_full_name": payload.repository_full_name,
            "branch": payload.branch,
            "target_sha": payload.target_sha,
            "environment": payload.environment,
            "ticket_id": payload.ticket_id,
            "decision": decision,
            "allowed": allowed,
            "requires_approval": requires_approval,
            "policy_checksum": policy_checksum,
            "principal_type": auth_user.principal_type,
            "agent_key_id": auth_user.agent_key_id,
            "agent_display_name": auth_user.agent_display_name,
            "scope": if auth_user.principal_type == "agent" {
                Some(AGENT_GOVERNANCE_EVALUATE_SCOPE)
            } else {
                None
            }
        }),
    )
    .await;

    let response = AgentGovernanceDryRunResponse {
        dry_run: true,
        would_persist_evaluation: false,
        would_authorize_execution: false,
        org_id,
        agent_id: payload.agent_id,
        agent_type,
        actor: payload.actor,
        action: payload.action,
        repository_full_name: payload.repository_full_name,
        branch: payload.branch,
        target_sha: payload.target_sha,
        environment: payload.environment,
        ticket_id: payload.ticket_id,
        operation_id: payload.operation_id,
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
        principal_type: Some(auth_user.principal_type),
        agent_key_id: auth_user.agent_key_id,
        agent_display_name: auth_user.agent_display_name,
        previewed_at,
    };

    (StatusCode::OK, Json(response)).into_response()
}
