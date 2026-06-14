use crate::db::Database;
use crate::models::{AdminAuditLogEntry, UserRole};
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::sync::Arc;

const AGENT_GOVERNANCE_EVALUATE_SCOPE: &str = "agent_governance:evaluate";
const AGENT_GOVERNANCE_READ_SCOPE: &str = "agent_governance:read";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub client_id: String,
    pub role: UserRole,
    pub org_id: Option<String>,
    pub platform_principal_id: Option<String>,
    pub is_platform_founder: bool,
    pub principal_type: String,
    pub scopes: Vec<String>,
    pub agent_key_id: Option<String>,
    pub agent_display_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuthError {
    message: String,
    status: StatusCode,
    code: &'static str,
}

impl AuthError {
    fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::UNAUTHORIZED,
            code: "UNAUTHORIZED",
        }
    }

    fn forbidden(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::FORBIDDEN,
            code: "FORBIDDEN",
        }
    }

    fn forbidden_with_code(message: impl Into<String>, code: &'static str) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::FORBIDDEN,
            code,
        }
    }

    fn unauthorized_with_code(message: impl Into<String>, code: &'static str) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::UNAUTHORIZED,
            code,
        }
    }

    fn service_unavailable(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "AUTH_BACKEND_UNAVAILABLE",
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            axum::Json(serde_json::json!({
                "error": self.message,
                "code": self.code
            })),
        )
            .into_response()
    }
}

pub async fn auth_middleware(
    State(db): State<Arc<Database>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<axum::response::Response, AuthError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            metrics::counter!("gitgov_auth_total", "result" => "missing_header", "role" => "unknown").increment(1);
            AuthError::unauthorized("Missing Authorization header")
        })?;

    let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        metrics::counter!("gitgov_auth_total", "result" => "bad_format", "role" => "unknown")
            .increment(1);
        AuthError::unauthorized("Invalid Authorization header format")
    })?;

    let key_hash = format!("{:x}", sha2::Sha256::digest(token.as_bytes()));

    let path = req.uri().path().to_string();
    let method = req.method().clone();
    let auth_validation = db.validate_api_key(&key_hash).await.map_err(|e| {
        tracing::error!("Authentication backend error: {}", e);
        AuthError::service_unavailable("Authentication backend unavailable")
    })?;
    let auth_user = match auth_validation.auth {
        Some(auth_user) => auth_user,
        None if token.starts_with("ggag_") => {
            match db
                .validate_agent_governance_agent_key(&key_hash)
                .await
                .map_err(|e| {
                    tracing::error!("Agent key authentication backend error: {}", e);
                    AuthError::service_unavailable("Authentication backend unavailable")
                })? {
                Some(agent_key) => {
                    if let Some(reason) = agent_key.denied_reason.as_deref() {
                        let audit_action = if reason == "agent_key_expired" {
                            "agent_key.denied_expired"
                        } else if reason == "agent_key_revoked" {
                            "agent_key.denied_revoked"
                        } else {
                            "agent_key.denied"
                        };
                        write_agent_key_auth_audit(
                            &db,
                            &agent_key,
                            audit_action,
                            serde_json::json!({
                                "reason": reason,
                                "path": path,
                                "method": method.as_str()
                            }),
                        )
                        .await;
                        metrics::counter!("gitgov_auth_total", "result" => reason.to_string(), "role" => "agent").increment(1);
                        return Err(AuthError::unauthorized_with_code(
                            "Invalid or expired agent key",
                            if reason == "agent_key_expired" {
                                "agent_key_expired"
                            } else {
                                "agent_key_revoked"
                            },
                        ));
                    }

                    let has_evaluate_scope = agent_key
                        .scopes
                        .iter()
                        .any(|scope| scope == AGENT_GOVERNANCE_EVALUATE_SCOPE);
                    let has_read_scope = agent_key
                        .scopes
                        .iter()
                        .any(|scope| scope == AGENT_GOVERNANCE_READ_SCOPE);
                    let is_agent_governance_decision_path = method == axum::http::Method::POST
                        && matches!(
                            path.as_str(),
                            "/agent-governance/evaluate" | "/agent-governance/dry-run"
                        );
                    let is_agent_governance_read_path = method == axum::http::Method::GET
                        && path.as_str() == "/agent-governance/context";
                    let requested_scope = if is_agent_governance_read_path {
                        AGENT_GOVERNANCE_READ_SCOPE
                    } else {
                        AGENT_GOVERNANCE_EVALUATE_SCOPE
                    };
                    let scope_allowed = (is_agent_governance_decision_path && has_evaluate_scope)
                        || (is_agent_governance_read_path && has_read_scope);
                    if !scope_allowed {
                        write_agent_key_auth_audit(
                            &db,
                            &agent_key,
                            "agent_key.invalid_scope",
                            serde_json::json!({
                                "reason": if is_agent_governance_read_path && !has_read_scope {
                                    "missing_agent_governance_read_scope"
                                } else if is_agent_governance_decision_path && !has_evaluate_scope {
                                    "missing_agent_governance_evaluate_scope"
                                } else {
                                    "path_not_allowed"
                                },
                                "path": path,
                                "method": method.as_str(),
                                "scopes": agent_key.scopes
                            }),
                        )
                        .await;
                        metrics::counter!("gitgov_auth_total", "result" => "invalid_scope", "role" => "agent").increment(1);
                        return Err(AuthError::forbidden_with_code(
                            "Agent key scope does not allow this request",
                            "invalid_scope",
                        ));
                    }

                    if let Err(e) = db
                        .mark_agent_governance_agent_key_used(&agent_key.agent_key_id)
                        .await
                    {
                        tracing::warn!(
                            error = %e,
                            agent_key_id = %agent_key.agent_key_id,
                            "Failed to update agent key last_used_at"
                        );
                    }
                    write_agent_key_auth_audit(
                        &db,
                        &agent_key,
                        "agent_key.used",
                        serde_json::json!({
                            "path": path,
                            "method": method.as_str(),
                            "scope": requested_scope
                        }),
                    )
                    .await;

                    let mut scopes = agent_key.scopes;
                    if !agent_key.allowed_actions.is_empty() {
                        scopes.push(format!(
                            "agent_actions:{}",
                            agent_key.allowed_actions.join(",")
                        ));
                    }

                    crate::db::ApiKeyAuthContext {
                        client_id: agent_key.client_id,
                        role: UserRole::Developer,
                        org_id: Some(agent_key.org_id),
                        platform_principal_id: None,
                        is_platform_founder: false,
                        principal_type: "agent".to_string(),
                        scopes,
                        agent_key_id: Some(agent_key.agent_key_id),
                        agent_display_name: Some(agent_key.display_name),
                    }
                }
                None => {
                    metrics::counter!("gitgov_auth_total", "result" => "invalid_key", "role" => "agent").increment(1);
                    return Err(AuthError::unauthorized("Invalid or expired API key"));
                }
            }
        }
        None => {
            metrics::counter!("gitgov_auth_total", "result" => "invalid_key", "role" => "unknown")
                .increment(1);
            return Err(AuthError::unauthorized("Invalid or expired API key"));
        }
    };

    if auth_validation.used_stale_cache
        && auth_user.role == UserRole::Admin
        && is_sensitive_admin_path(path.as_str())
    {
        tracing::warn!(
            path = %path,
            client_id = %auth_user.client_id,
            "Blocking stale auth cache for sensitive admin endpoint"
        );
        return Err(AuthError::service_unavailable(
            "Authentication temporarily unavailable for this admin endpoint; retry shortly",
        ));
    }

    let user = AuthUser {
        client_id: auth_user.client_id,
        role: auth_user.role,
        org_id: auth_user.org_id,
        platform_principal_id: auth_user.platform_principal_id,
        is_platform_founder: auth_user.is_platform_founder,
        principal_type: auth_user.principal_type,
        scopes: auth_user.scopes,
        agent_key_id: auth_user.agent_key_id,
        agent_display_name: auth_user.agent_display_name,
    };

    metrics::counter!("gitgov_auth_total", "result" => "success", "role" => user.role.as_str())
        .increment(1);

    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}

async fn write_agent_key_auth_audit(
    db: &Arc<Database>,
    agent_key: &crate::db::AgentKeyAuthContext,
    action: &str,
    metadata: serde_json::Value,
) {
    let entry = AdminAuditLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        actor_client_id: agent_key.client_id.clone(),
        action: action.to_string(),
        target_type: Some("agent_governance_agent_key".to_string()),
        target_id: Some(agent_key.agent_key_id.clone()),
        metadata: serde_json::json!({
            "org_id": agent_key.org_id,
            "agent_key_id": agent_key.agent_key_id,
            "agent_display_name": agent_key.display_name,
            "principal_type": "agent",
            "metadata": metadata
        }),
        created_at: chrono::Utc::now().timestamp_millis(),
    };
    if let Err(e) = db.insert_admin_audit_log(&entry).await {
        tracing::warn!(error = %e, action, "Failed to write agent key auth audit");
    }
}

fn is_sensitive_admin_path(path: &str) -> bool {
    path.starts_with("/api-keys")
        || path.starts_with("/org-users")
        || path.starts_with("/org-invitations")
        || path.starts_with("/dashboard")
        || path.starts_with("/enterprise/")
        || path.starts_with("/compliance/")
        || path.starts_with("/deployment-gates/")
        || path.starts_with("/agent-governance/")
        || path.starts_with("/jobs/metrics")
        || path.starts_with("/outbox/lease/metrics")
}

pub fn require_admin(user: &AuthUser) -> Result<(), AuthError> {
    if user.role != UserRole::Admin {
        return Err(AuthError::forbidden("Admin access required"));
    }
    Ok(())
}

pub fn is_founder_global_admin(user: &AuthUser) -> bool {
    user.role == UserRole::Admin && user.org_id.is_none() && user.is_platform_founder
}

#[cfg(test)]
pub fn require_same_user_or_admin(user: &AuthUser, target_login: &str) -> Result<(), AuthError> {
    if user.role == UserRole::Admin {
        return Ok(());
    }

    if user.client_id != target_login {
        return Err(AuthError::forbidden("Can only access your own data"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn admin_user() -> AuthUser {
        AuthUser {
            client_id: "admin1".to_string(),
            role: UserRole::Admin,
            org_id: None,
            platform_principal_id: None,
            is_platform_founder: false,
            principal_type: "human".to_string(),
            scopes: Vec::new(),
            agent_key_id: None,
            agent_display_name: None,
        }
    }

    fn dev_user(login: &str) -> AuthUser {
        AuthUser {
            client_id: login.to_string(),
            role: UserRole::Developer,
            org_id: None,
            platform_principal_id: None,
            is_platform_founder: false,
            principal_type: "human".to_string(),
            scopes: Vec::new(),
            agent_key_id: None,
            agent_display_name: None,
        }
    }

    #[test]
    fn require_admin_allows_admin() {
        assert!(require_admin(&admin_user()).is_ok());
    }

    #[test]
    fn require_admin_blocks_developer() {
        assert!(require_admin(&dev_user("dev1")).is_err());
    }

    #[test]
    fn founder_global_admin_detection_matches_expected_scope() {
        assert!(is_founder_global_admin(&AuthUser {
            client_id: "bootstrap-admin".to_string(),
            role: UserRole::Admin,
            org_id: None,
            platform_principal_id: Some("principal-1".to_string()),
            is_platform_founder: true,
            principal_type: "platform_founder".to_string(),
            scopes: Vec::new(),
            agent_key_id: None,
            agent_display_name: None,
        }));
        assert!(is_founder_global_admin(&AuthUser {
            client_id: "platform-service".to_string(),
            role: UserRole::Admin,
            org_id: None,
            platform_principal_id: Some("principal-2".to_string()),
            is_platform_founder: true,
            principal_type: "platform_founder".to_string(),
            scopes: Vec::new(),
            agent_key_id: None,
            agent_display_name: None,
        }));
        assert!(!is_founder_global_admin(&AuthUser {
            client_id: "bootstrap-admin".to_string(),
            role: UserRole::Admin,
            org_id: Some("org-123".to_string()),
            platform_principal_id: Some("principal-1".to_string()),
            is_platform_founder: true,
            principal_type: "human".to_string(),
            scopes: Vec::new(),
            agent_key_id: None,
            agent_display_name: None,
        }));
        assert!(!is_founder_global_admin(&AuthUser {
            client_id: "bootstrap-admin".to_string(),
            role: UserRole::Admin,
            org_id: None,
            platform_principal_id: None,
            is_platform_founder: false,
            principal_type: "human".to_string(),
            scopes: Vec::new(),
            agent_key_id: None,
            agent_display_name: None,
        }));
    }

    #[test]
    fn require_same_user_or_admin_allows_admin_for_any_target() {
        assert!(require_same_user_or_admin(&admin_user(), "anyone").is_ok());
    }

    #[test]
    fn require_same_user_or_admin_allows_self() {
        assert!(require_same_user_or_admin(&dev_user("dev1"), "dev1").is_ok());
    }

    #[test]
    fn require_same_user_or_admin_blocks_different_user() {
        assert!(require_same_user_or_admin(&dev_user("dev1"), "dev2").is_err());
    }

    #[test]
    fn sensitive_admin_path_detection_matches_expected_routes() {
        assert!(is_sensitive_admin_path("/api-keys"));
        assert!(is_sensitive_admin_path("/api-keys/revoke"));
        assert!(is_sensitive_admin_path("/org-users"));
        assert!(is_sensitive_admin_path("/org-users/user-1/api-key"));
        assert!(is_sensitive_admin_path("/org-invitations"));
        assert!(is_sensitive_admin_path("/org-invitations/inv-1/revoke"));
        assert!(is_sensitive_admin_path("/dashboard"));
        assert!(is_sensitive_admin_path("/enterprise/adoption-profile"));
        assert!(is_sensitive_admin_path(
            "/enterprise/onboarding-checklist-tracking"
        ));
        assert!(is_sensitive_admin_path("/enterprise/release-approvals"));
        assert!(is_sensitive_admin_path(
            "/enterprise/release-governance/evaluate"
        ));
        assert!(is_sensitive_admin_path("/compliance/acme"));
        assert!(is_sensitive_admin_path("/compliance/control-frameworks"));
        assert!(is_sensitive_admin_path("/compliance/evidence-exports"));
        assert!(is_sensitive_admin_path(
            "/compliance/evidence-exports/cee_123/download"
        ));
        assert!(is_sensitive_admin_path("/compliance/evidence-mappings"));
        assert!(is_sensitive_admin_path(
            "/compliance/evidence-mappings/cem_123"
        ));
        assert!(is_sensitive_admin_path("/compliance/review-packages"));
        assert!(is_sensitive_admin_path(
            "/compliance/review-packages/crp_123/download"
        ));
        assert!(is_sensitive_admin_path(
            "/compliance/framework-review-reports"
        ));
        assert!(is_sensitive_admin_path(
            "/compliance/framework-review-reports/frr_123/download"
        ));
        assert!(is_sensitive_admin_path("/deployment-gates/authorize"));
        assert!(is_sensitive_admin_path("/deployment-gates/authorizations"));
        assert!(is_sensitive_admin_path("/agent-governance/evaluate"));
        assert!(is_sensitive_admin_path("/agent-governance/settings"));
        assert!(is_sensitive_admin_path("/agent-governance/evaluations"));
        assert!(is_sensitive_admin_path("/jobs/metrics"));
        assert!(is_sensitive_admin_path("/outbox/lease/metrics"));
        assert!(!is_sensitive_admin_path("/logs"));
        assert!(!is_sensitive_admin_path("/stats"));
    }

    #[test]
    fn auth_error_service_unavailable_maps_to_503() {
        let response = AuthError::service_unavailable("backend down").into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn require_admin_error_maps_to_403() {
        let response = require_admin(&dev_user("dev1"))
            .expect_err("developer should be denied")
            .into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
