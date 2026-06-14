// ============================================================================
// AGENT GOVERNANCE AGENT KEY ADMINISTRATION
// ============================================================================

fn default_agent_key_allowed_actions() -> Vec<String> {
    ["commit", "push", "open_pr", "merge_pr", "deploy"]
        .iter()
        .map(|value| value.to_string())
        .collect()
}

fn normalize_agent_key_text(
    value: Option<String>,
    field_name: &str,
    max_chars: usize,
    required: bool,
    errors: &mut Vec<String>,
) -> Option<String> {
    let normalized = value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    if required && normalized.is_none() {
        errors.push(format!("{field_name} is required."));
        return None;
    }

    if let Some(value) = normalized.as_deref() {
        if value.len() > max_chars || has_control_chars(value) {
            errors.push(format!("{field_name} is invalid or too long."));
        }
    }

    normalized
}

fn normalize_agent_key_allowed_actions(actions: Vec<String>) -> Result<Vec<String>, Vec<String>> {
    let mut errors = Vec::new();
    let mut normalized = if actions.is_empty() {
        default_agent_key_allowed_actions()
    } else {
        actions
            .into_iter()
            .map(|action| action.trim().to_ascii_lowercase())
            .filter(|action| !action.is_empty())
            .collect::<Vec<_>>()
    };

    normalized.sort();
    normalized.dedup();

    if normalized.is_empty() {
        errors.push("allowed_actions must include at least one action.".to_string());
    }
    for action in &normalized {
        if !AGENT_GOVERNANCE_ACTIONS.contains(&action.as_str()) {
            errors.push(format!("allowed action {action} is not supported."));
        }
    }

    if errors.is_empty() {
        Ok(normalized)
    } else {
        Err(errors)
    }
}

fn parse_agent_key_expires_at(
    expires_at: Option<i64>,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, String> {
    let Some(expires_at) = expires_at else {
        return Ok(None);
    };
    let Some(datetime) = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(expires_at) else {
        return Err("expires_at must be a valid Unix timestamp in milliseconds.".to_string());
    };
    if datetime <= chrono::Utc::now() {
        return Err("expires_at must be in the future.".to_string());
    }
    Ok(Some(datetime))
}

pub async fn list_agent_governance_agent_keys(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<AgentGovernanceAgentKeyQuery>,
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

    match state.db.list_agent_governance_agent_keys(&org_id).await {
        Ok(items) => {
            let total = items.len() as i64;
            (
                StatusCode::OK,
                Json(AgentGovernanceAgentKeyListResponse { items, total }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to list agent governance agent keys");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn create_agent_governance_agent_key(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateAgentGovernanceAgentKeyRequest>,
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

    let mut errors = Vec::new();
    let display_name = normalize_agent_key_text(
        Some(payload.display_name),
        "display_name",
        120,
        true,
        &mut errors,
    )
    .unwrap_or_default();
    let description =
        normalize_agent_key_text(payload.description, "description", 500, false, &mut errors);
    let environment =
        normalize_agent_key_text(payload.environment, "environment", 80, false, &mut errors)
            .map(|value| value.to_ascii_lowercase());
    let allowed_actions = match normalize_agent_key_allowed_actions(payload.allowed_actions) {
        Ok(actions) => actions,
        Err(mut action_errors) => {
            errors.append(&mut action_errors);
            Vec::new()
        }
    };
    let expires_at = match parse_agent_key_expires_at(payload.expires_at) {
        Ok(value) => value,
        Err(error) => {
            errors.push(error);
            None
        }
    };

    if !errors.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid agent key request", "details": errors })),
        )
            .into_response();
    }

    let key_id = format!("agk_{}", Uuid::new_v4().simple());
    let token_secret = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let token = format!("ggag_{token_secret}");
    let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
    let token_last4 = token
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    let scopes = vec![AGENT_GOVERNANCE_EVALUATE_SCOPE.to_string()];

    match state
        .db
        .create_agent_governance_agent_key(&CreateAgentGovernanceAgentKeyInput {
            key_id: &key_id,
            org_id: &org_id,
            token_hash: &token_hash,
            token_prefix: "ggag_",
            token_last4: &token_last4,
            display_name: &display_name,
            description: description.as_deref(),
            environment: environment.as_deref(),
            scopes: &scopes,
            allowed_actions: &allowed_actions,
            expires_at,
            created_by: &auth_user.client_id,
        })
        .await
    {
        Ok(record) => {
            write_agent_governance_audit(
                &state,
                &auth_user.client_id,
                "agent_key.created",
                "agent_governance_agent_key",
                Some(record.key_id.clone()),
                json!({
                    "org_id": org_id,
                    "agent_key_id": record.key_id,
                    "display_name": record.display_name,
                    "scopes": record.scopes,
                    "allowed_actions": record.allowed_actions,
                    "expires_at": record.expires_at
                }),
            )
            .await;
            (
                StatusCode::CREATED,
                Json(CreateAgentGovernanceAgentKeyResponse { record, token }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to create agent governance agent key");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn revoke_agent_governance_agent_key(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<AgentGovernanceAgentKeyQuery>,
    Path(key_id): Path<String>,
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

    match state
        .db
        .revoke_agent_governance_agent_key(&org_id, &key_id, &auth_user.client_id)
        .await
    {
        Ok(Some(record)) => {
            write_agent_governance_audit(
                &state,
                &auth_user.client_id,
                "agent_key.revoked",
                "agent_governance_agent_key",
                Some(record.key_id.clone()),
                json!({
                    "org_id": org_id,
                    "agent_key_id": record.key_id,
                    "display_name": record.display_name,
                    "revoked_at": record.revoked_at
                }),
            )
            .await;
            (StatusCode::OK, Json(record)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Agent key not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, key_id = %key_id, "Failed to revoke agent governance agent key");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
