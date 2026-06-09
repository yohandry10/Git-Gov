use super::common::*;

#[tokio::test]
async fn health_endpoint_returns_ok() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, body) = json_request(&app, "GET", "/health", None, None).await;
    assert_eq!(status, StatusCode::OK);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["status"], "ok");

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn health_detailed_returns_database_info() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, body) = json_request(&app, "GET", "/health/detailed", None, None).await;
    assert_eq!(status, StatusCode::OK);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["status"], "ok");
    assert!(parsed["database"].is_object());

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn unauthenticated_request_returns_401() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, _) = json_request(&app, "GET", "/stats", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = json_request(&app, "GET", "/logs", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = json_request(&app, "GET", "/me", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn invalid_api_key_returns_401() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, _) = json_request(&app, "GET", "/stats", None, Some("invalid-key-12345")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn authenticated_me_returns_user_info() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let api_key = insert_test_api_key(&pool, "test-admin", "Admin").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    let (status, body) = json_request(&app, "GET", "/me", None, Some(&api_key)).await;
    assert_eq!(status, StatusCode::OK);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["client_id"], "test-admin");
    assert_eq!(parsed["role"], "Admin");

    teardown(&admin_pool, &schema).await;
}
