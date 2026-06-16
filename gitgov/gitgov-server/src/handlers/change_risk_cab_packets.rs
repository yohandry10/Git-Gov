const CHANGE_RISK_CAB_PACKET_SCHEMA_VERSION: &str = "gitgov_change_risk_cab_packet.v1";
const CHANGE_RISK_CAB_PACKET_MAX_EVALUATIONS: usize = 100;

fn normalize_change_risk_cab_packet_id(value: &str) -> Result<String, &'static str> {
    let normalized = value.trim().to_string();
    if normalized.starts_with("crcab_") && normalized.len() == 38 {
        Ok(normalized)
    } else {
        Err("packet_id must be a valid crcab_ identifier")
    }
}

fn deterministic_change_risk_cab_packet_id() -> String {
    format!("crcab_{}", Uuid::new_v4().simple())
}

fn looks_like_change_risk_cab_packet_secret(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.contains("bearer ")
        || lowered.contains("authorization:")
        || lowered.contains("api_key")
        || lowered.contains("apikey")
        || lowered.contains("secret")
        || lowered.contains("password")
        || lowered.contains("ghp_")
        || lowered.contains("github_pat_")
        || lowered.contains("glpat-")
        || lowered.contains("sk-")
}

fn change_risk_cab_packet_hash(artifact: &serde_json::Value) -> String {
    let bytes =
        serde_json::to_vec(artifact).expect("change risk CAB packet artifact should serialize");
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn normalize_change_risk_cab_packet_query(
    query: &mut ChangeRiskCabPacketQuery,
) -> Result<(i64, i64), Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut query.org_name);
    query.status = match query.status.take() {
        Some(value) if !value.trim().is_empty() => {
            let normalized = value.trim().to_ascii_lowercase();
            if matches!(normalized.as_str(), "active" | "archived") {
                Some(normalized)
            } else {
                errors.push("status must be active or archived".to_string());
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

fn normalize_change_risk_cab_packet_request(
    payload: &mut ChangeRiskCabPacketRequest,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut payload.org_name);
    payload.name = payload.name.trim().to_string();
    normalize_release_approval_optional_text(&mut payload.repository_full_name);
    normalize_release_approval_optional_text(&mut payload.branch);
    normalize_release_approval_optional_text(&mut payload.environment);
    normalize_release_approval_optional_text(&mut payload.risk_level);
    normalize_release_approval_optional_text(&mut payload.review_status);

    if payload.name.is_empty()
        || payload.name.len() > 160
        || has_control_chars(&payload.name)
        || looks_like_change_risk_cab_packet_secret(&payload.name)
    {
        errors.push("name is required, must be at most 160 characters, and must not contain secret-looking values.".to_string());
    }
    if let Some(repo) = payload.repository_full_name.as_deref() {
        if !is_valid_release_approval_repo(repo) {
            errors.push("repository_full_name must look like owner/repo.".to_string());
        }
    }
    if let Some(environment) = payload.environment.as_mut() {
        *environment = environment.to_ascii_lowercase();
        if environment.len() > 80 || has_control_chars(environment) {
            errors.push("environment is invalid or too long.".to_string());
        }
    }
    if let Some(branch) = payload.branch.as_deref() {
        if branch.len() > CHANGE_RISK_TEXT_MAX_CHARS || has_control_chars(branch) {
            errors.push("branch is invalid or too long.".to_string());
        }
    }
    if let Some(risk_level) = payload.risk_level.as_mut() {
        *risk_level = risk_level.to_ascii_lowercase();
        if !CHANGE_RISK_LEVELS.contains(&risk_level.as_str()) {
            errors.push("risk_level must be low, medium, high, or unknown.".to_string());
        }
    }
    if let Some(review_status) = payload.review_status.as_mut() {
        *review_status = review_status.to_ascii_lowercase();
        if ![
            CHANGE_RISK_REVIEW_NEEDS_REVIEW,
            CHANGE_RISK_REVIEW_REVIEWED,
            CHANGE_RISK_REVIEW_ACCEPTED_RISK,
            CHANGE_RISK_REVIEW_NEEDS_MITIGATION,
            CHANGE_RISK_REVIEW_REJECTED,
        ]
        .contains(&review_status.as_str())
        {
            errors.push(
                "review_status must be needs_review, reviewed, accepted_risk, needs_mitigation, or rejected."
                    .to_string(),
            );
        }
    }
    payload
        .evaluation_ids
        .retain(|value| !value.trim().is_empty());
    for value in &mut payload.evaluation_ids {
        *value = value.trim().to_string();
        if !value.starts_with("cra_") || value.len() > 80 || has_control_chars(value) {
            errors.push("evaluation_ids must contain valid cra_ identifiers.".to_string());
            break;
        }
    }
    payload.evaluation_ids.sort();
    payload.evaluation_ids.dedup();
    if payload.evaluation_ids.len() > CHANGE_RISK_CAB_PACKET_MAX_EVALUATIONS {
        errors.push("evaluation_ids must contain at most 100 evaluations.".to_string());
    }

    payload
        .deployment_gate_ids
        .retain(|value| !value.trim().is_empty());
    for value in &mut payload.deployment_gate_ids {
        *value = value.trim().to_string();
        if value.len() > 120 || has_control_chars(value) {
            errors.push("deployment_gate_ids contains an invalid identifier.".to_string());
            break;
        }
    }
    payload.deployment_gate_ids.sort();
    payload.deployment_gate_ids.dedup();
    if payload.deployment_gate_ids.len() > CHANGE_RISK_CAB_PACKET_MAX_EVALUATIONS {
        errors.push("deployment_gate_ids must contain at most 100 identifiers.".to_string());
    }

    if let (Some(start), Some(end)) = (payload.date_range_start, payload.date_range_end) {
        if start > end {
            errors.push("date_range_start must be before date_range_end.".to_string());
        }
    }
    let has_filter = payload.repository_full_name.is_some()
        || payload.branch.is_some()
        || payload.environment.is_some()
        || payload.risk_level.is_some()
        || payload.review_status.is_some()
        || payload.date_range_start.is_some()
        || payload.date_range_end.is_some()
        || !payload.evaluation_ids.is_empty()
        || !payload.deployment_gate_ids.is_empty();
    if !has_filter {
        errors.push("At least one filter, evaluation_id, or deployment_gate_id is required.".to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn change_risk_cab_packet_response(
    packet: ChangeRiskCabPacketRecord,
    artifact: Option<serde_json::Value>,
) -> ChangeRiskCabPacketResponse {
    let download_url = format!("/change-risk/cab-packets/{}/download", packet.packet_id);
    ChangeRiskCabPacketResponse {
        packet,
        download_url,
        artifact,
    }
}

fn change_risk_cab_packet_filters(payload: &ChangeRiskCabPacketRequest) -> serde_json::Value {
    json!({
        "repository_full_name": payload.repository_full_name,
        "branch": payload.branch,
        "environment": payload.environment,
        "risk_level": payload.risk_level,
        "review_status": payload.review_status,
        "date_range_start": payload.date_range_start,
        "date_range_end": payload.date_range_end,
        "evaluation_ids": payload.evaluation_ids,
        "deployment_gate_ids": payload.deployment_gate_ids
    })
}

fn summarize_change_risk_cab_packet_evaluations(
    evaluations: &[ChangeRiskEvaluationRecord],
) -> serde_json::Value {
    let mut risk_counts = serde_json::Map::new();
    let mut review_counts = serde_json::Map::new();
    let mut triggered_rules = HashSet::new();
    let mut missing_evidence = HashSet::new();

    for evaluation in evaluations {
        let risk_count = risk_counts
            .entry(evaluation.risk_level.clone())
            .or_insert_with(|| json!(0));
        *risk_count = json!(risk_count.as_i64().unwrap_or(0) + 1);

        let review_count = review_counts
            .entry(evaluation.review_status.clone())
            .or_insert_with(|| json!(0));
        *review_count = json!(review_count.as_i64().unwrap_or(0) + 1);

        for rule in &evaluation.triggered_rules {
            triggered_rules.insert(rule.clone());
        }
        for evidence in &evaluation.missing_evidence {
            missing_evidence.insert(evidence.clone());
        }
    }

    let mut triggered_rules = triggered_rules.into_iter().collect::<Vec<_>>();
    triggered_rules.sort();
    let mut missing_evidence = missing_evidence.into_iter().collect::<Vec<_>>();
    missing_evidence.sort();

    json!({
        "total_evaluations": evaluations.len(),
        "risk_level_counts": risk_counts,
        "review_status_counts": review_counts,
        "triggered_rules": triggered_rules,
        "missing_evidence": missing_evidence
    })
}

fn build_change_risk_cab_packet_artifact(
    packet_id: &str,
    created_at: i64,
    created_by: &str,
    name: &str,
    filters: &serde_json::Value,
    evaluations: &[ChangeRiskEvaluationRecord],
    artifact_hash: Option<&str>,
) -> serde_json::Value {
    let evaluation_snapshots = evaluations
        .iter()
        .map(|record| {
            json!({
                "evaluation_id": record.evaluation_id,
                "repository_full_name": record.repository_full_name,
                "branch": record.branch,
                "environment": record.environment,
                "change_id": record.change_id,
                "deployment_gate_id": record.deployment_gate_id,
                "release_id": record.release_id,
                "commit_sha": record.commit_sha,
                "evidence_packet_hash": record.evidence_packet_hash,
                "risk_level": record.risk_level,
                "ruleset_version": record.ruleset_version,
                "risk_reasons": record.risk_reasons,
                "missing_evidence": record.missing_evidence,
                "blocking_gaps": record.blocking_gaps,
                "recommended_manual_actions": record.recommended_manual_actions,
                "triggered_rules": record.triggered_rules,
                "non_triggered_rules": record.non_triggered_rules,
                "trace_hash": record.trace_hash,
                "review": {
                    "review_status": record.review_status,
                    "reviewed_by_user_id": record.reviewed_by_user_id,
                    "reviewed_at": record.reviewed_at,
                    "has_review_notes_safe": record.review_notes_safe.is_some(),
                    "review_notes_safe": record.review_notes_safe,
                    "mitigation_notes_safe": record.mitigation_notes_safe,
                    "decision_reason_safe": record.decision_reason_safe,
                    "review_updated_at": record.review_updated_at
                },
                "claims": {
                    "advisory_only": record.advisory_only,
                    "llm_used": record.llm_used,
                    "agent_governance_used": record.agent_governance_used,
                    "compliance_claim": record.compliance_claim,
                    "certification": record.certification
                },
                "created_by": record.created_by,
                "created_at": record.created_at
            })
        })
        .collect::<Vec<_>>();

    json!({
        "schema_version": CHANGE_RISK_CAB_PACKET_SCHEMA_VERSION,
        "packet_id": packet_id,
        "name": name,
        "generated_at": created_at,
        "created_by_user_id": created_by,
        "purpose": "Manual CAB review packet for deterministic Change Risk evaluations.",
        "filters": filters,
        "summary": summarize_change_risk_cab_packet_evaluations(evaluations),
        "evaluations": evaluation_snapshots,
        "claims": {
            "advisory_only": true,
            "manual_review_packet": true,
            "compliance_claim": false,
            "regulatory_claim": false,
            "certification": false,
            "legal_attestation": false,
            "compliance_score": false
        },
        "verification": {
            "packet_hash": artifact_hash,
            "hash_algorithm": "sha256",
            "verify": [
                "Recompute this packet hash from the canonical JSON preimage.",
                "Confirm every evaluation belongs to the requested tenant and selected filters.",
                "Confirm every evaluation has advisory_only=true, llm_used=false, agent_governance_used=false, compliance_claim=false, and certification=false.",
                "Compare each trace_hash with the source Change Risk evaluation trace."
            ]
        },
        "audit_metadata": {
            "advisory_only": true,
            "manual_on_demand": true,
            "manual_cab_review_only": true,
            "llm_used": false,
            "agent_governance_used": false,
            "agent_governance_required": false,
            "enforcement": false,
            "release_blocking": false,
            "deployment_execution": false,
            "provider_mutation": false,
            "repository_mutation": false,
            "source_evaluations_mutated": false,
            "public_link": false,
            "email_delivery": false,
            "scheduler": false,
            "pdf_export": false,
            "docx_export": false
        }
    })
}

pub async fn create_change_risk_cab_packet(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<ChangeRiskCabPacketRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    if let Err(errors) = normalize_change_risk_cab_packet_request(&mut payload) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid Change Risk CAB packet request", "details": errors })),
        )
            .into_response();
    }
    let org_id = match resolve_change_risk_org(&state, &auth_user, payload.org_name.as_deref()).await
    {
        Ok(org_id) => org_id,
        Err(response) => return response.into_response(),
    };

    let evaluations = match state
        .db
        .list_change_risk_evaluations_for_cab_packet(
            &org_id,
            &ChangeRiskCabPacketEvaluationFilter {
                repository_full_name: payload.repository_full_name.as_deref(),
                branch: payload.branch.as_deref(),
                environment: payload.environment.as_deref(),
                risk_level: payload.risk_level.as_deref(),
                review_status: payload.review_status.as_deref(),
                date_range_start: payload.date_range_start,
                date_range_end: payload.date_range_end,
                evaluation_ids: &payload.evaluation_ids,
                deployment_gate_ids: &payload.deployment_gate_ids,
                limit: CHANGE_RISK_CAB_PACKET_MAX_EVALUATIONS as i64,
            },
        )
        .await
    {
        Ok(items) => items,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to load change risk evaluations for CAB packet");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    if evaluations.is_empty() {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "No Change Risk evaluations match the CAB packet request",
                "code": "change_risk_cab_packet_empty"
            })),
        )
            .into_response();
    }
    if !payload.evaluation_ids.is_empty() && evaluations.len() != payload.evaluation_ids.len() {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "One or more requested Change Risk evaluations were not found in scope",
                "code": "change_risk_cab_packet_evaluation_mismatch"
            })),
        )
            .into_response();
    }
    for evaluation in &evaluations {
        if !evaluation.advisory_only
            || evaluation.llm_used
            || evaluation.agent_governance_used
            || evaluation.compliance_claim
            || evaluation.certification
        {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Change Risk evaluation claims are not eligible for CAB packaging"
                })),
            )
                .into_response();
        }
    }

    let packet_id = deterministic_change_risk_cab_packet_id();
    let created_at = chrono::Utc::now().timestamp_millis();
    let filters = change_risk_cab_packet_filters(&payload);
    let evaluation_ids_json = json!(
        evaluations
            .iter()
            .map(|record| record.evaluation_id.clone())
            .collect::<Vec<_>>()
    );
    let preimage = build_change_risk_cab_packet_artifact(
        &packet_id,
        created_at,
        &auth_user.client_id,
        &payload.name,
        &filters,
        &evaluations,
        None,
    );
    let artifact_hash = change_risk_cab_packet_hash(&preimage);
    let artifact = build_change_risk_cab_packet_artifact(
        &packet_id,
        created_at,
        &auth_user.client_id,
        &payload.name,
        &filters,
        &evaluations,
        Some(&artifact_hash),
    );

    match state
        .db
        .create_change_risk_cab_packet(&CreateChangeRiskCabPacketInput {
            packet_id: &packet_id,
            org_id: &org_id,
            name: &payload.name,
            filters_json: &filters,
            evaluation_ids_json: &evaluation_ids_json,
            artifact_hash: &artifact_hash,
            artifact_json: &artifact,
            created_by_user_id: &auth_user.client_id,
        })
        .await
    {
        Ok(packet) => {
            let audit = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "change_risk_cab_packet_created".to_string(),
                target_type: Some("change_risk_cab_packet".to_string()),
                target_id: Some(packet.packet_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "packet_id": packet.packet_id,
                    "artifact_hash": packet.artifact_hash,
                    "evaluation_count": evaluations.len(),
                    "manual_cab_review_only": true,
                    "advisory_only": true,
                    "llm_used": false,
                    "agent_governance_used": false,
                    "compliance_claim": false,
                    "certification": false
                }),
                created_at,
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
                tracing::warn!(error = %e, "Failed to write admin audit log (change risk CAB packet create)");
            }
            (
                StatusCode::CREATED,
                Json(change_risk_cab_packet_response(packet, Some(artifact))),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to create change risk CAB packet");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn list_change_risk_cab_packets(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(mut query): Query<ChangeRiskCabPacketQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let (limit, offset) = match normalize_change_risk_cab_packet_query(&mut query) {
        Ok(values) => values,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid Change Risk CAB packet query", "details": errors })),
            )
                .into_response();
        }
    };
    let org_id = match resolve_change_risk_org(&state, &auth_user, query.org_name.as_deref()).await
    {
        Ok(org_id) => org_id,
        Err(response) => return response.into_response(),
    };
    match state
        .db
        .list_change_risk_cab_packets(&ListChangeRiskCabPacketsInput {
            org_id: &org_id,
            status: query.status.as_deref(),
            limit,
            offset,
        })
        .await
    {
        Ok((items, total)) => (
            StatusCode::OK,
            Json(ChangeRiskCabPacketListResponse {
                items,
                total,
                limit,
                offset,
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to list change risk CAB packets");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn get_change_risk_cab_packet(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(packet_id): Path<String>,
    Query(mut query): Query<ChangeRiskCabPacketQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let packet_id = match normalize_change_risk_cab_packet_id(&packet_id) {
        Ok(value) => value,
        Err(error) => return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response(),
    };
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_change_risk_org(&state, &auth_user, query.org_name.as_deref()).await
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
            tracing::error!(error = %e, org_id = %org_id, packet_id = %packet_id, "Failed to get change risk CAB packet");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    let artifact = match state
        .db
        .get_change_risk_cab_packet_artifact(&org_id, &packet_id)
        .await
    {
        Ok(value) => value,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, packet_id = %packet_id, "Failed to get change risk CAB packet artifact");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    (
        StatusCode::OK,
        Json(change_risk_cab_packet_response(packet, artifact)),
    )
        .into_response()
}

pub async fn download_change_risk_cab_packet(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(packet_id): Path<String>,
    Query(mut query): Query<ChangeRiskCabPacketQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let packet_id = match normalize_change_risk_cab_packet_id(&packet_id) {
        Ok(value) => value,
        Err(error) => return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response(),
    };
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_change_risk_org(&state, &auth_user, query.org_name.as_deref()).await
    {
        Ok(org_id) => org_id,
        Err(response) => return response.into_response(),
    };
    let current = match state.db.get_change_risk_cab_packet(&org_id, &packet_id).await {
        Ok(Some(packet)) => packet,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Change Risk CAB packet not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, packet_id = %packet_id, "Failed to get change risk CAB packet before download");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    if current.status == "archived" {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Change Risk CAB packet has been archived",
                "code": "change_risk_cab_packet_archived"
            })),
        )
            .into_response();
    }
    match state.db.download_change_risk_cab_packet(&org_id, &packet_id).await {
        Ok(Some((packet, artifact))) => {
            let audit = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "change_risk_cab_packet_downloaded".to_string(),
                target_type: Some("change_risk_cab_packet".to_string()),
                target_id: Some(packet.packet_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "packet_id": packet.packet_id,
                    "artifact_hash": packet.artifact_hash,
                    "download_count": packet.download_count,
                    "manual_cab_review_only": true,
                    "advisory_only": true,
                    "agent_governance_used": false
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
                tracing::warn!(error = %e, "Failed to write admin audit log (change risk CAB packet download)");
            }
            let mut headers = HeaderMap::new();
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static("application/json"),
            );
            headers.insert(
                axum::http::header::CONTENT_DISPOSITION,
                axum::http::HeaderValue::from_str(&format!(
                    "attachment; filename=\"gitgov-change-risk-cab-packet-{}.json\"",
                    packet.packet_id
                ))
                .unwrap_or_else(|_| axum::http::HeaderValue::from_static("attachment")),
            );
            headers.insert(
                axum::http::HeaderName::from_static("x-gitgov-artifact-hash"),
                axum::http::HeaderValue::from_str(&packet.artifact_hash)
                    .unwrap_or_else(|_| axum::http::HeaderValue::from_static("invalid")),
            );
            (headers, Json(artifact)).into_response()
        }
        Ok(None) => (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Change Risk CAB packet has been archived",
                "code": "change_risk_cab_packet_archived"
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, packet_id = %packet_id, "Failed to download change risk CAB packet");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn archive_change_risk_cab_packet(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(packet_id): Path<String>,
    Json(mut payload): Json<ChangeRiskCabPacketRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    let packet_id = match normalize_change_risk_cab_packet_id(&packet_id) {
        Ok(value) => value,
        Err(error) => return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response(),
    };
    normalize_release_approval_optional_text(&mut payload.org_name);
    let org_id = match resolve_change_risk_org(&state, &auth_user, payload.org_name.as_deref()).await
    {
        Ok(org_id) => org_id,
        Err(response) => return response.into_response(),
    };
    match state
        .db
        .archive_change_risk_cab_packet(&ArchiveChangeRiskCabPacketInput {
            org_id: &org_id,
            packet_id: &packet_id,
            archived_by_user_id: &auth_user.client_id,
        })
        .await
    {
        Ok(Some(packet)) => {
            let audit = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "change_risk_cab_packet_archived".to_string(),
                target_type: Some("change_risk_cab_packet".to_string()),
                target_id: Some(packet.packet_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "packet_id": packet.packet_id,
                    "artifact_hash": packet.artifact_hash,
                    "archived_at": packet.archived_at,
                    "archived_by_user_id": packet.archived_by_user_id,
                    "manual_cab_review_only": true,
                    "advisory_only": true,
                    "agent_governance_used": false
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
                tracing::warn!(error = %e, "Failed to write admin audit log (change risk CAB packet archive)");
            }
            (
                StatusCode::OK,
                Json(change_risk_cab_packet_response(packet, None)),
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Change Risk CAB packet not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, packet_id = %packet_id, "Failed to archive change risk CAB packet");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
