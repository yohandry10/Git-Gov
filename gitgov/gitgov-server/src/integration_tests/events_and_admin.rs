use super::common::*;

async fn insert_repo_for_org(pool: &sqlx::PgPool, org_id: &str, full_name: &str) -> String {
    let repo_id = uuid::Uuid::new_v4().to_string();
    let repo_name = full_name
        .split('/')
        .next_back()
        .filter(|name| !name.is_empty())
        .unwrap_or(full_name)
        .to_string();

    sqlx::query(
        "INSERT INTO repos (id, org_id, full_name, name) VALUES ($1::uuid, $2::uuid, $3, $4)",
    )
    .bind(&repo_id)
    .bind(org_id)
    .bind(full_name)
    .bind(repo_name)
    .execute(pool)
    .await
    .expect("insert events test repo");

    repo_id
}

#[tokio::test]
async fn golden_path_ingest_events_and_query() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "golden-path-org").await;
    let api_key = insert_test_api_key_for_org(&pool, "test-admin", "Admin", &org_id).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    // Step 1: Ingest events (Golden Path: stage → commit → push)
    let events_payload = serde_json::json!({
        "events": [
            {
                "event_uuid": "aaaaaaaa-0000-0000-0000-000000000001",
                "event_type": "stage_files",
                "user_login": "test-admin",
                "files": ["src/main.rs"],
                "status": "success",
                "timestamp": 1700000000
            },
            {
                "event_uuid": "aaaaaaaa-0000-0000-0000-000000000002",
                "event_type": "commit",
                "user_login": "test-admin",
                "files": ["src/main.rs"],
                "status": "success",
                "branch": "main",
                "commit_sha": "abc123def456",
                "timestamp": 1700000001
            },
            {
                "event_uuid": "aaaaaaaa-0000-0000-0000-000000000003",
                "event_type": "successful_push",
                "user_login": "test-admin",
                "files": [],
                "status": "success",
                "branch": "main",
                "timestamp": 1700000002
            }
        ],
        "client_version": "integration-test"
    });

    let (status, body) = json_request(
        &app,
        "POST",
        "/events",
        Some(&events_payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "ingest failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["accepted"].as_array().unwrap().len(), 3);
    assert_eq!(parsed["duplicates"].as_array().unwrap().len(), 0);
    assert_eq!(parsed["errors"].as_array().unwrap().len(), 0);

    // Step 2: Query logs — should see the 3 events
    let (status, body) =
        json_request(&app, "GET", "/logs?limit=10&offset=0", None, Some(&api_key)).await;
    assert_eq!(status, StatusCode::OK, "logs failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    let events = parsed["events"].as_array().unwrap();
    assert!(
        events.len() >= 3,
        "expected ≥3 events, got {}",
        events.len()
    );

    // Step 3: Query stats — should reflect the ingested data
    let (status, body) = json_request(&app, "GET", "/stats", None, Some(&api_key)).await;
    assert_eq!(status, StatusCode::OK, "stats failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(
        parsed["client_events"]["total"].as_i64().unwrap() >= 3,
        "expected client_events.total ≥ 3"
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn webhook_replay_with_fresh_delivery_id_is_deduped_by_content_hash() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let db = Database::from_pool(pool.clone());

    use crate::db::WebhookIngestDecision;

    let payload = serde_json::json!({
        "ref": "refs/heads/main",
        "head_commit": { "id": "abc123" }
    });
    // Hash of the signed material (event_type + raw body). A replay reuses the
    // same signed body, so the content hash is identical even with a fresh
    // delivery_id.
    let content_hash = "fixed-content-hash-for-this-signed-body";

    // First delivery: stored and eligible for processing.
    let first = db
        .store_webhook_event(
            "delivery-original",
            "push",
            Some("sha256=valid-signature"),
            &payload,
            content_hash,
        )
        .await
        .expect("first webhook store");
    let first_id = match first {
        WebhookIngestDecision::Process(Some(id)) => id,
        other => panic!("first delivery should be processable, got {:?}", other),
    };

    // Retry-safety: a replay of a payload whose processing has NOT completed must
    // still be processable, so a transient processing failure is not silently lost.
    let replay_unprocessed = db
        .store_webhook_event(
            "delivery-replay-before-processed",
            "push",
            Some("sha256=valid-signature"),
            &payload,
            content_hash,
        )
        .await
        .expect("replay store (unprocessed)");
    assert!(
        matches!(replay_unprocessed, WebhookIngestDecision::Process(_)),
        "an unprocessed payload must remain reprocessable, not be dropped"
    );

    // Mark the first occurrence as processed.
    db.mark_webhook_processed(&first_id, None)
        .await
        .expect("mark processed");

    // Now a replay with a FRESH delivery_id but the same signed body is a true
    // duplicate of an already-processed event and must be skipped.
    let replay_processed = db
        .store_webhook_event(
            "delivery-replay-fresh",
            "push",
            Some("sha256=valid-signature"),
            &payload,
            content_hash,
        )
        .await
        .expect("replay store (processed)");
    assert!(
        matches!(replay_processed, WebhookIngestDecision::SkipDuplicate),
        "replay with a fresh delivery_id of an already-processed payload must be deduped"
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn events_with_future_timestamp_are_rejected() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "event-future-ts").await;
    let api_key = insert_test_api_key_for_org(&pool, "event-admin", "Admin", &org_id).await;
    let app = build_test_app(Arc::new(Database::from_pool(pool.clone())));

    let now_ms = chrono::Utc::now().timestamp_millis();
    let future_ms = now_ms + 24 * 60 * 60 * 1000; // +1 day: no legitimate event is in the future
    let past_ms = now_ms - 60 * 60 * 1000; // -1 hour: legitimate offline outbox backfill

    let payload = serde_json::json!({
        "events": [
            {
                "event_uuid": "ffffffff-0000-0000-0000-000000000001",
                "event_type": "commit",
                "user_login": "event-admin",
                "files": [],
                "status": "success",
                "commit_sha": "future1",
                "timestamp": future_ms
            },
            {
                "event_uuid": "ffffffff-0000-0000-0000-000000000002",
                "event_type": "commit",
                "user_login": "event-admin",
                "files": [],
                "status": "success",
                "commit_sha": "past1",
                "timestamp": past_ms
            }
        ],
        "client_version": "integration-test"
    });

    let (status, body) = json_request(
        &app,
        "POST",
        "/events",
        Some(&payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "ingest failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();

    let accepted: Vec<String> = parsed["accepted"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect();
    let errors = parsed["errors"].as_array().unwrap();

    // The legitimate past-dated (offline) event is accepted.
    assert!(
        accepted.contains(&"ffffffff-0000-0000-0000-000000000002".to_string()),
        "past-dated offline event should be accepted: {}",
        body
    );
    // The future-dated event must NOT be accepted...
    assert!(
        !accepted.contains(&"ffffffff-0000-0000-0000-000000000001".to_string()),
        "future-dated event must not be accepted: {}",
        body
    );
    // ...and is reported as an error mentioning the future timestamp.
    assert!(
        errors.iter().any(|e| {
            e["event_uuid"] == "ffffffff-0000-0000-0000-000000000001"
                && e["error"].as_str().unwrap_or_default().contains("future")
        }),
        "future-dated event should be rejected with a future-timestamp error: {}",
        body
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn event_ingest_rejects_unknown_event_type_and_status() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "event-fidelity-reject").await;
    let api_key = insert_test_api_key_for_org(&pool, "event-admin", "Admin", &org_id).await;
    let app = build_test_app(Arc::new(Database::from_pool(pool.clone())));

    let payload = serde_json::json!({
        "events": [
            {
                "event_uuid": "event-fidelity-unknown-type",
                "event_type": "not_a_real_event",
                "org_name": "event-fidelity-reject",
                "user_login": "event-admin",
                "files": [],
                "status": "success",
                "timestamp": 1700000000
            },
            {
                "event_uuid": "event-fidelity-unknown-status",
                "event_type": "commit",
                "org_name": "event-fidelity-reject",
                "user_login": "event-admin",
                "files": [],
                "status": "maybe",
                "timestamp": 1700000001
            }
        ],
        "client_version": "integration-test"
    });

    let (status, body) = json_request(
        &app,
        "POST",
        "/events",
        Some(&payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "ingest failed: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["accepted"].as_array().unwrap().len(), 0);
    assert_eq!(parsed["errors"].as_array().unwrap().len(), 2);
    assert!(parsed["errors"][0]["error"]
        .as_str()
        .unwrap()
        .contains("unsupported event_type"));
    assert!(parsed["errors"][1]["error"]
        .as_str()
        .unwrap()
        .contains("unsupported status"));

    let stored_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM client_events WHERE event_uuid LIKE 'event-fidelity-unknown-%'",
    )
    .fetch_one(&pool)
    .await
    .expect("count rejected events");
    assert_eq!(stored_count, 0);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn event_ingest_preserves_desktop_event_types_scope_and_sha() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "event-fidelity-org").await;
    let repo_id = insert_repo_for_org(&pool, &org_id, "event-fidelity-org/repo").await;
    let api_key = insert_test_api_key_for_org(&pool, "event-admin", "Admin", &org_id).await;
    let app = build_test_app(Arc::new(Database::from_pool(pool.clone())));
    let head_sha = "f1d2d2f924e986ac86fdf7b36c94bcdf32beec15";

    let payload = serde_json::json!({
        "events": [
            {
                "event_uuid": "event-fidelity-push-failed",
                "event_type": "push_failed",
                "org_name": "event-fidelity-org",
                "repo_full_name": "event-fidelity-org/repo",
                "user_login": "event-admin",
                "files": [],
                "branch": "main",
                "commit_sha": head_sha,
                "status": "failed",
                "timestamp": 1700000000
            },
            {
                "event_uuid": "event-fidelity-governance-blocked",
                "event_type": "governance_blocked_push",
                "org_name": "event-fidelity-org",
                "repo_full_name": "event-fidelity-org/repo",
                "user_login": "event-admin",
                "files": [],
                "branch": "main",
                "commit_sha": head_sha,
                "status": "blocked",
                "timestamp": 1700000001
            },
            {
                "event_uuid": "event-fidelity-governance-warned",
                "event_type": "governance_warned_push",
                "org_name": "event-fidelity-org",
                "repo_full_name": "event-fidelity-org/repo",
                "user_login": "event-admin",
                "files": [],
                "branch": "main",
                "commit_sha": head_sha,
                "status": "success",
                "timestamp": 1700000002
            },
            {
                "event_uuid": "event-fidelity-cli-start",
                "event_type": "cli_command",
                "org_name": "event-fidelity-org",
                "repo_full_name": "event-fidelity-org/repo",
                "user_login": "event-admin",
                "files": [],
                "branch": "main",
                "status": "success",
                "metadata": { "command_id": "cmd-1", "command": "git status" },
                "timestamp": 1700000003
            },
            {
                "event_uuid": "event-fidelity-cli-done",
                "event_type": "cli_command_completed",
                "org_name": "event-fidelity-org",
                "repo_full_name": "event-fidelity-org/repo",
                "user_login": "event-admin",
                "files": [],
                "branch": "main",
                "status": "success",
                "metadata": { "command_id": "cmd-1", "exit_code": 0 },
                "timestamp": 1700000004
            }
        ],
        "client_version": "integration-test"
    });

    let (status, body) = json_request(
        &app,
        "POST",
        "/events",
        Some(&payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "ingest failed: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["accepted"].as_array().unwrap().len(), 5);
    assert_eq!(parsed["errors"].as_array().unwrap().len(), 0);

    let rows = sqlx::query(
        r#"
        SELECT event_uuid, event_type, org_id::text, repo_id::text, branch, commit_sha, status
        FROM client_events
        WHERE event_uuid LIKE 'event-fidelity-%'
        ORDER BY created_at ASC
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("load fidelity events");
    assert_eq!(rows.len(), 5);

    let event_types: Vec<String> = rows.iter().map(|row| row.get("event_type")).collect();
    assert_eq!(
        event_types,
        vec![
            "push_failed",
            "governance_blocked_push",
            "governance_warned_push",
            "cli_command",
            "cli_command_completed"
        ]
    );
    assert!(!event_types
        .iter()
        .any(|event_type| event_type == "attempt_push"));
    for row in &rows {
        assert_eq!(row.get::<String, _>("org_id"), org_id);
        assert_eq!(row.get::<String, _>("repo_id"), repo_id);
        assert_eq!(
            row.get::<Option<String>, _>("branch").as_deref(),
            Some("main")
        );
    }
    for row in rows.iter().take(3) {
        assert_eq!(
            row.get::<Option<String>, _>("commit_sha").as_deref(),
            Some(head_sha)
        );
    }

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn event_ingest_rejects_repo_owner_outside_effective_org() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "event-owner-org-a").await;
    let api_key = insert_test_api_key_for_org(&pool, "event-owner-admin", "Admin", &org_id).await;
    let app = build_test_app(Arc::new(Database::from_pool(pool.clone())));

    let payload = serde_json::json!({
        "events": [
            {
                "event_uuid": "event-owner-cross-org-repo",
                "event_type": "commit",
                "repo_full_name": "event-owner-org-b/repo",
                "user_login": "event-owner-admin",
                "files": [],
                "branch": "main",
                "commit_sha": "1111111111111111111111111111111111111111",
                "status": "success",
                "timestamp": 1700000010
            },
            {
                "event_uuid": "event-owner-malformed-repo",
                "event_type": "commit",
                "repo_full_name": "repo-without-owner",
                "user_login": "event-owner-admin",
                "files": [],
                "branch": "main",
                "commit_sha": "2222222222222222222222222222222222222222",
                "status": "success",
                "timestamp": 1700000011
            },
            {
                "event_uuid": "event-owner-bad-char-repo",
                "event_type": "commit",
                "repo_full_name": "event-owner-org-a/repo with spaces",
                "user_login": "event-owner-admin",
                "files": [],
                "branch": "main",
                "commit_sha": "4444444444444444444444444444444444444444",
                "status": "success",
                "timestamp": 1700000012
            }
        ],
        "client_version": "integration-test"
    });

    let (status, body) = json_request(
        &app,
        "POST",
        "/events",
        Some(&payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "ingest failed: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["accepted"].as_array().unwrap().len(), 0);
    assert_eq!(parsed["errors"].as_array().unwrap().len(), 3);
    assert!(parsed["errors"][0]["error"]
        .as_str()
        .unwrap()
        .contains("repo_full_name owner does not match"));
    assert!(parsed["errors"][1]["error"]
        .as_str()
        .unwrap()
        .contains("owner/repo"));
    assert!(parsed["errors"][2]["error"]
        .as_str()
        .unwrap()
        .contains("owner/repo"));

    let stored_events: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM client_events WHERE event_uuid LIKE 'event-owner-%'",
    )
    .fetch_one(&pool)
    .await
    .expect("count rejected owner events");
    assert_eq!(stored_events, 0);

    let stored_repos: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM repos WHERE full_name IN ('event-owner-org-b/repo', 'repo-without-owner', 'event-owner-org-a/repo with spaces')",
    )
    .fetch_one(&pool)
    .await
    .expect("count rejected owner repos");
    assert_eq!(stored_repos, 0);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn global_admin_event_ingest_derives_org_from_repo_owner() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "event-derived-owner").await;
    let api_key = insert_test_api_key(&pool, "global-event-admin", "Admin").await;
    let app = build_test_app(Arc::new(Database::from_pool(pool.clone())));

    let payload = serde_json::json!({
        "events": [{
            "event_uuid": "event-owner-derived-global",
            "event_type": "commit",
            "repo_full_name": "event-derived-owner/new-repo",
            "user_login": "global-event-admin",
            "files": [],
            "branch": "main",
            "commit_sha": "3333333333333333333333333333333333333333",
            "status": "success",
            "timestamp": 1700000020
        }],
        "client_version": "integration-test"
    });

    let (status, body) = json_request(
        &app,
        "POST",
        "/events",
        Some(&payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "ingest failed: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["accepted"].as_array().unwrap().len(), 1);
    assert_eq!(parsed["errors"].as_array().unwrap().len(), 0);

    let stored_org_id: String = sqlx::query_scalar(
        "SELECT org_id::text FROM client_events WHERE event_uuid = 'event-owner-derived-global'",
    )
    .fetch_one(&pool)
    .await
    .expect("load derived owner event org");
    assert_eq!(stored_org_id, org_id);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn event_ingest_rejects_unknown_explicit_org_name() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let _org_id = insert_test_org(&pool, "event-known-owner").await;
    let api_key = insert_test_api_key(&pool, "global-event-admin-unknown-org", "Admin").await;
    let app = build_test_app(Arc::new(Database::from_pool(pool.clone())));

    let payload = serde_json::json!({
        "events": [{
            "event_uuid": "event-unknown-explicit-org",
            "event_type": "commit",
            "org_name": "missing-owner",
            "repo_full_name": "event-known-owner/repo",
            "user_login": "global-event-admin-unknown-org",
            "files": [],
            "branch": "main",
            "commit_sha": "5555555555555555555555555555555555555555",
            "status": "success",
            "timestamp": 1700000030
        }],
        "client_version": "integration-test"
    });

    let (status, body) = json_request(
        &app,
        "POST",
        "/events",
        Some(&payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "ingest failed: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["accepted"].as_array().unwrap().len(), 0);
    assert_eq!(parsed["errors"][0]["error"], "Event org_name was not found");

    let stored_events: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM client_events WHERE event_uuid = 'event-unknown-explicit-org'",
    )
    .fetch_one(&pool)
    .await
    .expect("count unknown org event");
    assert_eq!(stored_events, 0);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn cli_command_ingest_enforces_repo_owner_scope() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_a = insert_test_org(&pool, "cli-scope-a").await;
    let _org_b = insert_test_org(&pool, "cli-scope-b").await;
    let api_key = insert_test_api_key_for_org(&pool, "cli-scope-admin", "Admin", &org_a).await;
    let app = build_test_app(Arc::new(Database::from_pool(pool.clone())));

    let accepted_payload = serde_json::json!({
        "org_name": "cli-scope-a",
        "command": "git status",
        "origin": "manual_input",
        "branch": "main",
        "repo_name": "cli-scope-a/repo",
        "exit_code": 0,
        "duration_ms": 12,
        "metadata": { "command_id": "cli-scope-ok" }
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/cli/commands",
        Some(&accepted_payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "cli ingest failed: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["accepted"], true);

    let cross_org_payload = serde_json::json!({
        "org_name": "cli-scope-a",
        "command": "git status",
        "origin": "manual_input",
        "branch": "main",
        "repo_name": "cli-scope-b/repo",
        "exit_code": 0,
        "duration_ms": 12,
        "metadata": { "command_id": "cli-scope-cross" }
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/cli/commands",
        Some(&cross_org_payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "cross cli body: {body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        parsed["error"],
        "repo_name owner does not match organization"
    );

    let malformed_payload = serde_json::json!({
        "org_name": "cli-scope-a",
        "command": "git status",
        "origin": "manual_input",
        "branch": "main",
        "repo_name": "cli-scope-a/repo with spaces",
        "exit_code": 0,
        "duration_ms": 12,
        "metadata": { "command_id": "cli-scope-malformed" }
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/cli/commands",
        Some(&malformed_payload.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "bad cli body: {body}");

    let stored_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM cli_commands WHERE metadata->>'command_id' LIKE 'cli-scope-%'",
    )
    .fetch_one(&pool)
    .await
    .expect("count scoped cli commands");
    assert_eq!(stored_count, 1);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn event_deduplication_works() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "dedupe-org").await;
    let api_key = insert_test_api_key_for_org(&pool, "test-admin", "Admin", &org_id).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let event = serde_json::json!({
        "events": [{
            "event_uuid": "dedup-test-uuid-001",
            "event_type": "commit",
            "user_login": "test-admin",
            "files": [],
            "status": "success",
            "timestamp": 1700000000
        }],
        "client_version": "integration-test"
    });

    // First ingestion — accepted
    let (status, body) = json_request(
        &app,
        "POST",
        "/events",
        Some(&event.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["accepted"].as_array().unwrap().len(), 1);

    // Second ingestion — deduplicated
    let (status, body) = json_request(
        &app,
        "POST",
        "/events",
        Some(&event.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["accepted"].as_array().unwrap().len(), 0);
    assert_eq!(parsed["duplicates"].as_array().unwrap().len(), 1);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn developer_role_cannot_access_admin_endpoints() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let dev_key = insert_test_api_key(&pool, "test-dev", "Developer").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    // Developer can access /me
    let (status, body) = json_request(&app, "GET", "/me", None, Some(&dev_key)).await;
    assert_eq!(status, StatusCode::OK);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["role"], "Developer");

    // Developer cannot access /stats (admin-only)
    let (status, _) = json_request(&app, "GET", "/stats", None, Some(&dev_key)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Developer cannot access /dashboard (admin-only)
    let (status, _) = json_request(&app, "GET", "/dashboard", None, Some(&dev_key)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn enterprise_admin_routes_enforce_auth_and_org_scope() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_a = insert_test_org(&pool, "enterprise-a").await;
    let _org_b = insert_test_org(&pool, "enterprise-b").await;
    let global_admin_key = insert_test_api_key(&pool, "global-admin", "Admin").await;
    let scoped_admin_key =
        insert_test_api_key_for_org(&pool, "enterprise-a-admin", "Admin", &org_a).await;
    let scoped_dev_key =
        insert_test_api_key_for_org(&pool, "enterprise-a-dev", "Developer", &org_a).await;
    let release_hash = "a".repeat(64);
    sqlx::query(
        r#"
        INSERT INTO release_evidence_packets (
            org_id, ticket_id, release_id, repository_full_name, branch, target_sha,
            environment, evidence_packet_hash, evidence_packet_uri, packet, generated_by, generated_at
        ) VALUES (
            $1::uuid, 'KAN-1', 'rel-1', 'example-org/example-repo', 'main',
            'abcdef1234567890abcdef1234567890abcdef12', 'production', $2,
            '/evidence/packets/tickets/KAN-1?repo_full_name=example-org%2Fexample-repo&branch=main&target_sha=abcdef1234567890abcdef1234567890abcdef12&release_id=rel-1&environment=production&hours=72',
            '{}'::jsonb, 'test', NOW()
        )
        "#,
    )
    .bind(&org_a)
    .bind(&release_hash)
    .execute(&pool)
    .await
    .expect("insert enterprise route release evidence packet");
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    fn profile_payload(org_name: Option<&str>) -> String {
        let mut payload = serde_json::json!({
            "profile": {
                "customer_name": "ExampleCo",
                "repository_full_name": "example-org/example-repo",
                "default_branch": "main",
                "jira_project_key": "KAN",
                "policy_preset": "moderate",
                "providers": ["github", "jira"],
                "modules": ["traceability", "release-readiness"],
                "release_governance": {
                    "mode": "record-only",
                    "environment": "production",
                    "approval_required": false,
                    "enforcement": "disabled",
                    "quorum": {
                        "enabled": false,
                        "rules": []
                    }
                }
            }
        });
        if let Some(org_name) = org_name {
            payload["org_name"] = serde_json::json!(org_name);
        }
        payload.to_string()
    }

    fn tracking_payload(org_name: Option<&str>) -> String {
        let mut payload = serde_json::json!({
            "tracking": {
                "version": 1,
                "items": [
                    {
                        "stage_id": "providers",
                        "status": "in-progress",
                        "owner": "Platform owner",
                        "external_ref": "KAN-61"
                    }
                ]
            }
        });
        if let Some(org_name) = org_name {
            payload["org_name"] = serde_json::json!(org_name);
        }
        payload.to_string()
    }

    struct EnterpriseRouteCase<'a> {
        method: &'a str,
        implicit_uri: &'a str,
        cross_org_uri: &'a str,
        implicit_body: Option<String>,
        cross_org_body: Option<String>,
    }

    let evaluate_implicit_uri = format!("/enterprise/release-governance/evaluate?repository_full_name=example-org/example-repo&branch=main&target_sha=abcdef1234567890abcdef1234567890abcdef12&release_id=rel-1&environment=production&evidence_packet_hash={release_hash}");
    let evaluate_cross_org_uri = format!("/enterprise/release-governance/evaluate?org_name=enterprise-b&repository_full_name=example-org/example-repo&branch=main&target_sha=abcdef1234567890abcdef1234567890abcdef12&release_id=rel-1&environment=production&evidence_packet_hash={release_hash}");

    let cases = vec![
        EnterpriseRouteCase {
            method: "GET",
            implicit_uri: "/enterprise/adoption-profile",
            cross_org_uri: "/enterprise/adoption-profile?org_name=enterprise-b",
            implicit_body: None,
            cross_org_body: None,
        },
        EnterpriseRouteCase {
            method: "PUT",
            implicit_uri: "/enterprise/adoption-profile",
            cross_org_uri: "/enterprise/adoption-profile",
            implicit_body: Some(profile_payload(None)),
            cross_org_body: Some(profile_payload(Some("enterprise-b"))),
        },
        EnterpriseRouteCase {
            method: "GET",
            implicit_uri: "/enterprise/onboarding-checklist-tracking",
            cross_org_uri: "/enterprise/onboarding-checklist-tracking?org_name=enterprise-b",
            implicit_body: None,
            cross_org_body: None,
        },
        EnterpriseRouteCase {
            method: "PUT",
            implicit_uri: "/enterprise/onboarding-checklist-tracking",
            cross_org_uri: "/enterprise/onboarding-checklist-tracking",
            implicit_body: Some(tracking_payload(None)),
            cross_org_body: Some(tracking_payload(Some("enterprise-b"))),
        },
        EnterpriseRouteCase {
            method: "GET",
            implicit_uri: "/enterprise/release-approvals",
            cross_org_uri: "/enterprise/release-approvals?org_name=enterprise-b",
            implicit_body: None,
            cross_org_body: None,
        },
        EnterpriseRouteCase {
            method: "GET",
            implicit_uri: &evaluate_implicit_uri,
            cross_org_uri: &evaluate_cross_org_uri,
            implicit_body: None,
            cross_org_body: None,
        },
    ];

    for case in cases {
        let (status, body) = json_request(
            &app,
            case.method,
            case.implicit_uri,
            case.implicit_body.as_deref(),
            None,
        )
        .await;
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "{} {} should require auth: {}",
            case.method,
            case.implicit_uri,
            body
        );

        let (status, body) = json_request(
            &app,
            case.method,
            case.implicit_uri,
            case.implicit_body.as_deref(),
            Some(&scoped_dev_key),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "{} {} should require admin role: {}",
            case.method,
            case.implicit_uri,
            body
        );

        let (status, body) = json_request(
            &app,
            case.method,
            case.implicit_uri,
            case.implicit_body.as_deref(),
            Some(&global_admin_key),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "{} {} should require org_name for global admin keys: {}",
            case.method,
            case.implicit_uri,
            body
        );

        let (status, body) = json_request(
            &app,
            case.method,
            case.cross_org_uri,
            case.cross_org_body.as_deref(),
            Some(&scoped_admin_key),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "{} {} should block cross-org scoped admin access: {}",
            case.method,
            case.cross_org_uri,
            body
        );

        let (status, body) = json_request(
            &app,
            case.method,
            case.implicit_uri,
            case.implicit_body.as_deref(),
            Some(&scoped_admin_key),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "{} {} should allow scoped admin implicit org access: {}",
            case.method,
            case.implicit_uri,
            body
        );
    }

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn create_org_requires_founder_global_admin_key() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let founder_key = insert_test_api_key(&pool, "bootstrap-admin", "Admin").await;
    let non_founder_admin_key = insert_test_api_key(&pool, "admin-user", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let payload = serde_json::json!({
        "login": "scope-test-org",
        "name": "Scope Test Org"
    });

    let (status, body) = json_request(
        &app,
        "POST",
        "/orgs",
        Some(&payload.to_string()),
        Some(&non_founder_admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "non-founder admin should be blocked: {}",
        body
    );

    let (status, body) = json_request(
        &app,
        "POST",
        "/orgs",
        Some(&payload.to_string()),
        Some(&founder_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "founder should be allowed: {}",
        body
    );
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["login"], "scope-test-org");
    assert_eq!(parsed["created"], true);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn org_discovery_and_me_return_human_scope() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_a = insert_test_org(&pool, "enterprise-a").await;
    let _org_b = insert_test_org(&pool, "enterprise-b").await;
    let global_admin_key = insert_test_api_key(&pool, "bootstrap-admin", "Admin").await;
    let scoped_admin_key =
        insert_test_api_key_for_org(&pool, "enterprise-a-admin", "Admin", &org_a).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, body) = json_request(&app, "GET", "/orgs", None, Some(&global_admin_key)).await;
    assert_eq!(status, StatusCode::OK, "global org list failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    let logins: Vec<String> = parsed
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|org| org["login"].as_str().map(str::to_string))
        .collect();
    assert!(logins.contains(&"enterprise-a".to_string()));
    assert!(logins.contains(&"enterprise-b".to_string()));

    let (status, body) = json_request(
        &app,
        "GET",
        "/orgs/enterprise-a",
        None,
        Some(&scoped_admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "scoped org lookup failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["login"], "enterprise-a");

    let (status, body) = json_request(
        &app,
        "GET",
        "/orgs/enterprise-b",
        None,
        Some(&scoped_admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "cross-org lookup should fail: {}",
        body
    );

    let (status, body) = json_request(&app, "GET", "/me", None, Some(&scoped_admin_key)).await;
    assert_eq!(status, StatusCode::OK, "me failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["org_id"], org_a);
    assert_eq!(parsed["org_name"], "enterprise-a");

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn create_api_key_rejects_invalid_role_instead_of_silent_fallback() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let admin_key = insert_test_api_key(&pool, "bootstrap-admin", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let invalid_payload = serde_json::json!({
        "client_id": "role-case-test",
        "role": "admin"
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/api-keys",
        Some(&invalid_payload.to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "unexpected body: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        parsed["error"],
        "role must be one of: Admin, Architect, Developer, PM"
    );

    let valid_payload = serde_json::json!({
        "client_id": "role-case-test-valid",
        "role": "Admin"
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/api-keys",
        Some(&valid_payload.to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "unexpected body: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(
        parsed["api_key"].as_str().is_some(),
        "api_key should be present for valid role"
    );
    assert_eq!(parsed["client_id"], "role-case-test-valid");

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn api_keys_respect_requested_org_scope() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_a = insert_test_org(&pool, "enterprise-a").await;
    let org_b = insert_test_org(&pool, "enterprise-b").await;
    let global_admin_key = insert_test_api_key(&pool, "bootstrap-admin", "Admin").await;
    let enterprise_a_key =
        insert_test_api_key_for_org(&pool, "enterprise-a-dev", "Developer", &org_a).await;
    let enterprise_b_key =
        insert_test_api_key_for_org(&pool, "enterprise-b-dev", "Developer", &org_b).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, body) = json_request(
        &app,
        "GET",
        "/api-keys?org_name=enterprise-a",
        None,
        Some(&global_admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "scoped list failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    let keys = parsed.as_array().unwrap();
    assert!(
        keys.iter().all(|key| key["org_name"] == "enterprise-a"),
        "scoped list should contain only enterprise-a keys: {}",
        body
    );
    assert!(
        keys.iter()
            .any(|key| key["client_id"] == "enterprise-a-dev"),
        "enterprise-a key should be visible: {}",
        body
    );
    assert!(
        keys.iter()
            .all(|key| key["client_id"] != "enterprise-b-dev"),
        "enterprise-b key should not be visible in enterprise-a scope: {}",
        body
    );

    let invalid_payload = serde_json::json!({
        "client_id": "invalid-org-key",
        "role": "Developer",
        "org_name": "does-not-exist"
    });
    let (status, body) = json_request(
        &app,
        "POST",
        "/api-keys",
        Some(&invalid_payload.to_string()),
        Some(&global_admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "invalid org should fail: {}",
        body
    );

    let (status, body) = json_request(
        &app,
        "POST",
        "/api-keys/00000000-0000-0000-0000-000000000000/revoke?org_name=enterprise-a",
        None,
        Some(&global_admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "missing key revoke should be scoped 404: {}",
        body
    );

    let all_keys = sqlx::query(
        "SELECT id::text, client_id FROM api_keys WHERE client_id = $1 OR client_id = $2",
    )
    .bind("enterprise-a-dev")
    .bind("enterprise-b-dev")
    .fetch_all(&pool)
    .await
    .unwrap();
    let enterprise_b_id: String = all_keys
        .iter()
        .find(|row| row.get::<String, _>("client_id") == "enterprise-b-dev")
        .unwrap()
        .get("id");

    let (status, body) = json_request(
        &app,
        "POST",
        &format!("/api-keys/{}/revoke?org_name=enterprise-a", enterprise_b_id),
        None,
        Some(&global_admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "cross-scope revoke should fail: {}",
        body
    );

    let (status, body) = json_request(
        &app,
        "POST",
        &format!("/api-keys/{}/revoke?org_name=enterprise-b", enterprise_b_id),
        None,
        Some(&global_admin_key),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "matching scope revoke should pass: {}",
        body
    );
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["success"], true);

    assert!(!enterprise_a_key.trim().is_empty());
    assert!(!enterprise_b_key.trim().is_empty());

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn developer_only_sees_own_logs() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "logs-scope-org").await;
    let admin_key = insert_test_api_key_for_org(&pool, "admin-user", "Admin", &org_id).await;
    let dev_key = insert_test_api_key_for_org(&pool, "dev-user", "Developer", &org_id).await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    // Admin ingests events for two different users
    let events = serde_json::json!({
        "events": [
            {
                "event_uuid": "scope-test-001",
                "event_type": "commit",
                "user_login": "admin-user",
                "files": [],
                "status": "success",
                "timestamp": 1700000000
            },
            {
                "event_uuid": "scope-test-002",
                "event_type": "commit",
                "user_login": "dev-user",
                "files": [],
                "status": "success",
                "timestamp": 1700000001
            },
            {
                "event_uuid": "scope-test-003",
                "event_type": "commit",
                "user_login": "other-user",
                "files": [],
                "status": "success",
                "timestamp": 1700000002
            }
        ],
        "client_version": "integration-test"
    });

    let (status, _) = json_request(
        &app,
        "POST",
        "/events",
        Some(&events.to_string()),
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Developer queries logs — should only see their own events
    let (status, body) =
        json_request(&app, "GET", "/logs?limit=50&offset=0", None, Some(&dev_key)).await;
    assert_eq!(status, StatusCode::OK);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    let events = parsed["events"].as_array().unwrap();
    for event in events {
        let source = &event["source"];
        if source.is_string() && source.as_str().unwrap() == "client" {
            if let Some(login) = event["user_login"].as_str() {
                assert_eq!(login, "dev-user", "Developer saw another user's event");
            }
        }
    }

    // Admin queries logs — should see all events
    let (status, body) = json_request(
        &app,
        "GET",
        "/logs?limit=50&offset=0",
        None,
        Some(&admin_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    let events = parsed["events"].as_array().unwrap();
    assert!(events.len() >= 3, "admin should see all 3 events");

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn events_endpoint_validates_payload() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let api_key = insert_test_api_key(&pool, "test-admin", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    // Empty events array
    let empty = serde_json::json!({ "events": [], "client_version": "test" });
    let (status, _) = json_request(
        &app,
        "POST",
        "/events",
        Some(&empty.to_string()),
        Some(&api_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Malformed JSON
    let (status, _) = json_request(
        &app,
        "POST",
        "/events",
        Some("not json at all"),
        Some(&api_key),
    )
    .await;
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "expected 400 or 422 for malformed JSON, got {}",
        status
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn daily_activity_endpoint_returns_data() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let api_key = insert_test_api_key(&pool, "test-admin", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, body) = json_request(&app, "GET", "/stats/daily", None, Some(&api_key)).await;
    assert_eq!(status, StatusCode::OK, "daily activity failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(parsed.is_array(), "expected array response");

    teardown(&admin_pool, &schema).await;
}
