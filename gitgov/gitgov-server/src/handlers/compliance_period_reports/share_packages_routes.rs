pub async fn create_compliance_period_report_share_package(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(period_report_id): Path<String>,
    Json(mut payload): Json<CompliancePeriodReportSharePackageRequest>,
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
    if let Err(errors) = normalize_compliance_period_report_share_package_request(&mut payload) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid period report share package request", "details": errors })),
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

    let (period_report, period_artifact) = match state
        .db
        .get_compliance_period_report_with_payload(&org_id, &period_report_id, None)
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
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, "Failed to load period report before share package");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    if period_report.retention_status == "archived" || period_report.archived_at.is_some() {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Archived Period Compliance Reports cannot create new share packages",
                "code": "period_report_archived"
            })),
        )
            .into_response();
    }
    if period_report.review_status != "reviewed" {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Period Compliance Report must be reviewed before share package creation",
                "code": "period_report_not_reviewed"
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
            Json(json!({ "error": "Period Compliance Report claims are not eligible for share packaging" })),
        )
            .into_response();
    }

    let pdf_export = match state
        .db
        .get_latest_compliance_period_report_pdf_export(&org_id, &period_report_id)
        .await
    {
        Ok(Some(value)) => value,
        Ok(None) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "Period Compliance Report PDF export is required before share package creation",
                    "code": "period_report_pdf_required"
                })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, "Failed to load period report PDF before share package");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    let manifest = match state
        .db
        .get_latest_compliance_period_report_manifest(&org_id, &period_report_id)
        .await
    {
        Ok(Some(value)) => value,
        Ok(None) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "Period Compliance Report provenance manifest is required before share package creation",
                    "code": "period_report_manifest_required"
                })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, "Failed to load period report manifest before share package");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    let share_package_id = deterministic_compliance_period_report_share_package_id();
    let created_at = chrono::Utc::now().timestamp_millis();
    let no_claims_snapshot = json!({
        "compliance_claim": false,
        "regulatory_claim": false,
        "certification": false,
        "compliance_score": false,
        "requires_auditor_review": true,
        "official_regulatory_mapping": false,
        "legal_attestation": false,
        "agent_governance_required": false
    });
    let source_hashes = json!({
        "period_report_artifact_hash": period_report.artifact_hash,
        "pdf_artifact_hash": pdf_export.pdf_artifact_hash,
        "manifest_hash": manifest.manifest_hash,
        "period_report_source_hashes": period_artifact.get("source_hashes")
    });
    let review_snapshot = json!({
        "review_status": period_report.review_status,
        "reviewed_by_user_id": period_report.reviewed_by_user_id,
        "reviewed_at": period_report.reviewed_at,
        "has_review_notes_safe": period_report.review_notes_safe.is_some()
    });
    let retention_snapshot = json!({
        "retention_status": period_report.retention_status,
        "retention_until": period_report.retention_until,
        "download_count": period_report.download_count,
        "last_downloaded_at": period_report.last_downloaded_at,
        "archived_at": period_report.archived_at
    });
    let preimage =
        build_period_report_share_package_artifact(PeriodReportSharePackageArtifactInput {
            share_package_id: &share_package_id,
            created_at,
            created_by: &auth_user.client_id,
            period_report: &period_report,
            period_artifact: &period_artifact,
            pdf_export: &pdf_export,
            manifest: &manifest,
            no_claims_snapshot: &no_claims_snapshot,
            source_hashes: &source_hashes,
            review_snapshot: &review_snapshot,
            retention_snapshot: &retention_snapshot,
            artifact_hash: None,
        });
    let artifact_hash = compliance_review_package_hash(&preimage);
    let artifact =
        build_period_report_share_package_artifact(PeriodReportSharePackageArtifactInput {
            share_package_id: &share_package_id,
            created_at,
            created_by: &auth_user.client_id,
            period_report: &period_report,
            period_artifact: &period_artifact,
            pdf_export: &pdf_export,
            manifest: &manifest,
            no_claims_snapshot: &no_claims_snapshot,
            source_hashes: &source_hashes,
            review_snapshot: &review_snapshot,
            retention_snapshot: &retention_snapshot,
            artifact_hash: Some(&artifact_hash),
        });

    match state
        .db
        .create_compliance_period_report_share_package(
            &CreateCompliancePeriodReportSharePackageInput {
                share_package_id: &share_package_id,
                org_id: &org_id,
                period_report_id: &period_report.period_report_id,
                created_by_user_id: &auth_user.client_id,
                package_format: "json_bundle",
                artifact_hash: &artifact_hash,
                payload_json_redacted: &artifact,
                period_report_artifact_hash: &period_report.artifact_hash,
                pdf_export_id: &pdf_export.pdf_export_id,
                pdf_artifact_hash: &pdf_export.pdf_artifact_hash,
                manifest_id: &manifest.manifest_id,
                manifest_hash: &manifest.manifest_hash,
                no_claims_snapshot: &no_claims_snapshot,
                source_hashes: &source_hashes,
                review_snapshot: &review_snapshot,
                retention_snapshot: &retention_snapshot,
            },
        )
        .await
    {
        Ok(share_package) => {
            append_period_report_access_log(
                &state,
                PeriodReportAccessLogInput {
                    org_id: &org_id,
                    period_report_id: &period_report_id,
                    actor_client_id: &auth_user.client_id,
                    action: "share_package_created",
                    artifact_type: "share_package",
                    artifact_id: Some(&share_package.share_package_id),
                    artifact_hash: Some(&share_package.artifact_hash),
                    metadata: json!({
                        "pdf_export_id": share_package.pdf_export_id,
                        "manifest_id": share_package.manifest_id,
                        "manual_sharing_only": true,
                        "public_link": false,
                        "email_delivery": false,
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
                action: "compliance_period_report.share_package_created".to_string(),
                target_type: Some("compliance_period_report_share_package".to_string()),
                target_id: Some(share_package.share_package_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "period_report_id": period_report.period_report_id,
                    "share_package_id": share_package.share_package_id,
                    "artifact_hash": share_package.artifact_hash,
                    "pdf_export_id": share_package.pdf_export_id,
                    "manifest_id": share_package.manifest_id,
                    "compliance_claim": false,
                    "regulatory_claim": false,
                    "certification": false,
                    "agent_governance_required": false
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit_entry).await {
                tracing::warn!("Failed to write period report share package audit log: {}", e);
            }
            (
                StatusCode::CREATED,
                Json(compliance_period_report_share_package_response(
                    share_package,
                    Some(artifact),
                )),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, "Failed to create period report share package");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn list_compliance_period_report_share_packages(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(period_report_id): Path<String>,
    Query(mut query): Query<CompliancePeriodReportSharePackageQuery>,
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
    let limit = match normalize_compliance_period_report_share_package_query(&mut query) {
        Ok(value) => value,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid period report share package query", "details": errors })),
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
    if let Err(resp) =
        authorize_period_report_for_share_package(&state, &auth_user, &org_id, &period_report_id)
            .await
    {
        return resp;
    }
    match state
        .db
        .list_compliance_period_report_share_packages(
            &ListCompliancePeriodReportSharePackagesInput {
                org_id: &org_id,
                period_report_id: &period_report_id,
                status: query.status.as_deref(),
                limit,
            },
        )
        .await
    {
        Ok(items) => (
            StatusCode::OK,
            Json(CompliancePeriodReportSharePackageListResponse {
                count: items.len(),
                items,
                limit,
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, period_report_id = %period_report_id, "Failed to list period report share packages");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn get_compliance_period_report_share_package(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(share_package_id): Path<String>,
    Query(mut query): Query<CompliancePeriodReportSharePackageQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let share_package_id = match normalize_compliance_period_report_share_package_id(&share_package_id) {
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
    let share_package = match state
        .db
        .get_compliance_period_report_share_package(&org_id, &share_package_id)
        .await
    {
        Ok(Some(value)) => value,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Period report share package not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, share_package_id = %share_package_id, "Failed to load period report share package");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    if let Err(resp) = authorize_period_report_for_share_package(
        &state,
        &auth_user,
        &org_id,
        &share_package.period_report_id,
    )
    .await
    {
        return resp;
    }
    let artifact = match state
        .db
        .get_compliance_period_report_share_package_payload(&org_id, &share_package_id)
        .await
    {
        Ok(value) => value,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, share_package_id = %share_package_id, "Failed to load period report share package payload");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    (
        StatusCode::OK,
        Json(compliance_period_report_share_package_response(
            share_package,
            artifact,
        )),
    )
        .into_response()
}

pub async fn download_compliance_period_report_share_package(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(share_package_id): Path<String>,
    Query(mut query): Query<CompliancePeriodReportSharePackageQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let share_package_id = match normalize_compliance_period_report_share_package_id(&share_package_id) {
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
    let current = match state
        .db
        .get_compliance_period_report_share_package(&org_id, &share_package_id)
        .await
    {
        Ok(Some(value)) => value,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Period report share package not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, share_package_id = %share_package_id, "Failed to load period report share package before download");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    if current.status == "revoked" {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Period report share package has been revoked",
                "code": "share_package_revoked"
            })),
        )
            .into_response();
    }
    if let Err(resp) = authorize_period_report_for_share_package(
        &state,
        &auth_user,
        &org_id,
        &current.period_report_id,
    )
    .await
    {
        return resp;
    }
    match state
        .db
        .download_compliance_period_report_share_package(&org_id, &share_package_id)
        .await
    {
        Ok(Some((share_package, artifact))) => {
            append_period_report_access_log(
                &state,
                PeriodReportAccessLogInput {
                    org_id: &org_id,
                    period_report_id: &share_package.period_report_id,
                    actor_client_id: &auth_user.client_id,
                    action: "share_package_downloaded",
                    artifact_type: "share_package",
                    artifact_id: Some(&share_package.share_package_id),
                    artifact_hash: Some(&share_package.artifact_hash),
                    metadata: json!({
                        "download_count": share_package.download_count,
                        "last_downloaded_at": share_package.last_downloaded_at,
                        "manual_sharing_only": true,
                        "public_link": false,
                        "email_delivery": false,
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
                    "attachment; filename=\"gitgov-period-report-share-package-{}.json\"",
                    share_package.share_package_id
                ))
                .unwrap_or_else(|_| axum::http::HeaderValue::from_static("attachment")),
            );
            headers.insert(
                axum::http::HeaderName::from_static("x-gitgov-artifact-hash"),
                axum::http::HeaderValue::from_str(&share_package.artifact_hash)
                    .unwrap_or_else(|_| axum::http::HeaderValue::from_static("invalid")),
            );
            (headers, Json(artifact)).into_response()
        }
        Ok(None) => (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Period report share package has been revoked",
                "code": "share_package_revoked"
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, share_package_id = %share_package_id, "Failed to download period report share package");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn revoke_compliance_period_report_share_package(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(share_package_id): Path<String>,
    Json(mut payload): Json<CompliancePeriodReportSharePackageRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    let share_package_id = match normalize_compliance_period_report_share_package_id(&share_package_id) {
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
        .get_compliance_period_report_share_package(&org_id, &share_package_id)
        .await
    {
        Ok(Some(value)) => value,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Period report share package not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, share_package_id = %share_package_id, "Failed to load period report share package before revoke");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    match state
        .db
        .revoke_compliance_period_report_share_package(
            &RevokeCompliancePeriodReportSharePackageInput {
                org_id: &org_id,
                share_package_id: &share_package_id,
                revoked_by_user_id: &auth_user.client_id,
            },
        )
        .await
    {
        Ok(Some(share_package)) => {
            append_period_report_access_log(
                &state,
                PeriodReportAccessLogInput {
                    org_id: &org_id,
                    period_report_id: &existing.period_report_id,
                    actor_client_id: &auth_user.client_id,
                    action: "share_package_revoked",
                    artifact_type: "share_package",
                    artifact_id: Some(&share_package.share_package_id),
                    artifact_hash: Some(&share_package.artifact_hash),
                    metadata: json!({
                        "revoked_at": share_package.revoked_at,
                        "revoked_by_user_id": share_package.revoked_by_user_id,
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
                Json(compliance_period_report_share_package_response(
                    share_package,
                    None,
                )),
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Period report share package not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, share_package_id = %share_package_id, "Failed to revoke period report share package");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
