// ============================================================================
// COMPLIANCE FRAMEWORK REVIEW REPORT PDF EXPORTS
// ============================================================================

const COMPLIANCE_FRAMEWORK_REVIEW_REPORT_PDF_CONTENT_TYPE: &str = "application/pdf";
const COMPLIANCE_FRAMEWORK_REVIEW_REPORT_PDF_SCHEMA_VERSION: &str =
    "gitgov_framework_review_report_pdf_export.v1";

fn normalize_framework_review_report_pdf_export_id(value: &str) -> Result<String, &'static str> {
    let normalized = value.trim().to_string();
    if normalized.starts_with("frrpdf_") && normalized.len() <= 80 {
        Ok(normalized)
    } else {
        Err("pdf_export_id must be a valid frrpdf_ identifier")
    }
}

fn normalize_framework_review_report_pdf_manifest_id(
    value: &mut Option<String>,
) -> Result<(), String> {
    let Some(raw) = value.take() else {
        return Ok(());
    };
    let normalized = raw.trim().to_string();
    if normalized.is_empty() {
        return Ok(());
    }
    if !normalized.starts_with("frrm_") || normalized.len() > 80 {
        return Err("manifest_id must be a valid frrm_ identifier".to_string());
    }
    *value = Some(normalized);
    Ok(())
}

fn normalize_framework_review_report_pdf_request(
    payload: &mut ComplianceFrameworkReviewReportPdfExportRequest,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut payload.org_name);
    if let Err(error) = normalize_framework_review_report_pdf_manifest_id(&mut payload.manifest_id)
    {
        errors.push(error);
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn normalize_framework_review_report_pdf_query(
    query: &mut ComplianceFrameworkReviewReportPdfExportQuery,
) -> Result<Option<String>, Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut query.org_name);
    let pdf_export_id = match query.pdf_export_id.take() {
        Some(value) if !value.trim().is_empty() => {
            match normalize_framework_review_report_pdf_export_id(&value) {
                Ok(value) => Some(value),
                Err(error) => {
                    errors.push(error.to_string());
                    None
                }
            }
        }
        _ => None,
    };
    if errors.is_empty() {
        Ok(pdf_export_id)
    } else {
        Err(errors)
    }
}

fn pdf_escape_text(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            '(' => "\\(".to_string(),
            ')' => "\\)".to_string(),
            '\\' => "\\\\".to_string(),
            '\r' | '\n' | '\t' => " ".to_string(),
            ch if ch.is_ascii() && !ch.is_control() => ch.to_string(),
            _ => "?".to_string(),
        })
        .collect::<String>()
}

fn push_wrapped_pdf_line(lines: &mut Vec<String>, prefix: &str, value: impl ToString) {
    let text = format!("{prefix}{}", value.to_string());
    let max_len = 96usize;
    if text.len() <= max_len {
        lines.push(text);
        return;
    }
    let mut current = String::new();
    for part in text.split_whitespace() {
        if !current.is_empty() && current.len() + part.len() + 1 > max_len {
            lines.push(current);
            current = "  ".to_string();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(part);
    }
    if !current.is_empty() {
        lines.push(current);
    }
}

fn collect_framework_review_report_pdf_lines(
    report: &ComplianceFrameworkReviewReportRecord,
    manifest: &ComplianceFrameworkReviewReportProvenanceManifestRecord,
    report_artifact: &serde_json::Value,
    assignments: &[ComplianceFrameworkReviewReportAssignmentRecord],
    comments: &[ComplianceFrameworkReviewReportCommentRecord],
    generated_by: &str,
    generated_at: i64,
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push("GitGov Framework Review Report".to_string());
    lines.push(format!("Schema: {COMPLIANCE_FRAMEWORK_REVIEW_REPORT_PDF_SCHEMA_VERSION}"));
    lines.push("Purpose: customer/auditor review artifact".to_string());
    lines.push("Not a certification, compliance score, or official regulatory claim.".to_string());
    lines.push(String::new());
    push_wrapped_pdf_line(&mut lines, "Report ID: ", &report.report_id);
    push_wrapped_pdf_line(&mut lines, "Framework: ", &report.framework_id);
    push_wrapped_pdf_line(&mut lines, "Framework version: ", &report.framework_version);
    push_wrapped_pdf_line(&mut lines, "Framework owner type: ", &report.framework_owner_type);
    push_wrapped_pdf_line(&mut lines, "Review status: ", &report.review_status);
    push_wrapped_pdf_line(
        &mut lines,
        "Reviewed by: ",
        report.reviewed_by_user_id.as_deref().unwrap_or("not available"),
    );
    push_wrapped_pdf_line(
        &mut lines,
        "Reviewed at: ",
        report
            .reviewed_at
            .map(|value| value.to_string())
            .unwrap_or_else(|| "not available".to_string()),
    );
    push_wrapped_pdf_line(&mut lines, "Generated by: ", generated_by);
    push_wrapped_pdf_line(&mut lines, "Generated at: ", generated_at);
    lines.push(String::new());
    lines.push("Source hashes".to_string());
    push_wrapped_pdf_line(&mut lines, "Report artifact hash: ", &report.artifact_hash);
    push_wrapped_pdf_line(&mut lines, "Evidence export hash: ", &report.evidence_export_hash);
    push_wrapped_pdf_line(&mut lines, "Mapping hash: ", &report.mapping_hash);
    push_wrapped_pdf_line(&mut lines, "Review package hash: ", &report.review_package_hash);
    push_wrapped_pdf_line(&mut lines, "Manifest ID: ", &manifest.manifest_id);
    push_wrapped_pdf_line(&mut lines, "Manifest hash: ", &manifest.manifest_hash);
    if let Some(previous) = manifest.previous_manifest_hash.as_deref() {
        push_wrapped_pdf_line(&mut lines, "Previous manifest hash: ", previous);
    }
    lines.push(String::new());
    lines.push("No-claim flags".to_string());
    push_wrapped_pdf_line(&mut lines, "compliance_claim=", report.compliance_claim);
    push_wrapped_pdf_line(&mut lines, "regulatory_claim=", report.regulatory_claim);
    push_wrapped_pdf_line(&mut lines, "certification=", report.certification);
    push_wrapped_pdf_line(
        &mut lines,
        "requires_auditor_review=",
        report.requires_auditor_review,
    );
    lines.push(String::new());
    lines.push("Reviewer collaboration".to_string());
    push_wrapped_pdf_line(&mut lines, "Assignment count: ", assignments.len());
    push_wrapped_pdf_line(
        &mut lines,
        "Active assignments: ",
        assignments
            .iter()
            .filter(|assignment| assignment.assignment_status == "active")
            .count(),
    );
    for assignment in assignments.iter().take(12) {
        push_wrapped_pdf_line(
            &mut lines,
            "- Auditor: ",
            format!("{} ({})", assignment.auditor_client_id, assignment.assignment_status),
        );
    }
    push_wrapped_pdf_line(&mut lines, "Comment count: ", comments.len());
    for comment in comments.iter().take(8) {
        push_wrapped_pdf_line(
            &mut lines,
            "- Comment: ",
            format!(
                "{}: {}",
                comment.commenter_client_id, comment.comment_body_safe
            ),
        );
    }
    lines.push(String::new());
    lines.push("Evidence summary".to_string());
    if let Some(summary) = report_artifact.get("summary") {
        push_wrapped_pdf_line(&mut lines, "Summary: ", summary);
    }
    if let Some(missing) = report_artifact.get("missing_evidence") {
        push_wrapped_pdf_line(&mut lines, "Missing evidence: ", missing);
    }
    if let Some(controls) = report_artifact.get("controls").and_then(|value| value.as_array()) {
        push_wrapped_pdf_line(&mut lines, "Control count: ", controls.len());
        for control in controls.iter().take(16) {
            let control_id = control
                .get("control_id")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            let status = control
                .get("status")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            push_wrapped_pdf_line(&mut lines, "- Control: ", format!("{control_id} / {status}"));
        }
    }
    lines
}

fn build_pdf_content_stream(lines: &[String]) -> String {
    let mut content = String::from("BT\n/F1 10 Tf\n72 740 Td\n");
    for line in lines {
        content.push('(');
        content.push_str(&pdf_escape_text(line));
        content.push_str(") Tj\n0 -14 Td\n");
    }
    content.push_str("ET\n");
    content
}

fn build_framework_review_report_pdf(lines: &[String]) -> (Vec<u8>, i32) {
    let lines_per_page = 44usize;
    let pages: Vec<&[String]> = lines.chunks(lines_per_page).collect();
    let page_count = pages.len().max(1);
    let font_id = 3usize;
    let mut objects: Vec<(usize, String)> = Vec::new();
    let kids = (0..page_count)
        .map(|index| format!("{} 0 R", 4 + (index * 2)))
        .collect::<Vec<_>>()
        .join(" ");
    objects.push((1, "<< /Type /Catalog /Pages 2 0 R >>".to_string()));
    objects.push((
        2,
        format!("<< /Type /Pages /Kids [{kids}] /Count {page_count} >>"),
    ));
    objects.push((
        font_id,
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
    ));

    for (index, page_lines) in pages.iter().enumerate() {
        let page_id = 4 + (index * 2);
        let content_id = page_id + 1;
        let stream = build_pdf_content_stream(page_lines);
        objects.push((
            page_id,
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 {font_id} 0 R >> >> /Contents {content_id} 0 R >>"
            ),
        ));
        objects.push((
            content_id,
            format!(
                "<< /Length {} >>\nstream\n{}endstream",
                stream.len(),
                stream
            ),
        ));
    }

    objects.sort_by_key(|(id, _)| *id);
    let max_id = objects.iter().map(|(id, _)| *id).max().unwrap_or(1);
    let mut pdf = Vec::new();
    pdf.extend_from_slice(b"%PDF-1.4\n% GitGov\n");
    let mut offsets = vec![0usize; max_id + 1];
    for (id, body) in objects {
        offsets[id] = pdf.len();
        pdf.extend_from_slice(format!("{id} 0 obj\n{body}\nendobj\n").as_bytes());
    }
    let xref_offset = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n", max_id + 1).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets.iter().skip(1) {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            max_id + 1,
            xref_offset
        )
        .as_bytes(),
    );
    (pdf, page_count as i32)
}

fn framework_review_report_pdf_hash(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn deterministic_framework_review_report_pdf_export_id(pdf_hash: &str) -> String {
    let suffix = pdf_hash.trim_start_matches("sha256:");
    format!("frrpdf_{}", &suffix[..32.min(suffix.len())])
}

fn framework_review_report_pdf_response(
    pdf_export: ComplianceFrameworkReviewReportPdfExportRecord,
) -> ComplianceFrameworkReviewReportPdfExportResponse {
    ComplianceFrameworkReviewReportPdfExportResponse {
        download_url: format!(
            "/compliance/framework-review-reports/{}/pdf-export/download?pdf_export_id={}",
            pdf_export.report_id, pdf_export.pdf_export_id
        ),
        pdf_export,
    }
}

async fn load_framework_review_report_for_pdf(
    state: &Arc<AppState>,
    org_id: &str,
    report_id: &str,
) -> Result<ComplianceFrameworkReviewReportRecord, axum::response::Response> {
    match state
        .db
        .get_compliance_framework_review_report(org_id, report_id)
        .await
    {
        Ok(Some(record)) => Ok(record),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance framework review report not found" })),
        )
            .into_response()),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to load framework review report for PDF export");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response())
        }
    }
}

async fn resolve_framework_review_report_manifest_for_pdf(
    state: &Arc<AppState>,
    org_id: &str,
    report_id: &str,
    manifest_id: Option<&str>,
) -> Result<ComplianceFrameworkReviewReportProvenanceManifestRecord, axum::response::Response> {
    let result = match manifest_id {
        Some(manifest_id) => {
            state
                .db
                .get_compliance_framework_review_report_manifest(org_id, report_id, manifest_id)
                .await
        }
        None => {
            state
                .db
                .get_latest_compliance_framework_review_report_manifest(org_id, report_id)
                .await
        }
    };
    match result {
        Ok(Some(record)) => Ok(record),
        Ok(None) => Err((
            StatusCode::CONFLICT,
            Json(json!({
                "error": "A reviewed report provenance manifest is required before PDF export",
                "code": "manifest_required"
            })),
        )
            .into_response()),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to load framework review report manifest for PDF export");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response())
        }
    }
}

pub async fn create_compliance_framework_review_report_pdf_export(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(report_id): Path<String>,
    Json(mut payload): Json<ComplianceFrameworkReviewReportPdfExportRequest>,
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
    if let Err(errors) = normalize_framework_review_report_pdf_request(&mut payload) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid compliance framework review report PDF export request", "details": errors })),
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
    if let Some(resp) =
        require_framework_review_report_collaboration_access(&state, &auth_user, &org_id, &report_id)
            .await
    {
        return resp;
    }

    let report = match load_framework_review_report_for_pdf(&state, &org_id, &report_id).await {
        Ok(report) => report,
        Err(resp) => return resp,
    };
    if report.review_status != FRAMEWORK_REVIEW_REPORT_REVIEW_REVIEWED {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Framework review report must be reviewed before PDF export",
                "code": "report_not_reviewed"
            })),
        )
            .into_response();
    }
    if report.compliance_claim
        || report.regulatory_claim
        || report.certification
        || !report.requires_auditor_review
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Framework review report claims are not eligible for PDF export" })),
        )
            .into_response();
    }

    let manifest = match resolve_framework_review_report_manifest_for_pdf(
        &state,
        &org_id,
        &report_id,
        payload.manifest_id.as_deref(),
    )
    .await
    {
        Ok(manifest) => manifest,
        Err(resp) => return resp,
    };
    let report_artifact = match state
        .db
        .get_compliance_framework_review_report_payload_redacted(&org_id, &report_id)
        .await
    {
        Ok(Some(value)) => value,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Compliance framework review report artifact not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to load framework review report artifact for PDF export");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    let assignments = match state
        .db
        .list_compliance_framework_review_report_assignments(&org_id, &report_id)
        .await
    {
        Ok(value) => value,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to load assignments for PDF export");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    let comments = match state
        .db
        .list_compliance_framework_review_report_comments(&org_id, &report_id)
        .await
    {
        Ok(value) => value,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to load comments for PDF export");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    let generated_at = chrono::Utc::now().timestamp_millis();
    let lines = collect_framework_review_report_pdf_lines(
        &report,
        &manifest,
        &report_artifact,
        &assignments,
        &comments,
        &auth_user.client_id,
        generated_at,
    );
    let (pdf_bytes, page_count) = build_framework_review_report_pdf(&lines);
    let pdf_artifact_hash = framework_review_report_pdf_hash(&pdf_bytes);
    let pdf_export_id = deterministic_framework_review_report_pdf_export_id(&pdf_artifact_hash);

    match state
        .db
        .create_compliance_framework_review_report_pdf_export(
            &CreateComplianceFrameworkReviewReportPdfExportInput {
                pdf_export_id: &pdf_export_id,
                org_id: &org_id,
                report_id: &report.report_id,
                manifest_id: &manifest.manifest_id,
                created_by_user_id: &auth_user.client_id,
                source_report_hash: &report.artifact_hash,
                manifest_hash: &manifest.manifest_hash,
                pdf_artifact_hash: &pdf_artifact_hash,
                content_type: COMPLIANCE_FRAMEWORK_REVIEW_REPORT_PDF_CONTENT_TYPE,
                page_count,
                pdf_bytes: &pdf_bytes,
            },
        )
        .await
    {
        Ok(pdf_export) => {
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "compliance_framework_review_report.pdf_export_created".to_string(),
                target_type: Some("compliance_framework_review_report".to_string()),
                target_id: Some(report.report_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "report_id": report.report_id,
                    "pdf_export_id": pdf_export.pdf_export_id,
                    "pdf_artifact_hash": pdf_export.pdf_artifact_hash,
                    "source_report_hash": pdf_export.source_report_hash,
                    "manifest_id": pdf_export.manifest_id,
                    "manifest_hash": pdf_export.manifest_hash,
                    "content_type": pdf_export.content_type,
                    "page_count": pdf_export.page_count,
                    "compliance_claim": false,
                    "regulatory_claim": false,
                    "certification": false,
                    "agent_governance_required": false,
                    "source_report_artifact_mutated": false
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write framework review report PDF export audit log: {}", e);
            }
            (
                StatusCode::CREATED,
                Json(framework_review_report_pdf_response(pdf_export)),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to create framework review report PDF export");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn get_compliance_framework_review_report_pdf_export(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(report_id): Path<String>,
    Query(mut query): Query<ComplianceFrameworkReviewReportPdfExportQuery>,
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
    let pdf_export_id = match normalize_framework_review_report_pdf_query(&mut query) {
        Ok(value) => value,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid compliance framework review report PDF export query", "details": errors })),
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
    if let Some(resp) =
        require_framework_review_report_collaboration_access(&state, &auth_user, &org_id, &report_id)
            .await
    {
        return resp;
    }

    let result = match pdf_export_id.as_deref() {
        Some(pdf_export_id) => {
            state
                .db
                .get_compliance_framework_review_report_pdf_export(
                    &org_id,
                    &report_id,
                    pdf_export_id,
                )
                .await
        }
        None => {
            state
                .db
                .get_latest_compliance_framework_review_report_pdf_export(&org_id, &report_id)
                .await
        }
    };
    match result {
        Ok(Some(pdf_export)) => (
            StatusCode::OK,
            Json(framework_review_report_pdf_response(pdf_export)),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance framework review report PDF export not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to load framework review report PDF export");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn download_compliance_framework_review_report_pdf_export(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(report_id): Path<String>,
    Query(mut query): Query<ComplianceFrameworkReviewReportPdfExportQuery>,
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
    let pdf_export_id = match normalize_framework_review_report_pdf_query(&mut query) {
        Ok(value) => value,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid compliance framework review report PDF export download query", "details": errors })),
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
    if let Some(resp) =
        require_framework_review_report_collaboration_access(&state, &auth_user, &org_id, &report_id)
            .await
    {
        return resp;
    }

    match state
        .db
        .download_compliance_framework_review_report_pdf_export(
            &org_id,
            &report_id,
            pdf_export_id.as_deref(),
        )
        .await
    {
        Ok(Some((pdf_export, bytes))) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static(COMPLIANCE_FRAMEWORK_REVIEW_REPORT_PDF_CONTENT_TYPE),
            );
            headers.insert(
                axum::http::header::CONTENT_DISPOSITION,
                axum::http::HeaderValue::from_str(&format!(
                    "attachment; filename=\"gitgov-framework-review-{}.pdf\"",
                    pdf_export.pdf_export_id
                ))
                .unwrap_or_else(|_| axum::http::HeaderValue::from_static("attachment")),
            );
            headers.insert(
                axum::http::HeaderName::from_static("x-gitgov-artifact-hash"),
                axum::http::HeaderValue::from_str(&pdf_export.pdf_artifact_hash)
                    .unwrap_or_else(|_| axum::http::HeaderValue::from_static("invalid")),
            );
            (headers, Bytes::from(bytes)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance framework review report PDF export not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, report_id = %report_id, "Failed to download framework review report PDF export");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
