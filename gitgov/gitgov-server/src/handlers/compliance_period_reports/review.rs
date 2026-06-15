const COMPLIANCE_PERIOD_REPORT_REVIEW_NEEDS_REVIEW: &str = "needs_review";
const COMPLIANCE_PERIOD_REPORT_REVIEW_REVIEWED: &str = "reviewed";
const COMPLIANCE_PERIOD_REPORT_REVIEW_NEEDS_CHANGES: &str = "needs_changes";
const COMPLIANCE_PERIOD_REPORT_REVIEW_REJECTED: &str = "rejected";
const MAX_COMPLIANCE_PERIOD_REPORT_REVIEW_NOTE_LEN: usize = 1000;

fn normalize_safe_compliance_period_report_review_text(
    value: &mut Option<String>,
) -> Result<(), String> {
    let Some(text) = value.take() else {
        return Ok(());
    };
    let normalized = text.trim().to_string();
    if normalized.is_empty() {
        *value = None;
        return Ok(());
    }
    if normalized.len() > MAX_COMPLIANCE_PERIOD_REPORT_REVIEW_NOTE_LEN {
        return Err(format!(
            "review notes must be {MAX_COMPLIANCE_PERIOD_REPORT_REVIEW_NOTE_LEN} characters or less"
        ));
    }
    let lowered = normalized.to_ascii_lowercase();
    if lowered.contains("<script")
        || lowered.contains("</")
        || lowered.contains("<iframe")
        || lowered.contains("bearer ")
        || lowered.contains("ghp_")
        || lowered.contains("glpat-")
        || lowered.contains("sk-")
    {
        return Err("review notes must be plain text and cannot contain secrets".to_string());
    }
    *value = Some(normalized);
    Ok(())
}

fn normalize_compliance_period_report_review_request(
    payload: &mut CompliancePeriodReportReviewRequest,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut payload.org_name);
    payload.review_status = payload.review_status.trim().to_ascii_lowercase();
    if ![
        COMPLIANCE_PERIOD_REPORT_REVIEW_NEEDS_REVIEW,
        COMPLIANCE_PERIOD_REPORT_REVIEW_REVIEWED,
        COMPLIANCE_PERIOD_REPORT_REVIEW_NEEDS_CHANGES,
        COMPLIANCE_PERIOD_REPORT_REVIEW_REJECTED,
    ]
    .contains(&payload.review_status.as_str())
    {
        errors.push(
            "review_status must be needs_review, reviewed, needs_changes, or rejected."
                .to_string(),
        );
    }
    if let Err(error) =
        normalize_safe_compliance_period_report_review_text(&mut payload.review_notes_safe)
    {
        errors.push(error);
    }
    if payload.review_status == COMPLIANCE_PERIOD_REPORT_REVIEW_NEEDS_CHANGES
        && payload.review_notes_safe.is_none()
    {
        errors.push("review_notes_safe is required when review_status is needs_changes.".to_string());
    }
    if payload.review_status == COMPLIANCE_PERIOD_REPORT_REVIEW_REJECTED
        && payload.review_notes_safe.is_none()
    {
        errors.push("review_notes_safe is required when review_status is rejected.".to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub async fn get_compliance_period_report_review(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(period_report_id): Path<String>,
    Query(mut query): Query<CompliancePeriodReportQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let period_report_id = match normalize_compliance_period_report_id(&period_report_id) {
        Ok(value) => value,
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

    match state
        .db
        .get_compliance_period_report(
            &org_id,
            &period_report_id,
            period_report_auditor_filter(&auth_user),
        )
        .await
    {
        Ok(Some(period_report)) => (
            StatusCode::OK,
            Json(compliance_period_report_response(period_report, None)),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance period report not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, "Failed to load period compliance report review");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn review_compliance_period_report(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(period_report_id): Path<String>,
    Json(mut payload): Json<CompliancePeriodReportReviewRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let period_report_id = match normalize_compliance_period_report_id(&period_report_id) {
        Ok(value) => value,
        Err(error) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response();
        }
    };
    if let Err(errors) = normalize_compliance_period_report_review_request(&mut payload) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid compliance period report review request", "details": errors })),
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

    let current_report = match state
        .db
        .get_compliance_period_report(
            &org_id,
            &period_report_id,
            period_report_auditor_filter(&auth_user),
        )
        .await
    {
        Ok(Some(period_report)) => period_report,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Compliance period report not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, "Failed to authorize period compliance report review");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    if current_report.retention_status == "archived" {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Archived Period Compliance Reports cannot be reviewed",
                "code": "period_report_archived"
            })),
        )
            .into_response();
    }

    match state
        .db
        .update_compliance_period_report_review(
            &UpdateCompliancePeriodReportReviewInput {
                org_id: &org_id,
                period_report_id: &period_report_id,
                review_status: &payload.review_status,
                reviewed_by_user_id: &auth_user.client_id,
                review_notes_safe: payload.review_notes_safe.as_deref(),
            },
        )
        .await
    {
        Ok(Some(period_report)) => {
            append_period_report_access_log(
                &state,
                PeriodReportAccessLogInput {
                    org_id: &org_id,
                    period_report_id: &period_report_id,
                    actor_client_id: &auth_user.client_id,
                    action: "review_updated",
                    artifact_type: "review",
                    artifact_id: Some(&period_report.period_report_id),
                    artifact_hash: Some(&period_report.artifact_hash),
                    metadata: json!({
                        "previous_review_status": current_report.review_status,
                        "review_status": period_report.review_status,
                        "reviewed_by_user_id": period_report.reviewed_by_user_id,
                        "reviewed_at": period_report.reviewed_at,
                        "has_review_notes_safe": period_report.review_notes_safe.is_some(),
                        "hash_changed": false,
                        "source_period_report_artifact_mutated": false,
                        "compliance_claim": false,
                        "regulatory_claim": false,
                        "certification": false,
                        "legal_attestation": false,
                        "agent_governance_required": false
                    }),
                },
            )
            .await;
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "compliance_period_report.reviewed".to_string(),
                target_type: Some("compliance_period_report".to_string()),
                target_id: Some(period_report.period_report_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "period_report_id": period_report.period_report_id,
                    "previous_review_status": current_report.review_status,
                    "review_status": period_report.review_status,
                    "reviewed_by_user_id": period_report.reviewed_by_user_id,
                    "reviewed_at": period_report.reviewed_at,
                    "has_review_notes_safe": period_report.review_notes_safe.is_some(),
                    "artifact_hash": period_report.artifact_hash,
                    "hash_changed": false,
                    "source_period_report_artifact_mutated": false,
                    "compliance_claim": false,
                    "regulatory_claim": false,
                    "requires_auditor_review": period_report.requires_auditor_review,
                    "certification": false,
                    "legal_attestation": false,
                    "agent_governance_required": false
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write period compliance report review audit log: {}", e);
            }
            (
                StatusCode::OK,
                Json(compliance_period_report_response(period_report, None)),
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance period report not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, "Failed to review period compliance report");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
