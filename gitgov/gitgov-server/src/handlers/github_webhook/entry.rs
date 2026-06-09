// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookResponse {
    pub received: bool,
    pub delivery_id: String,
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub async fn handle_github_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let delivery_id = headers
        .get("X-GitHub-Delivery")
        .and_then(|v| v.to_str().ok())
        .unwrap_or(&Uuid::new_v4().to_string())
        .to_string();

    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let signature = headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Validate HMAC signature if secret is configured
    if let Some(ref secret) = state.github_webhook_secret {
        if let Some(ref sig) = signature {
            if !validate_github_signature(secret, &body, sig) {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(WebhookResponse {
                        received: false,
                        delivery_id: delivery_id.clone(),
                        event_type: event_type.clone(),
                        processed: Some(false),
                        error: Some("Invalid signature".to_string()),
                    }),
                );
            }
        } else {
            return (
                StatusCode::UNAUTHORIZED,
                Json(WebhookResponse {
                    received: false,
                    delivery_id: delivery_id.clone(),
                    event_type: event_type.clone(),
                    processed: Some(false),
                    error: Some("Missing signature".to_string()),
                }),
            );
        }
    }

    let payload: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(payload) => payload,
        Err(e) => {
            tracing::warn!("Invalid JSON webhook payload: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(WebhookResponse {
                    received: false,
                    delivery_id: delivery_id.clone(),
                    event_type: event_type.clone(),
                    processed: Some(false),
                    error: Some("Invalid JSON payload".to_string()),
                }),
            );
        }
    };

    // Store raw webhook event for debugging
    let webhook_id = match state.db.store_webhook_event(
        &delivery_id,
        &event_type,
        signature.as_deref(),
        &payload,
    ).await {
        Ok(id) => Some(id),
        Err(e) => {
            tracing::warn!("Failed to store webhook event: {}", e);
            None
        }
    };

    // Process the webhook based on event type
    let process_result = match event_type.as_str() {
        "push" => process_push_event(&state, &delivery_id, &payload).await,
        "create" => process_create_event(&state, &delivery_id, &payload).await,
        "pull_request" => process_pull_request_event(&state, &delivery_id, &payload).await,
        "pull_request_review" => {
            process_pull_request_review_event(&state, &delivery_id, &payload).await
        }
        "pull_request_review_comment" => {
            process_pull_request_review_comment_event(&state, &delivery_id, &payload).await
        }
        "issue_comment" => process_issue_comment_event(&state, &delivery_id, &payload).await,
        "check_run" => process_check_run_event(&state, &delivery_id, &payload).await,
        "check_suite" => process_check_suite_event(&state, &delivery_id, &payload).await,
        "status" => process_status_event(&state, &delivery_id, &payload).await,
        _ => {
            tracing::debug!("Unhandled event type: {}", event_type);
            Ok(())
        }
    };

    // Mark webhook as processed
    if let Some(ref id) = webhook_id {
        let error_msg = if process_result.is_err() {
            process_result.as_ref().err().map(|e| e.to_string())
        } else {
            None
        };
        let _ = state.db.mark_webhook_processed(id, error_msg.as_deref()).await;
    }

    match process_result {
        Ok(()) => (
            StatusCode::OK,
            Json(WebhookResponse {
                received: true,
                delivery_id,
                event_type,
                processed: Some(true),
                error: None,
            }),
        ),
        Err(e) if e.to_string().contains("duplicate") || e.to_string().contains("Duplicate") => {
            tracing::info!("Duplicate webhook received: delivery_id={}", delivery_id);
            (
                StatusCode::OK,
                Json(WebhookResponse {
                    received: true,
                    delivery_id,
                    event_type,
                    processed: Some(true),
                    error: Some("Duplicate delivery_id - already processed".to_string()),
                }),
            )
        }
        Err(e) => {
            let err_text = e.to_string();
            let is_internal_db_error = err_text.contains("Internal database error");
            let (status, error_msg) = if is_internal_db_error {
                // Return 5xx so GitHub can retry transient server/database failures.
                (StatusCode::SERVICE_UNAVAILABLE, "Internal database error")
            } else {
                (StatusCode::BAD_REQUEST, "Webhook payload could not be processed")
            };
            (
                status,
                Json(WebhookResponse {
                    received: true,
                    delivery_id,
                    event_type,
                    processed: Some(false),
                    error: Some(error_msg.to_string()),
                }),
            )
        }
    }
}

fn validate_github_signature(secret: &str, payload_bytes: &[u8], signature: &str) -> bool {
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
