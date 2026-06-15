const COMPLIANCE_PERIOD_REPORT_MANIFEST_SCHEMA_VERSION: &str =
    "gitgov_period_compliance_report_provenance_manifest.v1";
const COMPLIANCE_PERIOD_REPORT_MANIFEST_SIGNATURE_ALGORITHM: &str =
    "sha256-period-report-provenance-manifest-v1";

fn deterministic_compliance_period_report_manifest_id(
    period_report_id: &str,
    generated_by: &str,
    generated_at: i64,
    previous_manifest_hash: Option<&str>,
    content_hash: &str,
) -> String {
    let content = format!(
        "{COMPLIANCE_PERIOD_REPORT_MANIFEST_SCHEMA_VERSION}:{period_report_id}:{generated_by}:{generated_at}:{}:{content_hash}",
        previous_manifest_hash.unwrap_or("root")
    );
    let digest = format!("{:x}", Sha256::digest(content.as_bytes()));
    format!("cprm_{}", &digest[..32])
}

fn normalize_compliance_period_report_manifest_id(
    manifest_id: &str,
) -> Result<String, &'static str> {
    let normalized = manifest_id.trim().to_string();
    if normalized.starts_with("cprm_") && normalized.len() == 37 {
        Ok(normalized)
    } else {
        Err("manifest_id must be a valid cprm_ identifier")
    }
}

fn normalize_compliance_period_report_manifest_request(
    org_name: &mut Option<String>,
) {
    normalize_release_approval_optional_text(org_name);
}

struct CompliancePeriodReportManifestPayloadInput<'a> {
    manifest_id: &'a str,
    generated_at: i64,
    generated_by: &'a str,
    period_report: &'a CompliancePeriodReportRecord,
    period_artifact: &'a serde_json::Value,
    previous_manifest_hash: Option<&'a str>,
    manifest_hash: Option<&'a str>,
    pdf_exports: &'a [CompliancePeriodReportPdfExportRecord],
    access_logs: &'a [CompliancePeriodReportAccessLogRecord],
}

fn compliance_period_report_manifest_access_log_summary(
    logs: &[CompliancePeriodReportAccessLogRecord],
) -> serde_json::Value {
    let mut counts: HashMap<String, i64> = HashMap::new();
    for log in logs {
        *counts.entry(log.action.clone()).or_insert(0) += 1;
    }
    let mut action_counts = counts
        .into_iter()
        .map(|(action, count)| json!({ "action": action, "count": count }))
        .collect::<Vec<_>>();
    action_counts.sort_by(|left, right| {
        left.get("action")
            .and_then(|value| value.as_str())
            .cmp(&right.get("action").and_then(|value| value.as_str()))
    });
    let latest_events = logs
        .iter()
        .take(25)
        .map(|log| {
            json!({
                "access_log_id": log.access_log_id,
                "actor_client_id": log.actor_client_id,
                "action": log.action,
                "artifact_type": log.artifact_type,
                "artifact_id": log.artifact_id,
                "artifact_hash": log.artifact_hash,
                "created_at": log.created_at
            })
        })
        .collect::<Vec<_>>();

    json!({
        "total_loaded": logs.len(),
        "action_counts": action_counts,
        "latest_events": latest_events
    })
}

fn build_compliance_period_report_manifest_payload(
    input: &CompliancePeriodReportManifestPayloadInput<'_>,
) -> serde_json::Value {
    let pdf_exports = input
        .pdf_exports
        .iter()
        .map(|export| {
            json!({
                "pdf_export_id": export.pdf_export_id,
                "source_period_report_hash": export.source_period_report_hash,
                "pdf_artifact_hash": export.pdf_artifact_hash,
                "content_type": export.content_type,
                "page_count": export.page_count,
                "created_by_user_id": export.created_by_user_id,
                "created_at": export.created_at,
                "downloaded_at": export.downloaded_at,
                "claims": {
                    "compliance_claim": export.compliance_claim,
                    "regulatory_claim": export.regulatory_claim,
                    "certification": export.certification,
                    "requires_auditor_review": export.requires_auditor_review
                }
            })
        })
        .collect::<Vec<_>>();

    json!({
        "schema_version": COMPLIANCE_PERIOD_REPORT_MANIFEST_SCHEMA_VERSION,
        "manifest_id": input.manifest_id,
        "generated_at": input.generated_at,
        "generated_by_user_id": input.generated_by,
        "signature": {
            "algorithm": COMPLIANCE_PERIOD_REPORT_MANIFEST_SIGNATURE_ALGORITHM,
            "signed_by_user_id": input.generated_by,
            "signed_at": input.generated_at,
            "signature_hash": input.manifest_hash
        },
        "hash_chain": {
            "subject_type": "period_compliance_report",
            "subject_id": input.period_report.period_report_id,
            "previous_manifest_hash": input.previous_manifest_hash,
            "manifest_hash": input.manifest_hash
        },
        "period_report": {
            "period_report_id": input.period_report.period_report_id,
            "artifact_hash": input.period_report.artifact_hash,
            "format": input.period_report.format,
            "status": input.period_report.status,
            "created_at": input.period_report.created_at,
            "created_by_user_id": input.period_report.created_by_user_id,
            "date_range_start": input.period_report.date_range_start,
            "date_range_end": input.period_report.date_range_end,
            "framework_id": input.period_report.framework_id,
            "report_count": input.period_report.report_count,
            "source_report_ids": input.period_report.source_report_ids,
            "retention_status": input.period_report.retention_status,
            "retention_until": input.period_report.retention_until,
            "download_count": input.period_report.download_count,
            "last_downloaded_at": input.period_report.last_downloaded_at,
            "archived_at": input.period_report.archived_at
        },
        "period_artifact_summary": {
            "schema_version": input.period_artifact.get("schema_version"),
            "summary": input.period_artifact.get("summary"),
            "source_hashes": input.period_artifact.get("source_hashes"),
            "missing_evidence_summary": input.period_artifact.get("missing_evidence_summary"),
            "positioning": input.period_artifact.get("positioning"),
            "audit_metadata": input.period_artifact.get("audit_metadata")
        },
        "pdf_exports": {
            "count": pdf_exports.len(),
            "items": pdf_exports
        },
        "access_log": compliance_period_report_manifest_access_log_summary(input.access_logs),
        "claims": {
            "compliance_claim": input.period_report.compliance_claim,
            "regulatory_claim": input.period_report.regulatory_claim,
            "certification": input.period_report.certification,
            "requires_auditor_review": input.period_report.requires_auditor_review,
            "official_regulatory_mapping": false
        },
        "audit_metadata": {
            "artifact_redacted": true,
            "raw_payload_included": false,
            "manual_on_demand": true,
            "source_report_artifact_mutated": false,
            "source_period_report_artifact_mutated": false,
            "agent_governance_required": false,
            "llm_decision": false,
            "policy_mutation": false,
            "provider_mutation": false,
            "gate_mutation": false,
            "report_scope": "period_compliance_report_provenance_manifest"
        }
    })
}

fn compliance_period_report_manifest_response(
    manifest: CompliancePeriodReportProvenanceManifestRecord,
    artifact: serde_json::Value,
) -> CompliancePeriodReportProvenanceManifestResponse {
    let download_url = format!(
        "/compliance/period-reports/{}/provenance-manifests/{}",
        manifest.period_report_id, manifest.manifest_id
    );
    CompliancePeriodReportProvenanceManifestResponse {
        manifest,
        download_url,
        artifact,
    }
}

pub async fn create_compliance_period_report_provenance_manifest(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(period_report_id): Path<String>,
    Json(mut payload): Json<CompliancePeriodReportProvenanceManifestRequest>,
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
    normalize_compliance_period_report_manifest_request(&mut payload.org_name);
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

    let (period_report, period_artifact) = match state
        .db
        .get_compliance_period_report_with_payload(
            &org_id,
            &period_report_id,
            period_report_auditor_filter(&auth_user),
        )
        .await
    {
        Ok(Some(value)) => value,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Compliance period report not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, "Failed to load period compliance report before provenance manifest");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    if period_report.status != "generated" || period_report.format != "json" {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Period compliance report must be a generated JSON artifact before a provenance manifest can be generated",
                "code": "period_report_not_generated"
            })),
        )
            .into_response();
    }
    if period_report.compliance_claim
        || period_report.regulatory_claim
        || period_report.certification
        || !period_report.requires_auditor_review
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Period compliance report claims are not eligible for a non-claim provenance manifest" })),
        )
            .into_response();
    }

    let pdf_exports = match state
        .db
        .list_compliance_period_report_pdf_exports(&org_id, &period_report_id, 25)
        .await
    {
        Ok(value) => value,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, "Failed to load period report PDF exports for provenance manifest");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    let access_logs = match state
        .db
        .list_compliance_period_report_access_logs(&org_id, &period_report_id, 100)
        .await
    {
        Ok(value) => value,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, "Failed to load period report access log for provenance manifest");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    let previous_manifest_hash = match state
        .db
        .latest_compliance_period_report_manifest_hash(&org_id, &period_report_id)
        .await
    {
        Ok(value) => value,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, "Failed to load previous period report manifest hash");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    let generated_at = chrono::Utc::now().timestamp_millis();
    let preimage =
        build_compliance_period_report_manifest_payload(&CompliancePeriodReportManifestPayloadInput {
            manifest_id: "pending",
            generated_at,
            generated_by: &auth_user.client_id,
            period_report: &period_report,
            period_artifact: &period_artifact,
            previous_manifest_hash: previous_manifest_hash.as_deref(),
            manifest_hash: None,
            pdf_exports: &pdf_exports,
            access_logs: &access_logs,
        });
    let content_hash = compliance_review_package_hash(&preimage);
    let manifest_id = deterministic_compliance_period_report_manifest_id(
        &period_report.period_report_id,
        &auth_user.client_id,
        generated_at,
        previous_manifest_hash.as_deref(),
        &content_hash,
    );
    let mut artifact =
        build_compliance_period_report_manifest_payload(&CompliancePeriodReportManifestPayloadInput {
            manifest_id: &manifest_id,
            generated_at,
            generated_by: &auth_user.client_id,
            period_report: &period_report,
            period_artifact: &period_artifact,
            previous_manifest_hash: previous_manifest_hash.as_deref(),
            manifest_hash: None,
            pdf_exports: &pdf_exports,
            access_logs: &access_logs,
        });
    let manifest_hash = compliance_review_package_hash(&artifact);
    artifact =
        build_compliance_period_report_manifest_payload(&CompliancePeriodReportManifestPayloadInput {
            manifest_id: &manifest_id,
            generated_at,
            generated_by: &auth_user.client_id,
            period_report: &period_report,
            period_artifact: &period_artifact,
            previous_manifest_hash: previous_manifest_hash.as_deref(),
            manifest_hash: Some(&manifest_hash),
            pdf_exports: &pdf_exports,
            access_logs: &access_logs,
        });

    match state
        .db
        .create_compliance_period_report_provenance_manifest(
            &CreateCompliancePeriodReportProvenanceManifestInput {
                manifest_id: &manifest_id,
                org_id: &org_id,
                period_report_id: &period_report.period_report_id,
                generated_by_user_id: &auth_user.client_id,
                manifest_hash: &manifest_hash,
                previous_manifest_hash: previous_manifest_hash.as_deref(),
                signature_algorithm: COMPLIANCE_PERIOD_REPORT_MANIFEST_SIGNATURE_ALGORITHM,
                payload_json_redacted: &artifact,
            },
        )
        .await
    {
        Ok(manifest) => {
            append_period_report_access_log(
                &state,
                PeriodReportAccessLogInput {
                    org_id: &org_id,
                    period_report_id: &period_report_id,
                    actor_client_id: &auth_user.client_id,
                    action: "manifest_created",
                    artifact_type: "manifest",
                    artifact_id: Some(&manifest.manifest_id),
                    artifact_hash: Some(&manifest.manifest_hash),
                    metadata: json!({
                        "previous_manifest_hash": manifest.previous_manifest_hash,
                        "signature_algorithm": manifest.signature_algorithm,
                        "source_period_report_hash": period_report.artifact_hash,
                        "pdf_export_count": pdf_exports.len(),
                        "access_log_count": access_logs.len(),
                        "compliance_claim": false,
                        "regulatory_claim": false,
                        "certification": false,
                        "agent_governance_required": false,
                        "source_period_report_artifact_mutated": false
                    }),
                },
            )
            .await;
            let audit_entry = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "compliance_period_report.provenance_manifest_created".to_string(),
                target_type: Some("compliance_period_report".to_string()),
                target_id: Some(period_report.period_report_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "period_report_id": period_report.period_report_id,
                    "manifest_id": manifest.manifest_id,
                    "manifest_hash": manifest.manifest_hash,
                    "previous_manifest_hash": manifest.previous_manifest_hash,
                    "signature_algorithm": manifest.signature_algorithm,
                    "source_period_report_artifact_hash": period_report.artifact_hash,
                    "source_period_report_artifact_mutated": false,
                    "compliance_claim": false,
                    "regulatory_claim": false,
                    "certification": false,
                    "agent_governance_required": false
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write period report manifest audit log: {}", e);
            }
            (
                StatusCode::CREATED,
                Json(compliance_period_report_manifest_response(manifest, artifact)),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, "Failed to create period report provenance manifest");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn download_compliance_period_report_provenance_manifest(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path((period_report_id, manifest_id)): Path<(String, String)>,
    Query(mut query): Query<CompliancePeriodReportProvenanceManifestQuery>,
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
    let manifest_id = match normalize_compliance_period_report_manifest_id(&manifest_id) {
        Ok(value) => value,
        Err(error) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response();
        }
    };
    normalize_compliance_period_report_manifest_request(&mut query.org_name);
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
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, "Failed to authorize period report manifest download");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    }

    match state
        .db
        .get_compliance_period_report_manifest_payload(&org_id, &period_report_id, &manifest_id)
        .await
    {
        Ok(Some(payload)) => {
            let manifest_hash = payload
                .get("hash_chain")
                .and_then(|value| value.get("manifest_hash"))
                .and_then(|value| value.as_str())
                .map(ToString::to_string);
            append_period_report_access_log(
                &state,
                PeriodReportAccessLogInput {
                    org_id: &org_id,
                    period_report_id: &period_report_id,
                    actor_client_id: &auth_user.client_id,
                    action: "manifest_downloaded",
                    artifact_type: "manifest",
                    artifact_id: Some(&manifest_id),
                    artifact_hash: manifest_hash.as_deref(),
                    metadata: json!({
                        "manifest_id": manifest_id.clone(),
                        "manifest_hash": manifest_hash,
                        "compliance_claim": false,
                        "regulatory_claim": false,
                        "certification": false,
                        "agent_governance_required": false
                    }),
                },
            )
            .await;
            (StatusCode::OK, Json(payload)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compliance period report provenance manifest not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, manifest_id = %manifest_id, "Failed to download period report provenance manifest");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
