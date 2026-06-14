// ============================================================================
// AGENT GOVERNANCE ADMINISTRATION
// ============================================================================

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
