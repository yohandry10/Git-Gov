// ============================================================================
// POLICY CHANGE REQUESTS — request/approve/reject workflow
// ============================================================================

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PolicyChangeRequestQuery {
    #[serde(default)]
    pub status: Option<String>, // pending | approved | rejected
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
    #[serde(default)]
    pub include_config: Option<bool>,
}

fn normalize_policy_request_status(value: Option<&str>) -> Result<Option<String>, StatusCode> {
    let Some(raw) = value else {
        return Ok(None);
    };
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Ok(None);
    }
    if matches!(normalized.as_str(), "pending" | "approved" | "rejected") {
        Ok(Some(normalized))
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum StoredPolicyApprovalValidationError {
    InvalidConfig(String),
    ChecksumMismatch {
        expected: String,
        actual: String,
    },
    NonCanonical,
}

fn validate_stored_policy_change_request_for_approval(
    existing: &PolicyChangeRequestRecord,
) -> Result<(), StoredPolicyApprovalValidationError> {
    gitgov_policy_core::validate_policy_config(&existing.requested_config)
        .map_err(|e| StoredPolicyApprovalValidationError::InvalidConfig(e.to_string()))?;
    let checksum = gitgov_policy_core::policy_checksum(&existing.requested_config)
        .map_err(|_| StoredPolicyApprovalValidationError::NonCanonical)?;
    if checksum != existing.requested_checksum {
        return Err(StoredPolicyApprovalValidationError::ChecksumMismatch {
            expected: existing.requested_checksum.clone(),
            actual: checksum,
        });
    }
    Ok(())
}

pub async fn create_policy_change_request(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(repo_name): Path<String>,
    Json(payload): Json<PolicyChangeRequestInput>,
) -> impl IntoResponse {
    let repo = match state.db.get_repo_by_full_name(&repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(PolicyChangeRequestCreateResponse {
                    accepted: false,
                    request_id: None,
                    status: "error".to_string(),
                    error: Some("Repository not found".to_string()),
                }),
            );
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to resolve repository for policy request");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PolicyChangeRequestCreateResponse {
                    accepted: false,
                    request_id: None,
                    status: "error".to_string(),
                    error: Some("Internal database error".to_string()),
                }),
            );
        }
    };

    if let Err(e) = gitgov_policy_core::validate_policy_config(&payload.config) {
        tracing::warn!(error = %e, "Rejected invalid policy change request config");
        return (
            StatusCode::BAD_REQUEST,
            Json(PolicyChangeRequestCreateResponse {
                accepted: false,
                request_id: None,
                status: "error".to_string(),
                error: Some(e.to_string()),
            }),
        );
    }

    let checksum = match gitgov_policy_core::policy_checksum(&payload.config) {
        Ok(value) => value,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to canonicalize policy change request config");
            return (
                StatusCode::BAD_REQUEST,
                Json(PolicyChangeRequestCreateResponse {
                    accepted: false,
                    request_id: None,
                    status: "error".to_string(),
                    error: Some("Invalid policy config payload".to_string()),
                }),
            );
        }
    };
    let request_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();

    match state
        .db
        .create_policy_change_request(CreatePolicyChangeRequestInput {
            request_id: &request_id,
            org_id: repo.org_id.as_deref(),
            repo_id: &repo.id,
            repo_name: &repo_name,
            requested_by: &auth_user.client_id,
            requested_config: &payload.config,
            requested_checksum: &checksum,
            reason: payload.reason.as_deref(),
            created_at: now,
        })
        .await
    {
        Ok(()) => {
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "policy_change_request_created".to_string(),
                target_type: Some("repo".to_string()),
                target_id: Some(repo.id.clone()),
                metadata: serde_json::json!({
                    "repo_name": repo_name,
                    "request_id": request_id,
                    "requested_checksum": checksum
                }),
                created_at: now,
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write admin audit log (policy_change_request_created): {}", e);
            }

            (
                StatusCode::OK,
                Json(PolicyChangeRequestCreateResponse {
                    accepted: true,
                    request_id: Some(request_id),
                    status: "pending".to_string(),
                    error: None,
                }),
            )
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to create policy change request");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PolicyChangeRequestCreateResponse {
                    accepted: false,
                    request_id: None,
                    status: "error".to_string(),
                    error: Some("Failed to create policy change request".to_string()),
                }),
            )
        }
    }
}

pub async fn list_policy_change_requests(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(repo_name): Path<String>,
    Query(query): Query<PolicyChangeRequestQuery>,
) -> impl IntoResponse {
    let status = match normalize_policy_request_status(query.status.as_deref()) {
        Ok(value) => value,
        Err(code) => {
            return (
                code,
                Json(serde_json::json!({
                    "error": "Invalid status filter",
                    "valid_values": ["pending", "approved", "rejected"]
                })),
            )
                .into_response();
        }
    };
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let offset = query.offset.unwrap_or(0).max(0);
    let include_config = query.include_config.unwrap_or(true);

    let requested_by = if auth_user.role == UserRole::Admin {
        None
    } else {
        Some(auth_user.client_id.as_str())
    };

    match state
        .db
        .list_policy_change_requests(ListPolicyChangeRequestsInput {
            org_id: auth_user.org_id.as_deref(),
            repo_name: Some(repo_name.as_str()),
            requested_by,
            status: status.as_deref(),
            limit,
            offset,
            include_config,
        })
        .await
    {
        Ok((requests, total)) => (
            StatusCode::OK,
            Json(PolicyChangeRequestListResponse {
                requests,
                total,
                limit,
                offset,
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::warn!(error = %e, "Failed to list policy change requests");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PolicyChangeRequestListResponse {
                    requests: vec![],
                    total: 0,
                    limit,
                    offset,
                }),
            )
                .into_response()
        }
    }
}

pub async fn approve_policy_change_request(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(request_id): Path<String>,
    Json(payload): Json<PolicyChangeRequestDecisionInput>,
) -> impl IntoResponse {
    if let Err(e) = require_admin(&auth_user) {
        return e.into_response();
    }

    let existing = match state
        .db
        .get_policy_change_request_by_id(&request_id, auth_user.org_id.as_deref())
        .await
    {
        Ok(Some(item)) => item,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Policy change request not found"})),
            )
                .into_response();
        }
        Err(e) => {
            tracing::warn!(error = %e, request_id = %request_id, "Failed to load policy request");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal database error"})),
            )
                .into_response();
        }
    };

    if existing.status != "pending" {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "Policy change request already decided",
                "status": existing.status
            })),
        )
            .into_response();
    }

    if existing.requested_by == auth_user.client_id {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "Self-approval is not allowed for policy changes"
            })),
        )
            .into_response();
    }

    match validate_stored_policy_change_request_for_approval(&existing) {
        Ok(()) => {}
        Err(StoredPolicyApprovalValidationError::InvalidConfig(details)) => {
            tracing::warn!(
                error = %details,
                request_id = %request_id,
                "Rejected approval of invalid stored policy change request config"
            );
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Policy change request config is no longer valid",
                    "details": details
                })),
            )
                .into_response();
        }
        Err(StoredPolicyApprovalValidationError::ChecksumMismatch { expected, actual }) => {
            tracing::warn!(
                request_id = %request_id,
                expected_checksum = %expected,
                actual_checksum = %actual,
                "Rejected approval of policy change request with checksum mismatch"
            );
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "error": "Policy change request checksum no longer matches requested config"
                })),
            )
                .into_response();
        }
        Err(StoredPolicyApprovalValidationError::NonCanonical) => {
            tracing::warn!(
                request_id = %request_id,
                "Rejected approval of non-canonical policy change request config"
            );
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Policy change request config cannot be canonicalized"
                })),
            )
                .into_response();
        }
    }

    let decided_at = chrono::Utc::now().timestamp_millis();
    match state
        .db
        .approve_policy_change_request(
            &request_id,
            auth_user.org_id.as_deref(),
            &auth_user.client_id,
            payload.note.as_deref(),
            decided_at,
        )
        .await
    {
        Ok(record) => {
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "policy_change_request_approved".to_string(),
                target_type: Some("policy_change_request".to_string()),
                target_id: Some(request_id.clone()),
                metadata: serde_json::json!({
                    "repo_name": record.repo_name,
                    "requested_by": record.requested_by,
                    "requested_checksum": record.requested_checksum
                }),
                created_at: decided_at,
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write admin audit log (policy_change_request_approved): {}", e);
            }

            (StatusCode::OK, Json(record)).into_response()
        }
        Err(DbError::Duplicate(_)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": "Policy change request already decided"})),
        )
            .into_response(),
        Err(DbError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Policy change request not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::warn!(error = %e, request_id = %request_id, "Failed to approve policy request");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to approve policy change request"})),
            )
                .into_response()
        }
    }
}

pub async fn reject_policy_change_request(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(request_id): Path<String>,
    Json(payload): Json<PolicyChangeRequestDecisionInput>,
) -> impl IntoResponse {
    if let Err(e) = require_admin(&auth_user) {
        return e.into_response();
    }

    let existing = match state
        .db
        .get_policy_change_request_by_id(&request_id, auth_user.org_id.as_deref())
        .await
    {
        Ok(Some(item)) => item,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Policy change request not found"})),
            )
                .into_response();
        }
        Err(e) => {
            tracing::warn!(error = %e, request_id = %request_id, "Failed to load policy request");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal database error"})),
            )
                .into_response();
        }
    };

    if existing.status != "pending" {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "Policy change request already decided",
                "status": existing.status
            })),
        )
            .into_response();
    }

    if existing.requested_by == auth_user.client_id {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "Self-decision is not allowed for policy changes"
            })),
        )
            .into_response();
    }

    let decided_at = chrono::Utc::now().timestamp_millis();
    match state
        .db
        .reject_policy_change_request(
            &request_id,
            auth_user.org_id.as_deref(),
            &auth_user.client_id,
            payload.note.as_deref(),
            decided_at,
        )
        .await
    {
        Ok(record) => {
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "policy_change_request_rejected".to_string(),
                target_type: Some("policy_change_request".to_string()),
                target_id: Some(request_id.clone()),
                metadata: serde_json::json!({
                    "repo_name": record.repo_name,
                    "requested_by": record.requested_by,
                    "requested_checksum": record.requested_checksum
                }),
                created_at: decided_at,
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write admin audit log (policy_change_request_rejected): {}", e);
            }

            (StatusCode::OK, Json(record)).into_response()
        }
        Err(DbError::Duplicate(_)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": "Policy change request already decided"})),
        )
            .into_response(),
        Err(DbError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Policy change request not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::warn!(error = %e, request_id = %request_id, "Failed to reject policy request");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to reject policy change request"})),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod policy_change_request_tests {
    use super::*;

    fn request_record(config: GitGovConfig, checksum: String) -> PolicyChangeRequestRecord {
        PolicyChangeRequestRecord {
            id: "request-id".to_string(),
            org_id: Some("org-id".to_string()),
            repo_id: "repo-id".to_string(),
            repo_name: "acme/repo".to_string(),
            requested_by: "developer".to_string(),
            requested_checksum: checksum,
            requested_config: config,
            reason: Some("policy update".to_string()),
            status: "pending".to_string(),
            decided_by: None,
            decision_note: None,
            created_at: 1_700_000_000,
            decided_at: None,
        }
    }

    #[test]
    fn stored_policy_change_request_approval_accepts_valid_checksum() {
        let config = GitGovConfig::default();
        let checksum = gitgov_policy_core::policy_checksum(&config).unwrap();
        let record = request_record(config, checksum);

        assert!(validate_stored_policy_change_request_for_approval(&record).is_ok());
    }

    #[test]
    fn stored_policy_change_request_approval_rejects_invalid_opa_config() {
        let mut config = GitGovConfig::default();
        config.adapters.opa.enabled = true;
        config.adapters.opa.base_url = Some("http://opa.example.com:8181".to_string());
        let checksum = gitgov_policy_core::policy_checksum(&config).unwrap();
        let record = request_record(config, checksum);

        let error = validate_stored_policy_change_request_for_approval(&record).unwrap_err();

        assert!(matches!(
            error,
            StoredPolicyApprovalValidationError::InvalidConfig(_)
        ));
    }

    #[test]
    fn stored_policy_change_request_approval_rejects_checksum_mismatch() {
        let mut config = GitGovConfig::default();
        config.enforcement.branches = EnforcementLevel::Warn;
        let record = request_record(config, "stale-checksum".to_string());

        let error = validate_stored_policy_change_request_for_approval(&record).unwrap_err();

        assert!(matches!(
            error,
            StoredPolicyApprovalValidationError::ChecksumMismatch { .. }
        ));
    }
}
