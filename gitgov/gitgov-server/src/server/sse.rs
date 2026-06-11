use sqlx::postgres::PgListener;
use std::sync::Arc;
use std::time::Duration;

use crate::handlers;
use crate::server::config::{SSE_LISTENER_BACKOFF_MAX_SECS, SSE_LISTENER_BACKOFF_START_SECS};
pub(crate) fn spawn_distributed_sse_listener(state: Arc<handlers::AppState>, database_url: String) {
    let channel = state.sse_distributed_channel.clone();
    let source_node = state.worker_id.clone();
    tokio::spawn(async move {
        let mut backoff_secs = SSE_LISTENER_BACKOFF_START_SECS;
        loop {
            let mut listener = match PgListener::connect(&database_url).await {
                Ok(listener) => listener,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        channel = %channel,
                        backoff_secs,
                        "Distributed SSE listener failed to connect"
                    );
                    tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                    backoff_secs =
                        (backoff_secs.saturating_mul(2)).min(SSE_LISTENER_BACKOFF_MAX_SECS);
                    continue;
                }
            };

            if let Err(e) = listener.listen(&channel).await {
                tracing::warn!(
                    error = %e,
                    channel = %channel,
                    backoff_secs,
                    "Distributed SSE listener failed to subscribe"
                );
                tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                backoff_secs = (backoff_secs.saturating_mul(2)).min(SSE_LISTENER_BACKOFF_MAX_SECS);
                continue;
            }

            tracing::info!(channel = %channel, "Distributed SSE listener connected");
            backoff_secs = SSE_LISTENER_BACKOFF_START_SECS;
            loop {
                match listener.recv().await {
                    Ok(notification) => {
                        let payload = notification.payload();
                        let envelope =
                            match serde_json::from_str::<handlers::DistributedSseEnvelope>(payload)
                            {
                                Ok(parsed) => parsed,
                                Err(e) => {
                                    tracing::warn!(
                                        error = %e,
                                        channel = %channel,
                                        "Distributed SSE payload decode failed"
                                    );
                                    continue;
                                }
                            };
                        if envelope.source_node == source_node {
                            continue;
                        }
                        if let handlers::SseNotification::NewEvents { org_id, .. } =
                            &envelope.notification
                        {
                            handlers::invalidate_dashboard_caches_for_sse(
                                &state,
                                org_id.as_deref(),
                            );
                        }
                        let _ = state.sse_tx.send(envelope.notification);
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            channel = %channel,
                            "Distributed SSE listener lost connection; reconnecting"
                        );
                        break;
                    }
                }
            }

            tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
            backoff_secs = (backoff_secs.saturating_mul(2)).min(SSE_LISTENER_BACKOFF_MAX_SECS);
        }
    });
}
