// ============================================================================
// AGENT GOVERNANCE READ-ONLY CONTEXT
// ============================================================================

fn normalize_agent_governance_context_query(
    query: &mut AgentGovernanceReadContextQuery,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    normalize_release_approval_optional_text(&mut query.org_name);
    normalize_release_approval_optional_text(&mut query.branch);
    normalize_release_approval_optional_text(&mut query.target_sha);
    normalize_release_approval_optional_text(&mut query.environment);
    query.repository_full_name = query.repository_full_name.trim().to_string();

    if !is_valid_release_approval_repo(&query.repository_full_name) {
        errors.push("repository_full_name must look like owner/repo.".to_string());
    }
    if let Some(branch) = query.branch.as_deref() {
        if branch.len() > 200 || has_control_chars(branch) {
            errors.push("branch is invalid or too long.".to_string());
        }
    }
    if let Some(target_sha) = query.target_sha.as_mut() {
        *target_sha = target_sha.to_ascii_lowercase();
        if !is_valid_release_approval_sha(target_sha) {
            errors.push("target_sha must be a full 40 or 64 character hexadecimal commit SHA.".to_string());
        }
    }
    if let Some(environment) = query.environment.as_mut() {
        *environment = environment.to_ascii_lowercase();
        if environment.len() > 80 || has_control_chars(environment) {
            errors.push("environment is invalid or too long.".to_string());
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn agent_read_or_admin_denial(auth_user: &AuthUser) -> Option<axum::response::Response> {
    if auth_user.principal_type == "agent" {
        if auth_user
            .scopes
            .iter()
            .any(|scope| scope == AGENT_GOVERNANCE_READ_SCOPE)
        {
            return None;
        }
        return Some((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "Agent key scope does not allow this request",
                "code": "invalid_scope"
            })),
        )
            .into_response());
    }

    require_admin(auth_user).err().map(|error| error.into_response())
}

pub async fn get_agent_governance_context(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(mut query): Query<AgentGovernanceReadContextQuery>,
) -> impl IntoResponse {
    if let Some(resp) = agent_read_or_admin_denial(&auth_user) {
        return resp;
    }

    if let Err(errors) = normalize_agent_governance_context_query(&mut query) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid agent governance context query", "details": errors })),
        )
            .into_response();
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

    if auth_user.principal_type == "agent" {
        match state.db.get_agent_governance_settings(&org_id).await {
            Ok(settings) if !settings.enabled => {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "Agent Governance is disabled for this tenant",
                        "code": "agent_governance_disabled",
                        "enabled": false,
                        "mode": settings.mode,
                        "manual_governance_available": true,
                        "read_only": true,
                        "will_authorize_execution": false
                    })),
                )
                    .into_response();
            }
            Ok(_) => {}
            Err(e) => {
                tracing::error!(error = %e, org_id = %org_id, "Failed to load agent governance settings for read-only context");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Internal database error" })),
                )
                    .into_response();
            }
        }
    }

    match state
        .db
        .get_agent_governance_read_context(&AgentGovernanceReadContextInput {
            org_id: &org_id,
            repository_full_name: &query.repository_full_name,
            branch: query.branch.as_deref(),
            target_sha: query.target_sha.as_deref(),
            environment: query.environment.as_deref(),
        })
        .await
    {
        Ok(context) => {
            let response = AgentGovernanceReadContextResponse {
                context_id: format!("agctx_{}", Uuid::new_v4().simple()),
                org_id,
                repository_full_name: query.repository_full_name,
                branch: query.branch,
                target_sha: query.target_sha,
                environment: query.environment,
                read_only: true,
                will_authorize_execution: false,
                mcp_surface: false,
                generated_at: chrono::Utc::now().timestamp_millis(),
                principal: json!({
                    "principal_type": auth_user.principal_type,
                    "client_id": auth_user.client_id,
                    "agent_key_id": auth_user.agent_key_id,
                    "agent_display_name": auth_user.agent_display_name,
                    "scopes": auth_user.scopes
                }),
                branch_status: context["branch_status"].clone(),
                policy_compliance: context["policy_compliance"].clone(),
                pipeline_state: context["pipeline_state"].clone(),
                risk_score: context["risk_score"].clone(),
                recent_activity: context["recent_activity"].clone(),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to load agent governance read-only context");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
