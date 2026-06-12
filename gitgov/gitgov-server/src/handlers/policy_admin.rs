// ============================================================================
// POLICIES
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyApiResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<GitGovConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<PolicySourceMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PolicyOverridePayload {
    GovernedRequest(PolicyOverrideRequest),
    LegacyConfig(GitGovConfig),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyOverrideRequest {
    pub config: GitGovConfig,
    #[serde(default)]
    pub quality_gate_exception: Option<PolicyQualityGateExceptionRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyQualityGateExceptionRequest {
    pub reason: String,
    #[serde(default)]
    pub ticket_id: Option<String>,
    #[serde(default)]
    pub approved_by: Option<String>,
    pub expires_at: i64,
}

fn enforcement_rank(level: &EnforcementLevel) -> u8 {
    match level {
        EnforcementLevel::Off => 0,
        EnforcementLevel::Warn => 1,
        EnforcementLevel::Block => 2,
    }
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub async fn get_policy(
    Extension(_auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(repo_name): Path<String>,
) -> impl IntoResponse {
    // First get repo ID by full_name
    let repo = match state.db.get_repo_by_full_name(&repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(PolicyApiResponse {
                    version: None,
                    checksum: None,
                    config: None,
                    source: None,
                    updated_at: None,
                    error: Some("Repository not found".to_string()),
                }),
            );
        }
        Err(_e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PolicyApiResponse {
                    version: None,
                    checksum: None,
                    config: None,
                    source: None,
                    updated_at: None,
                    error: Some("Internal database error".to_string()),
                }),
            );
        }
    };

    match state.db.get_policy(&repo.id).await {
        Ok(Some(policy)) => (
            StatusCode::OK,
            Json(PolicyApiResponse {
                version: Some(policy.version),
                checksum: Some(policy.checksum),
                config: Some(policy.config),
                source: Some(policy.source),
                updated_at: Some(policy.updated_at),
                error: None,
            }),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(PolicyApiResponse {
                version: None,
                checksum: None,
                config: None,
                source: None,
                updated_at: None,
                error: Some("Policy not found".to_string()),
            }),
        ),
        Err(_e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(PolicyApiResponse {
                version: None,
                checksum: None,
                config: None,
                source: None,
                updated_at: None,
                error: Some("Internal database error".to_string()),
            }),
        ),
    }
}

pub async fn override_policy(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(repo_name): Path<String>,
    Json(payload): Json<PolicyOverridePayload>,
) -> impl IntoResponse {
    if require_admin(&auth_user).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(PolicyApiResponse {
                version: None,
                checksum: None,
                config: None,
                source: None,
                updated_at: None,
                error: Some("Admin access required".to_string()),
            }),
        );
    }

    // First get repo ID by full_name
    let repo = match state.db.get_repo_by_full_name(&repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(PolicyApiResponse {
                    version: None,
                    checksum: None,
                    config: None,
                    source: None,
                    updated_at: None,
                    error: Some("Repository not found".to_string()),
                }),
            );
        }
        Err(_e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PolicyApiResponse {
                    version: None,
                    checksum: None,
                    config: None,
                    source: None,
                    updated_at: None,
                    error: Some("Internal database error".to_string()),
                }),
            );
        }
    };

    let previous_policy = match state.db.get_policy(&repo.id).await {
        Ok(policy) => policy,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PolicyApiResponse {
                    version: None,
                    checksum: None,
                    config: None,
                    source: None,
                    updated_at: None,
                    error: Some("Internal database error".to_string()),
                }),
            );
        }
    };

    let (mut config, requested_exception, explicit_exception_control) = match payload {
        PolicyOverridePayload::LegacyConfig(config) => (config, None, false),
        PolicyOverridePayload::GovernedRequest(req) => {
            (req.config, req.quality_gate_exception, true)
        }
    };

    let now_ms = chrono::Utc::now().timestamp_millis();
    let max_exception_window_ms: i64 = 30 * 24 * 60 * 60 * 1000;

    if !explicit_exception_control {
        if let Some(prev) = previous_policy.as_ref() {
            config.quality_gate_exception = prev.config.quality_gate_exception.clone();
        }
    } else if let Some(req) = requested_exception {
        let reason = req.reason.trim().to_string();
        if reason.is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(PolicyApiResponse {
                    version: None,
                    checksum: None,
                    config: None,
                    source: None,
                    updated_at: None,
                    error: Some(
                        "quality_gate_exception.reason is required for governed overrides"
                            .to_string(),
                    ),
                }),
            );
        }
        if req.expires_at <= now_ms {
            return (
                StatusCode::BAD_REQUEST,
                Json(PolicyApiResponse {
                    version: None,
                    checksum: None,
                    config: None,
                    source: None,
                    updated_at: None,
                    error: Some(
                        "quality_gate_exception.expires_at must be in the future".to_string(),
                    ),
                }),
            );
        }
        if req.expires_at > now_ms + max_exception_window_ms {
            return (
                StatusCode::BAD_REQUEST,
                Json(PolicyApiResponse {
                    version: None,
                    checksum: None,
                    config: None,
                    source: None,
                    updated_at: None,
                    error: Some(
                        "quality_gate_exception.expires_at exceeds max window (30 days)"
                            .to_string(),
                    ),
                }),
            );
        }

        config.quality_gate_exception = Some(QualityGateExceptionConfig {
            enabled: true,
            reason,
            ticket_id: normalize_optional_string(req.ticket_id),
            approved_by: Some(
                normalize_optional_string(req.approved_by)
                    .unwrap_or_else(|| auth_user.client_id.clone()),
            ),
            expires_at: req.expires_at,
            created_at: Some(now_ms),
        });
    } else {
        // Explicit governed request with null exception clears the active exception.
        config.quality_gate_exception = None;
    }

    let previous_quality = previous_policy
        .as_ref()
        .map(|p| p.config.enforcement.quality_gates.clone())
        .unwrap_or_default();
    let new_quality = config.enforcement.quality_gates.clone();
    let quality_gate_weakened =
        enforcement_rank(&new_quality) < enforcement_rank(&previous_quality);

    if quality_gate_weakened {
        let has_active_exception = config
            .quality_gate_exception
            .as_ref()
            .map(|e| e.enabled && e.expires_at > now_ms)
            .unwrap_or(false);
        if !has_active_exception {
            return (
                StatusCode::BAD_REQUEST,
                Json(PolicyApiResponse {
                    version: None,
                    checksum: None,
                    config: None,
                    source: None,
                    updated_at: None,
                    error: Some(
                        "quality gate enforcement downgrade requires active quality_gate_exception"
                            .to_string(),
                    ),
                }),
            );
        }
    }

    if let Err(e) = gitgov_policy_core::validate_policy_config(&config) {
        return (
            StatusCode::BAD_REQUEST,
            Json(PolicyApiResponse {
                version: None,
                checksum: None,
                config: None,
                source: None,
                updated_at: None,
                error: Some(e.to_string()),
            }),
        );
    }

    let checksum = match gitgov_policy_core::policy_checksum(&config) {
        Ok(checksum) => checksum,
        Err(_e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(PolicyApiResponse {
                    version: None,
                    checksum: None,
                    config: None,
                    source: None,
                    updated_at: None,
                    error: Some("Invalid policy config payload".to_string()),
                }),
            );
        }
    };
    let source = PolicySourceMetadata::control_plane_managed(&auth_user.client_id, &checksum);

    // Record that this is an override
    tracing::warn!(
        "Policy override for {} by {} (is_override=true)",
        repo_name,
        auth_user.client_id
    );

    match state
        .db
        .save_policy_with_source(&repo.id, &config, &checksum, &auth_user.client_id, &source)
        .await
    {
        Ok(()) => {
            // Admin audit log — append-only, non-fatal
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "policy_override".to_string(),
                target_type: Some("repo".to_string()),
                target_id: Some(repo.id.clone()),
                metadata: serde_json::json!({
                    "repo_name": repo_name,
                    "checksum": checksum,
                    "quality_gate_previous": format!("{:?}", previous_quality).to_lowercase(),
                    "quality_gate_new": format!("{:?}", new_quality).to_lowercase(),
                    "quality_gate_downgrade": quality_gate_weakened,
                    "quality_gate_exception": config.quality_gate_exception
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write admin audit log (policy_override): {}", e);
            }

            (
                StatusCode::OK,
                Json(PolicyApiResponse {
                    version: Some("1.0".to_string()),
                    checksum: Some(checksum),
                    config: Some(config),
                    source: Some(source),
                    updated_at: Some(chrono::Utc::now().timestamp_millis()),
                    error: None,
                }),
            )
        }
        Err(_e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(PolicyApiResponse {
                version: None,
                checksum: None,
                config: None,
                source: None,
                updated_at: None,
                error: Some("Internal database error".to_string()),
            }),
        ),
    }
}
