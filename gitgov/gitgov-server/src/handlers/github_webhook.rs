// Split facade for GitHub webhook handlers.
// Live code is partitioned by webhook responsibility under handlers/github_webhook/.

include!("github_webhook/entry.rs");
include!("github_webhook/push_create_review.rs");
include!("github_webhook/repo_evidence.rs");
include!("github_webhook/pr_comments.rs");
include!("github_webhook/pr_events.rs");
include!("github_webhook/tests.rs");

// ============================================================================
// ARCHIVED ORIGINAL: src/handlers/github_webhook.rs before SRP split.
// Keep temporarily as commented evidence until the split is reviewed and accepted.
// ============================================================================
// ===== PART 1 -> src/handlers/github_webhook/entry.rs lines 1-191 =====
// // ============================================================================
//
// #[derive(Debug, Serialize, Deserialize)]
// pub struct WebhookResponse {
//     pub received: bool,
//     pub delivery_id: String,
//     pub event_type: String,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub processed: Option<bool>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub error: Option<String>,
// }
//
// pub async fn handle_github_webhook(
//     State(state): State<Arc<AppState>>,
//     headers: HeaderMap,
//     body: Bytes,
// ) -> impl IntoResponse {
//     let delivery_id = headers
//         .get("X-GitHub-Delivery")
//         .and_then(|v| v.to_str().ok())
//         .unwrap_or(&Uuid::new_v4().to_string())
//         .to_string();
//
//     let event_type = headers
//         .get("X-GitHub-Event")
//         .and_then(|v| v.to_str().ok())
//         .unwrap_or("unknown")
//         .to_string();
//
//     let signature = headers
//         .get("X-Hub-Signature-256")
//         .and_then(|v| v.to_str().ok())
//         .map(|s| s.to_string());
//
//     // Validate HMAC signature if secret is configured
//     if let Some(ref secret) = state.github_webhook_secret {
//         if let Some(ref sig) = signature {
//             if !validate_github_signature(secret, &body, sig) {
//                 return (
//                     StatusCode::UNAUTHORIZED,
//                     Json(WebhookResponse {
//                         received: false,
//                         delivery_id: delivery_id.clone(),
//                         event_type: event_type.clone(),
//                         processed: Some(false),
//                         error: Some("Invalid signature".to_string()),
//                     }),
//                 );
//             }
//         } else {
//             return (
//                 StatusCode::UNAUTHORIZED,
//                 Json(WebhookResponse {
//                     received: false,
//                     delivery_id: delivery_id.clone(),
//                     event_type: event_type.clone(),
//                     processed: Some(false),
//                     error: Some("Missing signature".to_string()),
//                 }),
//             );
//         }
//     }
//
//     let payload: serde_json::Value = match serde_json::from_slice(&body) {
//         Ok(payload) => payload,
//         Err(e) => {
//             tracing::warn!("Invalid JSON webhook payload: {}", e);
//             return (
//                 StatusCode::BAD_REQUEST,
//                 Json(WebhookResponse {
//                     received: false,
//                     delivery_id: delivery_id.clone(),
//                     event_type: event_type.clone(),
//                     processed: Some(false),
//                     error: Some("Invalid JSON payload".to_string()),
//                 }),
//             );
//         }
//     };
//
//     // Store raw webhook event for debugging
//     let webhook_id = match state.db.store_webhook_event(
//         &delivery_id,
//         &event_type,
//         signature.as_deref(),
//         &payload,
//     ).await {
//         Ok(id) => Some(id),
//         Err(e) => {
//             tracing::warn!("Failed to store webhook event: {}", e);
//             None
//         }
//     };
//
//     // Process the webhook based on event type
//     let process_result = match event_type.as_str() {
//         "push" => process_push_event(&state, &delivery_id, &payload).await,
//         "create" => process_create_event(&state, &delivery_id, &payload).await,
//         "pull_request" => process_pull_request_event(&state, &delivery_id, &payload).await,
//         "pull_request_review" => {
//             process_pull_request_review_event(&state, &delivery_id, &payload).await
//         }
//         "pull_request_review_comment" => {
//             process_pull_request_review_comment_event(&state, &delivery_id, &payload).await
//         }
//         "issue_comment" => process_issue_comment_event(&state, &delivery_id, &payload).await,
//         "check_run" => process_check_run_event(&state, &delivery_id, &payload).await,
//         "check_suite" => process_check_suite_event(&state, &delivery_id, &payload).await,
//         "status" => process_status_event(&state, &delivery_id, &payload).await,
//         _ => {
//             tracing::debug!("Unhandled event type: {}", event_type);
//             Ok(())
//         }
//     };
//
//     // Mark webhook as processed
//     if let Some(ref id) = webhook_id {
//         let error_msg = if process_result.is_err() {
//             process_result.as_ref().err().map(|e| e.to_string())
//         } else {
//             None
//         };
//         let _ = state.db.mark_webhook_processed(id, error_msg.as_deref()).await;
//     }
//
//     match process_result {
//         Ok(()) => (
//             StatusCode::OK,
//             Json(WebhookResponse {
//                 received: true,
//                 delivery_id,
//                 event_type,
//                 processed: Some(true),
//                 error: None,
//             }),
//         ),
//         Err(e) if e.to_string().contains("duplicate") || e.to_string().contains("Duplicate") => {
//             tracing::info!("Duplicate webhook received: delivery_id={}", delivery_id);
//             (
//                 StatusCode::OK,
//                 Json(WebhookResponse {
//                     received: true,
//                     delivery_id,
//                     event_type,
//                     processed: Some(true),
//                     error: Some("Duplicate delivery_id - already processed".to_string()),
//                 }),
//             )
//         }
//         Err(e) => {
//             let err_text = e.to_string();
//             let is_internal_db_error = err_text.contains("Internal database error");
//             let (status, error_msg) = if is_internal_db_error {
//                 // Return 5xx so GitHub can retry transient server/database failures.
//                 (StatusCode::SERVICE_UNAVAILABLE, "Internal database error")
//             } else {
//                 (StatusCode::BAD_REQUEST, "Webhook payload could not be processed")
//             };
//             (
//                 status,
//                 Json(WebhookResponse {
//                     received: true,
//                     delivery_id,
//                     event_type,
//                     processed: Some(false),
//                     error: Some(error_msg.to_string()),
//                 }),
//             )
//         }
//     }
// }
//
// fn validate_github_signature(secret: &str, payload_bytes: &[u8], signature: &str) -> bool {
//     let signature_hex = match signature.strip_prefix("sha256=") {
//         Some(hex) => hex,
//         None => return false,
//     };
//     let signature_bytes = match hex::decode(signature_hex) {
//         Ok(bytes) => bytes,
//         Err(_) => return false,
//     };
//     let mut mac = match <hmac::Hmac<Sha256> as Mac>::new_from_slice(secret.as_bytes()) {
//         Ok(m) => m,
//         Err(_) => return false,
//     };
//
//     mac.update(payload_bytes);
//     mac.verify_slice(&signature_bytes).is_ok()
// }
//
// ===== PART 2 -> src/handlers/github_webhook/push_create_review.rs lines 192-455 =====
// async fn process_push_event(
//     state: &Arc<AppState>,
//     delivery_id: &str,
//     payload: &serde_json::Value,
// ) -> Result<(), String> {
//     let push: PushEvent = serde_json::from_value(payload.clone())
//         .map_err(|e| format!("Failed to parse push event: {}", e))?;
//
//     // Extract org/repo info
//     let (org_id, repo_id) = get_or_create_org_repo(&state.db, &push.repository).await?;
//
//     // Extract commit SHAs
//     let commit_shas: Vec<String> = push.commits.iter().map(|c| c.id.clone()).collect();
//     let commits_count = commit_shas.len() as i32;
//
//     // Determine ref type
//     let ref_type = if push.r#ref.starts_with("refs/tags/") {
//         "tag"
//     } else {
//         "branch"
//     };
//
//     let ref_name = push.r#ref
//         .strip_prefix("refs/heads/")
//         .or_else(|| push.r#ref.strip_prefix("refs/tags/"))
//         .unwrap_or(&push.r#ref)
//         .to_string();
//
//     let actor_login = push.sender.login.clone();
//     // Keep canonical type as "push" for compatibility with existing stats/signals SQL.
//     let event_type = "push";
//
//     if push.forced {
//         tracing::warn!(
//             actor = %actor_login,
//             ref_name = %ref_name,
//             repo = %push.repository.full_name,
//             "Force push detected — history rewrite on branch"
//         );
//     }
//
//     let event = GitHubEvent {
//         id: Uuid::new_v4().to_string(),
//         org_id: Some(org_id),
//         repo_id: Some(repo_id),
//         delivery_id: delivery_id.to_string(),
//         event_type: event_type.to_string(),
//         actor_login: Some(push.sender.login),
//         actor_id: Some(push.sender.id),
//         ref_name: Some(ref_name.clone()),
//         ref_type: Some(ref_type.to_string()),
//         before_sha: Some(push.before),
//         after_sha: Some(push.after),
//         commit_shas,
//         commits_count,
//         payload: payload.clone(),
//         created_at: chrono::Utc::now().timestamp_millis(),
//     };
//
//     state.db.insert_github_event(&event).await
//         .map_err(|e| {
//             tracing::error!("Failed to insert github event: {}", e);
//             "Internal database error".to_string()
//         })?;
//
//     tracing::info!(
//         "Processed {} event: {} commits to {} by {}",
//         event_type,
//         event.commits_count,
//         ref_name,
//         actor_login
//     );
//
//     // Enqueue detection job instead of spawning directly (backpressure control)
//     if let Some(ref org_id) = event.org_id {
//         if let Err(e) = state.db.enqueue_job(org_id, "detect_signals", None).await {
//             tracing::warn!("Failed to enqueue detection job for org {}: {}", org_id, e);
//         }
//     }
//
//     Ok(())
// }
//
// async fn process_create_event(
//     state: &Arc<AppState>,
//     delivery_id: &str,
//     payload: &serde_json::Value,
// ) -> Result<(), String> {
//     let create: CreateEvent = serde_json::from_value(payload.clone())
//         .map_err(|e| format!("Failed to parse create event: {}", e))?;
//
//     // Extract org/repo info
//     let (org_id, repo_id) = get_or_create_org_repo(&state.db, &create.repository).await?;
//
//     let ref_name = create.r#ref.clone();
//     let ref_type = create.ref_type.clone();
//     let actor_login = create.sender.login.clone();
//
//     let event = GitHubEvent {
//         id: Uuid::new_v4().to_string(),
//         org_id: Some(org_id),
//         repo_id: Some(repo_id),
//         delivery_id: delivery_id.to_string(),
//         event_type: "create".to_string(),
//         actor_login: Some(create.sender.login),
//         actor_id: Some(create.sender.id),
//         ref_name: Some(create.r#ref),
//         ref_type: Some(create.ref_type),
//         before_sha: None,
//         after_sha: None,
//         commit_shas: vec![],
//         commits_count: 0,
//         payload: payload.clone(),
//         created_at: chrono::Utc::now().timestamp_millis(),
//     };
//
//     state.db.insert_github_event(&event).await
//         .map_err(|e| format!("Failed to insert github event: {}", e))?;
//
//     tracing::info!(
//         "Processed create event: {} {} by {}",
//         ref_type,
//         ref_name,
//         actor_login
//     );
//
//     Ok(())
// }
//
// async fn process_pull_request_review_event(
//     state: &Arc<AppState>,
//     delivery_id: &str,
//     payload: &serde_json::Value,
// ) -> Result<(), String> {
//     let action = payload
//         .get("action")
//         .and_then(|v| v.as_str())
//         .unwrap_or("unknown")
//         .to_string();
//
//     let repo_val = match payload.get("repository") {
//         Some(r) => r,
//         None => {
//             tracing::warn!(
//                 "pull_request_review event missing 'repository' field, delivery_id={}",
//                 delivery_id
//             );
//             return Ok(());
//         }
//     };
//     let repo: GitHubRepository = match serde_json::from_value(repo_val.clone()) {
//         Ok(r) => r,
//         Err(e) => {
//             tracing::warn!(
//                 "Failed to parse repository in pull_request_review event: {}, delivery_id={}",
//                 e,
//                 delivery_id
//             );
//             return Ok(());
//         }
//     };
//     let (org_id, repo_id) = get_or_create_org_repo(&state.db, &repo).await?;
//
//     let sender = payload
//         .get("sender")
//         .and_then(|v| serde_json::from_value::<GitHubUser>(v.clone()).ok());
//     let actor_login = sender.as_ref().map(|s| s.login.clone());
//     let actor_id = sender.as_ref().map(|s| s.id);
//
//     let pr = payload.get("pull_request");
//     let pr_number = pr
//         .and_then(|p| p.get("number"))
//         .and_then(|v| v.as_i64())
//         .unwrap_or(0) as i32;
//     let base_branch = pr
//         .and_then(|p| p.get("base"))
//         .and_then(|b| b.get("ref"))
//         .and_then(|v| v.as_str())
//         .map(str::to_string);
//     let head_sha = pr
//         .and_then(|p| p.get("head"))
//         .and_then(|b| b.get("sha"))
//         .and_then(|v| v.as_str())
//         .map(str::to_string);
//
//     let review = payload.get("review");
//     let review_state = review
//         .and_then(|r| r.get("state"))
//         .and_then(|v| v.as_str())
//         .unwrap_or("unknown")
//         .to_string();
//     let review_commit_sha = review
//         .and_then(|r| r.get("commit_id"))
//         .and_then(|v| v.as_str())
//         .map(str::to_string);
//     let after_sha = review_commit_sha.or(head_sha);
//     let commit_shas = after_sha.clone().map(|sha| vec![sha]).unwrap_or_default();
//     let ref_name = base_branch
//         .clone()
//         .or_else(|| (pr_number > 0).then_some(format!("pr/{}", pr_number)));
//
//     let mut enriched_payload = payload.clone();
//     if let Some(obj) = enriched_payload.as_object_mut() {
//         obj.insert(
//             "gitgov".to_string(),
//             serde_json::json!({
//                 "review_action": action,
//                 "review_state": review_state,
//                 "pr_number": pr_number
//             }),
//         );
//     }
//
//     let event = GitHubEvent {
//         id: Uuid::new_v4().to_string(),
//         org_id: Some(org_id),
//         repo_id: Some(repo_id),
//         delivery_id: delivery_id.to_string(),
//         event_type: "pull_request_review".to_string(),
//         actor_login,
//         actor_id,
//         ref_name,
//         ref_type: Some("pull_request".to_string()),
//         before_sha: None,
//         after_sha,
//         commit_shas: commit_shas.clone(),
//         commits_count: commit_shas.len() as i32,
//         payload: enriched_payload,
//         created_at: chrono::Utc::now().timestamp_millis(),
//     };
//
//     state.db.insert_github_event(&event).await.map_err(|e| {
//         tracing::error!("Failed to insert pull_request_review github event: {}", e);
//         "Internal database error".to_string()
//     })?;
//
//     if let Some(ref org_id) = event.org_id {
//         if let Err(e) = state.db.enqueue_job(org_id, "detect_signals", None).await {
//             tracing::warn!("Failed to enqueue detection job for org {}: {}", org_id, e);
//         }
//     }
//
//     tracing::info!(
//         "Processed pull_request_review event: repo={} pr=#{} action={} state={} actor={}",
//         repo.full_name,
//         pr_number,
//         action,
//         review_state,
//         event.actor_login.as_deref().unwrap_or("unknown")
//     );
//
//     Ok(())
// }
//
// fn extract_sender_actor(payload: &serde_json::Value) -> (Option<String>, Option<i64>) {
//     let sender = payload
//         .get("sender")
//         .and_then(|v| serde_json::from_value::<GitHubUser>(v.clone()).ok());
//     (
//         sender.as_ref().map(|s| s.login.clone()),
//         sender.as_ref().map(|s| s.id),
//     )
// }
//
// ===== PART 3 -> src/handlers/github_webhook/repo_evidence.rs lines 456-758 =====
// struct GenericRepoEvidenceEvent<'a> {
//     state: &'a Arc<AppState>,
//     delivery_id: &'a str,
//     payload: &'a serde_json::Value,
//     event_type: &'a str,
//     actor_login: Option<String>,
//     actor_id: Option<i64>,
//     ref_name: Option<String>,
//     ref_type: Option<String>,
//     after_sha: Option<String>,
//     metadata: serde_json::Value,
// }
//
// #[derive(Debug, PartialEq, Eq)]
// struct CheckRunEvidence {
//     action: String,
//     status: String,
//     conclusion: Option<String>,
//     after_sha: Option<String>,
//     ref_name: Option<String>,
//     details_url: Option<String>,
// }
//
// fn extract_check_run_evidence(payload: &serde_json::Value) -> CheckRunEvidence {
//     let check_run = payload.get("check_run");
//     let ref_name = check_run
//         .and_then(|v| v.get("check_suite"))
//         .and_then(|v| v.get("head_branch"))
//         .and_then(|v| v.as_str())
//         .map(str::to_string)
//         .or_else(|| {
//             check_run
//                 .and_then(|v| v.get("head_branch"))
//                 .and_then(|v| v.as_str())
//                 .map(str::to_string)
//         });
//
//     CheckRunEvidence {
//         action: payload
//             .get("action")
//             .and_then(|v| v.as_str())
//             .unwrap_or("unknown")
//             .to_string(),
//         status: check_run
//             .and_then(|v| v.get("status"))
//             .and_then(|v| v.as_str())
//             .unwrap_or("unknown")
//             .to_string(),
//         conclusion: check_run
//             .and_then(|v| v.get("conclusion"))
//             .and_then(|v| v.as_str())
//             .map(str::to_string),
//         after_sha: check_run
//             .and_then(|v| v.get("head_sha"))
//             .and_then(|v| v.as_str())
//             .map(str::to_string),
//         ref_name,
//         details_url: check_run
//             .and_then(|v| v.get("details_url"))
//             .and_then(|v| v.as_str())
//             .map(str::to_string),
//     }
// }
//
// #[derive(Debug, PartialEq, Eq)]
// struct CheckSuiteEvidence {
//     action: String,
//     status: String,
//     conclusion: Option<String>,
//     after_sha: Option<String>,
//     ref_name: Option<String>,
// }
//
// fn extract_check_suite_evidence(payload: &serde_json::Value) -> CheckSuiteEvidence {
//     let check_suite = payload.get("check_suite");
//
//     CheckSuiteEvidence {
//         action: payload
//             .get("action")
//             .and_then(|v| v.as_str())
//             .unwrap_or("unknown")
//             .to_string(),
//         status: check_suite
//             .and_then(|v| v.get("status"))
//             .and_then(|v| v.as_str())
//             .unwrap_or("unknown")
//             .to_string(),
//         conclusion: check_suite
//             .and_then(|v| v.get("conclusion"))
//             .and_then(|v| v.as_str())
//             .map(str::to_string),
//         after_sha: check_suite
//             .and_then(|v| v.get("head_sha"))
//             .and_then(|v| v.as_str())
//             .map(str::to_string),
//         ref_name: check_suite
//             .and_then(|v| v.get("head_branch"))
//             .and_then(|v| v.as_str())
//             .map(str::to_string),
//     }
// }
//
// #[derive(Debug, PartialEq, Eq)]
// struct CommitStatusEvidence {
//     state_name: String,
//     context: Option<String>,
//     description: Option<String>,
//     target_url: Option<String>,
//     after_sha: Option<String>,
//     ref_name: Option<String>,
// }
//
// fn extract_commit_status_evidence(payload: &serde_json::Value) -> CommitStatusEvidence {
//     CommitStatusEvidence {
//         state_name: payload
//             .get("state")
//             .and_then(|v| v.as_str())
//             .unwrap_or("unknown")
//             .to_string(),
//         context: payload
//             .get("context")
//             .and_then(|v| v.as_str())
//             .map(str::to_string),
//         description: payload
//             .get("description")
//             .and_then(|v| v.as_str())
//             .map(str::to_string),
//         target_url: payload
//             .get("target_url")
//             .and_then(|v| v.as_str())
//             .map(str::to_string),
//         after_sha: payload
//             .get("sha")
//             .and_then(|v| v.as_str())
//             .map(str::to_string),
//         ref_name: payload
//             .get("branches")
//             .and_then(|v| v.as_array())
//             .and_then(|branches| branches.first())
//             .and_then(|entry| entry.get("name"))
//             .and_then(|v| v.as_str())
//             .map(str::to_string),
//     }
// }
//
// async fn store_generic_repo_evidence_event(
//     input: GenericRepoEvidenceEvent<'_>,
// ) -> Result<(), String> {
//     let repo_val = match input.payload.get("repository") {
//         Some(r) => r,
//         None => {
//             tracing::warn!(
//                 "{} event missing 'repository' field, delivery_id={}",
//                 input.event_type,
//                 input.delivery_id
//             );
//             return Ok(());
//         }
//     };
//     let repo: GitHubRepository = match serde_json::from_value(repo_val.clone()) {
//         Ok(r) => r,
//         Err(e) => {
//             tracing::warn!(
//                 "Failed to parse repository in {} event: {}, delivery_id={}",
//                 input.event_type,
//                 e,
//                 input.delivery_id
//             );
//             return Ok(());
//         }
//     };
//     let (org_id, repo_id) = get_or_create_org_repo(&input.state.db, &repo).await?;
//
//     let commit_shas = input.after_sha.clone().map(|sha| vec![sha]).unwrap_or_default();
//     let mut enriched_payload = input.payload.clone();
//     if let Some(obj) = enriched_payload.as_object_mut() {
//         obj.insert("gitgov".to_string(), input.metadata);
//     }
//
//     let event = GitHubEvent {
//         id: Uuid::new_v4().to_string(),
//         org_id: Some(org_id),
//         repo_id: Some(repo_id),
//         delivery_id: input.delivery_id.to_string(),
//         event_type: input.event_type.to_string(),
//         actor_login: input.actor_login,
//         actor_id: input.actor_id,
//         ref_name: input.ref_name,
//         ref_type: input.ref_type,
//         before_sha: None,
//         after_sha: input.after_sha,
//         commit_shas: commit_shas.clone(),
//         commits_count: commit_shas.len() as i32,
//         payload: enriched_payload,
//         created_at: chrono::Utc::now().timestamp_millis(),
//     };
//
//     input.state.db.insert_github_event(&event).await.map_err(|e| {
//         tracing::error!("Failed to insert {} github event: {}", input.event_type, e);
//         "Internal database error".to_string()
//     })?;
//
//     if let Some(ref org_id) = event.org_id {
//         if let Err(e) = input.state.db.enqueue_job(org_id, "detect_signals", None).await {
//             tracing::warn!("Failed to enqueue detection job for org {}: {}", org_id, e);
//         }
//     }
//
//     tracing::info!(
//         "Processed {} event: repo={} ref={} sha={} actor={}",
//         input.event_type,
//         repo.full_name,
//         event.ref_name.as_deref().unwrap_or("n/a"),
//         event.after_sha.as_deref().unwrap_or("n/a"),
//         event.actor_login.as_deref().unwrap_or("unknown")
//     );
//
//     Ok(())
// }
//
// async fn process_check_run_event(
//     state: &Arc<AppState>,
//     delivery_id: &str,
//     payload: &serde_json::Value,
// ) -> Result<(), String> {
//     let evidence = extract_check_run_evidence(payload);
//     let (actor_login, actor_id) = extract_sender_actor(payload);
//
//     store_generic_repo_evidence_event(GenericRepoEvidenceEvent {
//         state,
//         delivery_id,
//         payload,
//         event_type: "check_run",
//         actor_login,
//         actor_id,
//         ref_name: evidence.ref_name,
//         ref_type: Some("branch".to_string()),
//         after_sha: evidence.after_sha,
//         metadata: serde_json::json!({
//             "action": evidence.action,
//             "status": evidence.status,
//             "conclusion": evidence.conclusion,
//             "details_url": evidence.details_url
//         }),
//     })
//     .await
// }
//
// async fn process_check_suite_event(
//     state: &Arc<AppState>,
//     delivery_id: &str,
//     payload: &serde_json::Value,
// ) -> Result<(), String> {
//     let evidence = extract_check_suite_evidence(payload);
//     let (actor_login, actor_id) = extract_sender_actor(payload);
//
//     store_generic_repo_evidence_event(GenericRepoEvidenceEvent {
//         state,
//         delivery_id,
//         payload,
//         event_type: "check_suite",
//         actor_login,
//         actor_id,
//         ref_name: evidence.ref_name,
//         ref_type: Some("branch".to_string()),
//         after_sha: evidence.after_sha,
//         metadata: serde_json::json!({
//             "action": evidence.action,
//             "status": evidence.status,
//             "conclusion": evidence.conclusion
//         }),
//     })
//     .await
// }
//
// async fn process_status_event(
//     state: &Arc<AppState>,
//     delivery_id: &str,
//     payload: &serde_json::Value,
// ) -> Result<(), String> {
//     let evidence = extract_commit_status_evidence(payload);
//     let (actor_login, actor_id) = extract_sender_actor(payload);
//
//     store_generic_repo_evidence_event(GenericRepoEvidenceEvent {
//         state,
//         delivery_id,
//         payload,
//         event_type: "status",
//         actor_login,
//         actor_id,
//         ref_name: evidence.ref_name,
//         ref_type: Some("branch".to_string()),
//         after_sha: evidence.after_sha,
//         metadata: serde_json::json!({
//             "state": evidence.state_name,
//             "context": evidence.context,
//             "description": evidence.description,
//             "target_url": evidence.target_url
//         }),
//     })
//     .await
// }
//
// ===== PART 4 -> src/handlers/github_webhook/pr_comments.rs lines 759-1211 =====
// fn json_string_at<'a>(value: &'a serde_json::Value, path: &[&str]) -> Option<&'a str> {
//     let mut cursor = value;
//     for key in path {
//         cursor = cursor.get(*key)?;
//     }
//     cursor.as_str().map(str::trim).filter(|s| !s.is_empty())
// }
//
// fn json_i64_at(value: &serde_json::Value, path: &[&str]) -> Option<i64> {
//     let mut cursor = value;
//     for key in path {
//         cursor = cursor.get(*key)?;
//     }
//     cursor.as_i64()
// }
//
// fn pr_ref(repo_full_name: &str, pr_number: i32) -> Option<String> {
//     (pr_number > 0).then(|| format!("{}#{}", repo_full_name, pr_number))
// }
//
// fn merged_pr_ticket_targets<'a>(
//     head_sha: Option<&'a str>,
//     merge_commit_sha: Option<&'a str>,
// ) -> Vec<(&'static str, &'a str)> {
//     let mut targets = Vec::new();
//
//     if let Some(sha) = merge_commit_sha.map(str::trim).filter(|s| !s.is_empty()) {
//         targets.push(("pr_title", sha));
//     }
//
//     if let Some(sha) = head_sha.map(str::trim).filter(|s| !s.is_empty()) {
//         let already_included = targets
//             .iter()
//             .any(|(_, existing_sha)| existing_sha.eq_ignore_ascii_case(sha));
//         if !already_included {
//             targets.push(("pr_title", sha));
//         }
//     }
//
//     targets
// }
//
// #[derive(Debug, PartialEq, Eq)]
// struct PullRequestReviewCommentEvidence {
//     action: String,
//     pr_number: i32,
//     pr_title: Option<String>,
//     base_branch: Option<String>,
//     head_sha: Option<String>,
//     comment_commit_sha: Option<String>,
//     comment_body: Option<String>,
//     commit_sha: Option<String>,
// }
//
// fn extract_pull_request_review_comment_evidence(
//     payload: &serde_json::Value,
// ) -> PullRequestReviewCommentEvidence {
//     let head_sha = json_string_at(payload, &["pull_request", "head", "sha"]).map(str::to_string);
//     let comment_commit_sha = json_string_at(payload, &["comment", "commit_id"]).map(str::to_string);
//     let commit_sha = comment_commit_sha.clone().or_else(|| head_sha.clone());
//
//     PullRequestReviewCommentEvidence {
//         action: json_string_at(payload, &["action"])
//             .unwrap_or("unknown")
//             .to_string(),
//         pr_number: json_i64_at(payload, &["pull_request", "number"]).unwrap_or(0) as i32,
//         pr_title: json_string_at(payload, &["pull_request", "title"]).map(str::to_string),
//         base_branch: json_string_at(payload, &["pull_request", "base", "ref"]).map(str::to_string),
//         head_sha,
//         comment_commit_sha,
//         comment_body: json_string_at(payload, &["comment", "body"]).map(str::to_string),
//         commit_sha,
//     }
// }
//
// struct TicketEvidenceCorrelation<'a> {
//     state: &'a Arc<AppState>,
//     org_id: Option<&'a str>,
//     repo_full_name: &'a str,
//     pr_number: i32,
//     commit_sha: Option<&'a str>,
//     branch: Option<&'a str>,
//     source: &'a str,
//     text_sources: &'a [&'a str],
// }
//
// async fn correlate_ticket_evidence_to_commit(
//     input: TicketEvidenceCorrelation<'_>,
// ) -> Result<Vec<String>, String> {
//     let Some(commit_sha) = input
//         .commit_sha
//         .map(str::trim)
//         .filter(|s| !s.is_empty())
//     else {
//         return Ok(vec![]);
//     };
//
//     let ticket_ids = extract_ticket_ids(input.text_sources);
//     if ticket_ids.is_empty() {
//         return Ok(vec![]);
//     }
//
//     let pr_ref = pr_ref(input.repo_full_name, input.pr_number);
//     let mut correlated = Vec::new();
//     for ticket_id in ticket_ids {
//         let correlation = CommitTicketCorrelation {
//             id: Uuid::new_v4().to_string(),
//             org_id: input.org_id.map(str::to_string),
//             commit_sha: commit_sha.to_string(),
//             ticket_id: ticket_id.clone(),
//             correlation_source: input.source.to_string(),
//             confidence: 0.9,
//             created_at: chrono::Utc::now().timestamp_millis(),
//         };
//
//         match input
//             .state
//             .db
//             .insert_commit_ticket_correlation(&correlation)
//             .await
//         {
//             Ok(created) => {
//                 if created {
//                     correlated.push(ticket_id.clone());
//                 }
//                 if let Err(e) = input
//                     .state
//                     .db
//                     .append_project_ticket_relations_full(
//                         &ticket_id,
//                         Some(commit_sha),
//                         input.branch,
//                         pr_ref.as_deref(),
//                     )
//                     .await
//                 {
//                     tracing::debug!(
//                         ticket_id = %ticket_id,
//                         commit_sha = %commit_sha,
//                         source = %input.source,
//                         error = %e,
//                         "Could not append ticket relations after GitHub comment evidence"
//                     );
//                 }
//             }
//             Err(e) => {
//                 tracing::warn!(
//                     ticket_id = %ticket_id,
//                     commit_sha = %commit_sha,
//                     source = %input.source,
//                     error = %e,
//                     "Failed to store ticket correlation from GitHub comment evidence"
//                 );
//             }
//         }
//     }
//
//     correlated.sort();
//     Ok(correlated)
// }
//
// async fn process_pull_request_review_comment_event(
//     state: &Arc<AppState>,
//     delivery_id: &str,
//     payload: &serde_json::Value,
// ) -> Result<(), String> {
//     let evidence = extract_pull_request_review_comment_evidence(payload);
//     let repo_val = match payload.get("repository") {
//         Some(r) => r,
//         None => {
//             tracing::warn!(
//                 "pull_request_review_comment event missing 'repository' field, delivery_id={}",
//                 delivery_id
//             );
//             return Ok(());
//         }
//     };
//     let repo: GitHubRepository = serde_json::from_value(repo_val.clone()).map_err(|e| {
//         format!(
//             "Failed to parse repository in pull_request_review_comment event: {}",
//             e
//         )
//     })?;
//     let (org_id, repo_id) = get_or_create_org_repo(&state.db, &repo).await?;
//     let (actor_login, actor_id) = extract_sender_actor(payload);
//
//     let review_comment_text_sources = [
//         evidence.comment_body.as_deref().unwrap_or_default(),
//         evidence.pr_title.as_deref().unwrap_or_default(),
//     ];
//     let correlated_tickets = correlate_ticket_evidence_to_commit(TicketEvidenceCorrelation {
//         state,
//         org_id: Some(&org_id),
//         repo_full_name: &repo.full_name,
//         pr_number: evidence.pr_number,
//         commit_sha: evidence.commit_sha.as_deref(),
//         branch: evidence.base_branch.as_deref(),
//         source: "github_pr_review_comment",
//         text_sources: &review_comment_text_sources,
//     })
//     .await?;
//
//     let mut enriched_payload = payload.clone();
//     if let Some(obj) = enriched_payload.as_object_mut() {
//         obj.insert(
//             "gitgov".to_string(),
//             serde_json::json!({
//                 "action": evidence.action,
//                 "pr_number": evidence.pr_number,
//                 "comment_kind": "pull_request_review_comment",
//                 "ticket_correlations": correlated_tickets
//             }),
//         );
//     }
//
//     let commit_shas = evidence.commit_sha.clone().map(|sha| vec![sha]).unwrap_or_default();
//     let event = GitHubEvent {
//         id: Uuid::new_v4().to_string(),
//         org_id: Some(org_id),
//         repo_id: Some(repo_id),
//         delivery_id: delivery_id.to_string(),
//         event_type: "pull_request_review_comment".to_string(),
//         actor_login,
//         actor_id,
//         ref_name: evidence
//             .base_branch
//             .or_else(|| (evidence.pr_number > 0).then_some(format!("pr/{}", evidence.pr_number))),
//         ref_type: Some("pull_request".to_string()),
//         before_sha: None,
//         after_sha: evidence.commit_sha,
//         commit_shas: commit_shas.clone(),
//         commits_count: commit_shas.len() as i32,
//         payload: enriched_payload,
//         created_at: chrono::Utc::now().timestamp_millis(),
//     };
//
//     match state.db.insert_github_event(&event).await {
//         Ok(()) => {}
//         Err(DbError::Duplicate(_)) => {
//             tracing::debug!(
//                 "Duplicate pull_request_review_comment event ignored: delivery_id={}",
//                 delivery_id
//             );
//             return Ok(());
//         }
//         Err(e) => {
//             tracing::error!(
//                 "Failed to insert pull_request_review_comment github event: {}",
//                 e
//             );
//             return Err("Internal database error".to_string());
//         }
//     }
//
//     tracing::info!(
//         repo = %repo.full_name,
//         pr_number = evidence.pr_number,
//         correlated_tickets = ?correlated_tickets,
//         "Processed pull_request_review_comment event"
//     );
//
//     Ok(())
// }
//
// #[derive(Debug, Deserialize)]
// struct GitHubPullRequestLookupHead {
//     sha: Option<String>,
//     #[serde(default)]
//     #[serde(rename = "ref")]
//     ref_field: Option<String>,
// }
//
// #[derive(Debug, Deserialize)]
// struct GitHubPullRequestLookupBase {
//     #[serde(default)]
//     #[serde(rename = "ref")]
//     ref_field: Option<String>,
// }
//
// #[derive(Debug, Deserialize)]
// struct GitHubPullRequestLookup {
//     #[serde(default)]
//     title: Option<String>,
//     #[serde(default)]
//     head: Option<GitHubPullRequestLookupHead>,
//     #[serde(default)]
//     base: Option<GitHubPullRequestLookupBase>,
// }
//
// async fn fetch_pr_lookup(
//     http_client: &reqwest::Client,
//     github_token: &str,
//     repo_full_name: &str,
//     pr_number: i32,
// ) -> Result<GitHubPullRequestLookup, String> {
//     let url = format!(
//         "https://api.github.com/repos/{}/pulls/{}",
//         repo_full_name, pr_number
//     );
//     let response = http_client
//         .get(&url)
//         .header("Authorization", format!("Bearer {}", github_token))
//         .header("Accept", "application/vnd.github+json")
//         .header("User-Agent", "gitgov-server")
//         .header("X-GitHub-Api-Version", "2022-11-28")
//         .send()
//         .await
//         .map_err(|e| format!("GitHub PR lookup request failed: {}", e))?;
//
//     let status = response.status();
//     if !status.is_success() {
//         return Err(format!("GitHub PR lookup API returned {}", status));
//     }
//
//     response
//         .json()
//         .await
//         .map_err(|e| format!("GitHub PR lookup decode failed: {}", e))
// }
//
// async fn process_issue_comment_event(
//     state: &Arc<AppState>,
//     delivery_id: &str,
//     payload: &serde_json::Value,
// ) -> Result<(), String> {
//     let issue_is_pr = payload
//         .get("issue")
//         .and_then(|issue| issue.get("pull_request"))
//         .is_some();
//     if !issue_is_pr {
//         tracing::debug!(
//             "Ignoring issue_comment not linked to PR, delivery_id={}",
//             delivery_id
//         );
//         return Ok(());
//     }
//
//     let action = json_string_at(payload, &["action"]).unwrap_or("unknown").to_string();
//     let repo_val = match payload.get("repository") {
//         Some(r) => r,
//         None => {
//             tracing::warn!("issue_comment event missing 'repository' field, delivery_id={}", delivery_id);
//             return Ok(());
//         }
//     };
//     let repo: GitHubRepository = serde_json::from_value(repo_val.clone())
//         .map_err(|e| format!("Failed to parse repository in issue_comment event: {}", e))?;
//     let (org_id, repo_id) = get_or_create_org_repo(&state.db, &repo).await?;
//     let (actor_login, actor_id) = extract_sender_actor(payload);
//
//     let pr_number = json_i64_at(payload, &["issue", "number"]).unwrap_or(0) as i32;
//     let comment_body = json_string_at(payload, &["comment", "body"]);
//     let issue_title = json_string_at(payload, &["issue", "title"]);
//
//     let mut lookup_title = None;
//     let mut head_sha = None;
//     let mut base_branch = None;
//     if pr_number > 0 {
//         if let Some(token) = state.github_personal_access_token.as_deref() {
//             match fetch_pr_lookup(&state.http_client, token, &repo.full_name, pr_number).await {
//                 Ok(lookup) => {
//                     lookup_title = lookup.title;
//                     head_sha = lookup.head.as_ref().and_then(|head| head.sha.clone());
//                     base_branch = lookup.base.as_ref().and_then(|base| base.ref_field.clone());
//                     if base_branch.is_none() {
//                         base_branch = lookup.head.as_ref().and_then(|head| head.ref_field.clone());
//                     }
//                 }
//                 Err(e) => {
//                     tracing::warn!(
//                         delivery_id = %delivery_id,
//                         repo = %repo.full_name,
//                         pr_number,
//                         error = %e,
//                         "Failed to fetch PR metadata for issue_comment evidence"
//                     );
//                 }
//             }
//         }
//     }
//
//     let issue_comment_text_sources = [
//         comment_body.unwrap_or_default(),
//         issue_title.unwrap_or_default(),
//         lookup_title.as_deref().unwrap_or_default(),
//     ];
//     let correlated_tickets = correlate_ticket_evidence_to_commit(TicketEvidenceCorrelation {
//         state,
//         org_id: Some(&org_id),
//         repo_full_name: &repo.full_name,
//         pr_number,
//         commit_sha: head_sha.as_deref(),
//         branch: base_branch.as_deref(),
//         source: "github_pr_issue_comment",
//         text_sources: &issue_comment_text_sources,
//     })
//     .await?;
//
//     let mut enriched_payload = payload.clone();
//     if let Some(obj) = enriched_payload.as_object_mut() {
//         obj.insert(
//             "gitgov".to_string(),
//             serde_json::json!({
//                 "action": action,
//                 "pr_number": pr_number,
//                 "comment_kind": "issue_comment_on_pull_request",
//                 "ticket_correlations": correlated_tickets
//             }),
//         );
//     }
//
//     let commit_shas = head_sha.clone().map(|sha| vec![sha]).unwrap_or_default();
//     let event = GitHubEvent {
//         id: Uuid::new_v4().to_string(),
//         org_id: Some(org_id),
//         repo_id: Some(repo_id),
//         delivery_id: delivery_id.to_string(),
//         event_type: "issue_comment".to_string(),
//         actor_login,
//         actor_id,
//         ref_name: base_branch.or_else(|| (pr_number > 0).then_some(format!("pr/{}", pr_number))),
//         ref_type: Some("pull_request".to_string()),
//         before_sha: None,
//         after_sha: head_sha,
//         commit_shas: commit_shas.clone(),
//         commits_count: commit_shas.len() as i32,
//         payload: enriched_payload,
//         created_at: chrono::Utc::now().timestamp_millis(),
//     };
//
//     match state.db.insert_github_event(&event).await {
//         Ok(()) => {}
//         Err(DbError::Duplicate(_)) => {
//             tracing::debug!("Duplicate issue_comment event ignored: delivery_id={}", delivery_id);
//             return Ok(());
//         }
//         Err(e) => {
//             tracing::error!("Failed to insert issue_comment github event: {}", e);
//             return Err("Internal database error".to_string());
//         }
//     }
//
//     tracing::info!(
//         repo = %repo.full_name,
//         pr_number,
//         correlated_tickets = ?correlated_tickets,
//         "Processed issue_comment event linked to PR"
//     );
//
//     Ok(())
// }
//
// #[derive(Debug, Deserialize)]
// ===== PART 5 -> src/handlers/github_webhook/pr_events.rs lines 1212-1578 =====
// struct GitHubPrReviewUser {
//     login: String,
// }
//
// #[derive(Debug, Deserialize)]
// struct GitHubPrReview {
//     #[serde(default)]
//     state: Option<String>,
//     #[serde(default)]
//     user: Option<GitHubPrReviewUser>,
// }
//
// fn extract_final_approvers(reviews: &[GitHubPrReview]) -> Vec<String> {
//     // GitHub reviews are evaluated per reviewer by latest review state.
//     let mut latest_state_by_user: HashMap<String, String> = HashMap::new();
//
//     for review in reviews {
//         let Some(user) = review.user.as_ref() else { continue };
//         let state = review
//             .state
//             .as_deref()
//             .unwrap_or_default()
//             .trim()
//             .to_ascii_uppercase();
//         if state.is_empty() {
//             continue;
//         }
//         latest_state_by_user.insert(user.login.clone(), state);
//     }
//
//     let mut approvers: Vec<String> = latest_state_by_user
//         .into_iter()
//         .filter_map(|(login, state)| (state == "APPROVED").then_some(login))
//         .collect();
//
//     approvers.sort();
//     approvers
// }
//
// async fn fetch_pr_approvers(
//     http_client: &reqwest::Client,
//     github_token: &str,
//     repo_full_name: &str,
//     pr_number: i32,
// ) -> Result<Vec<String>, String> {
//     let mut all_reviews = Vec::new();
//     let mut page = 1u8;
//
//     loop {
//         let url = format!(
//             "https://api.github.com/repos/{}/pulls/{}/reviews?per_page=100&page={}",
//             repo_full_name, pr_number, page
//         );
//
//         let response = http_client
//             .get(&url)
//             .header("Authorization", format!("Bearer {}", github_token))
//             .header("Accept", "application/vnd.github+json")
//             .header("User-Agent", "gitgov-server")
//             .header("X-GitHub-Api-Version", "2022-11-28")
//             .send()
//             .await
//             .map_err(|e| format!("GitHub reviews request failed: {}", e))?;
//
//         let status = response.status();
//         if !status.is_success() {
//             return Err(format!("GitHub reviews API returned {}", status));
//         }
//
//         let reviews: Vec<GitHubPrReview> = response
//             .json()
//             .await
//             .map_err(|e| format!("GitHub reviews decode failed: {}", e))?;
//
//         let chunk_len = reviews.len();
//         all_reviews.extend(reviews);
//
//         if chunk_len < 100 || page >= 10 {
//             break;
//         }
//
//         page += 1;
//     }
//
//     Ok(extract_final_approvers(&all_reviews))
// }
//
// // Processes pull_request webhook events.
// // Stores every pull_request action as first-class evidence in github_events.
// // Additionally stores merged PRs (action == "closed" && merged == true) in pr_merges.
// async fn process_pull_request_event(
//     state: &Arc<AppState>,
//     delivery_id: &str,
//     payload: &serde_json::Value,
// ) -> Result<(), String> {
//     let action = payload
//         .get("action")
//         .and_then(|v| v.as_str())
//         .unwrap_or("unknown")
//         .to_string();
//     let pr = match payload.get("pull_request") {
//         Some(pr) => pr,
//         None => {
//             tracing::debug!("pull_request event missing 'pull_request' field, delivery_id={}", delivery_id);
//             return Ok(());
//         }
//     };
//
//     // Extract repository info for org/repo lookup
//     let repo_val = match payload.get("repository") {
//         Some(r) => r,
//         None => {
//             tracing::warn!("pull_request event missing 'repository' field, delivery_id={}", delivery_id);
//             return Ok(());
//         }
//     };
//     let repo: GitHubRepository = match serde_json::from_value(repo_val.clone()) {
//         Ok(r) => r,
//         Err(e) => {
//             tracing::warn!("Failed to parse repository in pull_request event: {}, delivery_id={}", e, delivery_id);
//             return Ok(());
//         }
//     };
//
//     let (org_id, repo_id) = get_or_create_org_repo(&state.db, &repo).await?;
//
//     let merged = pr.get("merged").and_then(|v| v.as_bool()).unwrap_or(false);
//     let draft = pr.get("draft").and_then(|v| v.as_bool()).unwrap_or(false);
//     let pr_number = pr.get("number").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
//     let pr_title = pr.get("title").and_then(|v| v.as_str()).map(String::from);
//     let author_login = pr.get("user").and_then(|u| u.get("login")).and_then(|v| v.as_str()).map(String::from);
//     let merged_by_login = pr.get("merged_by").and_then(|u| u.get("login")).and_then(|v| v.as_str()).map(String::from);
//     let head_sha = pr.get("head").and_then(|h| h.get("sha")).and_then(|v| v.as_str()).map(String::from);
//     let merge_commit_sha = pr
//         .get("merge_commit_sha")
//         .and_then(|v| v.as_str())
//         .map(str::trim)
//         .filter(|s| !s.is_empty())
//         .map(String::from);
//     let base_branch = pr.get("base").and_then(|b| b.get("ref")).and_then(|v| v.as_str()).map(String::from);
//     let sender_actor = payload
//         .get("sender")
//         .and_then(|v| serde_json::from_value::<GitHubUser>(v.clone()).ok());
//     let actor_login = sender_actor.as_ref().map(|s| s.login.clone());
//     let actor_id = sender_actor.as_ref().map(|s| s.id);
//     let requested_reviewers_count = pr
//         .get("requested_reviewers")
//         .and_then(|v| v.as_array())
//         .map(|arr| arr.len())
//         .unwrap_or(0);
//
//     let mut pr_payload = payload.clone();
//     if let Some(obj) = pr_payload.as_object_mut() {
//         obj.insert(
//             "gitgov".to_string(),
//             serde_json::json!({
//                 "action": action.clone(),
//                 "merged": merged,
//                 "draft": draft,
//                 "pr_number": pr_number,
//                 "requested_reviewers_count": requested_reviewers_count,
//                 "merge_commit_sha": merge_commit_sha.clone()
//             }),
//         );
//     }
//
//     let pr_commit_shas = if action == "closed" && merged {
//         merged_pr_ticket_targets(head_sha.as_deref(), merge_commit_sha.as_deref())
//             .into_iter()
//             .map(|(_, sha)| sha.to_string())
//             .collect::<Vec<_>>()
//     } else {
//         head_sha.clone().map(|sha| vec![sha]).unwrap_or_default()
//     };
//
//     let pr_event = GitHubEvent {
//         id: Uuid::new_v4().to_string(),
//         org_id: Some(org_id.clone()),
//         repo_id: Some(repo_id.clone()),
//         delivery_id: delivery_id.to_string(),
//         event_type: "pull_request".to_string(),
//         actor_login: actor_login.clone(),
//         actor_id,
//         ref_name: base_branch
//             .clone()
//             .or_else(|| (pr_number > 0).then_some(format!("pr/{}", pr_number))),
//         ref_type: Some("pull_request".to_string()),
//         before_sha: None,
//         after_sha: merge_commit_sha.clone().or_else(|| head_sha.clone()),
//         commits_count: pr_commit_shas.len() as i32,
//         commit_shas: pr_commit_shas,
//         payload: pr_payload,
//         created_at: chrono::Utc::now().timestamp_millis(),
//     };
//
//     let inserted_github_event = match state.db.insert_github_event(&pr_event).await {
//         Ok(()) => true,
//         Err(DbError::Duplicate(_)) => {
//             tracing::debug!(
//                 "Duplicate pull_request github event observed: delivery_id={}",
//                 delivery_id
//             );
//             false
//         }
//         Err(e) => {
//             tracing::error!("Failed to insert pull_request github event: {}", e);
//             return Err("Internal database error".to_string());
//         }
//     };
//
//     if inserted_github_event {
//         if let Some(ref org_id) = pr_event.org_id {
//             if let Err(e) = state.db.enqueue_job(org_id, "detect_signals", None).await {
//                 tracing::warn!("Failed to enqueue detection job for org {}: {}", org_id, e);
//             }
//         }
//     }
//
//     // Only merged PRs are materialized into pr_merges.
//     if action != "closed" || !merged {
//         tracing::info!(
//             "Processed pull_request event: repo={} pr=#{} action={} actor={}",
//             repo.full_name,
//             pr_number,
//             action,
//             actor_login.as_deref().unwrap_or("unknown"),
//         );
//         return Ok(());
//     }
//
//     let approvers = match state.github_personal_access_token.as_deref() {
//         Some(token) => match fetch_pr_approvers(&state.http_client, token, &repo.full_name, pr_number).await {
//             Ok(v) => v,
//             Err(e) => {
//                 tracing::warn!(
//                     delivery_id = %delivery_id,
//                     repo = %repo.full_name,
//                     pr_number,
//                     error = %e,
//                     "Failed to fetch PR approvers from GitHub API"
//                 );
//                 vec![]
//             }
//         },
//         None => {
//             tracing::debug!(
//                 delivery_id = %delivery_id,
//                 repo = %repo.full_name,
//                 pr_number,
//                 "GITHUB_PERSONAL_ACCESS_TOKEN not configured; storing PR merge without approvers"
//             );
//             vec![]
//         }
//     };
//     let approvals_count = approvers.len() as i32;
//
//     let mut enriched_payload = payload.clone();
//     if let Some(obj) = enriched_payload.as_object_mut() {
//         obj.insert(
//             "gitgov".to_string(),
//             serde_json::json!({
//                 "action": action.clone(),
//                 "merged": merged,
//                 "draft": draft,
//                 "pr_number": pr_number,
//                 "requested_reviewers_count": requested_reviewers_count,
//                 "approvers": approvers,
//                 "approvals_count": approvals_count
//             }),
//         );
//     }
//
//     let head_sha_clone = head_sha.clone();
//     let merge_commit_sha_clone = merge_commit_sha.clone();
//     let base_branch_clone = base_branch.clone();
//     let record = PrMergeRecord {
//         id: Uuid::new_v4().to_string(),
//         org_id: Some(org_id),
//         repo_id: Some(repo_id),
//         delivery_id: delivery_id.to_string(),
//         pr_number,
//         pr_title: pr_title.clone(),
//         author_login: author_login.clone(),
//         merged_by_login: merged_by_login.clone(),
//         head_sha,
//         base_branch,
//         payload: enriched_payload,
//         created_at: chrono::Utc::now().timestamp_millis(),
//     };
//
//     let inserted_pr_merge = match state.db.insert_pr_merge(&record).await {
//         Ok(()) => true,
//         Err(DbError::Duplicate(_)) => {
//             tracing::debug!("Duplicate PR merge event observed: delivery_id={}", delivery_id);
//             false
//         }
//         Err(e) => return Err(format!("Failed to insert PR merge: {}", e)),
//     };
//
//     if inserted_pr_merge {
//         tracing::info!(
//             "Processed PR merge: #{} '{}' by {} merged by {} (approvals={}), delivery_id={}",
//             pr_number,
//             pr_title.as_deref().unwrap_or(""),
//             author_login.as_deref().unwrap_or("unknown"),
//             merged_by_login.as_deref().unwrap_or("unknown"),
//             approvals_count,
//             delivery_id,
//         );
//     }
//
//     // Auto-correlate on both fresh and duplicate PR merge deliveries. This keeps
//     // webhook processing idempotent and lets redelivery repair missing coverage.
//     let title_sources = [pr_title.as_deref().unwrap_or_default()];
//     let mut correlated_ticket_ids = std::collections::BTreeSet::new();
//     for (source, commit_sha) in merged_pr_ticket_targets(
//         head_sha_clone.as_deref(),
//         merge_commit_sha_clone.as_deref(),
//     ) {
//         let correlated = correlate_ticket_evidence_to_commit(TicketEvidenceCorrelation {
//             state,
//             org_id: record.org_id.as_deref(),
//             repo_full_name: &repo.full_name,
//             pr_number,
//             commit_sha: Some(commit_sha),
//             branch: base_branch_clone.as_deref(),
//             source,
//             text_sources: &title_sources,
//         })
//         .await?;
//         correlated_ticket_ids.extend(correlated);
//     }
//
//     if !correlated_ticket_ids.is_empty() {
//         tracing::info!(
//             pr_ref = format!("{}#{}", repo.full_name, pr_number),
//             tickets = ?correlated_ticket_ids,
//             "Auto-correlated merged PR commits with tickets from title"
//         );
//     }
//
//     Ok(())
// }
//
// async fn get_or_create_org_repo(db: &Database, repo: &GitHubRepository) -> Result<(String, String), String> {
//     // Get or create org
//     let org_id = if let Some(ref org) = repo.organization {
//         db.upsert_org(org.id, &org.login, None, None).await
//             .map_err(|e| e.to_string())?
//     } else {
//         // If no organization, use the owner as org
//         db.upsert_org(repo.owner.id, &repo.owner.login, None, None).await
//             .map_err(|e| e.to_string())?
//     };
//
//     // Get or create repo
//     let repo_id = db.upsert_repo(
//         Some(&org_id),
//         repo.id,
//         &repo.full_name,
//         &repo.name,
//         repo.private,
//     ).await.map_err(|e| e.to_string())?;
//
//     Ok((org_id, repo_id))
// }
//
// ===== PART 6 -> src/handlers/github_webhook/tests.rs lines 1579-1749 =====
// #[cfg(test)]
// mod github_webhook_tests {
//     use super::{
//         extract_check_run_evidence, extract_check_suite_evidence, extract_commit_status_evidence,
//         extract_pull_request_review_comment_evidence, merged_pr_ticket_targets,
//     };
//     use serde_json::json;
//
//     #[test]
//     fn merged_pr_ticket_targets_prefers_merge_commit_then_head() {
//         let targets = merged_pr_ticket_targets(Some("head-sha"), Some("merge-sha"));
//
//         assert_eq!(
//             targets,
//             vec![
//                 ("pr_title", "merge-sha"),
//                 ("pr_title", "head-sha")
//             ]
//         );
//     }
//
//     #[test]
//     fn merged_pr_ticket_targets_deduplicates_same_head_and_merge_commit() {
//         let targets = merged_pr_ticket_targets(Some("ABCDEF"), Some("abcdef"));
//
//         assert_eq!(targets, vec![("pr_title", "abcdef")]);
//     }
//
//     #[test]
//     fn check_run_evidence_prefers_nested_suite_branch() {
//         let payload = json!({
//             "action": "completed",
//             "check_run": {
//                 "status": "completed",
//                 "conclusion": "success",
//                 "head_sha": "abc123",
//                 "head_branch": "fallback-branch",
//                 "details_url": "https://github.com/example/actions/runs/1",
//                 "check_suite": {
//                     "head_branch": "main"
//                 }
//             }
//         });
//
//         let evidence = extract_check_run_evidence(&payload);
//
//         assert_eq!(evidence.action, "completed");
//         assert_eq!(evidence.status, "completed");
//         assert_eq!(evidence.conclusion.as_deref(), Some("success"));
//         assert_eq!(evidence.after_sha.as_deref(), Some("abc123"));
//         assert_eq!(evidence.ref_name.as_deref(), Some("main"));
//         assert_eq!(
//             evidence.details_url.as_deref(),
//             Some("https://github.com/example/actions/runs/1")
//         );
//     }
//
//     #[test]
//     fn check_run_evidence_falls_back_to_run_branch() {
//         let payload = json!({
//             "action": "rerequested",
//             "check_run": {
//                 "status": "queued",
//                 "head_sha": "def456",
//                 "head_branch": "feature/KAN-4"
//             }
//         });
//
//         let evidence = extract_check_run_evidence(&payload);
//
//         assert_eq!(evidence.action, "rerequested");
//         assert_eq!(evidence.status, "queued");
//         assert_eq!(evidence.conclusion, None);
//         assert_eq!(evidence.after_sha.as_deref(), Some("def456"));
//         assert_eq!(evidence.ref_name.as_deref(), Some("feature/KAN-4"));
//     }
//
//     #[test]
//     fn check_suite_evidence_extracts_branch_and_sha() {
//         let payload = json!({
//             "action": "completed",
//             "check_suite": {
//                 "status": "completed",
//                 "conclusion": "failure",
//                 "head_sha": "suite-sha",
//                 "head_branch": "main"
//             }
//         });
//
//         let evidence = extract_check_suite_evidence(&payload);
//
//         assert_eq!(evidence.action, "completed");
//         assert_eq!(evidence.status, "completed");
//         assert_eq!(evidence.conclusion.as_deref(), Some("failure"));
//         assert_eq!(evidence.after_sha.as_deref(), Some("suite-sha"));
//         assert_eq!(evidence.ref_name.as_deref(), Some("main"));
//     }
//
//     #[test]
//     fn commit_status_evidence_uses_first_branch() {
//         let payload = json!({
//             "state": "success",
//             "context": "ci/build",
//             "description": "Build passed",
//             "target_url": "https://ci.example/run/42",
//             "sha": "status-sha",
//             "branches": [
//                 { "name": "main" },
//                 { "name": "release" }
//             ]
//         });
//
//         let evidence = extract_commit_status_evidence(&payload);
//
//         assert_eq!(evidence.state_name, "success");
//         assert_eq!(evidence.context.as_deref(), Some("ci/build"));
//         assert_eq!(evidence.description.as_deref(), Some("Build passed"));
//         assert_eq!(evidence.target_url.as_deref(), Some("https://ci.example/run/42"));
//         assert_eq!(evidence.after_sha.as_deref(), Some("status-sha"));
//         assert_eq!(evidence.ref_name.as_deref(), Some("main"));
//     }
//
//     #[test]
//     fn review_comment_evidence_prefers_comment_commit_sha() {
//         let payload = json!({
//             "action": "created",
//             "pull_request": {
//                 "number": 47,
//                 "title": "KAN-4 harden traceability",
//                 "base": { "ref": "main" },
//                 "head": { "sha": "head-sha" }
//             },
//             "comment": {
//                 "commit_id": "comment-sha",
//                 "body": "Follow-up for KAN-4"
//             }
//         });
//
//         let evidence = extract_pull_request_review_comment_evidence(&payload);
//
//         assert_eq!(evidence.action, "created");
//         assert_eq!(evidence.pr_number, 47);
//         assert_eq!(evidence.pr_title.as_deref(), Some("KAN-4 harden traceability"));
//         assert_eq!(evidence.base_branch.as_deref(), Some("main"));
//         assert_eq!(evidence.head_sha.as_deref(), Some("head-sha"));
//         assert_eq!(evidence.comment_commit_sha.as_deref(), Some("comment-sha"));
//         assert_eq!(evidence.comment_body.as_deref(), Some("Follow-up for KAN-4"));
//         assert_eq!(evidence.commit_sha.as_deref(), Some("comment-sha"));
//     }
//
//     #[test]
//     fn review_comment_evidence_falls_back_to_head_sha() {
//         let payload = json!({
//             "pull_request": {
//                 "number": 6,
//                 "head": { "sha": "head-only-sha" }
//             },
//             "comment": {
//                 "body": "KAN-6 release evidence"
//             }
//         });
//
//         let evidence = extract_pull_request_review_comment_evidence(&payload);
//
//         assert_eq!(evidence.action, "unknown");
//         assert_eq!(evidence.pr_number, 6);
//         assert_eq!(evidence.comment_commit_sha, None);
//         assert_eq!(evidence.commit_sha.as_deref(), Some("head-only-sha"));
//     }
// }
//
