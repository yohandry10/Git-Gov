async fn load_audit_stats(state: &AppState, org_id: Option<&str>) -> Result<AuditStats, DbError> {
    if let Some(stats) = get_cached_stats(state, org_id) {
        return Ok(stats);
    }

    // Single-flight guard: if many requests miss cache simultaneously,
    // only one recomputes stats while others wait and reuse the result.
    let _refresh_guard = state.stats_cache_refresh_lock.lock().await;
    if let Some(stats) = get_cached_stats(state, org_id) {
        return Ok(stats);
    }

    let stats_fut = state.db.get_stats(org_id);
    let pipeline_fut = state.db.get_pipeline_health_stats(org_id);
    let desktop_pushes_fut = state.db.get_desktop_pushes_today(org_id);
    let (stats_result, pipeline_result, desktop_pushes_result) =
        tokio::join!(stats_fut, pipeline_fut, desktop_pushes_fut);

    let mut stats = stats_result?;
    stats.pipeline = pipeline_result.unwrap_or_default();
    stats.client_events.desktop_pushes_today = match desktop_pushes_result {
        Ok(count) => count,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to compute desktop pushes today for /stats");
            0
        }
    };
    put_cached_stats(state, org_id, &stats);
    Ok(stats)
}

pub async fn get_logs(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(filter): Query<EventFilter>,
) -> impl IntoResponse {
    // Non-admins can only see their own events
    let clamped_limit = if filter.limit == 0 {
        100
    } else {
        filter.limit.min(500)
    };
    let mut filter = if auth_user.role != UserRole::Admin {
        EventFilter {
            user_login: Some(auth_user.client_id.clone()),
            limit: clamped_limit,
            ..filter
        }
    } else {
        EventFilter {
            limit: clamped_limit,
            ..filter
        }
    };
    let deprecations = logs_deprecations_for_request(&filter);

    if filter.offset > 0 {
        tracing::warn!(
            requested_offset = filter.offset,
            "Deprecated /logs offset pagination requested; prefer keyset cursor"
        );
    }
    if should_reject_logs_offset(&filter, state.logs_reject_offset_pagination) {
        return (
            StatusCode::BAD_REQUEST,
            Json(LogsResponse {
                events: vec![],
                error: Some(
                    "offset pagination is disabled; use before_created_at and before_id"
                        .to_string(),
                ),
                stale: None,
                deprecations,
            }),
        );
    }

    let scoped_org_id = match resolve_and_check_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        filter.org_name.as_deref(),
        false,
    )
    .await
    {
        Ok(org_id) => org_id,
        Err(err) => {
            let error = match err {
                OrgScopeError::BadRequest => "org_name is required",
                OrgScopeError::NotFound => "Organization not found",
                OrgScopeError::Forbidden => "Requested org is outside API key scope",
                OrgScopeError::Internal => "Internal database error",
            };
            return (
                org_scope_status(err),
                Json(LogsResponse {
                    events: vec![],
                    error: Some(error.to_string()),
                    stale: None,
                    deprecations: deprecations.clone(),
                }),
            );
        }
    };
    if scoped_org_id.is_some() {
        // Prefer UUID scope to avoid extra org_name lookups in DB query path.
        filter.org_id = scoped_org_id;
        filter.org_name = None;
    }
    // Keyset pagination path should not also apply offset pagination.
    if filter.before_created_at.is_some() {
        filter.offset = 0;
    }
    let logs_key = logs_cache_key(&auth_user.role, &filter);
    if let Some(cache_key) = logs_key.as_deref() {
        if let Some(cached_events) = get_cached_logs(&state, cache_key) {
            return (
                StatusCode::OK,
                Json(LogsResponse {
                    events: cached_events,
                    error: None,
                    stale: None,
                    deprecations: deprecations.clone(),
                }),
            );
        }
    }

    match state.db.get_combined_events(&filter).await {
        Ok(events) => {
            if let Some(cache_key) = logs_key.as_deref() {
                put_cached_logs(&state, cache_key, &events);
            }
            (
                StatusCode::OK,
                Json(LogsResponse {
                    events,
                    error: None,
                    stale: None,
                    deprecations: deprecations.clone(),
                }),
            )
        }
        Err(e) => {
            if let Some(cache_key) = logs_key.as_deref() {
                if let Some(events) = get_cached_logs_on_error(&state, cache_key) {
                    tracing::warn!(
                        error = %e,
                        "Serving stale /logs cache due transient database error"
                    );
                    return (
                        StatusCode::OK,
                        Json(LogsResponse {
                            events,
                            error: None,
                            stale: Some(true),
                            deprecations: deprecations.clone(),
                        }),
                    );
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LogsResponse {
                    events: vec![],
                    error: Some("Internal database error".to_string()),
                    stale: None,
                    deprecations,
                }),
            )
        }
    }
}

pub async fn get_stats(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if require_admin(&auth_user).is_err() {
        return (StatusCode::FORBIDDEN, Json(AuditStats::default()));
    }

    let org_id = auth_user.org_id.as_deref();
    match load_audit_stats(&state, org_id).await {
        Ok(stats) => (StatusCode::OK, Json(stats)),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuditStats::default()),
        ),
    }
}

pub async fn get_team_overview(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<TeamOverviewQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    let status = if let Some(raw) = query.status.as_deref() {
        match normalize_org_user_status(Some(raw)) {
            Ok(s) => Some(s),
            Err(msg) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": msg })),
                )
                    .into_response();
            }
        }
    } else {
        None
    };

    let days = query.days.unwrap_or(30).clamp(1, 180);
    let limit = if query.limit == 0 {
        50
    } else {
        query.limit.min(500)
    } as i64;
    let offset = query.offset as i64;

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
                Json(serde_json::json!({ "error": "org_name is required for global admin keys" })),
            )
                .into_response();
        }
        Err(err) => {
            let error = match err {
                OrgScopeError::BadRequest => "org_name is required for global admin keys",
                OrgScopeError::NotFound => "Organization not found",
                OrgScopeError::Forbidden => "Requested org is outside API key scope",
                OrgScopeError::Internal => "Internal database error",
            };
            return (
                org_scope_status(err),
                Json(serde_json::json!({ "error": error })),
            )
                .into_response();
        }
    };

    match state
        .db
        .get_team_overview(&org_id, status.as_deref(), days, limit, offset)
        .await
    {
        Ok((entries, total)) => (
            StatusCode::OK,
            Json(TeamOverviewResponse { entries, total }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to load team overview");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn get_team_repos(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<TeamOverviewQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    let days = query.days.unwrap_or(30).clamp(1, 180);
    let limit = if query.limit == 0 {
        50
    } else {
        query.limit.min(500)
    } as i64;
    let offset = query.offset as i64;

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
                Json(serde_json::json!({ "error": "org_name is required for global admin keys" })),
            )
                .into_response();
        }
        Err(err) => {
            let error = match err {
                OrgScopeError::BadRequest => "org_name is required for global admin keys",
                OrgScopeError::NotFound => "Organization not found",
                OrgScopeError::Forbidden => "Requested org is outside API key scope",
                OrgScopeError::Internal => "Internal database error",
            };
            return (
                org_scope_status(err),
                Json(serde_json::json!({ "error": error })),
            )
                .into_response();
        }
    };

    match state.db.get_team_repos(&org_id, days, limit, offset).await {
        Ok((entries, total)) => {
            (StatusCode::OK, Json(TeamReposResponse { entries, total })).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to load team repo overview");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn get_daily_activity(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<DailyActivityQuery>,
) -> impl IntoResponse {
    if require_admin(&auth_user).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(Vec::<DailyActivityPoint>::new()),
        );
    }

    let days = query.days.unwrap_or(14).clamp(1, 90) as i64;
    let org_id = auth_user.org_id.as_deref();

    match state.db.get_daily_activity(org_id, days).await {
        Ok(points) => (StatusCode::OK, Json(points)),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Vec::<DailyActivityPoint>::new()),
        ),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardResponse {
    pub stats: AuditStats,
    pub recent_events: Vec<CombinedEvent>,
}

pub async fn get_dashboard(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if require_admin(&auth_user).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(DashboardResponse {
                stats: AuditStats::default(),
                recent_events: vec![],
            }),
        );
    }

    let org_id = auth_user.org_id.as_deref();
    let stats = load_audit_stats(&state, org_id).await.unwrap_or_default();

    let filter = EventFilter {
        limit: 10,
        org_id: auth_user.org_id.clone(),
        ..Default::default()
    };
    let recent = state
        .db
        .get_combined_events(&filter)
        .await
        .unwrap_or_default();

    (
        StatusCode::OK,
        Json(DashboardResponse {
            stats,
            recent_events: recent,
        }),
    )
}
