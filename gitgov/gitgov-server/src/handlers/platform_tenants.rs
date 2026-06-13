// ============================================================================
// PLATFORM TENANT ADMINISTRATION
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformTenantRequest {
    /// Tenant/workspace slug, normally the GitHub org/user login.
    pub login: String,
    /// Human-readable tenant name.
    pub name: Option<String>,
    /// customer, internal, or sandbox. Defaults to customer.
    pub tenant_type: Option<String>,
    /// trial, active, suspended, archived, or deleted. Defaults to active.
    pub lifecycle_status: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformTenantLifecycleRequest {
    pub lifecycle_status: String,
    pub reason: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformTenantResponse {
    pub tenant: Org,
    pub created: bool,
}

fn normalize_platform_tenant_type(raw: Option<&str>) -> Result<String, &'static str> {
    let normalized = raw.unwrap_or("customer").trim().to_ascii_lowercase();
    match normalized.as_str() {
        "customer" | "internal" | "sandbox" => Ok(normalized),
        _ => Err("tenant_type must be one of: customer, internal, sandbox"),
    }
}

fn normalize_platform_lifecycle_status(raw: Option<&str>) -> Result<String, &'static str> {
    let normalized = raw.unwrap_or("active").trim().to_ascii_lowercase();
    match normalized.as_str() {
        "trial" | "active" | "suspended" | "archived" | "deleted" => Ok(normalized),
        _ => Err("lifecycle_status must be one of: trial, active, suspended, archived, deleted"),
    }
}

async fn write_platform_tenant_audit(
    state: &AppState,
    actor_client_id: &str,
    action: &str,
    target_id: Option<String>,
    metadata: serde_json::Value,
) {
    let audit = AdminAuditLogEntry {
        id: Uuid::new_v4().to_string(),
        actor_client_id: actor_client_id.to_string(),
        action: action.to_string(),
        target_type: Some("platform_tenant".to_string()),
        target_id,
        metadata,
        created_at: chrono::Utc::now().timestamp_millis(),
    };
    if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
        tracing::warn!(error = %e, action, "Failed to write platform tenant audit log");
    }
}

async fn deny_non_founder_platform_action(
    state: &AppState,
    auth_user: &AuthUser,
    action: &str,
    login: Option<&str>,
) -> axum::response::Response {
    write_platform_tenant_audit(
        state,
        &auth_user.client_id,
        "platform.tenant.provision_denied",
        None,
        serde_json::json!({
            "attempted_action": action,
            "actor_scope": if auth_user.org_id.is_some() { "tenant" } else { "global" },
            "actor_role": auth_user.role.as_str(),
            "target_tenant_login": login
        }),
    )
    .await;

    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({
            "error": "Only Platform Founder can administer tenants"
        })),
    )
        .into_response()
}

pub async fn list_platform_tenants(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if !is_founder_global_admin(&auth_user) {
        return deny_non_founder_platform_action(&state, &auth_user, "list_tenants", None).await;
    }

    match state.db.list_orgs(None).await {
        Ok(tenants) => (StatusCode::OK, Json(tenants)).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list platform tenants");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn provision_platform_tenant_endpoint(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PlatformTenantRequest>,
) -> impl IntoResponse {
    match provision_platform_tenant(&auth_user, &state, payload).await {
        Ok((tenant, created)) => (
            if created {
                StatusCode::CREATED
            } else {
                StatusCode::OK
            },
            Json(PlatformTenantResponse { tenant, created }),
        )
            .into_response(),
        Err(resp) => resp,
    }
}

async fn provision_platform_tenant(
    auth_user: &AuthUser,
    state: &AppState,
    payload: PlatformTenantRequest,
) -> Result<(Org, bool), axum::response::Response> {
    let login = payload.login.trim();
    if !is_founder_global_admin(auth_user) {
        return Err(
            deny_non_founder_platform_action(state, auth_user, "provision_tenant", Some(login))
                .await,
        );
    }

    if login.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "tenant login is required" })),
        )
            .into_response());
    }

    let tenant_type = match normalize_platform_tenant_type(payload.tenant_type.as_deref()) {
        Ok(value) => value,
        Err(message) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": message })),
            )
                .into_response())
        }
    };
    let lifecycle_status =
        match normalize_platform_lifecycle_status(payload.lifecycle_status.as_deref()) {
            Ok(value) => value,
            Err(message) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": message })),
                )
                    .into_response())
            }
        };

    match state
        .db
        .upsert_platform_tenant(
            login,
            payload.name.as_deref(),
            &tenant_type,
            &lifecycle_status,
            &payload.metadata,
            &auth_user.client_id,
        )
        .await
    {
        Ok((tenant, created)) => {
            write_platform_tenant_audit(
                state,
                &auth_user.client_id,
                if created {
                    "platform.tenant.created"
                } else {
                    "platform.tenant.updated"
                },
                Some(tenant.id.clone()),
                serde_json::json!({
                    "actor_scope": "platform",
                    "target_tenant_login": tenant.login,
                    "tenant_type": tenant.tenant_type,
                    "lifecycle_status": tenant.lifecycle_status,
                    "created": created,
                    "source": "platform_provisioning",
                }),
            )
            .await;
            Ok((tenant, created))
        }
        Err(e) => {
            tracing::error!(error = %e, login = %payload.login, "Failed to provision platform tenant");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal database error" })),
            )
                .into_response())
        }
    }
}

pub async fn update_platform_tenant_lifecycle(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(login): Path<String>,
    Json(payload): Json<PlatformTenantLifecycleRequest>,
) -> impl IntoResponse {
    let requested_login = login.trim();
    if !is_founder_global_admin(&auth_user) {
        return deny_non_founder_platform_action(
            &state,
            &auth_user,
            "update_tenant_lifecycle",
            Some(requested_login),
        )
        .await;
    }

    if requested_login.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "tenant login is required" })),
        )
            .into_response();
    }

    let lifecycle_status =
        match normalize_platform_lifecycle_status(Some(payload.lifecycle_status.as_str())) {
            Ok(value) => value,
            Err(message) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": message })),
                )
                    .into_response()
            }
        };
    let metadata = serde_json::json!({
        "lifecycle_reason": payload.reason,
        "lifecycle_metadata": payload.metadata
    });

    match state
        .db
        .update_platform_tenant_lifecycle(requested_login, &lifecycle_status, &metadata)
        .await
    {
        Ok(Some(tenant)) => {
            write_platform_tenant_audit(
                &state,
                &auth_user.client_id,
                "platform.tenant.lifecycle_changed",
                Some(tenant.id.clone()),
                serde_json::json!({
                    "actor_scope": "platform",
                    "target_tenant_login": tenant.login,
                    "lifecycle_status": tenant.lifecycle_status,
                    "reason": metadata.get("lifecycle_reason").cloned().unwrap_or(serde_json::Value::Null)
                }),
            )
            .await;
            (StatusCode::OK, Json(tenant)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Tenant not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, login = %requested_login, "Failed to update tenant lifecycle");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
