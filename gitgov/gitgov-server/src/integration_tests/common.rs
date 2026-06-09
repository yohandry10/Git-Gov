use crate::auth;
pub(super) use crate::db::Database;
pub(super) use crate::handlers::PolicyCheckBlockingScope;
use crate::handlers::{self, AppState, ConversationalRuntime};
pub(super) use axum::http::StatusCode;
use axum::{
    body::Body,
    http::Request,
    middleware,
    routing::{get, post, put},
    Router,
};
use sha2::Digest;
use sqlx::PgPool;
pub(super) use sqlx::Row;
use std::collections::HashMap;
use std::sync::atomic::AtomicI64;
pub(super) use std::sync::Arc;
use std::sync::Mutex;
pub(super) use std::time::Duration;
use std::time::Instant;
use tokio::sync::Semaphore;
use tower::ServiceExt;

/// Try to connect to the test database and set up an isolated schema.
/// Returns None if TEST_DATABASE_URL is not set or connection fails (test will be skipped).
/// Returns (pool_with_schema, schema_name, admin_pool_for_teardown).
pub(super) async fn try_setup() -> Option<(PgPool, String, PgPool)> {
    let url = std::env::var("TEST_DATABASE_URL").ok()?;

    // Admin pool: used to create/drop schema only.
    let admin_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&url)
        .await
        .ok()?;

    let schema = format!("test_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));

    // Create schema using admin pool.
    sqlx::query(&format!("CREATE SCHEMA \"{}\"", schema))
        .execute(&admin_pool)
        .await
        .expect("create test schema");

    // Build a pool where EVERY connection sets search_path to the test schema.
    let schema_for_hook = schema.clone();
    let test_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .after_connect(move |conn, _meta| {
            let s = schema_for_hook.clone();
            Box::pin(async move {
                sqlx::query(&format!("SET search_path TO \"{}\"", s))
                    .execute(&mut *conn)
                    .await?;
                Ok(())
            })
        })
        .connect(&url)
        .await
        .expect("connect test pool with schema");

    // Apply minimal DDL needed for the Golden Path tests.
    let ddl = r#"
        CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
        CREATE EXTENSION IF NOT EXISTS "pgcrypto";

        CREATE TABLE IF NOT EXISTS orgs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            github_id BIGINT UNIQUE,
            login TEXT UNIQUE NOT NULL,
            name TEXT,
            avatar_url TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS enterprise_adoption_profiles (
            org_id UUID PRIMARY KEY REFERENCES orgs(id) ON DELETE CASCADE,
            profile JSONB NOT NULL,
            updated_by TEXT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS enterprise_onboarding_checklist_tracking (
            org_id UUID PRIMARY KEY REFERENCES orgs(id) ON DELETE CASCADE,
            tracking JSONB NOT NULL,
            updated_by TEXT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS enterprise_release_approvals (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
            release_id TEXT NOT NULL,
            repository_full_name TEXT NOT NULL,
            branch TEXT,
            target_sha TEXT,
            environment TEXT NOT NULL,
            decision TEXT NOT NULL CHECK (decision IN ('approved', 'rejected', 'accepted-risk')),
            approver TEXT NOT NULL,
            ticket_id TEXT,
            evidence_packet_hash TEXT,
            evidence_packet_uri TEXT,
            evidence_summary JSONB NOT NULL DEFAULT '{}'::jsonb,
            risk_severity TEXT NOT NULL DEFAULT 'none' CHECK (risk_severity IN ('none', 'low', 'medium', 'high', 'critical')),
            risk_acceptance_reason TEXT,
            expires_at TIMESTAMPTZ,
            approval_hash TEXT NOT NULL UNIQUE,
            created_by TEXT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS repos (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
            github_id BIGINT UNIQUE,
            full_name TEXT UNIQUE NOT NULL,
            name TEXT NOT NULL,
            private BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS api_keys (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            key_hash TEXT UNIQUE NOT NULL,
            client_id TEXT NOT NULL,
            org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
            role TEXT NOT NULL DEFAULT 'Developer',
            created_at TIMESTAMPTZ DEFAULT NOW(),
            last_used TIMESTAMPTZ,
            is_active BOOLEAN DEFAULT TRUE
        );

        CREATE TABLE IF NOT EXISTS client_events (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
            repo_id UUID REFERENCES repos(id) ON DELETE CASCADE,
            event_uuid TEXT UNIQUE NOT NULL,
            event_type TEXT NOT NULL,
            user_login TEXT NOT NULL,
            user_name TEXT,
            branch TEXT,
            commit_sha TEXT,
            files JSONB DEFAULT '[]',
            status TEXT NOT NULL,
            reason TEXT,
            metadata JSONB DEFAULT '{}',
            client_version TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            synced_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS github_events (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
            repo_id UUID REFERENCES repos(id) ON DELETE CASCADE,
            delivery_id TEXT UNIQUE NOT NULL,
            event_type TEXT NOT NULL,
            actor_login TEXT,
            actor_id BIGINT,
            ref_name TEXT,
            ref_type TEXT,
            before_sha TEXT,
            after_sha TEXT,
            commit_shas JSONB DEFAULT '[]',
            commits_count INTEGER DEFAULT 0,
            payload JSONB NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            processed_at TIMESTAMPTZ
        );

        CREATE TABLE IF NOT EXISTS violations (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
            repo_id UUID REFERENCES repos(id) ON DELETE CASCADE,
            github_event_id UUID REFERENCES github_events(id),
            client_event_id UUID REFERENCES client_events(id),
            violation_type TEXT NOT NULL,
            severity TEXT DEFAULT 'warning',
            confidence_level TEXT DEFAULT 'pending',
            reason TEXT,
            user_login TEXT,
            branch TEXT,
            commit_sha TEXT,
            details JSONB DEFAULT '{}',
            correlated_github_event_id UUID REFERENCES github_events(id),
            correlated_client_event_id UUID REFERENCES client_events(id),
            resolved BOOLEAN DEFAULT FALSE,
            resolved_at TIMESTAMPTZ,
            resolved_by TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS policies (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
            repo_id UUID REFERENCES repos(id) ON DELETE CASCADE UNIQUE,
            config JSONB NOT NULL,
            checksum TEXT NOT NULL,
            override_actor TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS webhook_events (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            delivery_id TEXT UNIQUE NOT NULL,
            event_type TEXT NOT NULL,
            payload JSONB NOT NULL,
            processed BOOLEAN DEFAULT FALSE,
            error TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS pipeline_events (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID,
            pipeline_id TEXT NOT NULL,
            pipeline_name TEXT NOT NULL,
            status TEXT NOT NULL,
            branch TEXT,
            commit_sha TEXT,
            trigger_user TEXT,
            stages JSONB DEFAULT '[]',
            duration_ms BIGINT,
            url TEXT,
            metadata JSONB DEFAULT '{}',
            ingested_at TIMESTAMPTZ DEFAULT NOW(),
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS project_tickets (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID,
            ticket_id TEXT UNIQUE NOT NULL,
            project_key TEXT NOT NULL,
            summary TEXT,
            status TEXT,
            assignee TEXT,
            reporter TEXT,
            ticket_type TEXT,
            priority TEXT,
            labels JSONB DEFAULT '[]',
            related_commits JSONB DEFAULT '[]',
            related_prs JSONB DEFAULT '[]',
            url TEXT,
            raw_payload JSONB DEFAULT '{}',
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS commit_ticket_correlations (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID,
            commit_sha TEXT NOT NULL,
            ticket_id TEXT NOT NULL,
            source TEXT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(commit_sha, ticket_id)
        );

        CREATE TABLE IF NOT EXISTS export_logs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID,
            requested_by TEXT NOT NULL,
            format TEXT NOT NULL,
            filters JSONB DEFAULT '{}',
            event_count INTEGER DEFAULT 0,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS governance_events (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID,
            event_type TEXT NOT NULL,
            actor TEXT,
            repo TEXT,
            branch TEXT,
            details JSONB DEFAULT '{}',
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS pr_merges (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID,
            repo TEXT NOT NULL,
            pr_number INTEGER NOT NULL,
            title TEXT,
            author TEXT,
            merged_by TEXT,
            base_branch TEXT,
            head_branch TEXT,
            commit_sha TEXT,
            reviewers JSONB DEFAULT '[]',
            approved_by JSONB DEFAULT '[]',
            review_count INTEGER DEFAULT 0,
            additions INTEGER DEFAULT 0,
            deletions INTEGER DEFAULT 0,
            changed_files INTEGER DEFAULT 0,
            url TEXT,
            merged_at TIMESTAMPTZ DEFAULT NOW(),
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS pull_request_merges (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID REFERENCES orgs(id),
            repo_id UUID REFERENCES repos(id),
            delivery_id TEXT NOT NULL UNIQUE,
            pr_number INT NOT NULL,
            pr_title TEXT,
            author_login TEXT,
            merged_by_login TEXT,
            head_sha TEXT,
            base_branch TEXT,
            payload JSONB NOT NULL DEFAULT '{}',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS admin_audit_log (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            actor_client_id TEXT NOT NULL,
            action TEXT NOT NULL,
            target_type TEXT,
            target_id TEXT,
            metadata JSONB DEFAULT '{}',
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS client_sessions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            client_id TEXT NOT NULL,
            org_id UUID,
            app_version TEXT,
            os TEXT,
            hostname TEXT,
            last_seen TIMESTAMPTZ DEFAULT NOW(),
            created_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(client_id)
        );

        CREATE TABLE IF NOT EXISTS identity_aliases (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            primary_login TEXT NOT NULL,
            alias_login TEXT UNIQUE NOT NULL,
            created_by TEXT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS noncompliance_signals (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID,
            signal_type TEXT NOT NULL,
            severity TEXT DEFAULT 'medium',
            status TEXT DEFAULT 'open',
            description TEXT,
            evidence JSONB DEFAULT '{}',
            user_login TEXT,
            repo TEXT,
            branch TEXT,
            commit_sha TEXT,
            detected_at TIMESTAMPTZ DEFAULT NOW(),
            reviewed_at TIMESTAMPTZ,
            reviewed_by TEXT,
            resolution TEXT,
            violation_id UUID,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS violation_decisions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            violation_id UUID NOT NULL,
            actor TEXT NOT NULL,
            decision TEXT NOT NULL,
            reason TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS policy_history (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            repo_id TEXT NOT NULL,
            actor TEXT NOT NULL,
            action TEXT NOT NULL,
            config JSONB,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS policy_drift_events (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
            user_login TEXT NOT NULL,
            action TEXT NOT NULL,
            repo_name TEXT NOT NULL,
            result TEXT NOT NULL,
            before_checksum TEXT,
            after_checksum TEXT,
            duration_ms BIGINT,
            metadata JSONB DEFAULT '{}'::jsonb,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS policy_change_requests (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
            repo_id UUID REFERENCES repos(id) ON DELETE CASCADE,
            repo_name TEXT NOT NULL,
            requested_by TEXT NOT NULL,
            requested_config JSONB NOT NULL,
            requested_checksum TEXT NOT NULL,
            reason TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS policy_change_request_decisions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            request_id UUID UNIQUE REFERENCES policy_change_requests(id) ON DELETE CASCADE,
            org_id UUID REFERENCES orgs(id) ON DELETE CASCADE,
            decision TEXT NOT NULL,
            decided_by TEXT NOT NULL,
            note TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS jobs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            job_type TEXT NOT NULL,
            payload JSONB DEFAULT '{}',
            status TEXT NOT NULL DEFAULT 'pending',
            attempts INTEGER DEFAULT 0,
            max_attempts INTEGER DEFAULT 3,
            error TEXT,
            worker_id TEXT,
            locked_at TIMESTAMPTZ,
            completed_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS org_users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID NOT NULL,
            login TEXT NOT NULL,
            display_name TEXT,
            email TEXT,
            role TEXT NOT NULL DEFAULT 'Developer',
            status TEXT NOT NULL DEFAULT 'active',
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(org_id, login)
        );

        CREATE TABLE IF NOT EXISTS org_invitations (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            org_id UUID NOT NULL,
            email TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'Developer',
            token_hash TEXT UNIQUE NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            invited_by TEXT NOT NULL,
            expires_at TIMESTAMPTZ NOT NULL,
            accepted_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS feature_requests (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_login TEXT NOT NULL,
            org_id TEXT,
            title TEXT NOT NULL,
            description TEXT,
            category TEXT DEFAULT 'general',
            priority TEXT DEFAULT 'normal',
            status TEXT DEFAULT 'open',
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        -- Indexes for performance
        CREATE INDEX IF NOT EXISTS idx_client_events_uuid ON client_events(event_uuid);
        CREATE INDEX IF NOT EXISTS idx_client_events_created ON client_events(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_client_events_type ON client_events(event_type);
        CREATE INDEX IF NOT EXISTS idx_client_events_user ON client_events(user_login);
        CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON api_keys(key_hash);
    "#;

    sqlx::raw_sql(ddl)
        .execute(&test_pool)
        .await
        .expect("apply test DDL");

    Some((test_pool, schema, admin_pool))
}

/// Drop the test schema after the test. Uses admin_pool (no search_path override).
pub(super) async fn teardown(admin_pool: &PgPool, schema: &str) {
    let _ = sqlx::query(&format!("DROP SCHEMA \"{}\" CASCADE", schema))
        .execute(admin_pool)
        .await;
}

/// Insert a test API key into the database. Returns the raw key.
pub(super) async fn insert_test_api_key(pool: &PgPool, client_id: &str, role: &str) -> String {
    let raw_key = format!("test-key-{}", uuid::Uuid::new_v4());
    let hash = format!("{:x}", sha2::Sha256::digest(raw_key.as_bytes()));
    sqlx::query(
        "INSERT INTO api_keys (key_hash, client_id, role, is_active) VALUES ($1, $2, $3, true)",
    )
    .bind(&hash)
    .bind(client_id)
    .bind(role)
    .execute(pool)
    .await
    .expect("insert test API key");
    raw_key
}

pub(super) async fn insert_test_api_key_for_org(
    pool: &PgPool,
    client_id: &str,
    role: &str,
    org_id: &str,
) -> String {
    let raw_key = format!("test-key-{}", uuid::Uuid::new_v4());
    let hash = format!("{:x}", sha2::Sha256::digest(raw_key.as_bytes()));
    sqlx::query(
        "INSERT INTO api_keys (key_hash, client_id, role, org_id, is_active) VALUES ($1, $2, $3, $4::uuid, true)",
    )
    .bind(&hash)
    .bind(client_id)
    .bind(role)
    .bind(org_id)
    .execute(pool)
    .await
    .expect("insert org-scoped test API key");
    raw_key
}

pub(super) async fn insert_test_org(pool: &PgPool, login: &str) -> String {
    let org_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO orgs (id, login, name) VALUES ($1::uuid, $2, $3)")
        .bind(&org_id)
        .bind(login)
        .bind(format!("{} Org", login))
        .execute(pool)
        .await
        .expect("insert test org");
    org_id
}

/// Insert a minimal org + repo for policy endpoints.
pub(super) async fn insert_test_repo(pool: &PgPool, full_name: &str) -> (String, String) {
    let org_id = uuid::Uuid::new_v4().to_string();
    let repo_id = uuid::Uuid::new_v4().to_string();
    let org_login = format!("org-{}", uuid::Uuid::new_v4().simple());
    let repo_name = full_name.split('/').nth(1).unwrap_or("repo").to_string();

    sqlx::query("INSERT INTO orgs (id, login, name) VALUES ($1::uuid, $2, $3)")
        .bind(&org_id)
        .bind(&org_login)
        .bind("Test Org")
        .execute(pool)
        .await
        .expect("insert test org");

    sqlx::query(
        "INSERT INTO repos (id, org_id, full_name, name, private) VALUES ($1::uuid, $2::uuid, $3, $4, false)",
    )
    .bind(&repo_id)
    .bind(&org_id)
    .bind(full_name)
    .bind(&repo_name)
    .execute(pool)
    .await
    .expect("insert test repo");

    (org_id, repo_id)
}

pub(super) async fn insert_test_policy(pool: &PgPool, repo_id: &str, config: serde_json::Value) {
    sqlx::query(
        r#"
        INSERT INTO policies (id, org_id, repo_id, config, checksum, override_actor)
        SELECT
            gen_random_uuid(),
            r.org_id,
            r.id,
            $2::jsonb,
            $3,
            'integration-test'
        FROM repos r
        WHERE r.id = $1::uuid
        ON CONFLICT (repo_id) DO UPDATE
        SET config = EXCLUDED.config,
            checksum = EXCLUDED.checksum,
            updated_at = NOW()
        "#,
    )
    .bind(repo_id)
    .bind(config)
    .bind(format!("checksum-{}", uuid::Uuid::new_v4()))
    .execute(pool)
    .await
    .expect("insert test policy");
}

/// Build a minimal Router with auth middleware for integration testing.
pub(super) fn build_test_app_with_options(
    db: Arc<Database>,
    alert_webhook_url: Option<String>,
    drift_alert_webhook_urls: Vec<String>,
    policy_check_block_scopes: Vec<PolicyCheckBlockingScope>,
) -> Router {
    let state = AppState {
        db: Arc::clone(&db),
        github_webhook_secret: None,
        github_personal_access_token: None,
        jenkins_webhook_secret: None,
        jira_webhook_secret: None,
        start_time: Instant::now(),
        worker_id: "test-worker".to_string(),
        http_client: reqwest::Client::new(),
        alert_webhook_url,
        drift_alert_webhook_urls,
        strict_actor_match: false,
        reject_synthetic_logins: false,
        events_max_batch: 1000,
        llm_api_key: None,
        llm_model: "test".to_string(),
        feature_request_webhook_url: None,
        conversational_runtime: Arc::new(Mutex::new(ConversationalRuntime::default())),
        chat_llm_semaphore: Arc::new(Semaphore::new(1)),
        chat_llm_queue_timeout_ms: 500,
        chat_llm_timeout_ms: 9000,
        stats_cache_ttl: Duration::from_millis(100),
        stats_cache: Arc::new(Mutex::new(HashMap::new())),
        org_lookup_cache_ttl: Duration::from_millis(0),
        org_lookup_cache: Arc::new(Mutex::new(HashMap::new())),
        repo_lookup_cache_ttl: Duration::from_millis(0),
        repo_lookup_cache: Arc::new(Mutex::new(HashMap::new())),
        repo_upsert_min_interval: Duration::from_millis(0),
        repo_upsert_last_attempt: Arc::new(Mutex::new(HashMap::new())),
        cache_invalidation_min_interval: Duration::from_millis(0),
        stats_cache_invalidation_min_interval: Duration::from_millis(0),
        logs_cache_invalidation_min_interval: Duration::from_millis(0),
        stats_cache_last_invalidation_ms: Arc::new(AtomicI64::new(0)),
        stats_cache_refresh_lock: Arc::new(tokio::sync::Mutex::new(())),
        logs_cache_ttl: Duration::from_millis(100),
        logs_cache_stale_on_error: Duration::from_millis(1000),
        logs_reject_offset_pagination: false,
        outbox_server_lease_enabled: false,
        outbox_server_lease_ttl_ms: 2000,
        outbox_lease_telemetry: Arc::new(Mutex::new(handlers::OutboxLeaseTelemetry::default())),
        logs_cache: Arc::new(Mutex::new(HashMap::new())),
        logs_cache_last_invalidation_ms: Arc::new(AtomicI64::new(0)),
        client_session_upsert_min_interval: Duration::from_millis(0),
        client_session_last_upsert: Arc::new(Mutex::new(HashMap::new())),
        sse_tx: tokio::sync::broadcast::channel::<handlers::SseNotification>(64).0,
        sse_max_connections: Arc::new(Semaphore::new(50)),
        sse_distributed_enabled: false,
        sse_distributed_channel: "test_sse".to_string(),
        policy_check_block_scopes,
    };

    let auth_routes = Router::new()
        .route("/events", post(handlers::ingest_client_events))
        .route("/logs", get(handlers::get_logs))
        .route("/stats", get(handlers::get_stats))
        .route("/stats/daily", get(handlers::get_daily_activity))
        .route("/dashboard", get(handlers::get_dashboard))
        .route(
            "/compliance/{org_name}",
            get(handlers::get_compliance_dashboard),
        )
        .route(
            "/signals/detect/{org_name}",
            post(handlers::trigger_detection),
        )
        .route("/me", get(handlers::get_me))
        .route("/orgs", get(handlers::list_orgs).post(handlers::create_org))
        .route("/orgs/{login}", get(handlers::get_org))
        .route(
            "/api-keys",
            get(handlers::list_api_keys).post(handlers::create_api_key),
        )
        .route("/api-keys/{id}/revoke", post(handlers::revoke_api_key))
        .route("/export", post(handlers::export_events))
        .route(
            "/enterprise/adoption-profile",
            get(handlers::get_enterprise_adoption_profile)
                .put(handlers::upsert_enterprise_adoption_profile),
        )
        .route(
            "/enterprise/onboarding-checklist-tracking",
            get(handlers::get_enterprise_onboarding_checklist_tracking)
                .put(handlers::upsert_enterprise_onboarding_checklist_tracking),
        )
        .route(
            "/enterprise/release-approvals",
            get(handlers::list_enterprise_release_approvals)
                .post(handlers::create_enterprise_release_approval),
        )
        .route(
            "/enterprise/release-governance/evaluate",
            get(handlers::evaluate_enterprise_release_governance),
        )
        .route("/policy/{repo_name}", get(handlers::get_policy))
        .route(
            "/policy/{repo_name}/override",
            put(handlers::override_policy),
        )
        .route("/policy/check", post(handlers::policy_check))
        .route(
            "/policy/{repo_name}/requests",
            post(handlers::create_policy_change_request).get(handlers::list_policy_change_requests),
        )
        .route(
            "/policy/requests/{request_id}/approve",
            post(handlers::approve_policy_change_request),
        )
        .route(
            "/policy/requests/{request_id}/reject",
            post(handlers::reject_policy_change_request),
        )
        .route(
            "/policy/drift-events",
            post(handlers::ingest_policy_drift_event).get(handlers::list_policy_drift_events),
        )
        .layer(middleware::from_fn_with_state(
            Arc::clone(&db),
            auth::auth_middleware,
        ));

    Router::new()
        .route("/health", get(handlers::health))
        .route("/health/detailed", get(handlers::detailed_health))
        .merge(auth_routes)
        .with_state(Arc::new(state))
}

pub(super) fn build_test_app_with_alerts(
    db: Arc<Database>,
    alert_webhook_url: Option<String>,
    drift_alert_webhook_urls: Vec<String>,
) -> Router {
    build_test_app_with_options(db, alert_webhook_url, drift_alert_webhook_urls, vec![])
}

pub(super) fn build_test_app_with_policy_check_scopes(
    db: Arc<Database>,
    policy_check_block_scopes: Vec<PolicyCheckBlockingScope>,
) -> Router {
    build_test_app_with_options(db, None, vec![], policy_check_block_scopes)
}

pub(super) fn build_test_app(db: Arc<Database>) -> Router {
    build_test_app_with_options(db, None, vec![], vec![])
}

/// Helper: make a JSON request to the test app.
pub(super) async fn json_request(
    app: &Router,
    method: &str,
    uri: &str,
    body: Option<&str>,
    api_key: Option<&str>,
) -> (StatusCode, String) {
    let mut builder = Request::builder().uri(uri);
    builder = match method {
        "GET" => builder.method("GET"),
        "POST" => builder.method("POST"),
        _ => builder.method(method),
    };
    if let Some(key) = api_key {
        builder = builder.header("Authorization", format!("Bearer {}", key));
    }
    if body.is_some() {
        builder = builder.header("Content-Type", "application/json");
    }
    let req_body = body
        .map(|b| Body::from(b.to_string()))
        .unwrap_or(Body::empty());
    let request = builder.body(req_body).unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), 1_000_000)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes).to_string();
    (status, body_str)
}

pub(super) async fn spawn_webhook_probe() -> (
    String,
    tokio::sync::oneshot::Receiver<String>,
    tokio::task::JoinHandle<()>,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind webhook probe listener");
    let addr = listener.local_addr().expect("listener local addr");
    let (body_tx, body_rx) = tokio::sync::oneshot::channel::<String>();
    let task = tokio::spawn(async move {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let (mut socket, _) = listener.accept().await.expect("accept webhook connection");
        let mut buf = vec![0u8; 16 * 1024];
        let read = socket.read(&mut buf).await.expect("read webhook request");
        let req = String::from_utf8_lossy(&buf[..read]).to_string();
        let body = req.split("\r\n\r\n").nth(1).unwrap_or_default().to_string();
        let _ = body_tx.send(body);
        let _ = socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
            .await;
    });

    (format!("http://{}", addr), body_rx, task)
}

/// Macro to reduce boilerplate: skip test if DB unavailable.
macro_rules! setup_or_skip {
    () => {
        match try_setup().await {
            Some(result) => result,
            None => {
                eprintln!("SKIPPED: TEST_DATABASE_URL not set or unreachable");
                return;
            }
        }
    };
}

// ========================================================================
// TESTS
// ========================================================================
