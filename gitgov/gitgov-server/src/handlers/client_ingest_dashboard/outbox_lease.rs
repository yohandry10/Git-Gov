#[derive(Debug, Clone, Copy)]
struct OutboxLeaseTelemetryInput {
    mode: OutboxLeaseTelemetryMode,
    requested_ttl_ms: u64,
    effective_ttl_ms: u64,
    wait_ms: u64,
    ttl_clamped: bool,
    wait_clamped: bool,
    request_started: Instant,
}

fn record_outbox_lease_telemetry(state: &Arc<AppState>, input: OutboxLeaseTelemetryInput) {
    match state.outbox_lease_telemetry.lock() {
        Ok(mut telemetry) => telemetry.record(OutboxLeaseTelemetryRecord {
            mode: input.mode,
            requested_ttl_ms: input.requested_ttl_ms,
            effective_ttl_ms: input.effective_ttl_ms,
            wait_ms: input.wait_ms,
            ttl_clamped: input.ttl_clamped,
            wait_clamped: input.wait_clamped,
            handler_duration_ms: input.request_started.elapsed().as_millis() as u64,
        }),
        Err(_) => tracing::warn!("Outbox lease telemetry lock poisoned; skipping telemetry record"),
    }
}

pub async fn acquire_outbox_flush_lease(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
    Json(request): Json<OutboxLeaseRequest>,
) -> impl IntoResponse {
    let request_started = Instant::now();
    let requested_ttl_ms = request
        .lease_ttl_ms
        .unwrap_or(state.outbox_server_lease_ttl_ms);
    let lease_ttl_ms = requested_ttl_ms.clamp(1_000, 60_000);
    let ttl_clamped = requested_ttl_ms != lease_ttl_ms;
    let requested_max_wait_ms = request.max_wait_ms.unwrap_or(lease_ttl_ms);
    let max_wait_ms = requested_max_wait_ms.clamp(250, 120_000);
    let wait_clamped = requested_max_wait_ms != max_wait_ms;

    if !state.outbox_server_lease_enabled {
        record_outbox_lease_telemetry(
            &state,
            OutboxLeaseTelemetryInput {
                mode: OutboxLeaseTelemetryMode::DisabledFailOpen,
                requested_ttl_ms,
                effective_ttl_ms: lease_ttl_ms,
                wait_ms: 0,
                ttl_clamped,
                wait_clamped,
                request_started,
            },
        );
        return (
            StatusCode::OK,
            Json(OutboxLeaseResponse {
                granted: true,
                wait_ms: 0,
                lease_ttl_ms: state.outbox_server_lease_ttl_ms,
                mode: "disabled_fail_open".to_string(),
            }),
        );
    }

    let scope = request
        .scope
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| auth_user.org_id.clone())
        .unwrap_or_else(|| "global".to_string());
    let scope_key = format!("flush:{}", scope);
    let holder = request
        .holder
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| {
            format!(
                "client:{}:{}",
                auth_user.client_id,
                auth_user.org_id.as_deref().unwrap_or("global")
            )
        });

    match state
        .db
        .try_acquire_outbox_flush_lease(&scope_key, &holder, Duration::from_millis(lease_ttl_ms))
        .await
    {
        Ok(decision) => {
            let response_wait_ms = decision.wait_ms.min(max_wait_ms);
            record_outbox_lease_telemetry(
                &state,
                OutboxLeaseTelemetryInput {
                    mode: if decision.granted {
                        OutboxLeaseTelemetryMode::Granted
                    } else {
                        OutboxLeaseTelemetryMode::Denied
                    },
                    requested_ttl_ms,
                    effective_ttl_ms: lease_ttl_ms,
                    wait_ms: response_wait_ms,
                    ttl_clamped,
                    wait_clamped,
                    request_started,
                },
            );
            (
                StatusCode::OK,
                Json(OutboxLeaseResponse {
                    granted: decision.granted,
                    wait_ms: response_wait_ms,
                    lease_ttl_ms,
                    mode: "server_lease".to_string(),
                }),
            )
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                scope_key = %scope_key,
                holder = %holder,
                "Outbox lease acquisition failed; returning fail-open grant"
            );
            record_outbox_lease_telemetry(
                &state,
                OutboxLeaseTelemetryInput {
                    mode: OutboxLeaseTelemetryMode::DbErrorFailOpen,
                    requested_ttl_ms,
                    effective_ttl_ms: lease_ttl_ms,
                    wait_ms: 0,
                    ttl_clamped,
                    wait_clamped,
                    request_started,
                },
            );
            (
                StatusCode::OK,
                Json(OutboxLeaseResponse {
                    granted: true,
                    wait_ms: 0,
                    lease_ttl_ms,
                    mode: "db_error_fail_open".to_string(),
                }),
            )
        }
    }
}

pub async fn get_outbox_lease_metrics(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin(&auth_user) {
        return resp.into_response();
    }

    let telemetry = match state.outbox_lease_telemetry.lock() {
        Ok(telemetry) => telemetry.snapshot(),
        Err(_) => {
            tracing::warn!("Outbox lease telemetry lock poisoned while reading");
            OutboxLeaseTelemetrySnapshot::default()
        }
    };

    (
        StatusCode::OK,
        Json(OutboxLeaseTelemetryResponse {
            enabled: state.outbox_server_lease_enabled,
            default_lease_ttl_ms: state.outbox_server_lease_ttl_ms,
            telemetry,
        }),
    )
        .into_response()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogsResponse {
    pub events: Vec<CombinedEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecations: Option<Vec<String>>,
}
