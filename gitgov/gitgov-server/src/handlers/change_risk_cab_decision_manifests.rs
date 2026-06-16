const CHANGE_RISK_CAB_DECISION_MANIFEST_SCHEMA_VERSION: &str =
    "gitgov_change_risk_cab_decision_manifest.v1";

fn deterministic_change_risk_cab_decision_manifest_id(
    packet_id: &str,
    created_by: &str,
    created_at: i64,
    content_hash: &str,
) -> String {
    let content = format!(
        "{CHANGE_RISK_CAB_DECISION_MANIFEST_SCHEMA_VERSION}:{packet_id}:{created_by}:{created_at}:{content_hash}"
    );
    let digest = format!("{:x}", Sha256::digest(content.as_bytes()));
    format!("crcabdm_{}", &digest[..32])
}

fn normalize_change_risk_cab_decision_manifest_id(
    value: &str,
) -> Result<String, &'static str> {
    let trimmed = value.trim();
    let cutoff = ["?", "#", "%3F", "%3f", "%23"]
        .iter()
        .filter_map(|marker| trimmed.find(marker))
        .min()
        .unwrap_or(trimmed.len());
    let normalized = trimmed[..cutoff].trim_end_matches('/').to_string();
    if normalized.starts_with("crcabdm_") && normalized.len() == 40 {
        Ok(normalized)
    } else {
        Err("manifest_id must be a valid crcabdm_ identifier")
    }
}

fn normalize_change_risk_cab_decision_manifest_query(
    query: &mut ChangeRiskCabDecisionManifestQuery,
) -> Result<(i64, i64), Vec<String>> {
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
    let offset = query.offset.unwrap_or(0).max(0);
    if errors.is_empty() {
        Ok((limit, offset))
    } else {
        Err(errors)
    }
}

fn normalize_change_risk_cab_decision_manifest_request(
    payload: &mut ChangeRiskCabDecisionManifestRequest,
) {
    normalize_release_approval_optional_text(&mut payload.org_name);
}

fn change_risk_cab_decision_manifest_response(
    manifest: ChangeRiskCabDecisionManifestRecord,
    artifact: Option<serde_json::Value>,
) -> ChangeRiskCabDecisionManifestResponse {
    let download_url = format!(
        "/change-risk/cab-decision-manifests/{}/download",
        manifest.manifest_id
    );
    ChangeRiskCabDecisionManifestResponse {
        manifest,
        download_url,
        artifact,
    }
}

fn summarize_cab_manifest_evaluations(packet_artifact: &serde_json::Value) -> serde_json::Value {
    let evaluations = packet_artifact
        .get("evaluations")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    let items = evaluations
        .iter()
        .map(|evaluation| {
            json!({
                "evaluation_id": evaluation.get("evaluation_id"),
                "repository_full_name": evaluation.get("repository_full_name"),
                "branch": evaluation.get("branch"),
                "environment": evaluation.get("environment"),
                "risk_level": evaluation.get("risk_level"),
                "ruleset_version": evaluation.get("ruleset_version"),
                "trace_hash": evaluation.get("trace_hash"),
                "review_status": evaluation.get("review").and_then(|review| review.get("review_status")),
                "claims": evaluation.get("claims")
            })
        })
        .collect::<Vec<_>>();
    let trace_hashes = evaluations
        .iter()
        .filter_map(|evaluation| {
            evaluation
                .get("trace_hash")
                .and_then(|value| value.as_str())
                .map(ToString::to_string)
        })
        .collect::<Vec<_>>();
    json!({
        "count": items.len(),
        "items": items,
        "trace_hashes": trace_hashes
    })
}

fn build_change_risk_cab_decision_manifest_artifact(
    manifest_id: &str,
    generated_at: i64,
    generated_by: &str,
    packet: &ChangeRiskCabPacketRecord,
    packet_artifact: &serde_json::Value,
    manifest_hash: Option<&str>,
) -> serde_json::Value {
    let included_evaluations = summarize_cab_manifest_evaluations(packet_artifact);
    json!({
        "schema_version": CHANGE_RISK_CAB_DECISION_MANIFEST_SCHEMA_VERSION,
        "manifest_id": manifest_id,
        "generated_at": generated_at,
        "generated_by_user_id": generated_by,
        "cab_packet": {
            "packet_id": packet.packet_id,
            "cab_packet_hash": packet.artifact_hash,
            "status": packet.status,
            "name": packet.name,
            "created_by_user_id": packet.created_by_user_id,
            "created_at": packet.created_at,
            "download_count": packet.download_count,
            "downloaded_at": packet.downloaded_at
        },
        "review": {
            "review_status": packet.review_status,
            "reviewed_by_user_id": packet.reviewed_by_user_id,
            "reviewed_at": packet.reviewed_at,
            "has_review_notes_safe": packet.review_notes_safe.is_some(),
            "review_notes_safe": packet.review_notes_safe,
            "mitigation_notes_safe": packet.mitigation_notes_safe,
            "decision_reason_safe": packet.decision_reason_safe,
            "follow_up_required": packet.follow_up_required,
            "follow_up_owner_safe": packet.follow_up_owner_safe,
            "review_updated_at": packet.review_updated_at
        },
        "risk_summary": packet_artifact.get("summary"),
        "included_evaluations": included_evaluations,
        "source_packet_verification": packet_artifact.get("verification"),
        "hash_chain": {
            "subject_type": "change_risk_cab_packet",
            "cab_packet_id": packet.packet_id,
            "cab_packet_hash": packet.artifact_hash,
            "manifest_hash": manifest_hash
        },
        "claims": {
            "advisory_only": true,
            "manual_evidence_only": true,
            "llm_used": false,
            "agent_governance_used": false,
            "compliance_claim": false,
            "certification": false,
            "legal_attestation": false,
            "regulatory_claim": false,
            "compliance_score": false
        },
        "audit_metadata": {
            "artifact_redacted": true,
            "manual_on_demand": true,
            "manual_cab_decision_manifest_only": true,
            "enforcement": false,
            "release_blocking": false,
            "deployment_execution": false,
            "provider_mutation": false,
            "repository_mutation": false,
            "source_cab_packet_mutated": false,
            "source_evaluations_mutated": false,
            "agent_governance_required": false,
            "llm_decision": false,
            "public_link": false,
            "email_delivery": false,
            "scheduler": false,
            "pdf_export": false,
            "docx_export": false
        }
    })
}

async fn audit_cab_decision_manifest(
    state: &Arc<AppState>,
    auth_user: &AuthUser,
    action: &str,
    manifest: &ChangeRiskCabDecisionManifestRecord,
    metadata: serde_json::Value,
) {
    let audit = AdminAuditLogEntry {
        id: Uuid::new_v4().to_string(),
        actor_client_id: auth_user.client_id.clone(),
        action: action.to_string(),
        target_type: Some("change_risk_cab_decision_manifest".to_string()),
        target_id: Some(manifest.manifest_id.clone()),
        metadata,
        created_at: chrono::Utc::now().timestamp_millis(),
    };
    if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
        tracing::warn!(error = %e, action, "Failed to write CAB decision manifest audit log");
    }
}

pub async fn create_change_risk_cab_decision_manifest(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(packet_id): Path<String>,
    Json(mut payload): Json<ChangeRiskCabDecisionManifestRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    let packet_id = match normalize_change_risk_cab_packet_id(&packet_id) {
        Ok(value) => value,
        Err(error) => return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response(),
    };
    normalize_change_risk_cab_decision_manifest_request(&mut payload);
    let org_id = match resolve_change_risk_org(&state, &auth_user, payload.org_name.as_deref()).await
    {
        Ok(org_id) => org_id,
        Err(response) => return response.into_response(),
    };
    let packet = match state.db.get_change_risk_cab_packet(&org_id, &packet_id).await {
        Ok(Some(packet)) => packet,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Change Risk CAB packet not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, packet_id = %packet_id, "Failed to load CAB packet before decision manifest");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    if packet.status != "active" {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Change Risk CAB packet must be active before a decision manifest can be created",
                "code": "change_risk_cab_packet_archived"
            })),
        )
            .into_response();
    }
    if packet.review_status == CHANGE_RISK_CAB_REVIEW_PENDING {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Change Risk CAB packet must have a manual disposition before a decision manifest can be created",
                "code": "change_risk_cab_packet_pending_review"
            })),
        )
            .into_response();
    }
    let packet_artifact = match state
        .db
        .get_change_risk_cab_packet_artifact(&org_id, &packet_id)
        .await
    {
        Ok(Some(value)) => value,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Change Risk CAB packet artifact not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, packet_id = %packet_id, "Failed to load CAB packet artifact before decision manifest");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };

    let generated_at = chrono::Utc::now().timestamp_millis();
    let preimage = build_change_risk_cab_decision_manifest_artifact(
        "pending",
        generated_at,
        &auth_user.client_id,
        &packet,
        &packet_artifact,
        None,
    );
    let content_hash = change_risk_cab_packet_hash(&preimage);
    let manifest_id = deterministic_change_risk_cab_decision_manifest_id(
        &packet.packet_id,
        &auth_user.client_id,
        generated_at,
        &content_hash,
    );
    let artifact_without_hash = build_change_risk_cab_decision_manifest_artifact(
        &manifest_id,
        generated_at,
        &auth_user.client_id,
        &packet,
        &packet_artifact,
        None,
    );
    let manifest_hash = change_risk_cab_packet_hash(&artifact_without_hash);
    let artifact = build_change_risk_cab_decision_manifest_artifact(
        &manifest_id,
        generated_at,
        &auth_user.client_id,
        &packet,
        &packet_artifact,
        Some(&manifest_hash),
    );

    match state
        .db
        .create_change_risk_cab_decision_manifest(
            &CreateChangeRiskCabDecisionManifestInput {
                manifest_id: &manifest_id,
                org_id: &org_id,
                cab_packet_id: &packet.packet_id,
                cab_packet_hash: &packet.artifact_hash,
                manifest_hash: &manifest_hash,
                manifest_json: &artifact,
                review_status_snapshot: &packet.review_status,
                reviewed_by_user_id: packet.reviewed_by_user_id.as_deref(),
                reviewed_at: packet.reviewed_at,
                created_by_user_id: &auth_user.client_id,
            },
        )
        .await
    {
        Ok(manifest) => {
            audit_cab_decision_manifest(
                &state,
                &auth_user,
                "cab_decision_manifest_created",
                &manifest,
                json!({
                    "org_id": org_id,
                    "manifest_id": manifest.manifest_id,
                    "manifest_hash": manifest.manifest_hash,
                    "cab_packet_id": manifest.cab_packet_id,
                    "cab_packet_hash": manifest.cab_packet_hash,
                    "review_status": manifest.review_status_snapshot,
                    "advisory_only": true,
                    "llm_used": false,
                    "agent_governance_used": false,
                    "release_blocking": false,
                    "deployment_execution": false,
                    "source_cab_packet_mutated": false,
                    "source_evaluations_mutated": false,
                    "compliance_claim": false,
                    "certification": false
                }),
            )
            .await;
            (
                StatusCode::CREATED,
                Json(change_risk_cab_decision_manifest_response(
                    manifest,
                    Some(artifact),
                )),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, packet_id = %packet_id, "Failed to create CAB decision manifest");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn list_change_risk_cab_decision_manifests(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(packet_id): Path<String>,
    Query(mut query): Query<ChangeRiskCabDecisionManifestQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let packet_id = match normalize_change_risk_cab_packet_id(&packet_id) {
        Ok(value) => value,
        Err(error) => return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response(),
    };
    let (limit, offset) = match normalize_change_risk_cab_decision_manifest_query(&mut query) {
        Ok(values) => values,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid CAB decision manifest query", "details": errors })),
            )
                .into_response();
        }
    };
    let org_id = match resolve_change_risk_org(&state, &auth_user, query.org_name.as_deref()).await
    {
        Ok(org_id) => org_id,
        Err(response) => return response.into_response(),
    };
    match state.db.get_change_risk_cab_packet(&org_id, &packet_id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Change Risk CAB packet not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, packet_id = %packet_id, "Failed to authorize CAB decision manifest list");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    }
    match state
        .db
        .list_change_risk_cab_decision_manifests(&ListChangeRiskCabDecisionManifestsInput {
            org_id: &org_id,
            cab_packet_id: &packet_id,
            status: query.status.as_deref(),
            limit,
            offset,
        })
        .await
    {
        Ok((items, total)) => (
            StatusCode::OK,
            Json(ChangeRiskCabDecisionManifestListResponse {
                items,
                total,
                limit,
                offset,
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, packet_id = %packet_id, "Failed to list CAB decision manifests");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn get_change_risk_cab_decision_manifest(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(manifest_id): Path<String>,
    Query(mut query): Query<ChangeRiskCabDecisionManifestQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let manifest_id = match normalize_change_risk_cab_decision_manifest_id(&manifest_id) {
        Ok(value) => value,
        Err(error) => return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response(),
    };
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_change_risk_org(&state, &auth_user, query.org_name.as_deref()).await
    {
        Ok(org_id) => org_id,
        Err(response) => return response.into_response(),
    };
    let manifest = match state
        .db
        .get_change_risk_cab_decision_manifest(&org_id, &manifest_id)
        .await
    {
        Ok(Some(value)) => value,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Change Risk CAB decision manifest not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, manifest_id = %manifest_id, "Failed to get CAB decision manifest");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    let artifact = match state
        .db
        .get_change_risk_cab_decision_manifest_artifact(&org_id, &manifest_id)
        .await
    {
        Ok(value) => value,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, manifest_id = %manifest_id, "Failed to get CAB decision manifest artifact");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    (
        StatusCode::OK,
        Json(change_risk_cab_decision_manifest_response(
            manifest, artifact,
        )),
    )
        .into_response()
}

pub async fn download_change_risk_cab_decision_manifest(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(manifest_id): Path<String>,
    Query(mut query): Query<ChangeRiskCabDecisionManifestQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let manifest_id = match normalize_change_risk_cab_decision_manifest_id(&manifest_id) {
        Ok(value) => value,
        Err(error) => return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response(),
    };
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_change_risk_org(&state, &auth_user, query.org_name.as_deref()).await
    {
        Ok(org_id) => org_id,
        Err(response) => return response.into_response(),
    };
    match state
        .db
        .get_change_risk_cab_decision_manifest(&org_id, &manifest_id)
        .await
    {
        Ok(Some(existing)) => {
            if existing.status == "revoked" {
                return (
                    StatusCode::CONFLICT,
                    Json(json!({
                        "error": "Change Risk CAB decision manifest has been revoked",
                        "code": "change_risk_cab_decision_manifest_revoked"
                    })),
                )
                    .into_response();
            }
        }
        Ok(None) => {}
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, manifest_id = %manifest_id, "Failed to precheck CAB decision manifest before download");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    }
    match state
        .db
        .download_change_risk_cab_decision_manifest(&org_id, &manifest_id)
        .await
    {
        Ok(Some((manifest, artifact))) => {
            audit_cab_decision_manifest(
                &state,
                &auth_user,
                "cab_decision_manifest_downloaded",
                &manifest,
                json!({
                    "org_id": org_id,
                    "manifest_id": manifest.manifest_id,
                    "manifest_hash": manifest.manifest_hash,
                    "cab_packet_id": manifest.cab_packet_id,
                    "cab_packet_hash": manifest.cab_packet_hash,
                    "download_count": manifest.download_count,
                    "advisory_only": true,
                    "llm_used": false,
                    "agent_governance_used": false,
                    "compliance_claim": false,
                    "certification": false
                }),
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
                    "attachment; filename=\"gitgov-change-risk-cab-decision-manifest-{}.json\"",
                    manifest.manifest_id
                ))
                .unwrap_or_else(|_| axum::http::HeaderValue::from_static("attachment")),
            );
            headers.insert(
                axum::http::HeaderName::from_static("x-gitgov-artifact-hash"),
                axum::http::HeaderValue::from_str(&manifest.manifest_hash)
                    .unwrap_or_else(|_| axum::http::HeaderValue::from_static("invalid")),
            );
            (headers, Json(artifact)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Change Risk CAB decision manifest not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, manifest_id = %manifest_id, "Failed to download CAB decision manifest");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn revoke_change_risk_cab_decision_manifest(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(manifest_id): Path<String>,
    Json(mut payload): Json<ChangeRiskCabDecisionManifestRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    let manifest_id = match normalize_change_risk_cab_decision_manifest_id(&manifest_id) {
        Ok(value) => value,
        Err(error) => return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response(),
    };
    normalize_change_risk_cab_decision_manifest_request(&mut payload);
    let org_id = match resolve_change_risk_org(&state, &auth_user, payload.org_name.as_deref()).await
    {
        Ok(org_id) => org_id,
        Err(response) => return response.into_response(),
    };
    match state
        .db
        .revoke_change_risk_cab_decision_manifest(&RevokeChangeRiskCabDecisionManifestInput {
            org_id: &org_id,
            manifest_id: &manifest_id,
            revoked_by_user_id: &auth_user.client_id,
        })
        .await
    {
        Ok(Some(manifest)) => {
            audit_cab_decision_manifest(
                &state,
                &auth_user,
                "cab_decision_manifest_revoked",
                &manifest,
                json!({
                    "org_id": org_id,
                    "manifest_id": manifest.manifest_id,
                    "manifest_hash": manifest.manifest_hash,
                    "cab_packet_id": manifest.cab_packet_id,
                    "cab_packet_hash": manifest.cab_packet_hash,
                    "revoked_at": manifest.revoked_at,
                    "revoked_by_user_id": manifest.revoked_by_user_id,
                    "source_cab_packet_mutated": false,
                    "source_evaluations_mutated": false,
                    "deployment_execution": false,
                    "agent_governance_used": false,
                    "compliance_claim": false,
                    "certification": false
                }),
            )
            .await;
            (
                StatusCode::OK,
                Json(change_risk_cab_decision_manifest_response(
                    manifest, None,
                )),
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Change Risk CAB decision manifest not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, manifest_id = %manifest_id, "Failed to revoke CAB decision manifest");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod change_risk_cab_decision_manifest_tests {
    use super::normalize_change_risk_cab_decision_manifest_id;

    #[test]
    fn normalizes_terminal_manifest_id_when_query_or_fragment_is_attached() {
        let manifest_id = "crcabdm_cf376cfc585cca7d8174c35f2eb03a6b";

        assert_eq!(
            normalize_change_risk_cab_decision_manifest_id(manifest_id).unwrap(),
            manifest_id
        );
        assert_eq!(
            normalize_change_risk_cab_decision_manifest_id(&format!(
                "{manifest_id}?org_name=yohandry10"
            ))
            .unwrap(),
            manifest_id
        );
        assert_eq!(
            normalize_change_risk_cab_decision_manifest_id(&format!(
                "{manifest_id}%3Forg_name=yohandry10"
            ))
            .unwrap(),
            manifest_id
        );
        assert_eq!(
            normalize_change_risk_cab_decision_manifest_id(&format!("{manifest_id}#download"))
                .unwrap(),
            manifest_id
        );
    }
}
