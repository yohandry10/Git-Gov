const COMPLIANCE_PERIOD_REPORT_PROFILE_FILTER_MAX_CHARS: usize = 4000;
const DEFAULT_COMPLIANCE_PERIOD_REPORT_PROFILE_RETENTION_DAYS: i32 = 2555;

fn normalize_compliance_period_report_profile_id(value: &str) -> Result<String, &'static str> {
    let normalized = value.trim().to_string();
    if normalized.starts_with("cprprof_") && normalized.len() <= 80 {
        Ok(normalized)
    } else {
        Err("profile_id must be a valid cprprof_ identifier")
    }
}

fn normalize_compliance_period_report_profile_name(
    value: Option<String>,
) -> Result<String, String> {
    let name = value
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if name.is_empty() {
        return Err("name is required".to_string());
    }
    if name.len() > 120 {
        return Err("name must be 120 characters or less".to_string());
    }
    Ok(name)
}

fn normalize_compliance_period_report_profile_period_type(
    value: Option<String>,
) -> Result<String, String> {
    let period_type = value
        .unwrap_or_else(|| "monthly".to_string())
        .trim()
        .to_ascii_lowercase();
    if matches!(period_type.as_str(), "monthly" | "quarterly" | "annual" | "custom") {
        Ok(period_type)
    } else {
        Err("period_type must be monthly, quarterly, annual, or custom".to_string())
    }
}

fn normalize_compliance_period_report_profile_status(
    status: &mut Option<String>,
) -> Result<Option<String>, Vec<String>> {
    let normalized = status
        .take()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    match normalized.as_deref() {
        Some("active" | "archived") | None => Ok(normalized),
        _ => Err(vec![
            "status must be active or archived when provided".to_string(),
        ]),
    }
}

fn normalize_compliance_period_report_profile_owner_type(
    value: Option<String>,
) -> Result<Option<String>, String> {
    let owner_type = value
        .map(|candidate| candidate.trim().to_ascii_lowercase())
        .filter(|candidate| !candidate.is_empty());
    match owner_type.as_deref() {
        Some("gitgov_managed" | "customer_provided") | None => Ok(owner_type),
        _ => Err("framework_owner_type must be gitgov_managed or customer_provided".to_string()),
    }
}

fn safe_compliance_period_report_profile_filters(value: &serde_json::Value) -> bool {
    let rendered = value.to_string();
    rendered.len() <= COMPLIANCE_PERIOD_REPORT_PROFILE_FILTER_MAX_CHARS
        && !rendered.to_ascii_lowercase().contains("<script")
        && !rendered.to_ascii_lowercase().contains("token")
        && !rendered.to_ascii_lowercase().contains("secret")
        && !rendered.to_ascii_lowercase().contains("password")
}

fn normalize_compliance_period_report_profile_filters(
    value: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let filters = value.unwrap_or_else(|| json!({}));
    if !filters.is_object() {
        return Err("filters must be a JSON object".to_string());
    }
    if !safe_compliance_period_report_profile_filters(&filters) {
        return Err("filters must be safe JSON metadata without secrets or HTML".to_string());
    }
    Ok(filters)
}

fn normalize_compliance_period_report_profile_retention_days(
    value: Option<i32>,
) -> Result<i32, String> {
    let retention_days = value.unwrap_or(DEFAULT_COMPLIANCE_PERIOD_REPORT_PROFILE_RETENTION_DAYS);
    if !(30..=3650).contains(&retention_days) {
        return Err("retention_days must be between 30 and 3650".to_string());
    }
    Ok(retention_days)
}

struct NormalizedCompliancePeriodReportProfile {
    name: String,
    period_type: String,
    framework_id: Option<String>,
    framework_owner_type: Option<String>,
    include_pdf: bool,
    include_manifest: bool,
    retention_days: i32,
    filters: serde_json::Value,
}

fn normalize_compliance_period_report_profile_request(
    payload: &mut CompliancePeriodReportProfileRequest,
) -> Result<NormalizedCompliancePeriodReportProfile, Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut payload.org_name);
    normalize_compliance_period_report_framework_id(&mut payload.framework_id);

    let name = match normalize_compliance_period_report_profile_name(Some(std::mem::take(
        &mut payload.name,
    ))) {
        Ok(value) => Some(value),
        Err(error) => {
            errors.push(error);
            None
        }
    };
    let period_type =
        match normalize_compliance_period_report_profile_period_type(Some(std::mem::take(
            &mut payload.period_type,
        ))) {
            Ok(value) => Some(value),
            Err(error) => {
                errors.push(error);
                None
            }
        };
    let framework_owner_type =
        match normalize_compliance_period_report_profile_owner_type(payload.framework_owner_type.take())
        {
            Ok(value) => Some(value),
            Err(error) => {
                errors.push(error);
                None
            }
        };
    let retention_days =
        match normalize_compliance_period_report_profile_retention_days(payload.retention_days) {
            Ok(value) => Some(value),
            Err(error) => {
                errors.push(error);
                None
            }
        };
    let filters = match normalize_compliance_period_report_profile_filters(payload.filters.take()) {
        Ok(value) => Some(value),
        Err(error) => {
            errors.push(error);
            None
        }
    };
    if let Some(framework_id) = payload.framework_id.as_deref() {
        if framework_id.len() > 160
            || !framework_id
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':' | '/'))
        {
            errors.push("framework_id contains unsupported characters".to_string());
        }
    }

    match (name, period_type, framework_owner_type, retention_days, filters) {
        (Some(name), Some(period_type), Some(framework_owner_type), Some(retention_days), Some(filters))
            if errors.is_empty() =>
        {
            Ok(NormalizedCompliancePeriodReportProfile {
                name,
                period_type,
                framework_id: payload.framework_id.clone(),
                framework_owner_type,
                include_pdf: payload.include_pdf.unwrap_or(true),
                include_manifest: payload.include_manifest.unwrap_or(true),
                retention_days,
                filters,
            })
        }
        _ => Err(errors),
    }
}

fn normalize_compliance_period_report_profile_query(
    query: &mut CompliancePeriodReportProfileQuery,
) -> Result<i64, Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut query.org_name);
    normalize_compliance_period_report_framework_id(&mut query.framework_id);
    let status = normalize_compliance_period_report_profile_status(&mut query.status);
    match status {
        Ok(normalized_status) => {
            query.status = normalized_status;
        }
        Err(mut status_errors) => {
            errors.append(&mut status_errors);
        }
    }
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
        Ok(query.limit.unwrap_or(25).clamp(1, 100))
    } else {
        Err(errors)
    }
}

fn merged_compliance_period_report_profile_patch(
    existing: &CompliancePeriodReportProfileRecord,
    payload: &mut CompliancePeriodReportProfilePatchRequest,
) -> Result<NormalizedCompliancePeriodReportProfile, Vec<String>> {
    let mut request = CompliancePeriodReportProfileRequest {
        org_name: payload.org_name.take(),
        name: payload.name.take().unwrap_or_else(|| existing.name.clone()),
        period_type: payload
            .period_type
            .take()
            .unwrap_or_else(|| existing.period_type.clone()),
        framework_id: payload
            .framework_id
            .take()
            .or_else(|| existing.framework_id.clone()),
        framework_owner_type: payload
            .framework_owner_type
            .take()
            .or_else(|| existing.framework_owner_type.clone()),
        include_pdf: payload.include_pdf.or(Some(existing.include_pdf)),
        include_manifest: payload.include_manifest.or(Some(existing.include_manifest)),
        retention_days: payload.retention_days.or(Some(existing.retention_days)),
        filters: payload.filters.take().or_else(|| Some(existing.filters.clone())),
    };
    normalize_compliance_period_report_profile_request(&mut request)
}

fn compliance_period_report_profile_response(
    profile: CompliancePeriodReportProfileRecord,
) -> CompliancePeriodReportProfileResponse {
    CompliancePeriodReportProfileResponse { profile }
}

fn utc_midnight_millis(year: i32, month: u32, day: u32) -> Result<i64, String> {
    let naive = chrono::NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .ok_or_else(|| "period boundary is outside the supported date range".to_string())?;
    Ok(chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(naive, chrono::Utc)
        .timestamp_millis())
}

fn derive_profile_run_period(
    profile: &CompliancePeriodReportProfileRecord,
    payload: &CompliancePeriodReportProfileRunRequest,
) -> Result<(i64, i64), Vec<String>> {
    let mut errors = Vec::new();
    if let (Some(start), Some(end)) = (payload.date_range_start, payload.date_range_end) {
        if start <= 0 || end <= 0 {
            errors.push("date_range_start and date_range_end must be positive Unix millisecond timestamps".to_string());
        } else if end <= start {
            errors.push("date_range_end must be after date_range_start".to_string());
        } else if end.saturating_sub(start) > MAX_COMPLIANCE_PERIOD_REPORT_RANGE_MS {
            errors.push("date range must be 366 days or less".to_string());
        }
        return if errors.is_empty() {
            Ok((start, end))
        } else {
            Err(errors)
        };
    }
    if payload.date_range_start.is_some() || payload.date_range_end.is_some() {
        return Err(vec![
            "date_range_start and date_range_end must be provided together".to_string(),
        ]);
    }
    if profile.period_type == "custom" {
        return Err(vec![
            "custom profiles require date_range_start and date_range_end for each run".to_string(),
        ]);
    }

    let now = chrono::Utc::now();
    let (start_year, start_month, end_year, end_month) = match profile.period_type.as_str() {
        "monthly" => {
            let end_month = if now.month() == 12 { 1 } else { now.month() + 1 };
            let end_year = if now.month() == 12 { now.year() + 1 } else { now.year() };
            (now.year(), now.month(), end_year, end_month)
        }
        "quarterly" => {
            let quarter_start_month = ((now.month() - 1) / 3) * 3 + 1;
            let next_quarter_month = quarter_start_month + 3;
            let (end_year, end_month) = if next_quarter_month > 12 {
                (now.year() + 1, next_quarter_month - 12)
            } else {
                (now.year(), next_quarter_month)
            };
            (now.year(), quarter_start_month, end_year, end_month)
        }
        "annual" => (now.year(), 1, now.year() + 1, 1),
        _ => {
            return Err(vec![
                "period_type must be monthly, quarterly, annual, or custom".to_string(),
            ]);
        }
    };
    let start = match utc_midnight_millis(start_year, start_month, 1) {
        Ok(value) => value,
        Err(error) => {
            errors.push(error);
            0
        }
    };
    let end = match utc_midnight_millis(end_year, end_month, 1) {
        Ok(value) => value,
        Err(error) => {
            errors.push(error);
            0
        }
    };
    if errors.is_empty() {
        Ok((start, end))
    } else {
        Err(errors)
    }
}

async fn create_period_report_for_profile_run(
    state: &Arc<AppState>,
    auth_user: &AuthUser,
    org_id: &str,
    profile: &CompliancePeriodReportProfileRecord,
    date_range_start: i64,
    date_range_end: i64,
) -> Result<(CompliancePeriodReportRecord, serde_json::Value), (StatusCode, serde_json::Value)> {
    let start = period_report_datetime_from_millis(date_range_start, "date_range_start")
        .map_err(|error| (StatusCode::BAD_REQUEST, json!({ "error": error })))?;
    let end = period_report_datetime_from_millis(date_range_end, "date_range_end")
        .map_err(|error| (StatusCode::BAD_REQUEST, json!({ "error": error })))?;

    let sources = state
        .db
        .list_reviewed_compliance_framework_review_reports_for_period(
            org_id,
            start,
            end,
            profile.framework_id.as_deref(),
            MAX_COMPLIANCE_PERIOD_REPORT_SOURCE_REPORTS + 1,
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, org_id = %org_id, profile_id = %profile.profile_id, "Failed to load reviewed reports for profile run");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Internal database error" }),
            )
        })?;
    if sources.is_empty() {
        return Err((
            StatusCode::CONFLICT,
            json!({
                "error": "No reviewed Framework Review Reports found for this profile run",
                "code": "period_report_profile_no_reviewed_reports"
            }),
        ));
    }
    if sources.len() as i64 > MAX_COMPLIANCE_PERIOD_REPORT_SOURCE_REPORTS {
        return Err((
            StatusCode::CONFLICT,
            json!({
                "error": "Period report source limit exceeded",
                "code": "period_report_source_limit_exceeded",
                "limit": MAX_COMPLIANCE_PERIOD_REPORT_SOURCE_REPORTS
            }),
        ));
    }

    let period_report_id = format!("cpr_{}", Uuid::new_v4().simple());
    let generated_at = chrono::Utc::now().timestamp_millis();
    let artifact = build_compliance_period_report_artifact(&CompliancePeriodReportArtifactInput {
        period_report_id: &period_report_id,
        org_id,
        created_by: &auth_user.client_id,
        date_range_start,
        date_range_end,
        framework_id: profile.framework_id.as_deref(),
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

    let mut period_report = state
        .db
        .create_compliance_period_report(&CreateCompliancePeriodReportInput {
            period_report_id: &period_report_id,
            org_id,
            created_by_user_id: &auth_user.client_id,
            framework_id: profile.framework_id.as_deref(),
            date_range_start: start,
            date_range_end: end,
            report_count: sources.len() as i32,
            source_report_ids: &source_report_ids,
            format: "json",
            status: "generated",
            artifact_hash: &artifact_hash,
            payload_json_redacted: &artifact,
        })
        .await
        .map_err(|e| {
            tracing::error!(error = %e, org_id = %org_id, profile_id = %profile.profile_id, "Failed to create period report for profile run");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Internal database error" }),
            )
        })?;

    let retention_until = chrono::Utc::now() + chrono::Duration::days(profile.retention_days as i64);
    if let Some(updated) = state
        .db
        .update_compliance_period_report_retention(&UpdateCompliancePeriodReportRetentionInput {
            org_id,
            period_report_id: &period_report.period_report_id,
            retention_until: Some(retention_until),
            archive: false,
        })
        .await
        .map_err(|e| {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report.period_report_id, "Failed to apply profile retention to period report");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Internal database error" }),
            )
        })?
    {
        period_report = updated;
    }

    append_period_report_access_log(
        state,
        PeriodReportAccessLogInput {
            org_id,
            period_report_id: &period_report.period_report_id,
            actor_client_id: &auth_user.client_id,
            action: "retention_updated",
            artifact_type: "retention",
            artifact_id: Some(&period_report.period_report_id),
            artifact_hash: Some(&period_report.artifact_hash),
            metadata: json!({
                "source": "period_report_profile_run",
                "profile_id": profile.profile_id,
                "retention_days": profile.retention_days,
                "retention_status": period_report.retention_status,
                "retention_until": period_report.retention_until,
                "manual_run": true,
                "scheduled_run": false,
                "compliance_claim": false,
                "regulatory_claim": false,
                "certification": false,
                "agent_governance_required": false
            }),
        },
    )
    .await;

    Ok((period_report, artifact))
}

async fn create_profile_pdf_export(
    state: &Arc<AppState>,
    auth_user: &AuthUser,
    org_id: &str,
    period_report: &CompliancePeriodReportRecord,
    artifact: &serde_json::Value,
) -> Result<CompliancePeriodReportPdfExportRecord, (StatusCode, serde_json::Value)> {
    let generated_at = chrono::Utc::now().timestamp_millis();
    let lines =
        collect_compliance_period_report_pdf_lines(period_report, artifact, &auth_user.client_id, generated_at);
    let (pdf_bytes, page_count) = build_framework_review_report_pdf(&lines);
    let pdf_artifact_hash = framework_review_report_pdf_hash(&pdf_bytes);
    let pdf_export_id = deterministic_compliance_period_report_pdf_export_id(&pdf_artifact_hash);
    state
        .db
        .create_compliance_period_report_pdf_export(&CreateCompliancePeriodReportPdfExportInput {
            pdf_export_id: &pdf_export_id,
            org_id,
            period_report_id: &period_report.period_report_id,
            created_by_user_id: &auth_user.client_id,
            source_period_report_hash: &period_report.artifact_hash,
            pdf_artifact_hash: &pdf_artifact_hash,
            content_type: COMPLIANCE_PERIOD_REPORT_PDF_CONTENT_TYPE,
            page_count,
            pdf_bytes: &pdf_bytes,
        })
        .await
        .map_err(|e| {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report.period_report_id, "Failed to create profile-run PDF export");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Internal database error" }),
            )
        })
}

async fn create_profile_provenance_manifest(
    state: &Arc<AppState>,
    auth_user: &AuthUser,
    org_id: &str,
    period_report: &CompliancePeriodReportRecord,
    period_artifact: &serde_json::Value,
) -> Result<(CompliancePeriodReportProvenanceManifestRecord, serde_json::Value), (StatusCode, serde_json::Value)> {
    let pdf_exports = state
        .db
        .list_compliance_period_report_pdf_exports(org_id, &period_report.period_report_id, 25)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report.period_report_id, "Failed to load profile-run PDF exports");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Internal database error" }),
            )
        })?;
    let access_logs = state
        .db
        .list_compliance_period_report_access_logs(org_id, &period_report.period_report_id, 100)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report.period_report_id, "Failed to load profile-run access log");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Internal database error" }),
            )
        })?;
    let previous_manifest_hash = state
        .db
        .latest_compliance_period_report_manifest_hash(org_id, &period_report.period_report_id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report.period_report_id, "Failed to load profile-run previous manifest hash");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Internal database error" }),
            )
        })?;

    let generated_at = chrono::Utc::now().timestamp_millis();
    let preimage = build_compliance_period_report_manifest_payload(
        &CompliancePeriodReportManifestPayloadInput {
            manifest_id: "pending",
            generated_at,
            generated_by: &auth_user.client_id,
            period_report,
            period_artifact,
            previous_manifest_hash: previous_manifest_hash.as_deref(),
            manifest_hash: None,
            pdf_exports: &pdf_exports,
            access_logs: &access_logs,
        },
    );
    let content_hash = compliance_review_package_hash(&preimage);
    let manifest_id = deterministic_compliance_period_report_manifest_id(
        &period_report.period_report_id,
        &auth_user.client_id,
        generated_at,
        previous_manifest_hash.as_deref(),
        &content_hash,
    );
    let artifact = build_compliance_period_report_manifest_payload(
        &CompliancePeriodReportManifestPayloadInput {
            manifest_id: &manifest_id,
            generated_at,
            generated_by: &auth_user.client_id,
            period_report,
            period_artifact,
            previous_manifest_hash: previous_manifest_hash.as_deref(),
            manifest_hash: None,
            pdf_exports: &pdf_exports,
            access_logs: &access_logs,
        },
    );
    let manifest_hash = compliance_review_package_hash(&artifact);
    let artifact = build_compliance_period_report_manifest_payload(
        &CompliancePeriodReportManifestPayloadInput {
            manifest_id: &manifest_id,
            generated_at,
            generated_by: &auth_user.client_id,
            period_report,
            period_artifact,
            previous_manifest_hash: previous_manifest_hash.as_deref(),
            manifest_hash: Some(&manifest_hash),
            pdf_exports: &pdf_exports,
            access_logs: &access_logs,
        },
    );

    let manifest = state
        .db
        .create_compliance_period_report_provenance_manifest(
            &CreateCompliancePeriodReportProvenanceManifestInput {
                manifest_id: &manifest_id,
                org_id,
                period_report_id: &period_report.period_report_id,
                generated_by_user_id: &auth_user.client_id,
                manifest_hash: &manifest_hash,
                previous_manifest_hash: previous_manifest_hash.as_deref(),
                signature_algorithm: COMPLIANCE_PERIOD_REPORT_MANIFEST_SIGNATURE_ALGORITHM,
                payload_json_redacted: &artifact,
            },
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report.period_report_id, "Failed to create profile-run provenance manifest");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Internal database error" }),
            )
        })?;

    append_period_report_access_log(
        state,
        PeriodReportAccessLogInput {
            org_id,
            period_report_id: &period_report.period_report_id,
            actor_client_id: &auth_user.client_id,
            action: "manifest_created",
            artifact_type: "manifest",
            artifact_id: Some(&manifest.manifest_id),
            artifact_hash: Some(&manifest.manifest_hash),
            metadata: json!({
                "source": "period_report_profile_run",
                "previous_manifest_hash": manifest.previous_manifest_hash,
                "signature_algorithm": manifest.signature_algorithm,
                "source_period_report_hash": period_report.artifact_hash,
                "pdf_export_count": pdf_exports.len(),
                "access_log_count": access_logs.len(),
                "manual_run": true,
                "scheduled_run": false,
                "compliance_claim": false,
                "regulatory_claim": false,
                "certification": false,
                "agent_governance_required": false,
                "source_period_report_artifact_mutated": false
            }),
        },
    )
    .await;

    Ok((manifest, artifact))
}

pub async fn create_compliance_period_report_profile(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<CompliancePeriodReportProfileRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    let normalized = match normalize_compliance_period_report_profile_request(&mut payload) {
        Ok(value) => value,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid compliance period report profile request", "details": errors })),
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
    let profile_id = format!("cprprof_{}", Uuid::new_v4().simple());
    match state
        .db
        .create_compliance_period_report_profile(&CreateCompliancePeriodReportProfileInput {
            profile_id: &profile_id,
            org_id: &org_id,
            created_by_user_id: &auth_user.client_id,
            name: &normalized.name,
            period_type: &normalized.period_type,
            framework_id: normalized.framework_id.as_deref(),
            framework_owner_type: normalized.framework_owner_type.as_deref(),
            include_pdf: normalized.include_pdf,
            include_manifest: normalized.include_manifest,
            retention_days: normalized.retention_days,
            filters: &normalized.filters,
        })
        .await
    {
        Ok(profile) => {
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "compliance_period_report_profile.created".to_string(),
                target_type: Some("compliance_period_report_profile".to_string()),
                target_id: Some(profile.profile_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "profile_id": profile.profile_id,
                    "period_type": profile.period_type,
                    "framework_id": profile.framework_id,
                    "include_pdf": profile.include_pdf,
                    "include_manifest": profile.include_manifest,
                    "retention_days": profile.retention_days,
                    "manual_run_template": true,
                    "scheduled_run": false,
                    "compliance_claim": false,
                    "regulatory_claim": false,
                    "certification": false,
                    "agent_governance_required": false
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write period report profile create audit log: {}", e);
            }
            (
                StatusCode::CREATED,
                Json(compliance_period_report_profile_response(profile)),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to create period report profile");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn list_compliance_period_report_profiles(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(mut query): Query<CompliancePeriodReportProfileQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let limit = match normalize_compliance_period_report_profile_query(&mut query) {
        Ok(value) => value,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid compliance period report profile query", "details": errors })),
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
        .list_compliance_period_report_profiles(&ListCompliancePeriodReportProfilesInput {
            org_id: &org_id,
            framework_id: query.framework_id.as_deref(),
            status: query.status.as_deref(),
            limit,
        })
        .await
    {
        Ok(items) => {
            let count = items.len();
            (
                StatusCode::OK,
                Json(CompliancePeriodReportProfileListResponse {
                    items,
                    count,
                    limit,
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to list period report profiles");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn get_compliance_period_report_profile(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(profile_id): Path<String>,
    Query(mut query): Query<CompliancePeriodReportProfileQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let profile_id = match normalize_compliance_period_report_profile_id(&profile_id) {
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
        .get_compliance_period_report_profile(&org_id, &profile_id)
        .await
    {
        Ok(Some(profile)) => (
            StatusCode::OK,
            Json(compliance_period_report_profile_response(profile)),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance period report profile not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, profile_id = %profile_id, "Failed to get period report profile");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn update_compliance_period_report_profile(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(profile_id): Path<String>,
    Json(mut payload): Json<CompliancePeriodReportProfilePatchRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    let profile_id = match normalize_compliance_period_report_profile_id(&profile_id) {
        Ok(value) => value,
        Err(error) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response();
        }
    };
    normalize_release_approval_optional_text(&mut payload.org_name);
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
    let existing = match state
        .db
        .get_compliance_period_report_profile(&org_id, &profile_id)
        .await
    {
        Ok(Some(value)) => value,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Compliance period report profile not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, profile_id = %profile_id, "Failed to load period report profile before update");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    if existing.status == "archived" {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Archived profiles cannot be edited",
                "code": "period_report_profile_archived"
            })),
        )
            .into_response();
    }
    let normalized = match merged_compliance_period_report_profile_patch(&existing, &mut payload) {
        Ok(value) => value,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid compliance period report profile patch", "details": errors })),
            )
                .into_response();
        }
    };
    match state
        .db
        .update_compliance_period_report_profile(&UpdateCompliancePeriodReportProfileInput {
            org_id: &org_id,
            profile_id: &profile_id,
            updated_by_user_id: &auth_user.client_id,
            name: &normalized.name,
            period_type: &normalized.period_type,
            framework_id: normalized.framework_id.as_deref(),
            framework_owner_type: normalized.framework_owner_type.as_deref(),
            include_pdf: normalized.include_pdf,
            include_manifest: normalized.include_manifest,
            retention_days: normalized.retention_days,
            filters: &normalized.filters,
        })
        .await
    {
        Ok(Some(profile)) => {
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "compliance_period_report_profile.updated".to_string(),
                target_type: Some("compliance_period_report_profile".to_string()),
                target_id: Some(profile.profile_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "profile_id": profile.profile_id,
                    "period_type": profile.period_type,
                    "framework_id": profile.framework_id,
                    "include_pdf": profile.include_pdf,
                    "include_manifest": profile.include_manifest,
                    "retention_days": profile.retention_days,
                    "manual_run_template": true,
                    "scheduled_run": false,
                    "agent_governance_required": false
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write period report profile update audit log: {}", e);
            }
            (
                StatusCode::OK,
                Json(compliance_period_report_profile_response(profile)),
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Archived profiles cannot be edited",
                "code": "period_report_profile_archived"
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, profile_id = %profile_id, "Failed to update period report profile");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn archive_compliance_period_report_profile(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(profile_id): Path<String>,
    Json(mut payload): Json<CompliancePeriodReportProfilePatchRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    let profile_id = match normalize_compliance_period_report_profile_id(&profile_id) {
        Ok(value) => value,
        Err(error) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response();
        }
    };
    normalize_release_approval_optional_text(&mut payload.org_name);
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
        .archive_compliance_period_report_profile(&ArchiveCompliancePeriodReportProfileInput {
            org_id: &org_id,
            profile_id: &profile_id,
            updated_by_user_id: &auth_user.client_id,
        })
        .await
    {
        Ok(Some(profile)) => {
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "compliance_period_report_profile.archived".to_string(),
                target_type: Some("compliance_period_report_profile".to_string()),
                target_id: Some(profile.profile_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "profile_id": profile.profile_id,
                    "archived_at": profile.archived_at,
                    "manual_run_template": true,
                    "scheduled_run": false,
                    "agent_governance_required": false
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write period report profile archive audit log: {}", e);
            }
            (
                StatusCode::OK,
                Json(compliance_period_report_profile_response(profile)),
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance period report profile not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, profile_id = %profile_id, "Failed to archive period report profile");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn run_compliance_period_report_profile(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(profile_id): Path<String>,
    Json(mut payload): Json<CompliancePeriodReportProfileRunRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    let profile_id = match normalize_compliance_period_report_profile_id(&profile_id) {
        Ok(value) => value,
        Err(error) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response();
        }
    };
    normalize_release_approval_optional_text(&mut payload.org_name);
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
    let profile = match state
        .db
        .get_compliance_period_report_profile(&org_id, &profile_id)
        .await
    {
        Ok(Some(value)) => value,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Compliance period report profile not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, profile_id = %profile_id, "Failed to load period report profile before run");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    if profile.status == "archived" {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Archived profiles cannot be run",
                "code": "period_report_profile_archived"
            })),
        )
            .into_response();
    }
    let (date_range_start, date_range_end) = match derive_profile_run_period(&profile, &payload) {
        Ok(value) => value,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid compliance period report profile run request", "details": errors })),
            )
                .into_response();
        }
    };
    let (period_report, artifact) = match create_period_report_for_profile_run(
        &state,
        &auth_user,
        &org_id,
        &profile,
        date_range_start,
        date_range_end,
    )
    .await
    {
        Ok(value) => value,
        Err((status, body)) => return (status, Json(body)).into_response(),
    };

    let pdf_export = if profile.include_pdf {
        match create_profile_pdf_export(&state, &auth_user, &org_id, &period_report, &artifact).await {
            Ok(value) => Some(value),
            Err((status, body)) => return (status, Json(body)).into_response(),
        }
    } else {
        None
    };
    let manifest = if profile.include_manifest {
        match create_profile_provenance_manifest(&state, &auth_user, &org_id, &period_report, &artifact).await {
            Ok((value, _artifact)) => Some(value),
            Err((status, body)) => return (status, Json(body)).into_response(),
        }
    } else {
        None
    };

    let profile = match state
        .db
        .record_compliance_period_report_profile_run(
            &RecordCompliancePeriodReportProfileRunInput {
                org_id: &org_id,
                profile_id: &profile.profile_id,
                period_report_id: &period_report.period_report_id,
                pdf_export_id: pdf_export.as_ref().map(|value| value.pdf_export_id.as_str()),
                manifest_id: manifest.as_ref().map(|value| value.manifest_id.as_str()),
                updated_by_user_id: &auth_user.client_id,
            },
        )
        .await
    {
        Ok(Some(value)) => value,
        Ok(None) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "Archived profiles cannot be run",
                    "code": "period_report_profile_archived"
                })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, profile_id = %profile_id, "Failed to record period report profile run");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    let audit_entry = AdminAuditLogEntry {
        id: Uuid::new_v4().to_string(),
        actor_client_id: auth_user.client_id.clone(),
        action: "compliance_period_report_profile.run".to_string(),
        target_type: Some("compliance_period_report_profile".to_string()),
        target_id: Some(profile.profile_id.clone()),
        metadata: json!({
            "org_id": org_id,
            "profile_id": profile.profile_id,
            "period_report_id": period_report.period_report_id,
            "pdf_export_id": pdf_export.as_ref().map(|value| value.pdf_export_id.clone()),
            "manifest_id": manifest.as_ref().map(|value| value.manifest_id.clone()),
            "date_range_start": period_report.date_range_start,
            "date_range_end": period_report.date_range_end,
            "retention_days": profile.retention_days,
            "manual_run": true,
            "scheduled_run": false,
            "compliance_claim": false,
            "regulatory_claim": false,
            "certification": false,
            "agent_governance_required": false,
            "llm_decision": false
        }),
        created_at: chrono::Utc::now().timestamp_millis(),
    };
    if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
        tracing::warn!("Failed to write period report profile run audit log: {}", e);
    }

    (
        StatusCode::CREATED,
        Json(CompliancePeriodReportProfileRunResponse {
            profile,
            period_report: period_report.clone(),
            pdf_export,
            manifest,
            download_url: format!(
                "/compliance/period-reports/{}/download",
                period_report.period_report_id
            ),
        }),
    )
        .into_response()
}
