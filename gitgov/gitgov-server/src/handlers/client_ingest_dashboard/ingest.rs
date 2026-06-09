// ============================================================================
// CLIENT EVENTS (Batch Ingest)
// ============================================================================

pub async fn ingest_client_events(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(batch): Json<ClientEventBatch>,
) -> impl IntoResponse {
    let batch_len = batch.events.len();
    metrics::histogram!("gitgov_events_batch_size").record(batch_len as f64);
    if state.events_max_batch > 0 && batch_len > state.events_max_batch {
        tracing::warn!(
            auth_user = %auth_user.client_id,
            batch_len,
            max_batch = state.events_max_batch,
            "Rejecting /events payload because it exceeds max configured batch size"
        );
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(ClientEventResponse {
                accepted: vec![],
                duplicates: vec![],
                errors: vec![EventError {
                    event_uuid: "batch".to_string(),
                    error: format!(
                        "Too many events in a single request: {} (max {})",
                        batch_len, state.events_max_batch
                    ),
                }],
            }),
        );
    }

    let mut events = Vec::new();
    let mut pre_validation_errors: Vec<EventError> = Vec::new();
    let strict_actor_match = state.strict_actor_match;
    let mut org_id_cache: HashMap<String, Option<String>> = HashMap::new();
    let mut repo_cache: HashMap<String, Option<Repo>> = HashMap::new();

    for input in batch.events {
        if strict_actor_match
            && auth_user.role != UserRole::Admin
            && input.user_login != auth_user.client_id
        {
            tracing::warn!(
                auth_user = %auth_user.client_id,
                requested_user_login = %input.user_login,
                event_uuid = %input.event_uuid,
                "Rejecting client event due to strict actor match enforcement"
            );
            pre_validation_errors.push(EventError {
                event_uuid: input.event_uuid,
                error: "user_login must match authenticated client_id (STRICT_ACTOR_MATCH)"
                    .to_string(),
            });
            continue;
        }

        let effective_user_login = if auth_user.role == UserRole::Admin {
            input.user_login.clone()
        } else {
            auth_user.client_id.clone()
        };

        if state.reject_synthetic_logins && is_likely_synthetic_login(&effective_user_login) {
            tracing::warn!(
                auth_user = %auth_user.client_id,
                rejected_user_login = %effective_user_login,
                event_uuid = %input.event_uuid,
                "Rejecting client event due to synthetic login policy"
            );
            pre_validation_errors.push(EventError {
                event_uuid: input.event_uuid,
                error: "synthetic user_login is not allowed in this environment".to_string(),
            });
            continue;
        }

        // Get org and repo IDs
        let requested_org_id = if let Some(ref org_name) = input.org_name {
            if let Some(cached) = org_id_cache.get(org_name) {
                cached.clone()
            } else {
                let resolved = resolve_org_id_with_cache(&state, org_name).await;
                org_id_cache.insert(org_name.clone(), resolved.clone());
                resolved
            }
        } else {
            None
        };

        if auth_user.role != UserRole::Admin {
            if let (Some(scoped_org_id), Some(requested_org_id)) =
                (auth_user.org_id.as_deref(), requested_org_id.as_deref())
            {
                if scoped_org_id != requested_org_id {
                    tracing::warn!(
                        auth_user = %auth_user.client_id,
                        requested_org_id = %requested_org_id,
                        scoped_org_id = %scoped_org_id,
                        event_uuid = %input.event_uuid,
                        "Rejecting client event with org mismatch"
                    );
                    pre_validation_errors.push(EventError {
                        event_uuid: input.event_uuid,
                        error: "Event org_name is outside API key scope".to_string(),
                    });
                    continue;
                }
            }
        }

        let org_id = if auth_user.role == UserRole::Admin {
            requested_org_id
        } else {
            auth_user.org_id.clone().or(requested_org_id)
        };

        let inferred_repo_full_name = input.repo_full_name.clone().or_else(|| {
            input
                .metadata
                .as_ref()
                .and_then(|m| m.get("repo_name"))
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToOwned::to_owned)
        });

        let repo = if let Some(ref repo_full_name) = inferred_repo_full_name {
            if let Some(cached) = repo_cache.get(repo_full_name) {
                cached.clone()
            } else {
                let resolved = resolve_repo_with_cache(&state, repo_full_name).await;
                repo_cache.insert(repo_full_name.clone(), resolved.clone());
                resolved
            }
        } else {
            None
        };
        if auth_user.role != UserRole::Admin {
            if let (Some(scoped_org_id), Some(repo)) = (auth_user.org_id.as_deref(), repo.as_ref())
            {
                if repo.org_id.as_deref() != Some(scoped_org_id) {
                    tracing::warn!(
                        auth_user = %auth_user.client_id,
                        repo = %repo.full_name,
                        event_uuid = %input.event_uuid,
                        "Rejecting client event with repo outside API key scope"
                    );
                    pre_validation_errors.push(EventError {
                        event_uuid: input.event_uuid,
                        error: "Event repo_full_name is outside API key scope".to_string(),
                    });
                    continue;
                }
            }
        }
        let repo_id = if let Some(repo) = repo {
            Some(repo.id)
        } else if let (Some(full_name), Some(effective_org_id)) =
            (inferred_repo_full_name.as_deref(), org_id.as_deref())
        {
            let repo_name = full_name
                .split('/')
                .next_back()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or(full_name);
            if should_schedule_repo_upsert(state.as_ref(), effective_org_id, full_name) {
                schedule_repo_upsert(
                    Arc::clone(&state),
                    effective_org_id.to_string(),
                    full_name.to_string(),
                    repo_name.to_string(),
                    input.event_uuid.clone(),
                );
            }
            None
        } else {
            None
        };

        let event = ClientEvent {
            id: Uuid::new_v4().to_string(),
            org_id,
            repo_id,
            event_uuid: input.event_uuid,
            event_type: ClientEventType::from_str(&input.event_type),
            user_login: effective_user_login,
            user_name: input.user_name,
            branch: input.branch,
            commit_sha: input.commit_sha,
            files: input.files,
            status: EventStatus::from_str(&input.status),
            reason: input.reason,
            metadata: input.metadata.unwrap_or(serde_json::Value::Null),
            client_version: batch.client_version.clone(),
            created_at: input
                .timestamp
                .unwrap_or_else(|| chrono::Utc::now().timestamp_millis()),
        };

        events.push(event);
    }

    if events.is_empty() {
        return (
            StatusCode::OK,
            Json(ClientEventResponse {
                accepted: vec![],
                duplicates: vec![],
                errors: pre_validation_errors,
            }),
        );
    }

    match state.db.insert_client_events_batch(&events).await {
        Ok(mut response) => {
            if !pre_validation_errors.is_empty() {
                response.errors.extend(pre_validation_errors);
            }
            // Prometheus counters
            metrics::counter!("gitgov_events_ingested_total", "status" => "accepted")
                .increment(response.accepted.len() as u64);
            metrics::counter!("gitgov_events_ingested_total", "status" => "duplicate")
                .increment(response.duplicates.len() as u64);
            metrics::counter!("gitgov_events_ingested_total", "status" => "error")
                .increment(response.errors.len() as u64);

            // Notify SSE subscribers about new events (fire-and-forget).
            // Single notification — frontend refreshes both logs and stats on new_events.
            let accepted_count = response.accepted.len() as u32;
            if accepted_count > 0 {
                fanout_sse_new_events(&state, accepted_count).await;
            }

            // Fire-and-forget (debounced): update client_sessions last_seen + device metadata.
            let should_touch_session =
                !response.accepted.is_empty() || !response.duplicates.is_empty();
            if should_touch_session {
                let client_id = auth_user.client_id.clone();
                let org_id = auth_user.org_id.clone();
                // Extract device metadata from the first event that has it
                let device_meta = events
                    .iter()
                    .find_map(|e| e.metadata.get("device").cloned())
                    .unwrap_or(serde_json::json!({}));
                if should_upsert_client_session(&state, &client_id) {
                    let db = Arc::clone(&state.db);
                    tokio::spawn(async move {
                        if let Err(e) = db
                            .upsert_client_session(&client_id, org_id.as_deref(), &device_meta)
                            .await
                        {
                            tracing::debug!(
                                error = %e,
                                "Failed to upsert client session (non-critical)"
                            );
                        }
                    });
                }
            }

            // Fire-and-forget alert for blocked_push events
            if let Some(ref webhook_url) = state.alert_webhook_url {
                let accepted_event_ids: HashSet<&str> =
                    response.accepted.iter().map(String::as_str).collect();
                for event in &events {
                    if event.event_type == ClientEventType::BlockedPush
                        && accepted_event_ids.contains(event.event_uuid.as_str())
                    {
                        let text = notifications::format_blocked_push_alert(
                            &event.user_login,
                            event.repo_id.as_deref().unwrap_or("unknown"),
                            event.branch.as_deref().unwrap_or("unknown"),
                        );
                        let client = state.http_client.clone();
                        let url = webhook_url.clone();
                        tokio::spawn(async move {
                            notifications::send_alert(&client, &url, text).await;
                        });
                    }
                }
            }
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to insert client events batch: {}", e);
            let mut errors = pre_validation_errors;
            errors.push(EventError {
                event_uuid: "batch".to_string(),
                error: "Internal database error".to_string(),
            });
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ClientEventResponse {
                    accepted: vec![],
                    duplicates: vec![],
                    errors,
                }),
            )
        }
    }
}

// ============================================================================
// QUERY ENDPOINTS
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct OutboxLeaseRequest {
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub holder: Option<String>,
    #[serde(default)]
    pub lease_ttl_ms: Option<u64>,
    #[serde(default)]
    pub max_wait_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutboxLeaseResponse {
    pub granted: bool,
    pub wait_ms: u64,
    pub lease_ttl_ms: u64,
    pub mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutboxLeaseTelemetryResponse {
    pub enabled: bool,
    pub default_lease_ttl_ms: u64,
    pub telemetry: OutboxLeaseTelemetrySnapshot,
}
