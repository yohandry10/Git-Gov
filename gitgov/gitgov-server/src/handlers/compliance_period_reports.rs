const COMPLIANCE_PERIOD_REPORT_SCHEMA_VERSION: &str = "gitgov_period_compliance_report.v1";
const MAX_COMPLIANCE_PERIOD_REPORT_SOURCE_REPORTS: i64 = 250;
const MAX_COMPLIANCE_PERIOD_REPORT_RANGE_MS: i64 = 366 * 24 * 60 * 60 * 1000;

fn normalize_compliance_period_report_id(value: &str) -> Result<String, &'static str> {
    let normalized = value.trim().to_string();
    if normalized.starts_with("cpr_") && normalized.len() <= 80 {
        Ok(normalized)
    } else {
        Err("period_report_id must be a valid cpr_ identifier")
    }
}

fn normalize_compliance_period_report_framework_id(value: &mut Option<String>) {
    *value = value
        .take()
        .map(|candidate| candidate.trim().to_string())
        .filter(|candidate| !candidate.is_empty());
}

fn normalize_compliance_period_report_request(
    payload: &mut CompliancePeriodReportRequest,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut payload.org_name);
    normalize_compliance_period_report_framework_id(&mut payload.framework_id);
    payload.format = payload
        .format
        .take()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    if payload.format.as_deref().unwrap_or("json") != "json" {
        errors.push("format must be json".to_string());
    }
    if payload.date_range_start <= 0 || payload.date_range_end <= 0 {
        errors.push("date_range_start and date_range_end are required Unix millisecond timestamps".to_string());
    }
    if payload.date_range_end <= payload.date_range_start {
        errors.push("date_range_end must be after date_range_start".to_string());
    }
    if payload.date_range_end.saturating_sub(payload.date_range_start)
        > MAX_COMPLIANCE_PERIOD_REPORT_RANGE_MS
    {
        errors.push("date range must be 366 days or less".to_string());
    }
    if let Some(framework_id) = payload.framework_id.as_deref() {
        if framework_id.len() > 160
            || !framework_id
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':' | '/'))
        {
            errors.push("framework_id contains unsupported characters".to_string());
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn normalize_compliance_period_report_query(
    query: &mut CompliancePeriodReportQuery,
) -> Result<i64, Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut query.org_name);
    normalize_compliance_period_report_framework_id(&mut query.framework_id);
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    if let Some(framework_id) = query.framework_id.as_deref() {
        if framework_id.len() > 160
            || !framework_id
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':' | '/'))
        {
            errors.push("framework_id contains unsupported characters".to_string());
        }
    }

    if errors.is_empty() {
        Ok(limit)
    } else {
        Err(errors)
    }
}

fn period_report_datetime_from_millis(
    value: i64,
    field_name: &str,
) -> Result<chrono::DateTime<chrono::Utc>, String> {
    chrono::DateTime::<chrono::Utc>::from_timestamp_millis(value)
        .ok_or_else(|| format!("{field_name} is outside the supported timestamp range"))
}

fn compliance_period_report_response(
    period_report: CompliancePeriodReportRecord,
    artifact: Option<serde_json::Value>,
) -> CompliancePeriodReportResponse {
    let download_url = format!(
        "/compliance/period-reports/{}/download",
        period_report.period_report_id
    );
    CompliancePeriodReportResponse {
        period_report,
        download_url,
        artifact,
    }
}

fn period_report_auditor_filter(auth_user: &AuthUser) -> Option<&str> {
    if auth_user.role == UserRole::Admin {
        None
    } else {
        Some(auth_user.client_id.as_str())
    }
}

fn missing_evidence_from_report_payload(payload: &serde_json::Value) -> Vec<String> {
    payload
        .get("missing_evidence")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn period_report_summary_value(payload: &serde_json::Value, key: &str) -> i64 {
    payload
        .get("summary")
        .and_then(|summary| summary.get(key))
        .and_then(|value| value.as_i64())
        .unwrap_or(0)
}

fn sorted_unique(values: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut output = Vec::new();
    for value in values {
        if seen.insert(value.clone()) {
            output.push(value);
        }
    }
    output.sort();
    output
}

struct CompliancePeriodReportArtifactInput<'a> {
    period_report_id: &'a str,
    org_id: &'a str,
    created_by: &'a str,
    date_range_start: i64,
    date_range_end: i64,
    framework_id: Option<&'a str>,
    generated_at: i64,
    sources: &'a [CompliancePeriodSourceReport],
}

fn build_compliance_period_report_artifact(
    input: &CompliancePeriodReportArtifactInput<'_>,
) -> serde_json::Value {
    let mut total_controls = 0;
    let mut evidence_present = 0;
    let mut partial = 0;
    let mut missing = 0;
    let mut not_applicable = 0;
    let mut manual_review_required = 0;
    let mut missing_by_evidence: HashMap<String, Vec<String>> = HashMap::new();

    let mut report_hashes = Vec::new();
    let mut manifest_hashes = Vec::new();
    let mut evidence_export_hashes = Vec::new();
    let mut mapping_hashes = Vec::new();
    let mut review_package_hashes = Vec::new();

    let reports = input
        .sources
        .iter()
        .map(|source| {
            let report = &source.report;
            total_controls += period_report_summary_value(&source.payload_json_redacted, "total_controls");
            evidence_present += period_report_summary_value(&source.payload_json_redacted, "evidence_present");
            partial += period_report_summary_value(&source.payload_json_redacted, "partial");
            missing += period_report_summary_value(&source.payload_json_redacted, "missing");
            not_applicable += period_report_summary_value(&source.payload_json_redacted, "not_applicable");
            manual_review_required += period_report_summary_value(
                &source.payload_json_redacted,
                "manual_review_required",
            );

            for evidence in missing_evidence_from_report_payload(&source.payload_json_redacted) {
                missing_by_evidence
                    .entry(evidence)
                    .or_default()
                    .push(report.report_id.clone());
            }

            report_hashes.push(report.artifact_hash.clone());
            evidence_export_hashes.push(report.evidence_export_hash.clone());
            mapping_hashes.push(report.mapping_hash.clone());
            review_package_hashes.push(report.review_package_hash.clone());
            if let Some(manifest_hash) = source.latest_manifest_hash.as_ref() {
                manifest_hashes.push(manifest_hash.clone());
            }

            json!({
                "report_id": report.report_id,
                "framework_id": report.framework_id,
                "framework_version": report.framework_version,
                "framework_owner_type": report.framework_owner_type,
                "framework_review_status": report.framework_review_status,
                "pack_hash": report.pack_hash,
                "review_status": report.review_status,
                "reviewed_by_user_id": report.reviewed_by_user_id,
                "reviewed_at": report.reviewed_at,
                "created_at": report.created_at,
                "artifact_hash": report.artifact_hash,
                "evidence_export_id": report.evidence_export_id,
                "evidence_export_hash": report.evidence_export_hash,
                "mapping_id": report.mapping_id,
                "mapping_hash": report.mapping_hash,
                "review_package_id": report.review_package_id,
                "review_package_hash": report.review_package_hash,
                "latest_manifest_id": source.latest_manifest_id,
                "latest_manifest_hash": source.latest_manifest_hash,
                "latest_manifest_created_at": source.latest_manifest_created_at,
                "manifest_count": source.manifest_count,
                "missing_evidence": missing_evidence_from_report_payload(&source.payload_json_redacted),
                "claims": {
                    "compliance_claim": false,
                    "regulatory_claim": false,
                    "certification": false,
                    "requires_auditor_review": true
                }
            })
        })
        .collect::<Vec<_>>();

    let mut missing_evidence_summary = missing_by_evidence
        .into_iter()
        .map(|(evidence_type, mut report_ids)| {
            report_ids.sort();
            report_ids.dedup();
            json!({
                "evidence_type": evidence_type,
                "report_count": report_ids.len(),
                "report_ids": report_ids
            })
        })
        .collect::<Vec<_>>();
    missing_evidence_summary.sort_by(|left, right| {
        left.get("evidence_type")
            .and_then(|value| value.as_str())
            .cmp(&right.get("evidence_type").and_then(|value| value.as_str()))
    });

    json!({
        "schema_version": COMPLIANCE_PERIOD_REPORT_SCHEMA_VERSION,
        "period_report_id": input.period_report_id,
        "org_id": input.org_id,
        "created_by_user_id": input.created_by,
        "generated_at": input.generated_at,
        "period": {
            "date_range_start": input.date_range_start,
            "date_range_end": input.date_range_end,
            "inclusive_start": true,
            "exclusive_end": true
        },
        "filters": {
            "framework_id": input.framework_id
        },
        "positioning": {
            "manual_on_demand": true,
            "period_summary_only": true,
            "requires_human_auditor_review": true,
            "official_regulatory_mapping": false
        },
        "claims": {
            "compliance_claim": false,
            "regulatory_claim": false,
            "certification": false,
            "requires_auditor_review": true
        },
        "summary": {
            "report_count": input.sources.len(),
            "framework_count": sorted_unique(
                input.sources.iter().map(|source| source.report.framework_id.clone()).collect()
            ).len(),
            "reviewed_report_count": input.sources.len(),
            "reports_with_manifest_count": input.sources.iter().filter(|source| source.latest_manifest_hash.is_some()).count(),
            "reports_missing_manifest_count": input.sources.iter().filter(|source| source.latest_manifest_hash.is_none()).count(),
            "total_controls": total_controls,
            "evidence_present": evidence_present,
            "partial": partial,
            "missing": missing,
            "not_applicable": not_applicable,
            "manual_review_required": manual_review_required,
            "missing_evidence_type_count": missing_evidence_summary.len()
        },
        "missing_evidence_summary": missing_evidence_summary,
        "source_hashes": {
            "report_hashes": sorted_unique(report_hashes),
            "manifest_hashes": sorted_unique(manifest_hashes),
            "evidence_export_hashes": sorted_unique(evidence_export_hashes),
            "mapping_hashes": sorted_unique(mapping_hashes),
            "review_package_hashes": sorted_unique(review_package_hashes)
        },
        "reports": reports,
        "audit_metadata": {
            "artifact_redacted": true,
            "raw_payload_included": false,
            "agent_governance_required": false,
            "llm_decision": false,
            "policy_mutation": false,
            "provider_mutation": false,
            "gate_mutation": false,
            "report_scope": "period_compliance_report"
        }
    })
}

pub async fn create_compliance_period_report(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<CompliancePeriodReportRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    if let Err(errors) = normalize_compliance_period_report_request(&mut payload) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid compliance period report request", "details": errors })),
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
    let date_range_start =
        match period_report_datetime_from_millis(payload.date_range_start, "date_range_start") {
            Ok(value) => value,
            Err(error) => {
                return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response();
            }
        };
    let date_range_end =
        match period_report_datetime_from_millis(payload.date_range_end, "date_range_end") {
            Ok(value) => value,
            Err(error) => {
                return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response();
            }
        };

    let source_limit = MAX_COMPLIANCE_PERIOD_REPORT_SOURCE_REPORTS + 1;
    let sources = match state
        .db
        .list_reviewed_compliance_framework_review_reports_for_period(
            &org_id,
            date_range_start,
            date_range_end,
            payload.framework_id.as_deref(),
            source_limit,
        )
        .await
    {
        Ok(value) => value,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to load reviewed framework review reports for period report");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    if sources.is_empty() {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "No reviewed Framework Review Reports found for this period",
                "code": "period_report_no_reviewed_reports"
            })),
        )
            .into_response();
    }
    if sources.len() as i64 > MAX_COMPLIANCE_PERIOD_REPORT_SOURCE_REPORTS {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Period report source limit exceeded",
                "code": "period_report_source_limit_exceeded",
                "limit": MAX_COMPLIANCE_PERIOD_REPORT_SOURCE_REPORTS
            })),
        )
            .into_response();
    }

    let period_report_id = format!("cpr_{}", Uuid::new_v4().simple());
    let generated_at = chrono::Utc::now().timestamp_millis();
    let artifact =
        build_compliance_period_report_artifact(&CompliancePeriodReportArtifactInput {
            period_report_id: &period_report_id,
            org_id: &org_id,
            created_by: &auth_user.client_id,
            date_range_start: payload.date_range_start,
            date_range_end: payload.date_range_end,
            framework_id: payload.framework_id.as_deref(),
            generated_at,
            sources: &sources,
        });
    let artifact_hash = compliance_review_package_hash(&artifact);
    let source_report_ids = json!(
        sources
            .iter()
            .map(|source| source.report.report_id.clone())
            .collect::<Vec<_>>()
    );

    match state
        .db
        .create_compliance_period_report(&CreateCompliancePeriodReportInput {
            period_report_id: &period_report_id,
            org_id: &org_id,
            created_by_user_id: &auth_user.client_id,
            framework_id: payload.framework_id.as_deref(),
            date_range_start,
            date_range_end,
            report_count: sources.len() as i32,
            source_report_ids: &source_report_ids,
            format: "json",
            status: "generated",
            artifact_hash: &artifact_hash,
            payload_json_redacted: &artifact,
        })
        .await
    {
        Ok(period_report) => {
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "compliance_period_report.created".to_string(),
                target_type: Some("compliance_period_report".to_string()),
                target_id: Some(period_report.period_report_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "period_report_id": period_report.period_report_id,
                    "artifact_hash": period_report.artifact_hash,
                    "report_count": period_report.report_count,
                    "framework_id": period_report.framework_id,
                    "date_range_start": period_report.date_range_start,
                    "date_range_end": period_report.date_range_end,
                    "compliance_claim": false,
                    "regulatory_claim": false,
                    "requires_auditor_review": true,
                    "certification": false,
                    "agent_governance_required": false
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write period compliance report audit log: {}", e);
            }
            (
                StatusCode::CREATED,
                Json(compliance_period_report_response(period_report, Some(artifact))),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to create period compliance report");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn list_compliance_period_reports(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(mut query): Query<CompliancePeriodReportQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let limit = match normalize_compliance_period_report_query(&mut query) {
        Ok(value) => value,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid compliance period report query", "details": errors })),
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
        .list_compliance_period_reports(&ListCompliancePeriodReportsInput {
            org_id: &org_id,
            framework_id: query.framework_id.as_deref(),
            auditor_client_id: period_report_auditor_filter(&auth_user),
            limit,
        })
        .await
    {
        Ok(items) => {
            let count = items.len();
            (
                StatusCode::OK,
                Json(CompliancePeriodReportListResponse {
                    items,
                    count,
                    limit,
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to list period compliance reports");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn get_compliance_period_report(
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
        Ok(Some(period_report)) => {
            append_period_report_access_log(
                &state,
                PeriodReportAccessLogInput {
                    org_id: &org_id,
                    period_report_id: &period_report_id,
                    actor_client_id: &auth_user.client_id,
                    action: "viewed",
                    artifact_type: "metadata",
                    artifact_id: Some(&period_report.period_report_id),
                    artifact_hash: Some(&period_report.artifact_hash),
                    metadata: json!({
                    "retention_status": period_report.retention_status.clone(),
                    "retention_until": period_report.retention_until,
                    "download_count": period_report.download_count,
                    "compliance_claim": false,
                    "regulatory_claim": false,
                    "certification": false,
                    "agent_governance_required": false
                    }),
                },
            )
            .await;
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
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, "Failed to load period compliance report");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn download_compliance_period_report(
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
        .download_compliance_period_report(
            &org_id,
            &period_report_id,
            period_report_auditor_filter(&auth_user),
        )
        .await
    {
        Ok(Some((period_report, artifact))) => {
            append_period_report_access_log(
                &state,
                PeriodReportAccessLogInput {
                    org_id: &org_id,
                    period_report_id: &period_report_id,
                    actor_client_id: &auth_user.client_id,
                    action: "downloaded_json",
                    artifact_type: "json",
                    artifact_id: Some(&period_report.period_report_id),
                    artifact_hash: Some(&period_report.artifact_hash),
                    metadata: json!({
                    "download_count": period_report.download_count,
                    "last_downloaded_at": period_report.last_downloaded_at,
                    "retention_status": period_report.retention_status.clone(),
                    "retention_until": period_report.retention_until,
                    "compliance_claim": false,
                    "regulatory_claim": false,
                    "certification": false,
                    "agent_governance_required": false
                    }),
                },
            )
            .await;
            let mut headers = HeaderMap::new();
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static("application/json"),
            );
            headers.insert(
                axum::http::header::CONTENT_DISPOSITION,
                axum::http::HeaderValue::from_str(&format!(
                    "attachment; filename=\"gitgov-period-compliance-{}.json\"",
                    period_report.period_report_id
                ))
                .unwrap_or_else(|_| axum::http::HeaderValue::from_static("attachment")),
            );
            headers.insert(
                axum::http::HeaderName::from_static("x-gitgov-artifact-hash"),
                axum::http::HeaderValue::from_str(&period_report.artifact_hash)
                    .unwrap_or_else(|_| axum::http::HeaderValue::from_static("invalid")),
            );
            (headers, Json(artifact)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance period report not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, "Failed to download period compliance report");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

include!("compliance_period_reports/retention.rs");
include!("compliance_period_reports/pdf_exports.rs");
include!("compliance_period_reports/provenance_manifests.rs");
