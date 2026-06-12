use super::common::*;
use sha2::Digest;

fn invitation_token_and_hash() -> (String, String) {
    let token = format!("invite-token-{}", uuid::Uuid::new_v4());
    let token_hash = format!("{:x}", sha2::Sha256::digest(token.as_bytes()));
    (token, token_hash)
}

async fn insert_invitation(
    pool: &sqlx::PgPool,
    org_id: &str,
    invite_email: Option<&str>,
    invite_login: Option<&str>,
    role: &str,
) -> (String, String) {
    let (token, token_hash) = invitation_token_and_hash();
    sqlx::query(
        r#"
        INSERT INTO org_invitations (
            org_id, invite_email, invite_login, role, token_hash, invited_by, expires_at
        )
        VALUES ($1::uuid, $2, $3, $4, $5, 'bootstrap-admin', NOW() + INTERVAL '1 day')
        "#,
    )
    .bind(org_id)
    .bind(invite_email)
    .bind(invite_login)
    .bind(role)
    .bind(&token_hash)
    .execute(pool)
    .await
    .expect("insert org invitation");
    (token, token_hash)
}

async fn accept_invitation(
    app: &axum::Router,
    token: &str,
    login: Option<&str>,
) -> (StatusCode, String) {
    let payload = serde_json::json!({
        "token": token,
        "login": login
    });
    json_request(
        app,
        "POST",
        "/org-invitations/accept",
        Some(&payload.to_string()),
        None,
    )
    .await
}

#[tokio::test]
async fn email_only_invitation_cannot_impersonate_existing_org_user() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "invite-collision-org").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    sqlx::query(
        r#"
        INSERT INTO org_users (
            org_id, login, display_name, email, role, status, created_by, updated_by
        )
        VALUES (
            $1::uuid, 'alice-admin', 'Alice Admin', 'alice@corp.test',
            'Admin', 'active', 'bootstrap-admin', 'bootstrap-admin'
        )
        "#,
    )
    .bind(&org_id)
    .execute(&pool)
    .await
    .expect("insert existing org user");

    let (invite_token, token_hash) = insert_invitation(
        &pool,
        &org_id,
        Some("alice-admin@contractor.test"),
        None,
        "Developer",
    )
    .await;

    let (status, body) = accept_invitation(&app, &invite_token, Some("alice-admin")).await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "email-only collision should fail closed: {}",
        body
    );

    let user_row = sqlx::query(
        r#"
        SELECT role, email, status
        FROM org_users
        WHERE org_id = $1::uuid AND login = 'alice-admin'
        "#,
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("load existing org user");
    assert_eq!(user_row.get::<String, _>("role"), "Admin");
    assert_eq!(user_row.get::<String, _>("email"), "alice@corp.test");
    assert_eq!(user_row.get::<String, _>("status"), "active");

    let issued_key_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM api_keys
        WHERE org_id = $1::uuid AND client_id = 'alice-admin'
        "#,
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("count issued keys");
    assert_eq!(issued_key_count, 0);

    let invitation_row = sqlx::query(
        r#"
        SELECT status, accepted_by
        FROM org_invitations
        WHERE token_hash = $1
        "#,
    )
    .bind(&token_hash)
    .fetch_one(&pool)
    .await
    .expect("load invitation");
    assert_eq!(invitation_row.get::<String, _>("status"), "pending");
    assert!(invitation_row
        .get::<Option<String>, _>("accepted_by")
        .is_none());

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn email_only_invitation_creates_new_user_once() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "invite-new-user-org").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);
    let (invite_token, token_hash) = insert_invitation(
        &pool,
        &org_id,
        Some("carol-dev@example.test"),
        None,
        "Developer",
    )
    .await;

    let (status, body) = accept_invitation(&app, &invite_token, Some("carol-dev")).await;
    assert_eq!(status, StatusCode::OK, "accept failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("accept json");
    assert_eq!(parsed["client_id"], "carol-dev");
    assert_eq!(parsed["role"], "Developer");
    assert_eq!(parsed["org_id"], org_id);
    assert!(parsed["api_key"].as_str().unwrap_or_default().len() >= 32);

    let user_row = sqlx::query(
        r#"
        SELECT email, role, status
        FROM org_users
        WHERE org_id = $1::uuid AND login = 'carol-dev'
        "#,
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("load accepted org user");
    assert_eq!(user_row.get::<String, _>("email"), "carol-dev@example.test");
    assert_eq!(user_row.get::<String, _>("role"), "Developer");
    assert_eq!(user_row.get::<String, _>("status"), "active");

    let key_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM api_keys
        WHERE org_id = $1::uuid AND client_id = 'carol-dev' AND role = 'Developer'
        "#,
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("count issued key");
    assert_eq!(key_count, 1);

    let (status, body) = accept_invitation(&app, &invite_token, Some("carol-dev")).await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "accepted invitation should be single-use: {}",
        body
    );

    let key_count_after_replay: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM api_keys
        WHERE org_id = $1::uuid AND client_id = 'carol-dev'
        "#,
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("count issued keys after replay");
    assert_eq!(key_count_after_replay, 1);

    let accepted_by: Option<String> = sqlx::query_scalar(
        "SELECT accepted_by FROM org_invitations WHERE token_hash = $1 AND status = 'accepted'",
    )
    .bind(&token_hash)
    .fetch_one(&pool)
    .await
    .expect("load accepted_by");
    assert_eq!(accepted_by.as_deref(), Some("carol-dev"));

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn explicit_login_invitation_can_target_existing_org_user() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "invite-explicit-login-org").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);

    sqlx::query(
        r#"
        INSERT INTO org_users (
            org_id, login, display_name, email, role, status, created_by, updated_by
        )
        VALUES (
            $1::uuid, 'existing-dev', 'Existing Dev', 'old@example.test',
            'Developer', 'disabled', 'bootstrap-admin', 'bootstrap-admin'
        )
        "#,
    )
    .bind(&org_id)
    .execute(&pool)
    .await
    .expect("insert existing disabled org user");

    let (invite_token, _) = insert_invitation(
        &pool,
        &org_id,
        Some("new@example.test"),
        Some("existing-dev"),
        "Architect",
    )
    .await;

    let (status, body) = accept_invitation(&app, &invite_token, Some("existing-dev")).await;
    assert_eq!(status, StatusCode::OK, "explicit accept failed: {}", body);
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("accept json");
    assert_eq!(parsed["client_id"], "existing-dev");
    assert_eq!(parsed["role"], "Architect");

    let user_row = sqlx::query(
        r#"
        SELECT email, role, status
        FROM org_users
        WHERE org_id = $1::uuid AND login = 'existing-dev'
        "#,
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("load updated org user");
    assert_eq!(user_row.get::<String, _>("email"), "new@example.test");
    assert_eq!(user_row.get::<String, _>("role"), "Architect");
    assert_eq!(user_row.get::<String, _>("status"), "active");

    let key_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM api_keys
        WHERE org_id = $1::uuid AND client_id = 'existing-dev' AND role = 'Architect'
        "#,
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("count issued explicit key");
    assert_eq!(key_count, 1);

    teardown(&admin_pool, &schema).await;
}

#[tokio::test]
async fn requested_login_mismatch_does_not_consume_invitation() {
    let (pool, schema, admin_pool) = setup_or_skip!();
    let org_id = insert_test_org(&pool, "invite-mismatch-org").await;
    let db = Arc::new(Database::from_pool(pool.clone()));
    let app = build_test_app(db);
    let (invite_token, token_hash) = insert_invitation(
        &pool,
        &org_id,
        Some("target-user@example.test"),
        None,
        "Developer",
    )
    .await;

    let (status, body) = accept_invitation(&app, &invite_token, Some("mallory")).await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "mismatched requested login should fail validation: {}",
        body
    );
    assert!(
        body.contains("login does not match"),
        "unexpected mismatch body: {}",
        body
    );

    let pending_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM org_invitations WHERE token_hash = $1 AND status = 'pending'",
    )
    .bind(&token_hash)
    .fetch_one(&pool)
    .await
    .expect("count pending invitation");
    assert_eq!(pending_count, 1);

    let (status, body) = accept_invitation(&app, &invite_token, Some("target-user")).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "correct login should still accept after mismatch: {}",
        body
    );

    let issued_key_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM api_keys
        WHERE org_id = $1::uuid AND client_id = 'target-user'
        "#,
    )
    .bind(&org_id)
    .fetch_one(&pool)
    .await
    .expect("count issued target key");
    assert_eq!(issued_key_count, 1);

    teardown(&admin_pool, &schema).await;
}
