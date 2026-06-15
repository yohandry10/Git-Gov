const COMPLIANCE_PERIOD_REPORT_SHARE_PACKAGE_SCHEMA_VERSION: &str =
    "gitgov_period_compliance_report_share_package.v1";

fn normalize_compliance_period_report_share_package_id(
    value: &str,
) -> Result<String, &'static str> {
    let normalized = value.trim().to_string();
    if normalized.starts_with("cprsp_") && normalized.len() <= 80 {
        Ok(normalized)
    } else {
        Err("share_package_id must be a valid cprsp_ identifier")
    }
}

fn normalize_compliance_period_report_share_package_request(
    payload: &mut CompliancePeriodReportSharePackageRequest,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut payload.org_name);
    payload.format = payload
        .format
        .take()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .or_else(|| Some("json_bundle".to_string()));
    if payload.format.as_deref() != Some("json_bundle") {
        errors.push("format must be json_bundle".to_string());
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn normalize_compliance_period_report_share_package_query(
    query: &mut CompliancePeriodReportSharePackageQuery,
) -> Result<i64, Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut query.org_name);
    query.status = match query.status.take() {
        Some(value) if !value.trim().is_empty() => {
            let normalized = value.trim().to_ascii_lowercase();
            if matches!(normalized.as_str(), "active" | "revoked") {
                Some(normalized)
            } else {
                errors.push("status must be active or revoked".to_string());
                None
            }
        }
        _ => None,
    };
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    if errors.is_empty() {
        Ok(limit)
    } else {
        Err(errors)
    }
}

fn compliance_period_report_share_package_response(
    share_package: CompliancePeriodReportSharePackageRecord,
    artifact: Option<serde_json::Value>,
) -> CompliancePeriodReportSharePackageResponse {
    let download_url = format!(
        "/compliance/period-report-share-packages/{}/download",
        share_package.share_package_id
    );
    CompliancePeriodReportSharePackageResponse {
        share_package,
        download_url,
        artifact,
    }
}

fn deterministic_compliance_period_report_share_package_id() -> String {
    format!("cprsp_{}", Uuid::new_v4().simple())
}

struct PeriodReportSharePackageArtifactInput<'a> {
    share_package_id: &'a str,
    created_at: i64,
    created_by: &'a str,
    period_report: &'a CompliancePeriodReportRecord,
    period_artifact: &'a serde_json::Value,
    pdf_export: &'a CompliancePeriodReportPdfExportRecord,
    manifest: &'a CompliancePeriodReportProvenanceManifestRecord,
    no_claims_snapshot: &'a serde_json::Value,
    source_hashes: &'a serde_json::Value,
    review_snapshot: &'a serde_json::Value,
    retention_snapshot: &'a serde_json::Value,
    artifact_hash: Option<&'a str>,
}

fn build_period_report_share_package_artifact(
    input: PeriodReportSharePackageArtifactInput<'_>,
) -> serde_json::Value {
    json!({
        "schema_version": COMPLIANCE_PERIOD_REPORT_SHARE_PACKAGE_SCHEMA_VERSION,
        "share_package_id": input.share_package_id,
        "generated_at": input.created_at,
        "created_by_user_id": input.created_by,
        "package_format": "json_bundle",
        "purpose": "Manual offline verification bundle for customer/auditor review.",
        "positioning": {
            "manual_sharing_only": true,
            "public_link": false,
            "email_delivery": false,
            "scheduler": false,
            "certification": false,
            "legal_attestation": false,
            "official_regulatory_claim": false,
            "compliance_score": false
        },
        "period_report": {
            "period_report_id": input.period_report.period_report_id,
            "artifact_hash": input.period_report.artifact_hash,
            "format": input.period_report.format,
            "status": input.period_report.status,
            "review_status": input.period_report.review_status,
            "date_range_start": input.period_report.date_range_start,
            "date_range_end": input.period_report.date_range_end,
            "framework_id": input.period_report.framework_id,
            "report_count": input.period_report.report_count,
            "source_report_ids": input.period_report.source_report_ids,
            "created_at": input.period_report.created_at,
            "created_by_user_id": input.period_report.created_by_user_id
        },
        "period_report_artifact_summary": {
            "schema_version": input.period_artifact.get("schema_version"),
            "summary": input.period_artifact.get("summary"),
            "source_hashes": input.period_artifact.get("source_hashes"),
            "missing_evidence_summary": input.period_artifact.get("missing_evidence_summary"),
            "positioning": input.period_artifact.get("positioning"),
            "audit_metadata": input.period_artifact.get("audit_metadata")
        },
        "pdf_export": {
            "pdf_export_id": input.pdf_export.pdf_export_id,
            "source_period_report_hash": input.pdf_export.source_period_report_hash,
            "pdf_artifact_hash": input.pdf_export.pdf_artifact_hash,
            "content_type": input.pdf_export.content_type,
            "page_count": input.pdf_export.page_count,
            "created_at": input.pdf_export.created_at
        },
        "provenance_manifest": {
            "manifest_id": input.manifest.manifest_id,
            "manifest_hash": input.manifest.manifest_hash,
            "previous_manifest_hash": input.manifest.previous_manifest_hash,
            "signature_algorithm": input.manifest.signature_algorithm,
            "created_at": input.manifest.created_at
        },
        "review": input.review_snapshot,
        "retention": input.retention_snapshot,
        "source_hashes": input.source_hashes,
        "claims": input.no_claims_snapshot,
        "verification": {
            "package_hash": input.artifact_hash,
            "hash_algorithm": "sha256",
            "verify": [
                "Recompute this JSON bundle hash after excluding transport headers.",
                "Compare period_report.artifact_hash with the downloaded Period Compliance Report JSON artifact.",
                "Compare pdf_export.pdf_artifact_hash with the downloaded Period Compliance Report PDF artifact.",
                "Compare provenance_manifest.manifest_hash with the downloaded Period Compliance Report provenance manifest.",
                "Confirm claims.compliance_claim=false, claims.regulatory_claim=false, claims.certification=false, and claims.requires_auditor_review=true."
            ]
        },
        "audit_metadata": {
            "artifact_redacted": true,
            "raw_payload_included": false,
            "manual_on_demand": true,
            "manual_sharing_only": true,
            "source_period_report_artifact_mutated": false,
            "source_pdf_artifact_mutated": false,
            "source_manifest_artifact_mutated": false,
            "source_review_mutated": false,
            "agent_governance_required": false,
            "agent_governance_used": false,
            "llm_decision": false,
            "policy_mutation": false,
            "provider_mutation": false,
            "gate_mutation": false,
            "public_link": false,
            "email_delivery": false,
            "scheduler": false,
            "docx_export": false,
            "report_scope": "period_compliance_report_share_package"
        }
    })
}

async fn authorize_period_report_for_share_package(
    state: &Arc<AppState>,
    auth_user: &AuthUser,
    org_id: &str,
    period_report_id: &str,
) -> Result<CompliancePeriodReportRecord, axum::response::Response> {
    match state
        .db
        .get_compliance_period_report(
            org_id,
            period_report_id,
            period_report_auditor_filter(auth_user),
        )
        .await
    {
        Ok(Some(period_report)) => Ok(period_report),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance period report not found" })),
        )
            .into_response()),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, "Failed to authorize period report share package access");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response())
        }
    }
}

