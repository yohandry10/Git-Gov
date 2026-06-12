// ============================================================================
// CLIENT EVENTS (Batch Ingest)
// ============================================================================

/// Maximum clock skew tolerated for a client-supplied event timestamp that
/// claims to be in the future. No legitimate event happens ahead of the
/// server's clock, so events beyond this skew are rejected. This prevents
/// postdating the (client-controlled) `created_at` to push events into future
/// time-window governance queries or to corrupt audit ordering. Past timestamps
/// remain allowed because the offline outbox legitimately backfills older events.
const EVENT_FUTURE_SKEW_MS: i64 = 5 * 60 * 1000;

/// Returns true when a client-supplied event timestamp is implausibly in the
/// future (beyond the allowed clock skew) relative to `now_ms`.
fn event_timestamp_too_far_in_future(timestamp_ms: i64, now_ms: i64) -> bool {
    timestamp_ms > now_ms.saturating_add(EVENT_FUTURE_SKEW_MS)
}

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
    let mut org_login_cache: HashMap<String, Option<String>> = HashMap::new();
    let mut repo_cache: HashMap<String, Option<Repo>> = HashMap::new();

    for input in batch.events {
        let event_type = match ClientEventType::parse(&input.event_type) {
            Some(event_type) => event_type,
            None => {
                pre_validation_errors.push(EventError {
                    event_uuid: input.event_uuid,
                    error: format!("unsupported event_type '{}'", input.event_type),
                });
                continue;
            }
        };
        let status = match EventStatus::parse(&input.status) {
            Some(status) => status,
            None => {
                pre_validation_errors.push(EventError {
                    event_uuid: input.event_uuid,
                    error: format!("unsupported status '{}'", input.status),
                });
                continue;
            }
        };

        if let Some(timestamp_ms) = input.timestamp {
            let now_ms = chrono::Utc::now().timestamp_millis();
            if event_timestamp_too_far_in_future(timestamp_ms, now_ms) {
                tracing::warn!(
                    auth_user = %auth_user.client_id,
                    event_uuid = %input.event_uuid,
                    timestamp_ms,
                    now_ms,
                    "Rejecting client event with timestamp too far in the future"
                );
                pre_validation_errors.push(EventError {
                    event_uuid: input.event_uuid,
                    error: "event timestamp is too far in the future".to_string(),
                });
                continue;
            }
        }

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
        if input.org_name.is_some() && requested_org_id.is_none() {
            tracing::warn!(
                auth_user = %auth_user.client_id,
                requested_org_name = %input.org_name.as_deref().unwrap_or_default(),
                event_uuid = %input.event_uuid,
                "Rejecting client event with unknown org_name"
            );
            pre_validation_errors.push(EventError {
                event_uuid: input.event_uuid,
                error: "Event org_name was not found".to_string(),
            });
            continue;
        }

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
        let repo_owner = if let Some(repo_full_name) = inferred_repo_full_name.as_deref() {
            match repo_full_name_owner(repo_full_name) {
                Some(owner) => Some(owner.to_string()),
                None => {
                    tracing::warn!(
                        repo = %repo_full_name,
                        event_uuid = %input.event_uuid,
                        "Rejecting client event with malformed repo_full_name"
                    );
                    pre_validation_errors.push(EventError {
                        event_uuid: input.event_uuid,
                        error: "repo_full_name must be in owner/repo format".to_string(),
                    });
                    continue;
                }
            }
        } else {
            None
        };

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
        let repo_org_id = repo.as_ref().and_then(|repo| repo.org_id.clone());
        if let (Some(scoped_org_id), Some(repo_org_id)) =
            (auth_user.org_id.as_deref(), repo_org_id.as_deref())
        {
            if scoped_org_id != repo_org_id {
                tracing::warn!(
                    auth_user = %auth_user.client_id,
                    repo = %repo.as_ref().map(|r| r.full_name.as_str()).unwrap_or("unknown"),
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
        if let (Some(requested_org_id), Some(repo_org_id)) =
            (requested_org_id.as_deref(), repo_org_id.as_deref())
        {
            if requested_org_id != repo_org_id {
                tracing::warn!(
                    requested_org_id = %requested_org_id,
                    repo_org_id = %repo_org_id,
                    event_uuid = %input.event_uuid,
                    "Rejecting client event with repo/org mismatch"
                );
                pre_validation_errors.push(EventError {
                    event_uuid: input.event_uuid,
                    error: "Event org_name does not match repo_full_name organization".to_string(),
                });
                continue;
            }
        }
        let owner_org_id = if auth_user.org_id.is_none()
            && requested_org_id.is_none()
            && repo_org_id.is_none()
        {
            if let Some(owner) = repo_owner.as_deref() {
                if let Some(cached) = org_id_cache.get(owner) {
                    cached.clone()
                } else {
                    let resolved = resolve_org_id_with_cache(&state, owner).await;
                    org_id_cache.insert(owner.to_string(), resolved.clone());
                    resolved
                }
            } else {
                None
            }
        } else {
            None
        };

        let org_id = auth_user
            .org_id
            .clone()
            .or_else(|| requested_org_id.clone())
            .or_else(|| repo_org_id.clone())
            .or_else(|| owner_org_id.clone());
        let effective_org_id = if let Some(org_id) = org_id.as_deref() {
            org_id
        } else {
            tracing::warn!(
                auth_user = %auth_user.client_id,
                event_uuid = %input.event_uuid,
                "Rejecting client event without tenant scope"
            );
            pre_validation_errors.push(EventError {
                event_uuid: input.event_uuid,
                error: "org_name or resolvable repo_full_name is required for global admin keys".to_string(),
            });
            continue;
        };

        if let (Some(repo_full_name), Some(repo_owner)) =
            (inferred_repo_full_name.as_deref(), repo_owner.as_deref())
        {
            let effective_org_login = resolve_org_login_for_event(
                &state,
                effective_org_id,
                input.org_name.as_deref(),
                requested_org_id.as_deref(),
                &mut org_login_cache,
            )
            .await;
            let Some(effective_org_login) = effective_org_login else {
                tracing::warn!(
                    effective_org_id = %effective_org_id,
                    event_uuid = %input.event_uuid,
                    "Rejecting client event because tenant login could not be resolved"
                );
                pre_validation_errors.push(EventError {
                    event_uuid: input.event_uuid,
                    error: "Event organization could not be resolved".to_string(),
                });
                continue;
            };

            if !repo_owner.eq_ignore_ascii_case(effective_org_login.trim()) {
                tracing::warn!(
                    repo = %repo_full_name,
                    repo_owner = %repo_owner,
                    org_login = %effective_org_login,
                    event_uuid = %input.event_uuid,
                    "Rejecting client event with repo owner outside effective organization"
                );
                pre_validation_errors.push(EventError {
                    event_uuid: input.event_uuid,
                    error: "repo_full_name owner does not match event organization".to_string(),
                });
                continue;
            }
        }

        if let Some(error) = validate_event_capture_fidelity(
            &event_type,
            inferred_repo_full_name.as_deref(),
            input.branch.as_deref(),
            input.commit_sha.as_deref(),
            &input.files,
        ) {
            tracing::warn!(
                event_uuid = %input.event_uuid,
                event_type = %input.event_type,
                "Rejecting client event with incomplete capture context"
            );
            pre_validation_errors.push(EventError {
                event_uuid: input.event_uuid,
                error,
            });
            continue;
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
            event_type,
            user_login: effective_user_login,
            user_name: input.user_name,
            branch: input.branch,
            commit_sha: input.commit_sha,
            files: input.files,
            status,
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
                fanout_sse_new_events(&state, accepted_count, auth_user.org_id.as_deref()).await;
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
                    if matches!(
                        event.event_type,
                        ClientEventType::BlockedPush | ClientEventType::GovernanceBlockedPush
                    )
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

async fn resolve_org_login_for_event(
    state: &AppState,
    org_id: &str,
    requested_org_name: Option<&str>,
    requested_org_id: Option<&str>,
    org_login_cache: &mut HashMap<String, Option<String>>,
) -> Option<String> {
    if requested_org_id == Some(org_id) {
        if let Some(name) = requested_org_name.map(str::trim).filter(|name| !name.is_empty()) {
            return Some(name.to_string());
        }
    }

    if let Some(cached) = org_login_cache.get(org_id) {
        return cached.clone();
    }

    let resolved = state
        .db
        .get_org_by_id(org_id)
        .await
        .ok()
        .flatten()
        .map(|org| org.login);
    org_login_cache.insert(org_id.to_string(), resolved.clone());
    resolved
}

fn repo_full_name_owner(repo_full_name: &str) -> Option<&str> {
    let mut parts = repo_full_name.trim().split('/');
    let owner = parts.next()?.trim();
    let repo = parts.next()?.trim();
    if owner.is_empty()
        || repo.is_empty()
        || parts.next().is_some()
        || !is_valid_repo_full_name_part(owner)
        || !is_valid_repo_full_name_part(repo)
    {
        return None;
    }
    Some(owner)
}

fn is_valid_repo_full_name_part(part: &str) -> bool {
    part.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}

fn validate_event_capture_fidelity(
    event_type: &ClientEventType,
    repo_full_name: Option<&str>,
    branch: Option<&str>,
    commit_sha: Option<&str>,
    files: &[String],
) -> Option<String> {
    if event_requires_repo_context(event_type) && is_blank(repo_full_name) {
        return Some("repo_full_name is required for evidence-bearing events".to_string());
    }

    if event_requires_branch(event_type) && is_blank(branch) {
        return Some("branch is required for evidence-bearing events".to_string());
    }

    if event_requires_commit_sha(event_type) && is_blank(commit_sha) {
        return Some("commit_sha is required for commit/push evidence events".to_string());
    }

    if matches!(event_type, ClientEventType::StageFiles)
        && files.iter().all(|file| file.trim().is_empty())
    {
        return Some("stage_files events must include at least one file".to_string());
    }

    None
}

fn event_requires_repo_context(event_type: &ClientEventType) -> bool {
    matches!(
        event_type,
        ClientEventType::StageFiles
            | ClientEventType::Commit
            | ClientEventType::AttemptPush
            | ClientEventType::SuccessfulPush
            | ClientEventType::PushFailed
            | ClientEventType::BlockedPush
            | ClientEventType::GovernanceBlockedPush
            | ClientEventType::GovernanceWarnedPush
            | ClientEventType::CreateBranch
            | ClientEventType::BlockedBranch
            | ClientEventType::CheckoutBranch
    )
}

fn event_requires_branch(event_type: &ClientEventType) -> bool {
    event_requires_repo_context(event_type)
}

fn event_requires_commit_sha(event_type: &ClientEventType) -> bool {
    matches!(
        event_type,
        ClientEventType::Commit
            | ClientEventType::AttemptPush
            | ClientEventType::SuccessfulPush
            | ClientEventType::PushFailed
            | ClientEventType::BlockedPush
            | ClientEventType::GovernanceBlockedPush
            | ClientEventType::GovernanceWarnedPush
    )
}

fn is_blank(value: Option<&str>) -> bool {
    value.map(str::trim).filter(|value| !value.is_empty()).is_none()
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
