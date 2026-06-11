// ============================================================================
// JENKINS INTEGRATION (V1.2-A)
// ============================================================================

#[derive(Debug, Clone, Deserialize, Default)]
pub struct IntegrationStatusQuery {
    #[serde(default)]
    org_name: Option<String>,
}

pub async fn ingest_jenkins_pipeline_event(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<JenkinsPipelineEventInput>,
) -> impl IntoResponse {
    if require_admin(&auth_user).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(JenkinsPipelineEventResponse {
                accepted: false,
                duplicate: false,
                pipeline_event_id: None,
                error: Some("Admin access required".to_string()),
            }),
        );
    }

    metrics::counter!("gitgov_jenkins_events_total", "status" => payload.status.clone())
        .increment(1);

    if let Some(expected_secret) = state.jenkins_webhook_secret.as_deref() {
        let provided_secret = headers
            .get("x-gitgov-jenkins-secret")
            .and_then(|v| v.to_str().ok())
            .map(str::trim)
            .unwrap_or_default();

        if provided_secret.is_empty()
            || provided_secret
                .as_bytes()
                .ct_eq(expected_secret.as_bytes())
                .unwrap_u8()
                != 1
        {
            tracing::warn!("Rejected Jenkins pipeline event due to missing/invalid secret header");
            return (
                StatusCode::UNAUTHORIZED,
                Json(JenkinsPipelineEventResponse {
                    accepted: false,
                    duplicate: false,
                    pipeline_event_id: None,
                    error: Some("Invalid Jenkins webhook secret".to_string()),
                }),
            );
        }
    }

    if payload.pipeline_id.trim().is_empty() || payload.job_name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(JenkinsPipelineEventResponse {
                accepted: false,
                duplicate: false,
                pipeline_event_id: None,
                error: Some("pipeline_id and job_name are required".to_string()),
            }),
        );
    }

    let Some(status) = PipelineStatus::from_str(payload.status.trim()) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(JenkinsPipelineEventResponse {
                accepted: false,
                duplicate: false,
                pipeline_event_id: None,
                error: Some("Invalid status. Use: success, failure, aborted, unstable".to_string()),
            }),
        );
    };

    let requested_org_id = match payload.org_name.as_deref() {
        Some(org_name) => match state.db.get_org_by_login(org_name).await {
            Ok(Some(org)) => Some(org.id),
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(JenkinsPipelineEventResponse {
                        accepted: false,
                        duplicate: false,
                        pipeline_event_id: None,
                        error: Some("Organization not found".to_string()),
                    }),
                );
            }
            Err(e) => {
                tracing::error!(error = %e, org_name = %org_name, "Failed to resolve Jenkins org scope");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(JenkinsPipelineEventResponse {
                        accepted: false,
                        duplicate: false,
                        pipeline_event_id: None,
                        error: Some("Internal database error".to_string()),
                    }),
                );
            }
        },
        None => None,
    };

    let derived_org_id = if let Some(repo_full_name) = payload.repo_full_name.as_deref() {
        match state.db.get_repo_by_full_name(repo_full_name).await {
            Ok(Some(repo)) => repo.org_id,
            Ok(None) => {
                let guessed_org = repo_full_name.split('/').next().unwrap_or_default();
                if guessed_org.is_empty() {
                    None
                } else {
                    state
                        .db
                        .get_org_by_login(guessed_org)
                        .await
                        .ok()
                        .flatten()
                        .map(|o| o.id)
                }
            }
            Err(_) => None,
        }
    } else {
        None
    };
    let org_id = match apply_required_ingest_org_scope(
        auth_user.org_id.as_deref(),
        requested_org_id.as_deref(),
        derived_org_id.as_deref(),
    ) {
        Ok(value) => value,
        Err(error) => {
            return (
                if error == "Organization is required for global admin keys" {
                    StatusCode::BAD_REQUEST
                } else {
                    StatusCode::FORBIDDEN
                },
                Json(JenkinsPipelineEventResponse {
                    accepted: false,
                    duplicate: false,
                    pipeline_event_id: None,
                    error: Some(error.to_string()),
                }),
            );
        }
    };

    let raw_payload = serde_json::to_value(&payload).unwrap_or(serde_json::Value::Null);
    let event = PipelineEvent {
        id: Uuid::new_v4().to_string(),
        org_id,
        pipeline_id: payload.pipeline_id,
        job_name: payload.job_name,
        status,
        commit_sha: payload
            .commit_sha
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        branch: payload
            .branch
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        repo_full_name: payload
            .repo_full_name
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        duration_ms: payload.duration_ms,
        triggered_by: payload
            .triggered_by
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        stages: payload.stages,
        artifacts: payload.artifacts,
        payload: raw_payload,
        ingested_at: payload
            .timestamp
            .unwrap_or_else(|| chrono::Utc::now().timestamp_millis()),
    };

    tracing::info!(
        pipeline_id = %event.pipeline_id,
        job_name = %event.job_name,
        status = %event.status.as_str(),
        commit_sha = ?event.commit_sha,
        repo = ?event.repo_full_name,
        "Received Jenkins pipeline event"
    );

    match state.db.insert_pipeline_event(&event).await {
        Ok(pipeline_event_id) => (
            StatusCode::OK,
            Json(JenkinsPipelineEventResponse {
                accepted: true,
                duplicate: false,
                pipeline_event_id: Some(pipeline_event_id),
                error: None,
            }),
        ),
        Err(DbError::Duplicate(_)) => (
            StatusCode::OK,
            Json(JenkinsPipelineEventResponse {
                accepted: false,
                duplicate: true,
                pipeline_event_id: None,
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JenkinsPipelineEventResponse {
                accepted: false,
                duplicate: false,
                pipeline_event_id: None,
                error: Some(sanitize_db_error(&e)),
            }),
        ),
    }
}

fn apply_required_ingest_org_scope(
    auth_org_id: Option<&str>,
    requested_org_id: Option<&str>,
    derived_org_id: Option<&str>,
) -> Result<Option<String>, &'static str> {
    let effective = requested_org_id.or(derived_org_id).or(auth_org_id);
    let Some(effective_org_id) = effective else {
        return Err("Organization is required for global admin keys");
    };

    if let Some(scoped_org_id) = auth_org_id {
        if scoped_org_id != effective_org_id {
            return Err("Requested org is outside API key scope");
        }
    }
    if let (Some(requested), Some(derived)) = (requested_org_id, derived_org_id) {
        if requested != derived {
            return Err("Requested org does not match repo organization");
        }
    }

    Ok(Some(effective_org_id.to_string()))
}

// ============================================================================
// JIRA INTEGRATION (V1.2-B groundwork)
// ============================================================================

fn jira_issue_text(value: Option<&serde_json::Value>) -> Option<String> {
    value?
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn jira_issue_timestamp_ms(value: Option<&serde_json::Value>) -> Option<i64> {
    let raw = value?.as_str()?;
    chrono::DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|dt| dt.timestamp_millis())
}

fn jira_org_name_hint(payload: &JiraWebhookEvent) -> Option<String> {
    let candidates = [
        payload.extra.get("org_name"),
        payload.extra.get("organization"),
        payload.extra.get("org"),
        payload.extra.get("tenant"),
    ];
    candidates
        .into_iter()
        .flatten()
        .find_map(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

#[derive(Debug, Deserialize, Default)]
pub struct JiraSignedWebhookQuery {
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub organization: Option<String>,
    #[serde(default)]
    pub org: Option<String>,
    #[serde(default)]
    pub tenant: Option<String>,
}

fn jira_query_org_name(query: &JiraSignedWebhookQuery) -> Option<&str> {
    [
        query.org_name.as_deref(),
        query.organization.as_deref(),
        query.org.as_deref(),
        query.tenant.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(str::trim)
    .find(|value| !value.is_empty())
}

fn validate_jira_signature(secret: &str, payload_bytes: &[u8], signature: &str) -> bool {
    let signature_hex = match signature.strip_prefix("sha256=") {
        Some(hex) => hex,
        None => return false,
    };
    let signature_bytes = match hex::decode(signature_hex) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    let mut mac = match <hmac::Hmac<Sha256> as Mac>::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };

    mac.update(payload_bytes);
    mac.verify_slice(&signature_bytes).is_ok()
}

fn build_project_ticket_from_jira_payload(
    org_id: Option<String>,
    payload: &JiraWebhookEvent,
) -> Result<ProjectTicket, String> {
    let issue = payload
        .issue
        .as_ref()
        .ok_or_else(|| "Missing issue object".to_string())?;
    let key = issue
        .get("key")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "Missing issue.key".to_string())?
        .to_ascii_uppercase();

    let fields = issue.get("fields");
    let title = jira_issue_text(fields.and_then(|f| f.get("summary")));
    let status = jira_issue_text(
        fields
            .and_then(|f| f.get("status"))
            .and_then(|s| s.get("name")),
    );
    let assignee = jira_issue_text(
        fields
            .and_then(|f| f.get("assignee"))
            .and_then(|a| a.get("displayName")),
    )
    .or_else(|| {
        jira_issue_text(
            fields
                .and_then(|f| f.get("assignee"))
                .and_then(|a| a.get("name")),
        )
    });
    let reporter = jira_issue_text(
        fields
            .and_then(|f| f.get("reporter"))
            .and_then(|a| a.get("displayName")),
    )
    .or_else(|| {
        jira_issue_text(
            fields
                .and_then(|f| f.get("reporter"))
                .and_then(|a| a.get("name")),
        )
    });
    let priority = jira_issue_text(
        fields
            .and_then(|f| f.get("priority"))
            .and_then(|p| p.get("name")),
    );
    let ticket_type = jira_issue_text(
        fields
            .and_then(|f| f.get("issuetype"))
            .and_then(|t| t.get("name")),
    );
    let created_at = jira_issue_timestamp_ms(fields.and_then(|f| f.get("created")));
    let updated_at = jira_issue_timestamp_ms(fields.and_then(|f| f.get("updated")));

    let self_url = issue
        .get("self")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let mut text_candidates: Vec<&str> = Vec::new();
    if let Some(summary) = title.as_deref() {
        text_candidates.push(summary);
    }
    if let Some(description) = fields
        .and_then(|f| f.get("description"))
        .and_then(|d| d.as_str())
    {
        text_candidates.push(description);
    }
    let related_branches = extract_ticket_ids(&text_candidates);

    Ok(ProjectTicket {
        id: Uuid::new_v4().to_string(),
        org_id,
        ticket_id: key,
        ticket_url: self_url,
        title,
        status,
        assignee,
        reporter,
        priority,
        ticket_type,
        related_commits: vec![],
        related_prs: vec![],
        related_branches,
        created_at,
        updated_at,
        ingested_at: chrono::Utc::now().timestamp_millis(),
    })
}

pub async fn handle_jira_signed_webhook(
    State(state): State<Arc<AppState>>,
    Query(query): Query<JiraSignedWebhookQuery>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    metrics::counter!("gitgov_jira_events_total").increment(1);

    let Some(expected_secret) = state.jira_webhook_secret.as_deref() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(JiraWebhookIngestResponse {
                accepted: false,
                duplicate: false,
                ticket_id: None,
                error: Some("Jira webhook secret is not configured".to_string()),
            }),
        );
    };

    let signature = headers
        .get("x-hub-signature")
        .or_else(|| headers.get("x-hub-signature-256"))
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .unwrap_or_default();

    if signature.is_empty() || !validate_jira_signature(expected_secret, &body, signature) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(JiraWebhookIngestResponse {
                accepted: false,
                duplicate: false,
                ticket_id: None,
                error: Some("Invalid Jira signature".to_string()),
            }),
        );
    }

    let mut payload_value: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(value) => value,
        Err(e) => {
            tracing::warn!(error = %e, "Invalid JSON Jira webhook payload");
            return (
                StatusCode::BAD_REQUEST,
                Json(JiraWebhookIngestResponse {
                    accepted: false,
                    duplicate: false,
                    ticket_id: None,
                    error: Some("Invalid JSON payload".to_string()),
                }),
            );
        }
    };

    if let (Some(org_name), Some(payload_obj)) =
        (jira_query_org_name(&query), payload_value.as_object_mut())
    {
        if !payload_obj.contains_key("org_name")
            && !payload_obj.contains_key("organization")
            && !payload_obj.contains_key("org")
            && !payload_obj.contains_key("tenant")
        {
            payload_obj.insert("org_name".to_string(), serde_json::json!(org_name));
        }
    }

    let payload: JiraWebhookEvent = match serde_json::from_value(payload_value) {
        Ok(payload) => payload,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to parse Jira webhook payload");
            return (
                StatusCode::BAD_REQUEST,
                Json(JiraWebhookIngestResponse {
                    accepted: false,
                    duplicate: false,
                    ticket_id: None,
                    error: Some("Invalid Jira payload".to_string()),
                }),
            );
        }
    };

    let requested_org_name = jira_org_name_hint(&payload);
    let org_id = match resolve_and_check_org_scope(
        &state,
        None,
        requested_org_name.as_deref(),
        true,
    )
    .await
    {
        Ok(org_id) => org_id,
        Err(err) => {
            let error = match err {
                OrgScopeError::BadRequest => "org_name is required for public Jira webhooks",
                OrgScopeError::NotFound => "Organization not found for Jira org hint",
                OrgScopeError::Forbidden => "Requested org is outside API key scope",
                OrgScopeError::Internal => "Internal database error while resolving org scope",
            };
            return (
                org_scope_status(err),
                Json(JiraWebhookIngestResponse {
                    accepted: false,
                    duplicate: false,
                    ticket_id: None,
                    error: Some(error.to_string()),
                }),
            );
        }
    };

    let ticket = match build_project_ticket_from_jira_payload(org_id, &payload) {
        Ok(ticket) => ticket,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(JiraWebhookIngestResponse {
                    accepted: false,
                    duplicate: false,
                    ticket_id: None,
                    error: Some(error),
                }),
            )
        }
    };

    let ticket_id = ticket.ticket_id.clone();
    match state.db.upsert_project_ticket(&ticket).await {
        Ok(()) => (
            StatusCode::OK,
            Json(JiraWebhookIngestResponse {
                accepted: true,
                duplicate: false,
                ticket_id: Some(ticket_id),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JiraWebhookIngestResponse {
                accepted: false,
                duplicate: false,
                ticket_id: Some(ticket_id),
                error: Some(sanitize_db_error(&e)),
            }),
        ),
    }
}

pub async fn ingest_jira_webhook(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<JiraWebhookEvent>,
) -> impl IntoResponse {
    if require_admin(&auth_user).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(JiraWebhookIngestResponse {
                accepted: false,
                duplicate: false,
                ticket_id: None,
                error: Some("Admin access required".to_string()),
            }),
        );
    }

    metrics::counter!("gitgov_jira_events_total").increment(1);

    if let Some(expected_secret) = state.jira_webhook_secret.as_deref() {
        let provided_secret = headers
            .get("x-gitgov-jira-secret")
            .and_then(|v| v.to_str().ok())
            .map(str::trim)
            .unwrap_or_default();
        if provided_secret.is_empty()
            || provided_secret
                .as_bytes()
                .ct_eq(expected_secret.as_bytes())
                .unwrap_u8()
                != 1
        {
            return (
                StatusCode::UNAUTHORIZED,
                Json(JiraWebhookIngestResponse {
                    accepted: false,
                    duplicate: false,
                    ticket_id: None,
                    error: Some("Invalid Jira secret".to_string()),
                }),
            );
        }
    }

    let requested_org_name = jira_org_name_hint(&payload);
    let org_id = match resolve_and_check_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        requested_org_name.as_deref(),
        true,
    )
    .await
    {
        Ok(org_id) => org_id,
        Err(err) => {
            let error = match err {
                OrgScopeError::BadRequest => "org_name is required for global admin keys",
                OrgScopeError::NotFound => "Organization not found for Jira org hint",
                OrgScopeError::Forbidden => "Requested org is outside API key scope",
                OrgScopeError::Internal => "Internal database error while resolving org scope",
            };
            return (
                org_scope_status(err),
                Json(JiraWebhookIngestResponse {
                    accepted: false,
                    duplicate: false,
                    ticket_id: None,
                    error: Some(error.to_string()),
                }),
            );
        }
    };

    let ticket = match build_project_ticket_from_jira_payload(org_id, &payload) {
        Ok(ticket) => ticket,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(JiraWebhookIngestResponse {
                    accepted: false,
                    duplicate: false,
                    ticket_id: None,
                    error: Some(error),
                }),
            )
        }
    };

    let ticket_id = ticket.ticket_id.clone();
    match state.db.upsert_project_ticket(&ticket).await {
        Ok(()) => (
            StatusCode::OK,
            Json(JiraWebhookIngestResponse {
                accepted: true,
                duplicate: false,
                ticket_id: Some(ticket_id),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JiraWebhookIngestResponse {
                accepted: false,
                duplicate: false,
                ticket_id: Some(ticket_id),
                error: Some(sanitize_db_error(&e)),
            }),
        ),
    }
}

pub async fn get_jira_integration_status(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<IntegrationStatusQuery>,
) -> impl IntoResponse {
    if require_admin(&auth_user).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(JiraIntegrationStatusResponse::default()),
        );
    }

    let scoped_org = match resolve_required_product_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        query.org_name.as_deref(),
    )
    .await
    {
        Ok(org) => org,
        Err(err) => {
            return (
                org_scope_status(err),
                Json(JiraIntegrationStatusResponse::default()),
            )
        }
    };

    match state
        .db
        .get_jira_integration_status(Some(scoped_org.id.as_str()))
        .await
    {
        Ok(status) => (StatusCode::OK, Json(status)),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JiraIntegrationStatusResponse::default()),
        ),
    }
}

pub async fn get_jira_ticket_detail(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(ticket_id): Path<String>,
    Query(query): Query<JiraTicketDetailQuery>,
) -> impl IntoResponse {
    if require_admin(&auth_user).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(JiraTicketDetailResponse::default()),
        );
    }

    let normalized = ticket_id.trim().to_ascii_uppercase();
    if normalized.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(JiraTicketDetailResponse::default()),
        );
    }

    let scoped_org = match resolve_required_product_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        query.org_name.as_deref(),
    )
    .await
    {
        Ok(org) => org,
        Err(err) => {
            return (
                org_scope_status(err),
                Json(JiraTicketDetailResponse::default()),
            )
        }
    };

    match state
        .db
        .get_project_ticket_by_ticket_id(&normalized, Some(scoped_org.id.as_str()))
        .await
    {
        Ok(Some(ticket)) => (
            StatusCode::OK,
            Json(JiraTicketDetailResponse {
                found: true,
                ticket: Some(ticket),
            }),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(JiraTicketDetailResponse {
                found: false,
                ticket: None,
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JiraTicketDetailResponse::default()),
        ),
    }
}

fn is_evidence_packet_quality_gate_job(job_name: &str) -> bool {
    let job = job_name.to_ascii_lowercase();
    job.contains("sonar")
        || job.contains("quality")
        || job.contains("clippy")
        || job.contains("lint")
        || job.contains("readiness")
}

fn evidence_packet_quality_gate_runs(pipelines: &[CommitPipelineRun]) -> Vec<CommitPipelineRun> {
    let mut seen = HashSet::new();
    let mut quality_gates = Vec::new();

    for pipeline in pipelines {
        if is_evidence_packet_quality_gate_job(&pipeline.job_name)
            && seen.insert(pipeline.pipeline_event_id.clone())
        {
            quality_gates.push(pipeline.clone());
        }
    }

    quality_gates
}

fn align_commit_embedded_pipelines_with_packet_runs(
    commits: &mut [TicketFlowCorrelation],
    pipelines: &[CommitPipelineRun],
) {
    let packet_pipeline_ids: HashSet<&str> = pipelines
        .iter()
        .map(|pipeline| pipeline.pipeline_event_id.as_str())
        .collect();

    for commit in commits {
        if commit
            .pipeline
            .as_ref()
            .is_some_and(|pipeline| !packet_pipeline_ids.contains(pipeline.pipeline_event_id.as_str()))
        {
            commit.pipeline = None;
        }
    }
}

fn build_evidence_packet_completeness(
    ticket_found: bool,
    commits: &[TicketFlowCorrelation],
    pull_requests: &[PrMergeEvidenceEntry],
    pipelines: &[CommitPipelineRun],
    quality_gates: &[CommitPipelineRun],
) -> EvidencePacketCompleteness {
    let pipeline_count = pipelines.len() as i64;
    let mut missing = Vec::new();
    if !ticket_found {
        missing.push("ticket".to_string());
    }
    if commits.is_empty() {
        missing.push("commits".to_string());
    }
    if pull_requests.is_empty() {
        missing.push("pull_requests".to_string());
    }
    if pipeline_count == 0 {
        missing.push("pipelines".to_string());
    }
    if quality_gates.is_empty() {
        missing.push("quality_gates".to_string());
    }

    EvidencePacketCompleteness {
        ticket_found,
        commits: commits.len() as i64,
        pull_requests: pull_requests.len() as i64,
        pipelines: pipeline_count,
        quality_gates: quality_gates.len() as i64,
        missing,
    }
}

fn collect_evidence_packet_shas(
    commits: &[TicketFlowCorrelation],
    pull_requests: &[PrMergeEvidenceEntry],
) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut shas = Vec::new();
    for commit in commits {
        let sha = commit.commit_sha.trim().to_ascii_lowercase();
        if !sha.is_empty() && seen.insert(sha.clone()) {
            shas.push(sha);
        }
    }
    for pull_request in pull_requests {
        for sha in [
            pull_request.head_sha.as_deref(),
            pull_request.merge_commit_sha.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            let sha = sha.trim().to_ascii_lowercase();
            if !sha.is_empty() && seen.insert(sha.clone()) {
                shas.push(sha);
            }
        }
    }
    shas
}

struct EvidencePacketReconstructionInput<'a> {
    query: &'a EvidencePacketQuery,
    scoped_org_login: &'a str,
    ticket_id: &'a str,
    hours: i64,
    commits: &'a [TicketFlowCorrelation],
    pull_requests: &'a [PrMergeEvidenceEntry],
    pipelines: &'a [CommitPipelineRun],
    quality_gates: &'a [CommitPipelineRun],
}

fn build_evidence_packet_reconstruction(
    input: EvidencePacketReconstructionInput<'_>,
) -> EvidencePacketReconstruction {
    let legacy_pipeline_scope_fallbacks = input
        .pipelines
        .iter()
        .filter(|pipeline| {
            (input.query.repo_full_name.is_some() && pipeline.repo_full_name.is_none())
                || (input.query.branch.is_some() && pipeline.branch.is_none())
        })
        .count() as i64;
    let mut warnings = Vec::new();
    if legacy_pipeline_scope_fallbacks > 0 {
        warnings.push(
            "Some legacy pipeline events lacked repo_full_name and/or branch; org and SHA still matched."
                .to_string(),
        );
    }

    EvidencePacketReconstruction {
        filters: EvidencePacketReconstructionFilters {
            org_name: Some(input.scoped_org_login.to_string()),
            repo_full_name: input.query.repo_full_name.clone(),
            branch: input.query.branch.clone(),
            target_sha: input.query.target_sha.clone(),
            ticket_id: input.ticket_id.to_string(),
            hours: input.hours,
        },
        sources: EvidencePacketReconstructionSources {
            commit_correlations: input.commits.len() as i64,
            client_events: input
                .commits
                .iter()
                .filter(|commit| commit.evidence_source.as_deref() == Some("client_event"))
                .count() as i64,
            pull_request_merge_commits: input
                .commits
                .iter()
                .filter(|commit| commit.evidence_source.as_deref() == Some("pull_request_merge"))
                .count() as i64,
            pull_request_merges: input.pull_requests.len() as i64,
            pipeline_events: input.pipelines.len() as i64,
            quality_gate_pipeline_events: input.quality_gates.len() as i64,
            legacy_pipeline_scope_fallbacks,
        },
        warnings,
    }
}

fn evidence_packet_hash(packet: &EvidencePacket) -> String {
    let mut value = serde_json::to_value(packet).unwrap_or_else(|_| serde_json::json!({}));
    if let Some(obj) = value.as_object_mut() {
        obj.remove("content_hash");
    }
    let bytes = serde_json::to_vec(&value).unwrap_or_default();
    format!("{:x}", Sha256::digest(&bytes))
}

fn normalize_evidence_packet_optional_text(value: &mut Option<String>) {
    *value = value
        .take()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
}

fn has_evidence_packet_control_chars(value: &str) -> bool {
    value.chars().any(|ch| ch.is_control())
}

fn is_valid_evidence_packet_repo(value: &str) -> bool {
    let parts: Vec<&str> = value.split('/').collect();
    parts.len() == 2
        && value.len() <= 200
        && !has_evidence_packet_control_chars(value)
        && parts
            .iter()
            .all(|part| !part.is_empty() && !part.contains(char::is_whitespace))
}

fn is_valid_evidence_packet_branch(value: &str) -> bool {
    value.len() <= 200 && !has_evidence_packet_control_chars(value)
}

fn is_valid_evidence_packet_release_id(value: &str) -> bool {
    value.len() <= 120 && !has_evidence_packet_control_chars(value)
}

fn is_valid_evidence_packet_environment(value: &str) -> bool {
    value.len() <= 80 && !has_evidence_packet_control_chars(value)
}

fn is_valid_release_bound_evidence_ticket_id(value: &str) -> bool {
    static TICKET_ID_RE: OnceLock<Regex> = OnceLock::new();
    let re = TICKET_ID_RE
        .get_or_init(|| Regex::new(r"^[A-Z][A-Z0-9]+-[1-9][0-9]*$").expect("valid ticket regex"));
    value.len() <= 32 && re.is_match(value)
}

fn is_valid_evidence_packet_sha(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.chars().all(|c| c.is_ascii_hexdigit())
}

fn sha_matches_evidence_value(candidate: &str, evidence_value: &str) -> bool {
    candidate.eq_ignore_ascii_case(evidence_value)
}

fn target_sha_exists_in_evidence(
    target_sha: &str,
    commits: &[TicketFlowCorrelation],
    pull_requests: &[PrMergeEvidenceEntry],
) -> bool {
    commits
        .iter()
        .any(|commit| sha_matches_evidence_value(target_sha, &commit.commit_sha))
        || pull_requests.iter().any(|pull_request| {
            pull_request
                .head_sha
                .as_deref()
                .is_some_and(|sha| sha_matches_evidence_value(target_sha, sha))
                || pull_request
                    .merge_commit_sha
                    .as_deref()
                    .is_some_and(|sha| sha_matches_evidence_value(target_sha, sha))
        })
}

fn evidence_packet_uri_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

pub async fn get_ticket_evidence_packet(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Path(ticket_id): Path<String>,
    Query(mut query): Query<EvidencePacketQuery>,
) -> impl IntoResponse {
    if require_admin(&auth_user).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(EvidencePacketResponse::default()),
        );
    }

    let normalized = ticket_id.trim().to_ascii_uppercase();
    if normalized.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(EvidencePacketResponse::default()),
        );
    }
    normalize_evidence_packet_optional_text(&mut query.org_name);
    normalize_evidence_packet_optional_text(&mut query.repo_full_name);
    normalize_evidence_packet_optional_text(&mut query.branch);
    normalize_evidence_packet_optional_text(&mut query.release_id);
    normalize_evidence_packet_optional_text(&mut query.environment);
    normalize_evidence_packet_optional_text(&mut query.target_sha);
    if let Some(environment) = query.environment.as_mut() {
        *environment = environment.to_ascii_lowercase();
    }
    if query
        .repo_full_name
        .as_deref()
        .is_some_and(|repo| !is_valid_evidence_packet_repo(repo))
        || query
            .branch
            .as_deref()
            .is_some_and(|branch| !is_valid_evidence_packet_branch(branch))
        || query
            .release_id
            .as_deref()
            .is_some_and(|release_id| !is_valid_evidence_packet_release_id(release_id))
        || query
            .environment
            .as_deref()
            .is_some_and(|environment| !is_valid_evidence_packet_environment(environment))
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(EvidencePacketResponse::default()),
        );
    }
    if let Some(target_sha) = query.target_sha.as_mut() {
        if !is_valid_evidence_packet_sha(target_sha) {
            return (
                StatusCode::BAD_REQUEST,
                Json(EvidencePacketResponse::default()),
            );
        }
        *target_sha = target_sha.to_ascii_lowercase();
    }
    let release_context_requested =
        query.target_sha.is_some() || query.release_id.is_some() || query.environment.is_some();
    if release_context_requested
        && (query.repo_full_name.is_none()
            || query.branch.is_none()
            || query.target_sha.is_none()
            || query.release_id.is_none()
            || query.environment.is_none())
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(EvidencePacketResponse::default()),
        );
    }
    if release_context_requested && !is_valid_release_bound_evidence_ticket_id(&normalized) {
        return (
            StatusCode::BAD_REQUEST,
            Json(EvidencePacketResponse::default()),
        );
    }

    let scoped_org = match resolve_required_product_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        query.org_name.as_deref(),
    )
    .await
    {
        Ok(org) => org,
        Err(err) => {
            return (
                org_scope_status(err),
                Json(EvidencePacketResponse::default()),
            )
        }
    };

    let hours = query.hours.unwrap_or(24 * 30).clamp(1, 24 * 90);
    let ticket = match state
        .db
        .get_project_ticket_by_ticket_id(&normalized, Some(scoped_org.id.as_str()))
        .await
    {
        Ok(ticket) => ticket,
        Err(e) => {
            tracing::error!(ticket_id = %normalized, error = %e, "Failed to load ticket for evidence packet");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(EvidencePacketResponse::default()),
            );
        }
    };

    let flow_query = CorrelationV2Query {
        org_name: None,
        org_id: Some(scoped_org.id.clone()),
        repo_full_name: query.repo_full_name.clone(),
        branch: query.branch.clone(),
        target_sha: query.target_sha.clone(),
        ticket_id: Some(normalized.clone()),
        hours: Some(hours),
        limit: 500,
        offset: 0,
    };
    let mut commits = match state.db.get_ticket_flow_correlations_v2(&flow_query).await {
        Ok((items, _)) => items,
        Err(e) => {
            tracing::error!(ticket_id = %normalized, error = %e, "Failed to load commit correlations for evidence packet");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(EvidencePacketResponse::default()),
            );
        }
    };

    let commit_shas: Vec<String> = commits
        .iter()
        .map(|item| item.commit_sha.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let pull_requests = match state
        .db
        .get_pr_merge_evidence_for_ticket_packet(PrMergeEvidenceForTicketPacketQuery {
            scope_org_id: Some(scoped_org.id.as_str()),
            org_name: None,
            repo_full_name: query.repo_full_name.as_deref(),
            branch: query.branch.as_deref(),
            target_sha: query.target_sha.as_deref(),
            ticket_id: &normalized,
            commit_shas: &commit_shas,
            hours,
        })
        .await
    {
        Ok(items) => items,
        Err(e) => {
            tracing::error!(ticket_id = %normalized, error = %e, "Failed to load PR evidence for evidence packet");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(EvidencePacketResponse::default()),
            );
        }
    };

    let evidence_shas = collect_evidence_packet_shas(&commits, &pull_requests);
    let pipelines = match state
        .db
        .get_pipeline_runs_for_evidence_packet(PipelineRunsForEvidencePacketQuery {
            scope_org_id: scoped_org.id.as_str(),
            repo_full_name: query.repo_full_name.as_deref(),
            branch: query.branch.as_deref(),
            commit_shas: &evidence_shas,
            allow_legacy_scope_fallback: !release_context_requested,
        })
        .await
    {
        Ok(items) => items,
        Err(e) => {
            tracing::error!(ticket_id = %normalized, error = %e, "Failed to load pipeline evidence for evidence packet");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(EvidencePacketResponse::default()),
            );
        }
    };

    align_commit_embedded_pipelines_with_packet_runs(&mut commits, &pipelines);

    let quality_gates = evidence_packet_quality_gate_runs(&pipelines);
    let completeness = build_evidence_packet_completeness(
        ticket.is_some(),
        &commits,
        &pull_requests,
        &pipelines,
        &quality_gates,
    );
    let found = completeness.ticket_found
        || completeness.commits > 0
        || completeness.pull_requests > 0
        || completeness.pipelines > 0;

    if !found
        || query.target_sha.as_deref().is_some_and(|target_sha| {
            !target_sha_exists_in_evidence(target_sha, &commits, &pull_requests)
        })
    {
        return (
            StatusCode::NOT_FOUND,
            Json(EvidencePacketResponse {
                found: false,
                packet: None,
            }),
        );
    }

    let reconstruction =
        build_evidence_packet_reconstruction(EvidencePacketReconstructionInput {
            query: &query,
            scoped_org_login: scoped_org.login.as_str(),
            ticket_id: &normalized,
            hours,
            commits: &commits,
            pull_requests: &pull_requests,
            pipelines: &pipelines,
            quality_gates: &quality_gates,
        });

    let mut packet = EvidencePacket {
        packet_type: "ticket".to_string(),
        subject: normalized,
        generated_at: chrono::Utc::now().timestamp_millis(),
        org_name: Some(scoped_org.login.clone()),
        repo_full_name: query.repo_full_name,
        branch: query.branch,
        target_sha: query.target_sha,
        release_id: query.release_id.clone(),
        environment: query.environment.clone(),
        period: format!("last_{}h", hours),
        ticket,
        commits,
        pull_requests,
        pipelines,
        quality_gates,
        reconstruction,
        completeness,
        content_hash: String::new(),
    };
    packet.content_hash = evidence_packet_hash(&packet);

    let release_binding_context = packet
        .repo_full_name
        .as_deref()
        .zip(packet.branch.as_deref())
        .zip(packet.target_sha.as_deref())
        .zip(packet.release_id.as_deref())
        .zip(packet.environment.as_deref());
    if let Some(((((repo_full_name, branch), target_sha), release_id), environment)) =
        release_binding_context
    {
        let evidence_packet_uri = format!(
            "/evidence/packets/tickets/{}?repo_full_name={}&branch={}&target_sha={}&release_id={}&environment={}&hours={}",
            evidence_packet_uri_encode(&packet.subject),
            evidence_packet_uri_encode(repo_full_name),
            evidence_packet_uri_encode(branch),
            evidence_packet_uri_encode(target_sha),
            evidence_packet_uri_encode(release_id),
            evidence_packet_uri_encode(environment),
            hours
        );
        let binding = ReleaseEvidencePacketBinding {
            id: Uuid::new_v4().to_string(),
            org_id: scoped_org.id.clone(),
            ticket_id: packet.subject.clone(),
            release_id: release_id.to_string(),
            repository_full_name: repo_full_name.to_string(),
            branch: branch.to_string(),
            target_sha: target_sha.to_string(),
            environment: environment.to_string(),
            evidence_packet_hash: packet.content_hash.clone(),
            evidence_packet_uri,
            packet: serde_json::to_value(&packet).unwrap_or_else(|_| serde_json::json!({})),
            generated_by: auth_user.client_id.clone(),
            generated_at: packet.generated_at,
            created_at: packet.generated_at,
        };
        if let Err(e) = state
            .db
            .store_release_evidence_packet_binding(&binding)
            .await
        {
            tracing::error!(
                error = %e,
                org_id = %binding.org_id,
                ticket_id = %binding.ticket_id,
                "Failed to store release evidence packet binding"
            );
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(EvidencePacketResponse::default()),
            );
        }
    }

    (
        StatusCode::OK,
        Json(EvidencePacketResponse {
            found: true,
            packet: Some(packet),
        }),
    )
}

pub async fn get_jenkins_integration_status(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<IntegrationStatusQuery>,
) -> impl IntoResponse {
    if require_admin(&auth_user).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(JenkinsIntegrationStatusResponse {
                ok: false,
                ..Default::default()
            }),
        );
    }

    let scoped_org = match resolve_required_product_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        query.org_name.as_deref(),
    )
    .await
    {
        Ok(org) => org,
        Err(err) => {
            return (
                org_scope_status(err),
                Json(JenkinsIntegrationStatusResponse {
                    ok: false,
                    ..Default::default()
                }),
            )
        }
    };

    match state
        .db
        .get_jenkins_integration_status(Some(scoped_org.id.as_str()))
        .await
    {
        Ok(status) => (StatusCode::OK, Json(status)),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JenkinsIntegrationStatusResponse {
                ok: false,
                ..Default::default()
            }),
        ),
    }
}

fn read_metadata_commit_message(metadata: &serde_json::Value) -> Option<&str> {
    metadata
        .as_object()
        .and_then(|m| m.get("commit_message"))
        .and_then(|v| v.as_str())
}

fn collect_pr_ticket_matches(
    head_sha: Option<&str>,
    pr_title: Option<&str>,
    tickets_by_commit_sha: &HashMap<String, HashSet<String>>,
    phase1_tickets: &HashSet<String>,
) -> Vec<String> {
    let mut matched_tickets: HashSet<String> = HashSet::new();

    if let Some(sha) = head_sha {
        if let Some(commit_tickets) = tickets_by_commit_sha.get(sha) {
            matched_tickets.extend(commit_tickets.iter().cloned());
        }
    }

    if let Some(title) = pr_title {
        for ticket_id in extract_ticket_ids(&[title]) {
            if phase1_tickets.contains(&ticket_id) {
                matched_tickets.insert(ticket_id);
            }
        }
    }

    let mut matched_tickets: Vec<String> = matched_tickets.into_iter().collect();
    matched_tickets.sort();
    matched_tickets
}

#[cfg(test)]
mod jira_pr_correlation_tests {
    use super::*;

    fn sign_jira_payload(secret: &str, payload: &[u8]) -> String {
        let mut mac =
            <hmac::Hmac<Sha256> as Mac>::new_from_slice(secret.as_bytes()).expect("test HMAC key");
        mac.update(payload);
        format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
    }

    fn ticket_set(values: &[&str]) -> HashSet<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn head_sha_match_only_returns_tickets_for_that_commit() {
        let mut tickets_by_commit_sha: HashMap<String, HashSet<String>> = HashMap::new();
        tickets_by_commit_sha.insert("sha-a".to_string(), ticket_set(&["ABC-1"]));
        tickets_by_commit_sha.insert("sha-b".to_string(), ticket_set(&["ABC-2"]));
        let phase1_tickets = ticket_set(&["ABC-1", "ABC-2"]);

        let matched =
            collect_pr_ticket_matches(Some("sha-a"), None, &tickets_by_commit_sha, &phase1_tickets);

        assert_eq!(matched, vec!["ABC-1".to_string()]);
    }

    #[test]
    fn title_match_is_limited_to_phase1_candidates() {
        let mut tickets_by_commit_sha: HashMap<String, HashSet<String>> = HashMap::new();
        tickets_by_commit_sha.insert("sha-a".to_string(), ticket_set(&["ABC-1"]));
        let phase1_tickets = ticket_set(&["ABC-1", "ABC-2"]);

        let matched = collect_pr_ticket_matches(
            Some("sha-a"),
            Some("Implements ABC-2 and XYZ-99"),
            &tickets_by_commit_sha,
            &phase1_tickets,
        );

        assert_eq!(matched, vec!["ABC-1".to_string(), "ABC-2".to_string()]);
    }

    #[test]
    fn jira_org_name_hint_prefers_explicit_org_name() {
        let payload = JiraWebhookEvent {
            webhook_event: Some("jira:issue_updated".to_string()),
            timestamp: Some(1_700_000_000_000),
            issue: None,
            user: None,
            extra: HashMap::from([
                ("organization".to_string(), serde_json::json!("legacy-org")),
                ("org_name".to_string(), serde_json::json!("gitgov-team")),
            ]),
        };

        let hint = jira_org_name_hint(&payload);
        assert_eq!(hint.as_deref(), Some("gitgov-team"));
    }

    #[test]
    fn jira_org_name_hint_returns_none_when_missing() {
        let payload = JiraWebhookEvent {
            webhook_event: Some("jira:issue_created".to_string()),
            timestamp: Some(1_700_000_000_000),
            issue: None,
            user: None,
            extra: HashMap::new(),
        };

        assert_eq!(jira_org_name_hint(&payload), None);
    }

    #[test]
    fn validates_jira_hmac_signature() {
        let payload = br#"{"issue":{"key":"KAN-6"}}"#;
        let signature = sign_jira_payload("secret", payload);

        assert!(validate_jira_signature("secret", payload, &signature));
        assert!(!validate_jira_signature(
            "other-secret",
            payload,
            &signature
        ));
        assert!(!validate_jira_signature(
            "secret",
            br#"{"issue":{"key":"KAN-7"}}"#,
            &signature
        ));
    }

    #[test]
    fn evidence_packet_completeness_reports_missing_signals() {
        let completeness = build_evidence_packet_completeness(true, &[], &[], &[], &[]);

        assert!(completeness.ticket_found);
        assert_eq!(completeness.commits, 0);
        assert_eq!(completeness.pull_requests, 0);
        assert_eq!(completeness.pipelines, 0);
        assert_eq!(completeness.quality_gates, 0);
        assert_eq!(
            completeness.missing,
            vec![
                "commits".to_string(),
                "pull_requests".to_string(),
                "pipelines".to_string(),
                "quality_gates".to_string(),
            ]
        );
    }

    #[test]
    fn evidence_packet_hash_ignores_existing_hash_field() {
        let mut packet = EvidencePacket {
            packet_type: "ticket".to_string(),
            subject: "KAN-23".to_string(),
            generated_at: 1_777_560_000_000,
            org_name: Some("yohandry10".to_string()),
            repo_full_name: Some("yohandry10/Git-Gov".to_string()),
            branch: Some("main".to_string()),
            target_sha: Some("abcdef1234567890abcdef1234567890abcdef12".to_string()),
            release_id: Some("KAN-23".to_string()),
            environment: Some("production".to_string()),
            period: "last_720h".to_string(),
            ticket: None,
            commits: vec![],
            pull_requests: vec![],
            pipelines: vec![],
            quality_gates: vec![],
            reconstruction: EvidencePacketReconstruction::default(),
            completeness: EvidencePacketCompleteness {
                ticket_found: false,
                commits: 0,
                pull_requests: 0,
                pipelines: 0,
                quality_gates: 0,
                missing: vec!["ticket".to_string()],
            },
            content_hash: String::new(),
        };

        let first = evidence_packet_hash(&packet);
        packet.content_hash = "different-existing-value".to_string();
        let second = evidence_packet_hash(&packet);

        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
    }
}

pub async fn correlate_jira_tickets(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<JiraCorrelateRequest>,
) -> impl IntoResponse {
    if require_admin(&auth_user).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(JiraCorrelateResponse::default()),
        );
    }

    let scoped_org = match resolve_required_product_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        payload.org_name.as_deref(),
    )
    .await
    {
        Ok(scope) => scope,
        Err(err) => {
            return (
                org_scope_status(err),
                Json(JiraCorrelateResponse::default()),
            )
        }
    };

    let hours = payload.hours.unwrap_or(24).clamp(1, 24 * 30);
    let limit = payload.limit.unwrap_or(500).clamp(1, 5000);

    let commits = match state
        .db
        .get_recent_commit_events_for_ticket_correlation(
            Some(scoped_org.login.as_str()),
            payload.repo_full_name.as_deref(),
            hours,
            limit,
        )
        .await
    {
        Ok(commits) => commits,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JiraCorrelateResponse::default()),
            )
        }
    };

    let mut created = 0i64;
    let mut scanned_prs = 0i64;
    let mut correlated_tickets: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    let mut phase1_tickets: HashSet<String> = HashSet::new();
    let mut tickets_by_commit_sha: HashMap<String, HashSet<String>> = HashMap::new();

    for (commit_sha, branch, org_id, metadata, _repo_name) in &commits {
        let mut commit_sources: Vec<(&str, Vec<String>)> = Vec::new();
        if let Some(msg) = read_metadata_commit_message(metadata) {
            let tickets = extract_ticket_ids(&[msg]);
            if !tickets.is_empty() {
                commit_sources.push(("commit_message", tickets));
            }
        }
        if let Some(branch_name) = branch.as_deref() {
            let tickets = extract_ticket_ids(&[branch_name]);
            if !tickets.is_empty() {
                commit_sources.push(("branch_name", tickets));
            }
        }

        for (source, tickets) in commit_sources {
            for ticket_id in tickets {
                let correlation = CommitTicketCorrelation {
                    id: Uuid::new_v4().to_string(),
                    org_id: org_id.clone(),
                    commit_sha: commit_sha.clone(),
                    ticket_id: ticket_id.clone(),
                    correlation_source: source.to_string(),
                    confidence: if source == "commit_message" { 1.0 } else { 0.8 },
                    created_at: chrono::Utc::now().timestamp_millis(),
                };
                if let Ok(was_created) = state
                    .db
                    .insert_commit_ticket_correlation(&correlation)
                    .await
                {
                    phase1_tickets.insert(ticket_id.clone());
                    tickets_by_commit_sha
                        .entry(correlation.commit_sha.clone())
                        .or_default()
                        .insert(ticket_id.clone());

                    if was_created {
                        created += 1;
                        correlated_tickets.insert(ticket_id);
                        if let Err(e) = state
                            .db
                            .append_project_ticket_relations(
                                &correlation.ticket_id,
                                correlation.org_id.as_deref(),
                                Some(&correlation.commit_sha),
                                branch.as_deref(),
                            )
                            .await
                        {
                            tracing::warn!(
                                ticket_id = %correlation.ticket_id,
                                commit_sha = %correlation.commit_sha,
                                error = %e,
                                "Failed to append Jira ticket relations after correlation"
                            );
                        }
                    }
                }
            }
        }
    }

    // --- Phase 1b: PR title -> merge commit backfill ---
    // Merged PRs are often the commits that land on main. If the PR title has a
    // ticket ID, correlate both merge_commit_sha and head_sha so ticket coverage
    // reflects the governed merge path rather than only workstation commits.
    match state
        .db
        .get_recent_pr_merges_for_ticket_correlation(
            Some(scoped_org.login.as_str()),
            payload.repo_full_name.as_deref(),
            hours,
            limit,
        )
        .await
    {
        Ok(prs) => {
            scanned_prs = prs.len() as i64;
            for (
                pr_org_id,
                pr_number,
                pr_title,
                head_sha,
                merge_commit_sha,
                base_branch,
                repo_full_name,
            ) in prs
            {
                let Some(title) = pr_title.as_deref() else {
                    continue;
                };
                let ticket_ids = extract_ticket_ids(&[title]);
                if ticket_ids.is_empty() {
                    continue;
                }

                let mut targets: Vec<(&str, &str)> = Vec::new();
                if let Some(sha) = merge_commit_sha
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                {
                    targets.push(("pr_title", sha));
                }
                if let Some(sha) = head_sha.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
                    let already_included = targets
                        .iter()
                        .any(|(_, existing_sha)| existing_sha.eq_ignore_ascii_case(sha));
                    if !already_included {
                        targets.push(("pr_title", sha));
                    }
                }

                let pr_ref = repo_full_name
                    .as_deref()
                    .map(|repo| format!("{}#{}", repo, pr_number))
                    .unwrap_or_else(|| format!("#{}", pr_number));

                for ticket_id in ticket_ids {
                    for (source, commit_sha) in &targets {
                        let correlation = CommitTicketCorrelation {
                            id: Uuid::new_v4().to_string(),
                            org_id: pr_org_id.clone(),
                            commit_sha: (*commit_sha).to_string(),
                            ticket_id: ticket_id.clone(),
                            correlation_source: (*source).to_string(),
                            confidence: 0.9,
                            created_at: chrono::Utc::now().timestamp_millis(),
                        };
                        if let Ok(was_created) = state
                            .db
                            .insert_commit_ticket_correlation(&correlation)
                            .await
                        {
                            tickets_by_commit_sha
                                .entry(correlation.commit_sha.clone())
                                .or_default()
                                .insert(ticket_id.clone());
                            phase1_tickets.insert(ticket_id.clone());

                            if was_created {
                                created += 1;
                                correlated_tickets.insert(ticket_id.clone());
                            }

                            if let Err(e) = state
                                .db
                                .append_project_ticket_relations_full(
                                    &ticket_id,
                                    correlation.org_id.as_deref(),
                                    Some(commit_sha),
                                    base_branch.as_deref(),
                                    Some(&pr_ref),
                                )
                                .await
                            {
                                tracing::warn!(
                                    ticket_id = %ticket_id,
                                    commit_sha = %commit_sha,
                                    pr_ref = %pr_ref,
                                    error = %e,
                                    "Failed to append Jira ticket relations after PR title backfill"
                                );
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to scan merged PRs for ticket backfill");
        }
    }

    // --- Phase 2: PR auto-correlation ---
    // Find PRs whose head_sha matches any correlated commit, or whose title
    // mentions any correlated ticket ID. Then append them to related_prs.
    if !phase1_tickets.is_empty() {
        let correlated_shas: Vec<String> = tickets_by_commit_sha.keys().cloned().collect();
        let ticket_list: Vec<String> = phase1_tickets.iter().cloned().collect();

        match state
            .db
            .find_prs_related_to_tickets(
                &correlated_shas,
                &ticket_list,
                Some(scoped_org.id.as_str()),
                payload.repo_full_name.as_deref(),
                hours,
            )
            .await
        {
            Ok(prs) => {
                for (pr_number, pr_title, head_sha, repo_full_name) in &prs {
                    let pr_ref = if let Some(repo) = repo_full_name {
                        format!("{}#{}", repo, pr_number)
                    } else {
                        format!("#{}", pr_number)
                    };
                    let matched_tickets = collect_pr_ticket_matches(
                        head_sha.as_deref(),
                        pr_title.as_deref(),
                        &tickets_by_commit_sha,
                        &phase1_tickets,
                    );

                    for ticket_id in &matched_tickets {
                        if let Err(e) = state
                            .db
                            .append_project_ticket_relations_full(
                                ticket_id,
                                Some(scoped_org.id.as_str()),
                                None,
                                None,
                                Some(&pr_ref),
                            )
                            .await
                        {
                            tracing::warn!(
                                ticket_id = %ticket_id,
                                pr_ref = %pr_ref,
                                error = %e,
                                "Failed to append PR relation to ticket"
                            );
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to find PRs for ticket correlation");
            }
        }
    }

    let mut correlated_tickets: Vec<String> = correlated_tickets.into_iter().collect();
    correlated_tickets.sort();

    (
        StatusCode::OK,
        Json(JiraCorrelateResponse {
            scanned_commits: commits.len() as i64,
            scanned_prs,
            correlations_created: created,
            correlated_tickets,
        }),
    )
}

pub async fn get_jira_ticket_coverage(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<TicketCoverageQuery>,
) -> impl IntoResponse {
    if require_admin(&auth_user).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(TicketCoverageResponse::default()),
        );
    }

    let scoped_org = match resolve_required_product_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        query.org_name.as_deref(),
    )
    .await
    {
        Ok(org) => org,
        Err(err) => {
            return (
                org_scope_status(err),
                Json(TicketCoverageResponse::default()),
            )
        }
    };

    let hours = query.hours.unwrap_or(24).clamp(1, 24 * 30);
    match state
        .db
        .get_ticket_coverage(
            Some(scoped_org.login.as_str()),
            Some(scoped_org.id.as_str()),
            query.repo_full_name.as_deref(),
            query.branch.as_deref(),
            hours,
        )
        .await
    {
        Ok(resp) => (StatusCode::OK, Json(resp)),
        Err(e) => {
            tracing::error!(error = %e, "Failed to compute Jira ticket coverage");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TicketCoverageResponse::default()),
            )
        }
    }
}

pub async fn get_jenkins_commit_correlations(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(filter): Query<JenkinsCorrelationFilter>,
) -> impl IntoResponse {
    if require_admin(&auth_user).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(JenkinsCorrelationsResponse::default()),
        );
    }

    let scoped_org = match resolve_required_product_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        filter.org_name.as_deref(),
    )
    .await
    {
        Ok(org) => org,
        Err(err) => {
            return (
                org_scope_status(err),
                Json(JenkinsCorrelationsResponse::default()),
            )
        }
    };

    let filter = JenkinsCorrelationFilter {
        limit: if filter.limit == 0 { 20 } else { filter.limit },
        org_name: None,
        org_id: Some(scoped_org.id),
        ..filter
    };

    match state.db.get_commit_pipeline_correlations(&filter).await {
        Ok(correlations) => (
            StatusCode::OK,
            Json(JenkinsCorrelationsResponse { correlations }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JenkinsCorrelationsResponse::default()),
        ),
    }
}

pub async fn get_correlation_v2(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<CorrelationV2Query>,
) -> impl IntoResponse {
    if require_admin(&auth_user).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(CorrelationV2Response::default()),
        );
    }

    let scoped_org = match resolve_required_product_org_scope(
        &state,
        auth_user.org_id.as_deref(),
        query.org_name.as_deref(),
    )
    .await
    {
        Ok(org) => org,
        Err(err) => {
            return (
                org_scope_status(err),
                Json(CorrelationV2Response::default()),
            )
        }
    };

    let limit = if query.limit == 0 {
        50
    } else {
        query.limit.min(500)
    };
    let offset = query.offset;

    let filter = CorrelationV2Query {
        ticket_id: query
            .ticket_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_ascii_uppercase()),
        limit,
        offset,
        org_name: None,
        org_id: Some(scoped_org.id),
        ..query
    };

    match state.db.get_ticket_flow_correlations_v2(&filter).await {
        Ok((items, total)) => (
            StatusCode::OK,
            Json(CorrelationV2Response {
                items,
                total,
                limit: limit as i64,
                offset: offset as i64,
            }),
        ),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get correlation v2 view");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CorrelationV2Response::default()),
            )
        }
    }
}
