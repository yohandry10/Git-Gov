const EXECUTIVE_GOVERNANCE_SNAPSHOT_SCHEMA_VERSION: &str =
    "gitgov_executive_governance_snapshot.v1";

fn normalize_executive_governance_snapshot_id(value: &str) -> Result<String, &'static str> {
    let normalized = value.trim().to_string();
    if normalized.starts_with("egs_") && normalized.len() == 36 {
        Ok(normalized)
    } else {
        Err("snapshot_id must be a valid egs_ identifier")
    }
}

fn deterministic_executive_governance_snapshot_id() -> String {
    format!("egs_{}", Uuid::new_v4().simple())
}

fn executive_governance_snapshot_hash(artifact: &serde_json::Value) -> String {
    let bytes = serde_json::to_vec(artifact)
        .expect("executive governance snapshot artifact should serialize");
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn looks_like_executive_governance_snapshot_secret(value: &str) -> bool {
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

fn normalize_executive_governance_snapshot_request(
    payload: &mut ExecutiveGovernanceSnapshotRequest,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    normalize_release_approval_optional_text(&mut payload.org_name);
    payload.name = payload.name.trim().to_string();
    if payload.name.is_empty()
        || payload.name.len() > 160
        || has_control_chars(&payload.name)
        || looks_like_executive_governance_snapshot_secret(&payload.name)
    {
        errors.push("name is required, must be at most 160 characters, and must not contain secret-looking values.".to_string());
    }
    if !payload.include_repository_rows && !payload.include_summary {
        errors.push("At least one of include_repository_rows or include_summary must be true.".to_string());
    }
    if payload.filters.limit.is_none() {
        payload.filters.limit = Some(100);
    }
    if payload.filters.offset.is_none() {
        payload.filters.offset = Some(0);
    }
    match normalize_multi_repo_executive_governance_query(&mut payload.filters) {
        Ok((_limit, _offset)) => {}
        Err(mut query_errors) => errors.append(&mut query_errors),
    }
    payload.filters.org_name = None;
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn normalize_executive_governance_snapshot_query(
    query: &mut ExecutiveGovernanceSnapshotQuery,
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

fn executive_governance_snapshot_filters(query: &MultiRepoExecutiveGovernanceQuery) -> serde_json::Value {
    json!({
        "repository": query.repository,
        "environment": query.environment,
        "posture": query.posture,
        "gate_decision": query.gate_decision,
        "risk_level": query.risk_level,
        "review_status": query.review_status,
        "limit": query.limit,
        "offset": query.offset
    })
}

struct ExecutiveGovernanceSnapshotArtifactInput<'a> {
    snapshot_id: &'a str,
    org_id: &'a str,
    created_at: i64,
    created_by: &'a str,
    name: &'a str,
    filters: &'a serde_json::Value,
    view: &'a MultiRepoExecutiveGovernanceResponse,
    include_repository_rows: bool,
    include_summary: bool,
    artifact_hash: Option<&'a str>,
}

fn build_executive_governance_snapshot_artifact(
    input: ExecutiveGovernanceSnapshotArtifactInput<'_>,
) -> serde_json::Value {
    json!({
        "schema_version": EXECUTIVE_GOVERNANCE_SNAPSHOT_SCHEMA_VERSION,
        "snapshot_id": input.snapshot_id,
        "org_id": input.org_id,
        "name": input.name,
        "generated_at": input.created_at,
        "created_by_user_id": input.created_by,
        "source_endpoint": "/executive/repositories",
        "filters": input.filters,
        "repository_count": input.view.repositories.len(),
        "summary": if input.include_summary { json!(input.view.totals) } else { serde_json::Value::Null },
        "repositories": if input.include_repository_rows { json!(input.view.repositories) } else { json!([]) },
        "artifact_hash": input.artifact_hash,
        "flags": {
            "read_only": true,
            "manual_first": true,
            "advisory_only": true,
            "enforcement_used": false,
            "deployment_execution": false,
            "provider_mutation": false,
            "repository_mutation": false,
            "llm_used": false,
            "agent_governance_used": false,
            "compliance_claim": false,
            "certification": false,
            "compliance_score": false,
            "release_blocking": false,
            "deploy_execution": false
        }
    })
}

fn executive_governance_snapshot_response(
    snapshot: ExecutiveGovernanceSnapshotRecord,
    artifact: Option<serde_json::Value>,
) -> ExecutiveGovernanceSnapshotResponse {
    ExecutiveGovernanceSnapshotResponse { snapshot, artifact }
}

pub async fn create_executive_governance_snapshot(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<ExecutiveGovernanceSnapshotRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    if let Err(errors) = normalize_executive_governance_snapshot_request(&mut payload) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid executive governance snapshot request", "details": errors })),
        )
            .into_response();
    }
    let org_id = match resolve_and_check_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        payload.org_name.as_deref(),
        true,
    )
    .await
    {
        Ok(Some(org_id)) => org_id,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "org_name is required for global admin keys" })),
            )
                .into_response();
        }
        Err(err) => {
            return (
                org_scope_status(err),
                Json(json!({ "error": release_approval_scope_error_message(err) })),
            )
                .into_response();
        }
    };
    let limit = payload.filters.limit.unwrap_or(100).clamp(1, 100);
    let offset = payload.filters.offset.unwrap_or(0).max(0);
    let repositories = match state
        .db
        .get_multi_repo_executive_governance(&org_id, &payload.filters, limit, offset)
        .await
    {
        Ok(repositories) => repositories,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to load executive governance view for snapshot");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    let view = build_multi_repo_executive_governance_response(
        org_id.clone(),
        repositories,
        limit,
        offset,
    );
    let snapshot_id = deterministic_executive_governance_snapshot_id();
    let created_at = chrono::Utc::now().timestamp_millis();
    let filters = executive_governance_snapshot_filters(&payload.filters);
    let preimage = build_executive_governance_snapshot_artifact(
        ExecutiveGovernanceSnapshotArtifactInput {
            snapshot_id: &snapshot_id,
            org_id: &org_id,
            created_at,
            created_by: &auth_user.client_id,
            name: &payload.name,
            filters: &filters,
            view: &view,
            include_repository_rows: payload.include_repository_rows,
            include_summary: payload.include_summary,
            artifact_hash: None,
        },
    );
    let artifact_hash = executive_governance_snapshot_hash(&preimage);
    let artifact = build_executive_governance_snapshot_artifact(
        ExecutiveGovernanceSnapshotArtifactInput {
            snapshot_id: &snapshot_id,
            org_id: &org_id,
            created_at,
            created_by: &auth_user.client_id,
            name: &payload.name,
            filters: &filters,
            view: &view,
            include_repository_rows: payload.include_repository_rows,
            include_summary: payload.include_summary,
            artifact_hash: Some(&artifact_hash),
        },
    );

    match state
        .db
        .create_executive_governance_snapshot(&CreateExecutiveGovernanceSnapshotInput {
            snapshot_id: &snapshot_id,
            org_id: &org_id,
            name: &payload.name,
            filters_json: &filters,
            artifact_hash: &artifact_hash,
            artifact_json: &artifact,
            repository_count: view.repositories.len() as i64,
            created_by_user_id: &auth_user.client_id,
        })
        .await
    {
        Ok(snapshot) => {
            let audit = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "executive_governance_snapshot_created".to_string(),
                target_type: Some("executive_governance_snapshot".to_string()),
                target_id: Some(snapshot.snapshot_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "snapshot_id": snapshot.snapshot_id,
                    "artifact_hash": snapshot.artifact_hash,
                    "repository_count": snapshot.repository_count,
                    "manual_first": true,
                    "read_only": true,
                    "advisory_only": true,
                    "enforcement_used": false,
                    "deployment_execution": false,
                    "provider_mutation": false,
                    "repository_mutation": false,
                    "llm_used": false,
                    "agent_governance_used": false,
                    "compliance_claim": false,
                    "certification": false
                }),
                created_at,
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
                tracing::warn!(error = %e, "Failed to write admin audit log (executive governance snapshot create)");
            }
            (
                StatusCode::CREATED,
                Json(executive_governance_snapshot_response(snapshot, Some(artifact))),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to create executive governance snapshot");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn list_executive_governance_snapshots(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(mut query): Query<ExecutiveGovernanceSnapshotQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let (limit, offset) = match normalize_executive_governance_snapshot_query(&mut query) {
        Ok(values) => values,
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid executive governance snapshot query", "details": errors })),
            )
                .into_response();
        }
    };
    let org_id = match resolve_and_check_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        query.org_name.as_deref(),
        true,
    )
    .await
    {
        Ok(Some(org_id)) => org_id,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "org_name is required for global admin keys" })),
            )
                .into_response();
        }
        Err(err) => {
            return (
                org_scope_status(err),
                Json(json!({ "error": release_approval_scope_error_message(err) })),
            )
                .into_response();
        }
    };

    match state
        .db
        .list_executive_governance_snapshots(&ListExecutiveGovernanceSnapshotsInput {
            org_id: &org_id,
            status: query.status.as_deref(),
            limit,
            offset,
        })
        .await
    {
        Ok((items, total)) => (
            StatusCode::OK,
            Json(ExecutiveGovernanceSnapshotListResponse {
                items,
                total,
                limit,
                offset,
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, "Failed to list executive governance snapshots");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn get_executive_governance_snapshot(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(snapshot_id): Path<String>,
    Query(mut query): Query<ExecutiveGovernanceSnapshotQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let snapshot_id = match normalize_executive_governance_snapshot_id(&snapshot_id) {
        Ok(value) => value,
        Err(error) => return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response(),
    };
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_and_check_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        query.org_name.as_deref(),
        true,
    )
    .await
    {
        Ok(Some(org_id)) => org_id,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "org_name is required for global admin keys" })),
            )
                .into_response();
        }
        Err(err) => {
            return (
                org_scope_status(err),
                Json(json!({ "error": release_approval_scope_error_message(err) })),
            )
                .into_response();
        }
    };

    let snapshot = match state
        .db
        .get_executive_governance_snapshot(&org_id, &snapshot_id)
        .await
    {
        Ok(Some(snapshot)) => snapshot,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Executive governance snapshot not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, snapshot_id = %snapshot_id, "Failed to get executive governance snapshot");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    let artifact = match state
        .db
        .get_executive_governance_snapshot_artifact(&org_id, &snapshot_id)
        .await
    {
        Ok(value) => value,
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, snapshot_id = %snapshot_id, "Failed to get executive governance snapshot artifact");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response();
        }
    };
    (
        StatusCode::OK,
        Json(executive_governance_snapshot_response(snapshot, artifact)),
    )
        .into_response()
}

pub async fn download_executive_governance_snapshot(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(snapshot_id): Path<String>,
    Query(mut query): Query<ExecutiveGovernanceSnapshotQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_compliance_reviewer(&auth_user) {
        return resp.into_response();
    }
    let snapshot_id = match normalize_executive_governance_snapshot_id(&snapshot_id) {
        Ok(value) => value,
        Err(error) => return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response(),
    };
    normalize_release_approval_optional_text(&mut query.org_name);
    let org_id = match resolve_and_check_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        query.org_name.as_deref(),
        true,
    )
    .await
    {
        Ok(Some(org_id)) => org_id,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "org_name is required for global admin keys" })),
            )
                .into_response();
        }
        Err(err) => {
            return (
                org_scope_status(err),
                Json(json!({ "error": release_approval_scope_error_message(err) })),
            )
                .into_response();
        }
    };
    let current = match state
        .db
        .get_executive_governance_snapshot(&org_id, &snapshot_id)
        .await
    {
        Ok(Some(snapshot)) => snapshot,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Executive governance snapshot not found" })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, snapshot_id = %snapshot_id, "Failed to get executive governance snapshot before download");
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
                "error": "Executive governance snapshot has been archived",
                "code": "executive_governance_snapshot_archived"
            })),
        )
            .into_response();
    }
    match state
        .db
        .download_executive_governance_snapshot(&org_id, &snapshot_id)
        .await
    {
        Ok(Some((snapshot, artifact))) => {
            let audit = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "executive_governance_snapshot_downloaded".to_string(),
                target_type: Some("executive_governance_snapshot".to_string()),
                target_id: Some(snapshot.snapshot_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "snapshot_id": snapshot.snapshot_id,
                    "artifact_hash": snapshot.artifact_hash,
                    "download_count": snapshot.download_count,
                    "manual_first": true,
                    "read_only": true,
                    "agent_governance_used": false
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
                tracing::warn!(error = %e, "Failed to write admin audit log (executive governance snapshot download)");
            }
            let mut headers = HeaderMap::new();
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static("application/json"),
            );
            headers.insert(
                axum::http::header::CONTENT_DISPOSITION,
                axum::http::HeaderValue::from_str(&format!(
                    "attachment; filename=\"gitgov-executive-governance-snapshot-{}.json\"",
                    snapshot.snapshot_id
                ))
                .unwrap_or_else(|_| axum::http::HeaderValue::from_static("attachment")),
            );
            headers.insert(
                axum::http::HeaderName::from_static("x-gitgov-artifact-hash"),
                axum::http::HeaderValue::from_str(&snapshot.artifact_hash)
                    .unwrap_or_else(|_| axum::http::HeaderValue::from_static("invalid")),
            );
            (headers, Json(artifact)).into_response()
        }
        Ok(None) => (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Executive governance snapshot has been archived",
                "code": "executive_governance_snapshot_archived"
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, snapshot_id = %snapshot_id, "Failed to download executive governance snapshot");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}

pub async fn archive_executive_governance_snapshot(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(snapshot_id): Path<String>,
    Json(mut payload): Json<ArchiveExecutiveGovernanceSnapshotRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }
    let snapshot_id = match normalize_executive_governance_snapshot_id(&snapshot_id) {
        Ok(value) => value,
        Err(error) => return (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))).into_response(),
    };
    normalize_release_approval_optional_text(&mut payload.org_name);
    let org_id = match resolve_and_check_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        payload.org_name.as_deref(),
        true,
    )
    .await
    {
        Ok(Some(org_id)) => org_id,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "org_name is required for global admin keys" })),
            )
                .into_response();
        }
        Err(err) => {
            return (
                org_scope_status(err),
                Json(json!({ "error": release_approval_scope_error_message(err) })),
            )
                .into_response();
        }
    };
    match state
        .db
        .archive_executive_governance_snapshot(&ArchiveExecutiveGovernanceSnapshotInput {
            org_id: &org_id,
            snapshot_id: &snapshot_id,
            archived_by_user_id: &auth_user.client_id,
        })
        .await
    {
        Ok(Some(snapshot)) => {
            let audit = AdminAuditLogEntry {
                id: Uuid::new_v4().to_string(),
                actor_client_id: auth_user.client_id.clone(),
                action: "executive_governance_snapshot_archived".to_string(),
                target_type: Some("executive_governance_snapshot".to_string()),
                target_id: Some(snapshot.snapshot_id.clone()),
                metadata: json!({
                    "org_id": org_id,
                    "snapshot_id": snapshot.snapshot_id,
                    "artifact_hash": snapshot.artifact_hash,
                    "archived_at": snapshot.archived_at,
                    "archived_by_user_id": snapshot.archived_by_user_id,
                    "manual_first": true,
                    "read_only": true,
                    "source_evidence_mutated": false,
                    "agent_governance_used": false
                }),
                created_at: chrono::Utc::now().timestamp_millis(),
            };
            if let Err(e) = state.db.insert_admin_audit_log(&audit).await {
                tracing::warn!(error = %e, "Failed to write admin audit log (executive governance snapshot archive)");
            }
            (
                StatusCode::OK,
                Json(executive_governance_snapshot_response(snapshot, None)),
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Executive governance snapshot not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, org_id = %org_id, snapshot_id = %snapshot_id, "Failed to archive executive governance snapshot");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal database error" })),
            )
                .into_response()
        }
    }
}
