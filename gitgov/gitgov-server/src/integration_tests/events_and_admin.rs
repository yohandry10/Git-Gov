use super::common::*;

#[tokio::test]
async fn golden_path_ingest_events_and_query() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let api_key = insert_test_api_key(&pool, "test-admin", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    // Step 1: Ingest events (Golden Path: stage → commit → push)
    let events_payload = serde_json::json!({
        "events": [
            {
                "event_uuid": "aaaaaaaa-0000-0000-0000-000000000001",
                "event_type": "stage_files",
                "user_login": "test-admin",
                "files": [{"path": "src/main.rs", "status": "modified"}],
                "status": "success",
                "timestamp": 1700000000
            },
            {
                "event_uuid": "aaaaaaaa-0000-0000-0000-000000000002",
                "event_type": "commit",
                "user_login": "test-admin",
                "files": [{"path": "src/main.rs", "status": "modified"}],
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
        parsed["total_events"].as_i64().unwrap() >= 3,
        "expected total_events ≥ 3"
    );

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn event_deduplication_works() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let api_key = insert_test_api_key(&pool, "test-admin", "Admin").await;
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
            implicit_uri: "/enterprise/release-governance/evaluate?repository_full_name=example-org/example-repo&release_id=rel-1&environment=production",
            cross_org_uri: "/enterprise/release-governance/evaluate?org_name=enterprise-b&repository_full_name=example-org/example-repo&release_id=rel-1&environment=production",
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
    let admin_key = insert_test_api_key(&pool, "admin-user", "Admin").await;
    let dev_key = insert_test_api_key(&pool, "dev-user", "Developer").await;
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
