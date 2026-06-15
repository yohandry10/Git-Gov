// ============================================================================
// COMPLIANCE FRAMEWORK REVIEW REPORT COLLABORATION
// ============================================================================

const MAX_FRAMEWORK_REVIEW_REPORT_ASSIGNMENTS: usize = 20;
const MAX_FRAMEWORK_REVIEW_REPORT_COMMENT_LEN: usize = 2000;

fn normalize_framework_review_report_id(report_id: &str) -> Result<String, &'static str> {
    let normalized = report_id.trim().to_string();
    if normalized.starts_with("frr_") && normalized.len() <= 80 {
        Ok(normalized)
    } else {
        Err("report_id must be a valid frr_ identifier")
    }
}

fn normalize_auditor_client_id(value: &str) -> Result<String, String> {
    let normalized = value.trim().to_string();
    if normalized.is_empty() || normalized.len() > 128 {
        return Err("auditor_client_id must be 1-128 characters".to_string());
    }
    if !normalized
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '@'))
    {
        return Err("auditor_client_id contains unsupported characters".to_string());
    }
    let lowered = normalized.to_ascii_lowercase();
    if lowered.contains("bearer ")
        || lowered.contains("ghp_")
        || lowered.contains("glpat-")
        || lowered.contains("sk-")
    {
        return Err("auditor_client_id cannot contain secret-like values".to_string());
    }
    Ok(normalized)
}

fn normalize_framework_review_report_assignments_request(
    payload: &mut ComplianceFrameworkReviewReportAssignmentsRequest,
) -> Result<Vec<String>, Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut payload.org_name);
    if let Err(error) =
        normalize_safe_framework_review_report_review_text(&mut payload.assignment_notes_safe)
    {
        errors.push(error);
    }

    let mut seen = HashSet::new();
    let mut auditor_client_ids = Vec::new();
    for value in &payload.auditor_client_ids {
        match normalize_auditor_client_id(value) {
            Ok(normalized) => {
                if seen.insert(normalized.clone()) {
                    auditor_client_ids.push(normalized);
                }
            }
            Err(error) => errors.push(error),
        }
    }

    if auditor_client_ids.len() > MAX_FRAMEWORK_REVIEW_REPORT_ASSIGNMENTS {
        errors.push(format!(
            "auditor_client_ids supports at most {MAX_FRAMEWORK_REVIEW_REPORT_ASSIGNMENTS} active assignments"
        ));
    }

    if errors.is_empty() {
        Ok(auditor_client_ids)
    } else {
        Err(errors)
    }
}

fn normalize_framework_review_report_comment_request(
    payload: &mut ComplianceFrameworkReviewReportCommentRequest,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut payload.org_name);
    payload.comment_body_safe = payload.comment_body_safe.trim().to_string();
    if payload.comment_body_safe.is_empty() {
        errors.push("comment_body_safe is required".to_string());
    }
    if payload.comment_body_safe.len() > MAX_FRAMEWORK_REVIEW_REPORT_COMMENT_LEN {
        errors.push(format!(
            "comment_body_safe must be {MAX_FRAMEWORK_REVIEW_REPORT_COMMENT_LEN} characters or less"
        ));
    }
    let lowered = payload.comment_body_safe.to_ascii_lowercase();
    if lowered.contains("<script")
        || lowered.contains("</")
        || lowered.contains("<iframe")
        || lowered.contains("bearer ")
        || lowered.contains("ghp_")
        || lowered.contains("glpat-")
        || lowered.contains("sk-")
    {
        errors.push("comment_body_safe must be plain text and cannot contain secrets".to_string());
    }

    payload.review_status_suggestion = payload
        .review_status_suggestion
        .take()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    if let Some(status) = payload.review_status_suggestion.as_deref() {
        if ![
            FRAMEWORK_REVIEW_REPORT_REVIEW_NEEDS_REVIEW,
            FRAMEWORK_REVIEW_REPORT_REVIEW_REVIEWED,
            FRAMEWORK_REVIEW_REPORT_REVIEW_NEEDS_CHANGES,
            FRAMEWORK_REVIEW_REPORT_REVIEW_REJECTED,
        ]
        .contains(&status)
        {
            errors.push(
                "review_status_suggestion must be needs_review, reviewed, needs_changes, or rejected."
                    .to_string(),
            );
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

async fn require_framework_review_report_collaboration_access(
    state: &Arc<AppState>,
    auth_user: &AuthUser,
    org_id: &str,
    report_id: &str,
) -> Option<axum::response::Response> {
    if auth_user.role == UserRole::Admin {
        return None;
    }

    let has_assignments = match state
        .db
        .compliance_framework_review_report_has_active_assignments(org_id, report_id)
        .await
    {
        Ok(value) => value,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to check framework review report assignments");
            return Some((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response());
        }
    };
    if !has_assignments {
        return None;
    }

    match state
        .db
        .compliance_framework_review_report_is_assigned_to(
            org_id,
            report_id,
            &auth_user.client_id,
        )
        .await
    {
        Ok(true) => None,
        Ok(false) => Some((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "Auditor is not assigned to this framework review report",
                "code": "auditor_not_assigned"
            })),
        )
            .into_response()),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, auditor_client_id = %auth_user.client_id, "Failed to check assigned auditor access");
            Some((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response())
        }
    }
}

pub async fn list_assigned_compliance_framework_review_reports(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(mut query): Query<ComplianceFrameworkReviewReportQuery>,
) -> impl IntoResponse {
    query.assigned_to_me = Some(true);
    list_compliance_framework_review_reports(
        Extension(auth_user),
        State(state),
        Query(query),
    )
    .await
}

pub async fn list_compliance_framework_review_report_assignments(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(report_id): Path<String>,
    Query(mut query): Query<ComplianceFrameworkReviewReportAssignmentQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let report_id = match normalize_framework_review_report_id(&report_id) {
        Ok(report_id) => report_id,
        Err(error) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response();
        }
    };
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_compliance_framework_review_report_org(
        &state,
        &auth_user,
        query.org_name.as_deref(),
    )
    .await
    {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    if let Some(resp) =
        require_framework_review_report_collaboration_access(&state, &auth_user, &org_id, &report_id)
            .await
    {
        return resp;
    }

    match state
        .db
        .list_compliance_framework_review_report_assignments(&org_id, &report_id)
        .await
    {
        Ok(assignments) => {
            let count = assignments.len();
            (
                StatusCode::OK,
                Json(ComplianceFrameworkReviewReportAssignmentsResponse { assignments, count }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to list framework review report assignments");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn upsert_compliance_framework_review_report_assignments(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(report_id): Path<String>,
    Json(mut payload): Json<ComplianceFrameworkReviewReportAssignmentsRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    let report_id = match normalize_framework_review_report_id(&report_id) {
        Ok(report_id) => report_id,
        Err(error) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response();
        }
    };
    let auditor_client_ids =
        match normalize_framework_review_report_assignments_request(&mut payload) {
            Ok(auditor_client_ids) => auditor_client_ids,
            Err(errors) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Invalid framework review report assignments request", "details": errors })),
                )
                    .into_response();
            }
        };

    let org_id = match resolve_compliance_framework_review_report_org(
        &state,
        &auth_user,
        payload.org_name.as_deref(),
    )
    .await
    {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    match state
        .db
        .get_compliance_framework_review_report(&org_id, &report_id)
        .await
    {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Compliance framework review report not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to load framework review report before assignment");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    }

    for auditor_client_id in &auditor_client_ids {
        match state
            .db
            .tenant_principal_is_auditor(&org_id, auditor_client_id)
            .await
        {
            Ok(true) => {}
            Ok(false) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Assigned principal must be an active tenant Auditor",
                        "auditor_client_id": auditor_client_id
                    })),
                )
                    .into_response();
            }
            Err(e) => {
                tracing::error!(error = %e, org_id = %org_id, auditor_client_id = %auditor_client_id, "Failed to validate Auditor principal");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Internal database error" })),
                )
                    .into_response();
            }
        }
    }

    match state
        .db
        .upsert_compliance_framework_review_report_assignments(
            &UpsertComplianceFrameworkReviewReportAssignmentsInput {
                org_id: &org_id,
                report_id: &report_id,
                auditor_client_ids: &auditor_client_ids,
                assigned_by_user_id: &auth_user.client_id,
                assignment_notes_safe: payload.assignment_notes_safe.as_deref(),
            },
        )
        .await
    {
        Ok(assignments) => {
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "compliance_framework_review_report.assignments_updated".to_string(),
                target_type: Some("compliance_framework_review_report".to_string()),
                target_id: Some(report_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "report_id": report_id,
                    "active_assignment_count": auditor_client_ids.len(),
                    "artifact_mutation": false,
                    "compliance_claim": false,
                    "regulatory_claim": false,
                    "certification": false
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write framework review report assignment audit log: {}", e);
            }
            let count = assignments.len();
            (
                StatusCode::OK,
                Json(ComplianceFrameworkReviewReportAssignmentsResponse { assignments, count }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to upsert framework review report assignments");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn list_compliance_framework_review_report_comments(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(report_id): Path<String>,
    Query(mut query): Query<ComplianceFrameworkReviewReportCommentsQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let report_id = match normalize_framework_review_report_id(&report_id) {
        Ok(report_id) => report_id,
        Err(error) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response();
        }
    };
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_compliance_framework_review_report_org(
        &state,
        &auth_user,
        query.org_name.as_deref(),
    )
    .await
    {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    if let Some(resp) =
        require_framework_review_report_collaboration_access(&state, &auth_user, &org_id, &report_id)
            .await
    {
        return resp;
    }

    match state
        .db
        .list_compliance_framework_review_report_comments(&org_id, &report_id)
        .await
    {
        Ok(comments) => {
            let count = comments.len();
            (
                StatusCode::OK,
                Json(ComplianceFrameworkReviewReportCommentsResponse { comments, count }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to list framework review report comments");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn create_compliance_framework_review_report_comment(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(report_id): Path<String>,
    Json(mut payload): Json<ComplianceFrameworkReviewReportCommentRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let report_id = match normalize_framework_review_report_id(&report_id) {
        Ok(report_id) => report_id,
        Err(error) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response();
        }
    };
    if let Err(errors) = normalize_framework_review_report_comment_request(&mut payload) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid framework review report comment request", "details": errors })),
        )
            .into_response();
    }

    let org_id = match resolve_compliance_framework_review_report_org(
        &state,
        &auth_user,
        payload.org_name.as_deref(),
    )
    .await
    {
        Ok(org_id) => org_id,
        Err(resp) => return resp,
    };

    match state
        .db
        .get_compliance_framework_review_report(&org_id, &report_id)
        .await
    {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Compliance framework review report not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to load framework review report before comment");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    }

    if let Some(resp) =
        require_framework_review_report_collaboration_access(&state, &auth_user, &org_id, &report_id)
            .await
    {
        return resp;
    }

    match state
        .db
        .create_compliance_framework_review_report_comment(
            &CreateComplianceFrameworkReviewReportCommentInput {
                org_id: &org_id,
                report_id: &report_id,
                commenter_client_id: &auth_user.client_id,
                comment_body_safe: &payload.comment_body_safe,
                review_status_suggestion: payload.review_status_suggestion.as_deref(),
            },
        )
        .await
    {
        Ok(comment) => {
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "compliance_framework_review_report.comment_created".to_string(),
                target_type: Some("compliance_framework_review_report".to_string()),
                target_id: Some(report_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "report_id": report_id,
                    "comment_id": comment.id,
                    "commenter_client_id": comment.commenter_client_id,
                    "review_status_suggestion": comment.review_status_suggestion,
                    "artifact_mutation": false,
                    "compliance_claim": false,
                    "regulatory_claim": false,
                    "certification": false
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write framework review report comment audit log: {}", e);
            }
            (StatusCode::CREATED, Json(comment)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to create framework review report comment");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
