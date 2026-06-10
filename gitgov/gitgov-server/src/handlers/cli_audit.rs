// ============================================================================
// CLI COMMAND AUDIT — /cli/commands endpoint
// ============================================================================

pub async fn ingest_cli_command(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CliCommandInput>,
) -> impl IntoResponse {
    let org_scope = match resolve_required_product_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        payload.org_name.as_deref(),
    )
    .await
    {
        Ok(org_scope) => org_scope,
        Err(err) => {
            return (
                org_scope_status(err),
                Json(CliCommandResponse {
                    accepted: false,
                    id: None,
                    error: Some(cli_scope_error_message(err).to_string()),
                }),
            )
        }
    };

    if let Some(repo_name) = payload
        .repo_name
        .as_deref()
        .map(str::trim)
        .filter(|repo_name| !repo_name.is_empty())
    {
        let repo_owner = match cli_repo_name_owner(repo_name) {
            Some(owner) => owner,
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(CliCommandResponse {
                        accepted: false,
                        id: None,
                        error: Some("repo_name must be in owner/repo format".to_string()),
                    }),
                )
            }
        };
        if !repo_owner.eq_ignore_ascii_case(org_scope.login.trim()) {
            return (
                StatusCode::FORBIDDEN,
                Json(CliCommandResponse {
                    accepted: false,
                    id: None,
                    error: Some("repo_name owner does not match organization".to_string()),
                }),
            );
        }
    }

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();

    let record = CliCommandRecord {
        id: id.clone(),
        org_id: Some(org_scope.id),
        user_login: auth_user.client_id.clone(),
        command: payload.command.clone(),
        origin: payload.origin.clone(),
        branch: payload.branch.clone(),
        repo_name: payload.repo_name.clone(),
        exit_code: payload.exit_code,
        duration_ms: payload.duration_ms,
        metadata: payload.metadata.clone(),
        created_at: now,
    };

    match state.db.insert_cli_command(&record).await {
        Ok(()) => (
            StatusCode::OK,
            Json(CliCommandResponse {
                accepted: true,
                id: Some(id),
                error: None,
            }),
        ),
        Err(e) => {
            tracing::warn!(error = %e, user = %auth_user.client_id, "Failed to insert CLI command audit");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CliCommandResponse {
                    accepted: false,
                    id: None,
                    error: Some("Failed to record command".to_string()),
                }),
            )
        }
    }
}

pub async fn list_cli_commands(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<CliCommandQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let offset = query.offset.unwrap_or(0);
    let org_scope = match resolve_required_product_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        query.org_name.as_deref(),
    )
    .await
    {
        Ok(org_scope) => org_scope,
        Err(err) => {
            return (
                org_scope_status(err),
                Json(serde_json::json!({
                    "commands": [],
                    "total": 0,
                    "limit": limit,
                    "offset": offset,
                    "error": cli_scope_error_message(err),
                })),
            )
        }
    };

    // Admin sees all, developer sees only their own
    let user_filter = if auth_user.role == UserRole::Admin {
        query.user_login.as_deref()
    } else {
        Some(auth_user.client_id.as_str())
    };

    match state
        .db
        .list_cli_commands(Some(org_scope.id.as_str()), user_filter, limit, offset)
        .await
    {
        Ok((records, total)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "commands": records,
                "total": total,
                "limit": limit,
                "offset": offset,
            })),
        ),
        Err(e) => {
            tracing::warn!(error = %e, "Failed to list CLI commands");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "commands": [],
                    "total": 0,
                    "error": "Failed to list commands",
                })),
            )
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CliCommandQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    pub user_login: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

fn cli_scope_error_message(error: OrgScopeError) -> &'static str {
    match error {
        OrgScopeError::BadRequest => "org_name is required for global admin keys",
        OrgScopeError::NotFound => "Organization not found",
        OrgScopeError::Forbidden => "Requested org is outside API key scope",
        OrgScopeError::Internal => "Internal database error",
    }
}

fn cli_repo_name_owner(repo_name: &str) -> Option<&str> {
    let mut parts = repo_name.trim().split('/');
    let owner = parts.next()?.trim();
    let repo = parts.next()?.trim();
    if owner.is_empty()
        || repo.is_empty()
        || parts.next().is_some()
        || !cli_repo_name_part_is_valid(owner)
        || !cli_repo_name_part_is_valid(repo)
    {
        return None;
    }
    Some(owner)
}

fn cli_repo_name_part_is_valid(part: &str) -> bool {
    part.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}
