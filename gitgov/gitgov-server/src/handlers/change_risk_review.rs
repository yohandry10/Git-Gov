// ============================================================================
// CHANGE RISK MANUAL REVIEW
// ============================================================================

const CHANGE_RISK_REVIEW_NOTE_MAX_CHARS: usize = 1000;
const CHANGE_RISK_REVIEW_NEEDS_REVIEW: &str = "needs_review";
const CHANGE_RISK_REVIEW_REVIEWED: &str = "reviewed";
const CHANGE_RISK_REVIEW_ACCEPTED_RISK: &str = "accepted_risk";
const CHANGE_RISK_REVIEW_NEEDS_MITIGATION: &str = "needs_mitigation";
const CHANGE_RISK_REVIEW_REJECTED: &str = "rejected";

fn normalize_safe_change_risk_review_text(
    value: &mut Option<String>,
    field: &str,
) -> Result<(), String> {
    let Some(text) = value.take() else {
        return Ok(());
    };
    let normalized = text.trim().to_string();
    if normalized.is_empty() {
        *value = None;
        return Ok(());
    }
    if normalized.len() > CHANGE_RISK_REVIEW_NOTE_MAX_CHARS {
        return Err(format!(
            "{field} must be {CHANGE_RISK_REVIEW_NOTE_MAX_CHARS} characters or less"
        ));
    }
    let lowered = normalized.to_ascii_lowercase();
    if lowered.contains("<script")
        || lowered.contains("</")
        || lowered.contains("<iframe")
        || lowered.contains("bearer ")
        || lowered.contains("authorization:")
        || lowered.contains("api_key")
        || lowered.contains("apikey")
        || lowered.contains("secret")
        || lowered.contains("password")
        || lowered.contains("ghp_")
        || lowered.contains("github_pat_")
        || lowered.contains("glpat-")
        || lowered.contains("sk-")
    {
        return Err(format!("{field} must be plain text and cannot contain secrets"));
    }
    *value = Some(normalized);
    Ok(())
}

fn normalize_change_risk_review_request(
    payload: &mut ChangeRiskEvaluationReviewRequest,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut payload.org_name);
    payload.review_status = payload.review_status.trim().to_ascii_lowercase();
    if ![
        CHANGE_RISK_REVIEW_NEEDS_REVIEW,
        CHANGE_RISK_REVIEW_REVIEWED,
        CHANGE_RISK_REVIEW_ACCEPTED_RISK,
        CHANGE_RISK_REVIEW_NEEDS_MITIGATION,
        CHANGE_RISK_REVIEW_REJECTED,
    ]
    .contains(&payload.review_status.as_str())
    {
        errors.push(
            "review_status must be needs_review, reviewed, accepted_risk, needs_mitigation, or rejected."
                .to_string(),
        );
    }
    if let Err(error) =
        normalize_safe_change_risk_review_text(&mut payload.review_notes, "review_notes")
    {
        errors.push(error);
    }
    if let Err(error) =
        normalize_safe_change_risk_review_text(&mut payload.mitigation_notes, "mitigation_notes")
    {
        errors.push(error);
    }
    if let Err(error) =
        normalize_safe_change_risk_review_text(&mut payload.decision_reason, "decision_reason")
    {
        errors.push(error);
    }
    if matches!(
        payload.review_status.as_str(),
        CHANGE_RISK_REVIEW_ACCEPTED_RISK | CHANGE_RISK_REVIEW_REJECTED
    ) && payload.decision_reason.is_none()
    {
        errors.push(
            "decision_reason is required when review_status is accepted_risk or rejected."
                .to_string(),
        );
    }
    if payload.review_status == CHANGE_RISK_REVIEW_NEEDS_MITIGATION
        && payload.mitigation_notes.is_none()
    {
        errors.push(
            "mitigation_notes is required when review_status is needs_mitigation.".to_string(),
        );
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn change_risk_review_response_from_record(
    record: ChangeRiskEvaluationRecord,
) -> ChangeRiskEvaluationReviewResponse {
    ChangeRiskEvaluationReviewResponse {
        evaluation_id: record.evaluation_id,
        org_id: record.org_id,
        risk_level: record.risk_level,
        ruleset_version: record.ruleset_version,
        trace_hash: record.trace_hash,
        review_status: record.review_status,
        reviewed_by_user_id: record.reviewed_by_user_id,
        reviewed_at: record.reviewed_at,
        review_notes_safe: record.review_notes_safe,
        mitigation_notes_safe: record.mitigation_notes_safe,
        decision_reason_safe: record.decision_reason_safe,
        review_updated_at: record.review_updated_at,
        advisory_only: record.advisory_only,
        llm_used: record.llm_used,
        agent_governance_used: record.agent_governance_used,
        compliance_claim: record.compliance_claim,
        certification: record.certification,
    }
}

pub async fn get_change_risk_evaluation_review(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(evaluation_id): Path<String>,
    Query(mut query): Query<ChangeRiskEvaluationQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_change_risk_org(&state, &auth_user, query.org_name.as_deref()).await
    {
        Ok(org_id) => org_id,
        Err(response) => return response.into_response(),
    };

    match state
        .db
        .get_change_risk_evaluation(&org_id, evaluation_id.trim())
        .await
    {
        Ok(Some(record)) => (
            StatusCode::OK,
            Json(change_risk_review_response_from_record(record)),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Change risk evaluation not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to get change risk evaluation review");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn update_change_risk_evaluation_review(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(evaluation_id): Path<String>,
    Json(mut payload): Json<ChangeRiskEvaluationReviewRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    if !evaluation_id.starts_with("cra_") || evaluation_id.len() > 80 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "evaluation_id must be a valid cra_ identifier" })),
        )
            .into_response();
    }
    if let Err(errors) = normalize_change_risk_review_request(&mut payload) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid change risk review request", "details": errors })),
        )
            .into_response();
    }

    let org_id = match resolve_change_risk_org(&state, &auth_user, payload.org_name.as_deref()).await
    {
        Ok(org_id) => org_id,
        Err(response) => return response.into_response(),
    };

    match state
        .db
        .update_change_risk_evaluation_review(&UpdateChangeRiskEvaluationReviewInput {
            org_id: &org_id,
            evaluation_id: evaluation_id.trim(),
            review_status: &payload.review_status,
            reviewed_by_user_id: &auth_user.client_id,
            review_notes_safe: payload.review_notes.as_deref(),
            mitigation_notes_safe: payload.mitigation_notes.as_deref(),
            decision_reason_safe: payload.decision_reason.as_deref(),
        })
        .await
    {
        Ok(Some(record)) => {
            let audit = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "change_risk_review_updated".to_string(),
                target_type: Some("change_risk_evaluation".to_string()),
                target_id: Some(record.evaluation_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "evaluation_id": record.evaluation_id,
                    "risk_level": record.risk_level,
                    "ruleset_version": record.ruleset_version,
                    "trace_hash": record.trace_hash,
                    "review_status": record.review_status,
                    "reviewed_by_user_id": record.reviewed_by_user_id,
                    "reviewed_at": record.reviewed_at,
                    "has_review_notes_safe": record.review_notes_safe.is_some(),
                    "has_mitigation_notes_safe": record.mitigation_notes_safe.is_some(),
                    "has_decision_reason_safe": record.decision_reason_safe.is_some(),
                    "trace_changed": false,
                    "advisory_only": record.advisory_only,
                    "llm_used": record.llm_used,
                    "agent_governance_used": record.agent_governance_used,
                    "compliance_claim": record.compliance_claim,
                    "certification": record.certification
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
                tracing::warn!(error = %e, "Failed to write admin audit log (change risk review)");
            }
            (
                StatusCode::OK,
                Json(change_risk_review_response_from_record(record)),
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Change risk evaluation not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to update change risk evaluation review");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
