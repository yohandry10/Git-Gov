fn normalize_compliance_period_report_access_log_query(
    query: &mut CompliancePeriodReportAccessLogQuery,
) -> Result<i64, Vec<String>> {
    normalize_release_approval_optional_text(&mut query.org_name);
    Ok(query.limit.unwrap_or(50).clamp(1, 200))
}

fn normalize_compliance_period_report_retention_request(
    payload: &mut CompliancePeriodReportRetentionRequest,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut payload.org_name);
    let retention_until = match payload.retention_until {
        Some(value) if value > 0 => match period_report_datetime_from_millis(value, "retention_until") {
            Ok(value) => Some(value),
            Err(error) => {
                errors.push(error);
                None
            }
        },
        Some(_) => {
            errors.push("retention_until must be a positive Unix millisecond timestamp".to_string());
            None
        }
        None => None,
    };
    if !payload.archive && retention_until.is_none() {
        errors.push("retention_until is required unless archive is true".to_string());
    }
    if errors.is_empty() {
        Ok(retention_until)
    } else {
        Err(errors)
    }
}

struct PeriodReportAccessLogInput<'a> {
    org_id: &'a str,
    period_report_id: &'a str,
    actor_client_id: &'a str,
    action: &'a str,
    artifact_type: &'a str,
    artifact_id: Option<&'a str>,
    artifact_hash: Option<&'a str>,
    metadata: serde_json::Value,
}

async fn append_period_report_access_log(
    state: &Arc<AppState>,
    input: PeriodReportAccessLogInput<'_>,
) {
    let access_log_id = format!("cprlog_{}", Uuid::new_v4().simple());
    if let Err(e) = state
        .db
        .create_compliance_period_report_access_log(
            &CreateCompliancePeriodReportAccessLogInput {
                access_log_id: &access_log_id,
                org_id: input.org_id,
                period_report_id: input.period_report_id,
                actor_client_id: input.actor_client_id,
                action: input.action,
                artifact_type: input.artifact_type,
                artifact_id: input.artifact_id,
                artifact_hash: input.artifact_hash,
                metadata: &input.metadata,
            },
        )
        .await
    {
        tracing::warn!(
            error = %e,
            org_id = %input.org_id,
            period_report_id = %input.period_report_id,
            action = %input.action,
            "Failed to append period compliance report access log"
        );
    }
}

pub async fn update_compliance_period_report_retention(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(period_report_id): Path<String>,
    Json(mut payload): Json<CompliancePeriodReportRetentionRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    let period_report_id = match normalize_compliance_period_report_id(&period_report_id) {
        Ok(value) => value,
        Err(error) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response();
        }
    };
    let retention_until = match normalize_compliance_period_report_retention_request(&mut payload) {
        Ok(value) => value,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid compliance period report retention request", "details": errors })),
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
        .update_compliance_period_report_retention(
            &UpdateCompliancePeriodReportRetentionInput {
                org_id: &org_id,
                period_report_id: &period_report_id,
                retention_until,
                archive: payload.archive,
            },
        )
        .await
    {
        Ok(Some(period_report)) => {
            let action = if payload.archive {
                "archived"
            } else {
                "retention_updated"
            };
            append_period_report_access_log(
                &state,
                PeriodReportAccessLogInput {
                    org_id: &org_id,
                    period_report_id: &period_report_id,
                    actor_client_id: &auth_user.client_id,
                    action,
                    artifact_type: "retention",
                    artifact_id: Some(&period_report.period_report_id),
                    artifact_hash: Some(&period_report.artifact_hash),
                    metadata: json!({
                        "retention_status": period_report.retention_status.clone(),
                        "retention_until": period_report.retention_until,
                        "archived_at": period_report.archived_at,
                        "download_count": period_report.download_count,
                        "physical_delete": false,
                        "compliance_claim": false,
                        "regulatory_claim": false,
                        "certification": false,
                        "agent_governance_required": false
                    }),
                },
            )
            .await;
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: format!("compliance_period_report.{action}"),
                target_type: Some("compliance_period_report".to_string()),
                target_id: Some(period_report.period_report_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "period_report_id": period_report.period_report_id.clone(),
                    "retention_status": period_report.retention_status.clone(),
                    "retention_until": period_report.retention_until,
                    "archived_at": period_report.archived_at,
                    "physical_delete": false,
                    "compliance_claim": false,
                    "regulatory_claim": false,
                    "certification": false,
                    "agent_governance_required": false
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write period compliance report retention audit log: {}", e);
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
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, "Failed to update period compliance report retention");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn list_compliance_period_report_access_log(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(period_report_id): Path<String>,
    Query(mut query): Query<CompliancePeriodReportAccessLogQuery>,
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
    let limit = match normalize_compliance_period_report_access_log_query(&mut query) {
        Ok(value) => value,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid compliance period report access log query", "details": errors })),
            )
                .into_response();
        }
    };
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
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Compliance period report not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, "Failed to authorize period compliance report access log");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    }

    match state
        .db
        .list_compliance_period_report_access_logs(&org_id, &period_report_id, limit)
        .await
    {
        Ok(items) => {
            let count = items.len();
            (
                StatusCode::OK,
                Json(CompliancePeriodReportAccessLogResponse {
                    items,
                    count,
                    limit,
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, "Failed to list period compliance report access log");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
