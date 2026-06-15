// ============================================================================
// ORGS (admin provisioning)
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrgRequest {
    /// GitHub login / slug — must be unique (e.g. "rimac")
    pub login: String,
    /// Human-readable display name (e.g. "Rimac Seguros")
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrgResponse {
    pub org_id: String,
    pub login: String,
    /// true = newly created, false = already existed (idempotent)
    pub created: bool,
}

pub async fn list_orgs(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    match state.db.list_orgs(auth_user.org_id.as_deref()).await {
        Ok(orgs) => (StatusCode::OK, Json(orgs)).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list orgs");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal database error" })),
            ).into_response()
        }
    }
}

pub async fn get_org(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(login): Path<String>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    let requested = login.trim();
    if requested.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "organization login is required" })),
        ).into_response();
    }

    let org_id = match resolve_and_check_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        Some(requested),
        true,
    ).await {
        Ok(Some(org_id)) => org_id,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "org_name is required for global admin keys" })),
            ).into_response();
        }
        Err(err) => {
            let error = match err {
                OrgScopeError::BadRequest => "org_name is required for global admin keys",
                OrgScopeError::NotFound => "Organization not found",
                OrgScopeError::Forbidden => "Requested org is outside API key scope",
                OrgScopeError::Internal => "Internal database error",
            };
            return (org_scope_status(err), Json(serde_json::json!({ "error": error }))).into_response();
        }
    };

    match state.db.get_org_by_id(&org_id).await {
        Ok(Some(org)) => (StatusCode::OK, Json(org)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Organization not found" })),
        ).into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to load org");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal database error" })),
            ).into_response()
        }
    }
}

pub async fn create_org(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateOrgRequest>,
) -> impl IntoResponse {
    let response = provision_platform_tenant(
        &auth_user,
        &state,
        PlatformTenantRequest {
            login: payload.login,
            name: payload.name,
            tenant_type: Some("customer".to_string()),
            lifecycle_status: Some("active".to_string()),
            metadata: serde_json::json!({ "compat_route": "/orgs" }),
        },
    )
    .await;

    match response {
        Ok((tenant, created)) => (
            if created { StatusCode::CREATED } else { StatusCode::OK },
            Json(CreateOrgResponse {
                org_id: tenant.id,
                login: tenant.login,
                created,
            }),
        )
            .into_response(),
        Err(resp) => resp,
    }
}

fn parse_user_role_strict(raw: Option<&str>) -> Result<UserRole, &'static str> {
    match raw.unwrap_or("Developer").trim() {
        "Admin" => Ok(UserRole::Admin),
        "Auditor" => Ok(UserRole::Auditor),
        "Architect" => Ok(UserRole::Architect),
        "Developer" => Ok(UserRole::Developer),
        "PM" => Ok(UserRole::PM),
        _ => Err("role must be one of: Admin, Auditor, Architect, Developer, PM"),
    }
}

fn normalize_org_user_status(raw: Option<&str>) -> Result<String, &'static str> {
    let normalized = raw.unwrap_or("active").trim().to_ascii_lowercase();
    match normalized.as_str() {
        "active" | "disabled" => Ok(normalized),
        _ => Err("status must be 'active' or 'disabled'"),
    }
}

fn normalize_org_invitation_status_filter(raw: Option<&str>) -> Result<String, &'static str> {
    let normalized = raw.unwrap_or("").trim().to_ascii_lowercase();
    match normalized.as_str() {
        "pending" | "accepted" | "revoked" | "expired" => Ok(normalized),
        _ => Err("status must be one of: pending, accepted, revoked, expired"),
    }
}

// ============================================================================
// ORG INVITATIONS (admin onboarding)
// ============================================================================

pub async fn create_org_invitation(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateOrgInvitationRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    let invite_email = payload.invite_email.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let invite_login = payload.invite_login.as_deref().map(str::trim).filter(|s| !s.is_empty());
    if invite_email.is_none() && invite_login.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "invite_email or invite_login is required" })),
        ).into_response();
    }

    let role = match parse_user_role_strict(payload.role.as_deref()) {
        Ok(role) => role,
        Err(msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": msg })),
            ).into_response();
        }
    };

    let org_id = match resolve_and_check_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        payload.org_name.as_deref(),
        true,
    ).await {
        Ok(Some(org_id)) => org_id,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "org_name is required for global admin keys" })),
            ).into_response();
        }
        Err(err) => {
            let error = match err {
                OrgScopeError::BadRequest => "org_name is required for global admin keys",
                OrgScopeError::NotFound => "Organization not found",
                OrgScopeError::Forbidden => "Requested org is outside API key scope",
                OrgScopeError::Internal => "Internal database error",
            };
            return (org_scope_status(err), Json(serde_json::json!({ "error": error }))).into_response();
        }
    };

    let expires_days = payload.expires_in_days.unwrap_or(7).clamp(1, 30);
    let expires_at = chrono::Utc::now() + chrono::Duration::days(expires_days);
    let invite_token = Uuid::new_v4().to_string();
    let token_hash = format!("{:x}", Sha256::digest(invite_token.as_bytes()));

    match state.db.create_org_invitation(&crate::db::CreateOrgInvitationInput {
        org_id: &org_id,
        invite_email,
        invite_login,
        role: role.as_str(),
        token_hash: &token_hash,
        invited_by: &auth_user.client_id,
        expires_at,
    }).await {
        Ok(invitation) => {
            let audit = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "create_org_invitation".to_string(),
                target_type: Some("org_invitation".to_string()),
                target_id: Some(invitation.id.clone()),
                metadata: serde_json::json!({
                    "org_id": invitation.org_id,
                    "invite_email": invitation.invite_email,
                    "invite_login": invitation.invite_login,
                    "role": invitation.role,
                    "expires_at": invitation.expires_at
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
                tracing::warn!(error = %e, "Failed to write admin audit log (create_org_invitation)");
            }

            (
                StatusCode::CREATED,
                Json(CreateOrgInvitationResponse {
                    invitation,
                    invite_token,
                }),
            ).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to create org invitation");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal database error" })),
            ).into_response()
        }
    }
}

pub async fn list_org_invitations(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<OrgInvitationsQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    let status = if let Some(raw) = query.status.as_deref() {
        match normalize_org_invitation_status_filter(Some(raw)) {
            Ok(value) => Some(value),
            Err(msg) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": msg })),
                ).into_response();
            }
        }
    } else {
        None
    };

    let org_id = match resolve_and_check_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        query.org_name.as_deref(),
        true,
    ).await {
        Ok(Some(org_id)) => org_id,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "org_name is required for global admin keys" })),
            ).into_response();
        }
        Err(err) => {
            let error = match err {
                OrgScopeError::BadRequest => "org_name is required for global admin keys",
                OrgScopeError::NotFound => "Organization not found",
                OrgScopeError::Forbidden => "Requested org is outside API key scope",
                OrgScopeError::Internal => "Internal database error",
            };
            return (org_scope_status(err), Json(serde_json::json!({ "error": error }))).into_response();
        }
    };

    let limit = if query.limit == 0 { 50 } else { query.limit.min(500) } as i64;
    let offset = query.offset as i64;

    match state
        .db
        .list_org_invitations(&org_id, status.as_deref(), limit, offset)
        .await
    {
        Ok((entries, total)) => (StatusCode::OK, Json(OrgInvitationsResponse { entries, total })).into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to list org invitations");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal database error" })),
            ).into_response()
        }
    }
}

pub async fn resend_org_invitation(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(invitation_id): Path<String>,
    Json(payload): Json<ResendOrgInvitationRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    let invite_token = Uuid::new_v4().to_string();
    let token_hash = format!("{:x}", Sha256::digest(invite_token.as_bytes()));
    let expires_days = payload.expires_in_days.unwrap_or(7).clamp(1, 30);
    let expires_at = chrono::Utc::now() + chrono::Duration::days(expires_days);

    match state
        .db
        .resend_org_invitation(
            &invitation_id,
            auth_user.org_id.as_deref(),
            &token_hash,
            &auth_user.client_id,
            expires_at,
        )
        .await
    {
        Ok(Some(invitation)) => {
            let audit = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "resend_org_invitation".to_string(),
                target_type: Some("org_invitation".to_string()),
                target_id: Some(invitation.id.clone()),
                metadata: serde_json::json!({
                    "org_id": invitation.org_id,
                    "invite_email": invitation.invite_email,
                    "invite_login": invitation.invite_login,
                    "role": invitation.role,
                    "expires_at": invitation.expires_at
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
                tracing::warn!(error = %e, "Failed to write admin audit log (resend_org_invitation)");
            }

            (
                StatusCode::OK,
                Json(CreateOrgInvitationResponse {
                    invitation,
                    invite_token,
                }),
            ).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "invitation not found or already accepted" })),
        ).into_response(),
        Err(e) => {
            tracing::error!(error = %e, invitation_id = %invitation_id, "Failed to resend org invitation");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal database error" })),
            ).into_response()
        }
    }
}

pub async fn revoke_org_invitation(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(invitation_id): Path<String>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    match state
        .db
        .revoke_org_invitation(&invitation_id, auth_user.org_id.as_deref(), &auth_user.client_id)
        .await
    {
        Ok(Some(invitation)) => {
            let audit = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "revoke_org_invitation".to_string(),
                target_type: Some("org_invitation".to_string()),
                target_id: Some(invitation.id.clone()),
                metadata: serde_json::json!({
                    "org_id": invitation.org_id,
                    "invite_email": invitation.invite_email,
                    "invite_login": invitation.invite_login
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
                tracing::warn!(error = %e, "Failed to write admin audit log (revoke_org_invitation)");
            }
            (StatusCode::OK, Json(invitation)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "invitation not found or not pending" })),
        ).into_response(),
        Err(e) => {
            tracing::error!(error = %e, invitation_id = %invitation_id, "Failed to revoke org invitation");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal database error" })),
            ).into_response()
        }
    }
}

pub async fn preview_org_invitation(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "token is required" })),
        ).into_response();
    }

    let token_hash = format!("{:x}", Sha256::digest(trimmed.as_bytes()));
    match state.db.get_active_org_invitation_by_token_hash(&token_hash).await {
        Ok(Some(invitation)) => (StatusCode::OK, Json(invitation)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "invitation not found" })),
        ).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to preview invitation");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal database error" })),
            ).into_response()
        }
    }
}

pub async fn accept_org_invitation(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AcceptOrgInvitationRequest>,
) -> impl IntoResponse {
    let token = payload.token.trim();
    if token.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "token is required" })),
        ).into_response();
    }
    let login = payload.login.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));

    if let Some(requested_login) = login {
        match state.db.get_active_org_invitation_by_token_hash(&token_hash).await {
            Ok(Some(invitation)) if !invitation.accepts_requested_login(Some(requested_login)) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "login does not match the invitation target"
                    })),
                )
                    .into_response();
            }
            Ok(_) => {}
            Err(e) => {
                tracing::error!(error = %e, "Failed to validate invitation login");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "Internal database error" })),
                )
                    .into_response();
            }
        }
    }

    match state.db.accept_org_invitation(&token_hash).await {
        Ok(Some(result)) => {
            let audit = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: result.org_user.login.clone(),
                action: "accept_org_invitation".to_string(),
                target_type: Some("org_invitation".to_string()),
                target_id: Some(result.invitation.id.clone()),
                metadata: serde_json::json!({
                    "org_id": result.invitation.org_id,
                    "client_id": result.org_user.login,
                    "role": result.invitation.role
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
                tracing::warn!(error = %e, "Failed to write admin audit log (accept_org_invitation)");
            }

            (
                StatusCode::OK,
                Json(AcceptOrgInvitationResponse {
                    invitation: result.invitation,
                    client_id: result.org_user.login,
                    role: result.org_user.role,
                    org_id: result.org_user.org_id,
                    api_key: result.api_key,
                }),
            ).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "invitation not found, expired or already used" })),
        ).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to accept invitation");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal database error" })),
            ).into_response()
        }
    }
}

