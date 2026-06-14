// ============================================================================
// AGENT GOVERNANCE AGENT KEY ADMINISTRATION
// ============================================================================

fn default_agent_key_allowed_actions() -> Vec<String> {
    ["commit", "push", "open_pr", "merge_pr", "deploy"]
        .iter()
        .map(|value| value.to_string())
        .collect()
}

fn default_agent_key_scopes() -> Vec<String> {
    vec![AGENT_GOVERNANCE_EVALUATE_SCOPE.to_string()]
}

fn normalize_agent_key_scopes(scopes: Vec<String>) -> Result<Vec<String>, Vec<String>> {
    let mut errors = Vec::new();
    let mut normalized = if scopes.is_empty() {
        default_agent_key_scopes()
    } else {
        scopes
            .into_iter()
            .map(|scope| scope.trim().to_ascii_lowercase())
            .filter(|scope| !scope.is_empty())
            .collect::<Vec<_>>()
    };

    normalized.sort();
    normalized.dedup();

    if normalized.is_empty() {
        errors.push("scopes must include at least one scope.".to_string());
    }
    for scope in &normalized {
        if !matches!(
            scope.as_str(),
            AGENT_GOVERNANCE_EVALUATE_SCOPE | AGENT_GOVERNANCE_READ_SCOPE
        ) {
            errors.push(format!("scope {scope} is not supported."));
        }
    }

    if errors.is_empty() {
        Ok(normalized)
    } else {
        Err(errors)
    }
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
    no_expiry: bool,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, String> {
    if no_expiry && expires_at.is_some() {
        return Err("expires_at cannot be combined with no_expiry=true.".to_string());
    }
    if no_expiry {
        return Ok(None);
    }
    let Some(expires_at) = expires_at else {
        return Ok(Some(chrono::Utc::now() + chrono::Duration::days(90)));
    };
    let Some(datetime) = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(expires_at) else {
        return Err("expires_at must be a valid Unix timestamp in milliseconds.".to_string());
    };
    if datetime <= chrono::Utc::now() {
        return Err("expires_at must be in the future.".to_string());
    }
    if datetime > chrono::Utc::now() + chrono::Duration::days(366) {
        return Err("expires_at cannot be more than 366 days in the future.".to_string());
    }
    Ok(Some(datetime))
}

fn parse_agent_key_rotation_grace_period(
    grace_period_hours: Option<i64>,
) -> Result<i64, String> {
    let hours = grace_period_hours.unwrap_or(24);
    if !(0..=72).contains(&hours) {
        return Err("grace_period_hours must be between 0 and 72.".to_string());
    }
    Ok(hours)
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
    let scopes = match normalize_agent_key_scopes(payload.scopes) {
        Ok(scopes) => scopes,
        Err(mut scope_errors) => {
            errors.append(&mut scope_errors);
            Vec::new()
        }
    };
    let expires_at = match parse_agent_key_expires_at(payload.expires_at, payload.no_expiry) {
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
                    "expires_at": record.expires_at,
                    "status": record.status,
                    "no_expiry": record.no_expiry
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

pub async fn rotate_agent_governance_agent_key(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(key_id): Path<String>,
    Json(payload): Json<RotateAgentGovernanceAgentKeyRequest>,
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
    let rotation_reason =
        normalize_agent_key_text(payload.reason, "reason", 200, false, &mut errors);
    let grace_period_hours = match parse_agent_key_rotation_grace_period(payload.grace_period_hours)
    {
        Ok(value) => value,
        Err(error) => {
            errors.push(error);
            24
        }
    };

    if !errors.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid agent key rotation request", "details": errors })),
        )
            .into_response();
    }

    let replacement_key_id = format!("agk_{}", Uuid::new_v4().simple());
    let token_secret = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let token = format!("ggag_{token_secret}");
    let replacement_token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
    let replacement_token_last4 = token
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    let grace_expires_at = chrono::Utc::now() + chrono::Duration::hours(grace_period_hours);
    let replacement_expires_at = chrono::Utc::now() + chrono::Duration::days(90);

    match state
        .db
        .rotate_agent_governance_agent_key(&crate::db::RotateAgentGovernanceAgentKeyInput {
            org_id: &org_id,
            key_id: &key_id,
            replacement_key_id: &replacement_key_id,
            replacement_token_hash: &replacement_token_hash,
            replacement_token_prefix: "ggag_",
            replacement_token_last4: &replacement_token_last4,
            rotated_by: &auth_user.client_id,
            rotation_reason: rotation_reason.as_deref(),
            grace_expires_at,
            replacement_expires_at,
        })
        .await
    {
        Ok(crate::db::RotateAgentGovernanceAgentKeyOutcome::Rotated(records)) => {
            write_agent_governance_audit(
                &state,
                &auth_user.client_id,
                "agent_key.rotated",
                "agent_governance_agent_key",
                Some(records.replaced.key_id.clone()),
                json!({
                    "org_id": org_id,
                    "agent_key_id": records.replaced.key_id,
                    "replacement_agent_key_id": records.replacement.key_id,
                    "display_name": records.replaced.display_name,
                    "expires_at": records.replaced.expires_at,
                    "replacement_expires_at": records.replacement.expires_at,
                    "grace_period_hours": grace_period_hours,
                    "rotation_reason": records.replaced.rotation_reason
                }),
            )
            .await;
            (
                StatusCode::OK,
                Json(RotateAgentGovernanceAgentKeyResponse {
                    replacement: records.replacement,
                    replaced: records.replaced,
                    token,
                }),
            )
                .into_response()
        }
        Ok(crate::db::RotateAgentGovernanceAgentKeyOutcome::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Agent key not found" })),
        )
            .into_response(),
        Ok(crate::db::RotateAgentGovernanceAgentKeyOutcome::Revoked) => (
            StatusCode::CONFLICT,
            Json(json!({ "error": "Cannot rotate a revoked agent key", "code": "agent_key_revoked" })),
        )
            .into_response(),
        Ok(crate::db::RotateAgentGovernanceAgentKeyOutcome::Expired) => (
            StatusCode::CONFLICT,
            Json(json!({ "error": "Cannot rotate an expired agent key", "code": "agent_key_expired" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, key_id = %key_id, "Failed to rotate agent governance agent key");
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
