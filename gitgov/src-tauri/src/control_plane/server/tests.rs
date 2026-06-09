use super::{CombinedEvent, ServerStats};
use serde::Deserialize;

#[test]
fn contract_server_stats_deserializes_with_defaults() {
    let payload = serde_json::json!({
        "github_events": {
            "total": 0,
            "today": 0,
            "pushes_today": 0
        },
        "client_events": {
            "total": 0,
            "today": 0,
            "blocked_today": 0
        },
        "violations": {
            "total": 0,
            "unresolved": 0,
            "critical": 0
        },
        "active_devs_week": 0,
        "active_repos": 0
    });

    let stats: ServerStats = serde_json::from_value(payload).expect("deserialize ServerStats");
    assert!(stats.github_events.by_type.is_empty());
    assert_eq!(stats.client_events.desktop_pushes_today, 0);
    assert!(stats.client_events.by_type.is_empty());
    assert!(stats.client_events.by_status.is_empty());
    assert_eq!(stats.pipeline.total_7d, 0);
}

#[test]
fn contract_combined_event_defaults_details_to_empty_object() {
    let payload = serde_json::json!({
        "id": "evt-1",
        "source": "client",
        "event_type": "commit",
        "created_at": 123
    });

    let event: CombinedEvent = serde_json::from_value(payload).expect("deserialize CombinedEvent");
    assert_eq!(event.details, serde_json::json!({}));
}

#[test]
fn contract_logs_envelope_ignores_optional_backend_metadata() {
    #[derive(Deserialize)]
    struct LogsResponse {
        events: Vec<CombinedEvent>,
    }

    let payload = serde_json::json!({
        "events": [{
            "id": "evt-1",
            "source": "client",
            "event_type": "commit",
            "created_at": 123,
            "details": {}
        }],
        "error": "legacy-offset",
        "stale": true,
        "deprecations": ["offset"]
    });

    let response: LogsResponse =
        serde_json::from_value(payload).expect("deserialize logs response");
    assert_eq!(response.events.len(), 1);
}
