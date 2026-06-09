use clap::Parser;
use dotenvy::dotenv;
use sha2::Digest;
use std::cmp::min;
use std::collections::HashMap;
use std::io::IsTerminal;
use std::net::SocketAddr;
use std::sync::atomic::AtomicI64;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::handlers::AppState;
use crate::server::config::*;
use crate::server::jobs;
use crate::server::rate_limit::{DistributedDbRateLimiter, InMemoryRateLimiter, RateLimiterState};
use crate::server::routes;
use crate::server::sse::spawn_distributed_sse_listener;
use crate::{db, handlers, models};
pub(crate) async fn run() {
    dotenv().ok();

    // Keep shared/legacy API model types linked under strict clippy settings.
    models::touch_contract_types();

    let args = Args::parse();
    let (runtime_env, is_dev_env, runtime_env_explicit) = parse_runtime_env();
    if !runtime_env_explicit {
        tracing::warn!(
            runtime_env = %runtime_env,
            "GITGOV_ENV not set explicitly; using compile-profile default"
        );
    }

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gitgov_server=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Prometheus metrics recorder — must be installed before any metrics::* calls.
    let prometheus_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus metrics recorder");
    tracing::info!("Prometheus metrics recorder installed");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let insecure_jwt_fallback_requested =
        parse_bool_env("GITGOV_ALLOW_INSECURE_JWT_FALLBACK", false);
    let allow_insecure_jwt_fallback = insecure_jwt_fallback_requested && is_dev_env;
    if insecure_jwt_fallback_requested && !is_dev_env {
        tracing::warn!(
            runtime_env = %runtime_env,
            "Ignoring GITGOV_ALLOW_INSECURE_JWT_FALLBACK outside dev/test environments"
        );
    }
    let _jwt_secret = match std::env::var("GITGOV_JWT_SECRET") {
        Ok(secret) if !secret.trim().is_empty() => secret,
        _ if allow_insecure_jwt_fallback => {
            tracing::warn!(
                runtime_env = %runtime_env,
                "Using insecure JWT secret fallback; set GITGOV_JWT_SECRET for hardened environments"
            );
            "gitgov-secret-key-change-in-production".to_string()
        }
        _ => {
            tracing::error!(
                runtime_env = %runtime_env,
                "Missing GITGOV_JWT_SECRET in non-dev hardening mode"
            );
            std::process::exit(1);
        }
    };

    let github_webhook_secret = std::env::var("GITHUB_WEBHOOK_SECRET")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    if !is_dev_env && github_webhook_secret.is_none() {
        tracing::error!(
            runtime_env = %runtime_env,
            "Missing GITHUB_WEBHOOK_SECRET in non-dev hardening mode"
        );
        std::process::exit(1);
    }
    if is_dev_env && github_webhook_secret.is_none() {
        tracing::warn!(
            "GITHUB_WEBHOOK_SECRET is not configured; GitHub webhook signature validation is disabled"
        );
    }
    let github_personal_access_token = std::env::var("GITHUB_PERSONAL_ACCESS_TOKEN").ok();
    let jenkins_webhook_secret = std::env::var("JENKINS_WEBHOOK_SECRET").ok();
    let jira_webhook_secret = std::env::var("JIRA_WEBHOOK_SECRET").ok();

    let db = match db::Database::new(&database_url).await {
        Ok(db) => Arc::new(db),
        Err(e) => {
            tracing::error!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    tracing::info!("Connected to Supabase database");

    // Bootstrap: create first admin API key if none exist
    // Or use the key from GITGOV_API_KEY env if configured
    let should_print_key = args.print_bootstrap_key || std::io::stderr().is_terminal();

    // Check if GITGOV_API_KEY is configured and keep it active as global founder admin key
    if let Ok(env_api_key) = std::env::var("GITGOV_API_KEY") {
        let key_hash = format!("{:x}", sha2::Sha256::digest(env_api_key.as_bytes()));

        match db.ensure_admin_api_key(&key_hash, "bootstrap-admin").await {
            Ok(_) => {
                tracing::info!("GITGOV_API_KEY ensured as active global Admin key");
                if should_print_key {
                    eprintln!();
                    eprintln!("╔════════════════════════════════════════════════════════════════╗");
                    eprintln!(
                        "║  GITGOV_API_KEY configured and ready                            ║"
                    );
                    eprintln!("╚════════════════════════════════════════════════════════════════╝");
                    eprintln!();
                }
            }
            Err(e) => {
                tracing::error!("Failed to ensure GITGOV_API_KEY: {}", e);
            }
        }
    } else {
        // No GITGOV_API_KEY in env, use normal bootstrap
        match db.bootstrap_admin_key().await {
            Ok(Some(api_key)) => {
                if should_print_key {
                    eprintln!();
                    eprintln!("╔════════════════════════════════════════════════════════════════╗");
                    eprintln!("║  BOOTSTRAP ADMIN KEY - SAVE NOW, WILL NOT BE SHOWN AGAIN       ║");
                    eprintln!("╠════════════════════════════════════════════════════════════════╣");
                    eprintln!("║  {}", api_key);
                    eprintln!("╚════════════════════════════════════════════════════════════════╝");
                    eprintln!();
                }
                tracing::info!(
                    print_key = should_print_key,
                    "Bootstrap admin key created. Use --print-bootstrap-key to display."
                );
            }
            Ok(None) => {
                tracing::info!("API keys already exist, skipping bootstrap");
            }
            Err(e) => {
                tracing::error!("Failed to bootstrap admin key: {}", e);
            }
        }
    }

    let worker_id_clone = jobs::start_job_worker(Arc::clone(&db));

    let alert_webhook_url = std::env::var("GITGOV_ALERT_WEBHOOK_URL").ok();
    let mut drift_alert_webhook_urls = parse_csv_env("GITGOV_DRIFT_ALERT_WEBHOOK_URLS");
    if drift_alert_webhook_urls.is_empty() {
        if let Ok(single_url) = std::env::var("GITGOV_DRIFT_ALERT_WEBHOOK_URL") {
            let trimmed = single_url.trim();
            if !trimmed.is_empty() {
                drift_alert_webhook_urls.push(trimmed.to_string());
            }
        }
    }
    drift_alert_webhook_urls.sort();
    drift_alert_webhook_urls.dedup();
    let strict_actor_match = parse_bool_env("GITGOV_STRICT_ACTOR_MATCH", true);
    let reject_synthetic_logins = parse_bool_env("GITGOV_REJECT_SYNTHETIC_LOGINS", false);
    let llm_api_key = std::env::var("GEMINI_API_KEY").ok();
    let llm_model =
        std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.5-flash".to_string());
    let feature_request_webhook_url = std::env::var("FEATURE_REQUEST_WEBHOOK_URL").ok();
    let chat_llm_max_concurrency = parse_usize_env("GITGOV_CHAT_LLM_MAX_CONCURRENCY", 16);
    let chat_llm_queue_timeout_ms =
        parse_usize_env("GITGOV_CHAT_LLM_QUEUE_TIMEOUT_MS", 3000) as u64;
    let chat_llm_timeout_ms = parse_usize_env("GITGOV_CHAT_LLM_TIMEOUT_MS", 9000) as u64;
    let stats_cache_ttl_ms = parse_usize_env("GITGOV_STATS_CACHE_TTL_MS", 3000) as u64;
    let logs_cache_ttl_ms = parse_usize_env("GITGOV_LOGS_CACHE_TTL_MS", 800) as u64;
    let org_lookup_cache_ttl_ms = parse_usize_env("GITGOV_ORG_LOOKUP_CACHE_TTL_MS", 30_000) as u64;
    let repo_lookup_cache_ttl_ms =
        parse_usize_env("GITGOV_REPO_LOOKUP_CACHE_TTL_MS", 30_000) as u64;
    let repo_upsert_min_interval_ms =
        parse_usize_env("GITGOV_REPO_UPSERT_MIN_INTERVAL_MS", 30_000) as u64;
    let cache_invalidation_min_interval_ms =
        parse_usize_env("GITGOV_CACHE_INVALIDATION_MIN_INTERVAL_MS", 120) as u64;
    let stats_cache_invalidation_min_interval_ms = parse_usize_env(
        "GITGOV_STATS_CACHE_INVALIDATION_MIN_INTERVAL_MS",
        cache_invalidation_min_interval_ms as usize,
    ) as u64;
    let logs_cache_invalidation_min_interval_ms = parse_usize_env(
        "GITGOV_LOGS_CACHE_INVALIDATION_MIN_INTERVAL_MS",
        cache_invalidation_min_interval_ms as usize,
    ) as u64;
    let logs_cache_stale_on_error_ms =
        parse_usize_env("GITGOV_LOGS_CACHE_STALE_ON_ERROR_MS", 5000) as u64;
    let client_session_upsert_min_interval_ms =
        parse_usize_env("GITGOV_CLIENT_SESSION_UPSERT_MIN_INTERVAL_MS", 15_000) as u64;
    let logs_reject_offset_pagination =
        parse_bool_env("GITGOV_LOGS_REJECT_OFFSET_PAGINATION", false);
    let outbox_server_lease_requested = parse_bool_env("GITGOV_OUTBOX_SERVER_LEASE_ENABLED", false);
    let outbox_server_lease_ttl_ms =
        parse_usize_env("GITGOV_OUTBOX_SERVER_LEASE_TTL_MS", 2_000).clamp(1_000, 60_000) as u64;
    let sse_distributed_enabled = parse_bool_env("GITGOV_SSE_DISTRIBUTED_ENABLED", false);
    let sse_distributed_channel = std::env::var("GITGOV_SSE_DISTRIBUTED_CHANNEL")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| SSE_DISTRIBUTED_CHANNEL_DEFAULT.to_string());
    let events_max_batch = parse_usize_env("GITGOV_EVENTS_MAX_BATCH", 1000);
    let policy_check_block_scopes =
        parse_policy_check_block_scopes_env("GITGOV_POLICY_CHECK_BLOCK_SCOPES");
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("failed to build HTTP client for notifications");
    let outbox_server_lease_enabled = if outbox_server_lease_requested {
        match db.ensure_outbox_lease_storage().await {
            Ok(()) => true,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Failed to initialize outbox lease storage; disabling server lease endpoint"
                );
                false
            }
        }
    } else {
        false
    };

    let state = Arc::new(AppState {
        db: Arc::clone(&db),
        github_webhook_secret,
        github_personal_access_token,
        jenkins_webhook_secret,
        jira_webhook_secret,
        start_time: Instant::now(),
        worker_id: worker_id_clone.clone(),
        http_client,
        alert_webhook_url,
        drift_alert_webhook_urls,
        strict_actor_match,
        reject_synthetic_logins,
        events_max_batch,
        llm_api_key,
        llm_model,
        feature_request_webhook_url,
        conversational_runtime: Arc::new(std::sync::Mutex::new(
            handlers::ConversationalRuntime::default(),
        )),
        chat_llm_semaphore: Arc::new(Semaphore::new(chat_llm_max_concurrency)),
        chat_llm_queue_timeout_ms,
        chat_llm_timeout_ms,
        stats_cache_ttl: Duration::from_millis(stats_cache_ttl_ms),
        stats_cache: Arc::new(Mutex::new(HashMap::new())),
        org_lookup_cache_ttl: Duration::from_millis(org_lookup_cache_ttl_ms),
        org_lookup_cache: Arc::new(Mutex::new(HashMap::new())),
        repo_lookup_cache_ttl: Duration::from_millis(repo_lookup_cache_ttl_ms),
        repo_lookup_cache: Arc::new(Mutex::new(HashMap::new())),
        repo_upsert_min_interval: Duration::from_millis(repo_upsert_min_interval_ms),
        repo_upsert_last_attempt: Arc::new(Mutex::new(HashMap::new())),
        cache_invalidation_min_interval: Duration::from_millis(cache_invalidation_min_interval_ms),
        stats_cache_invalidation_min_interval: Duration::from_millis(
            stats_cache_invalidation_min_interval_ms,
        ),
        logs_cache_invalidation_min_interval: Duration::from_millis(
            logs_cache_invalidation_min_interval_ms,
        ),
        stats_cache_last_invalidation_ms: Arc::new(AtomicI64::new(0)),
        stats_cache_refresh_lock: Arc::new(tokio::sync::Mutex::new(())),
        logs_cache_ttl: Duration::from_millis(logs_cache_ttl_ms),
        logs_cache_stale_on_error: Duration::from_millis(logs_cache_stale_on_error_ms),
        logs_reject_offset_pagination,
        outbox_server_lease_enabled,
        outbox_server_lease_ttl_ms,
        outbox_lease_telemetry: Arc::new(Mutex::new(handlers::OutboxLeaseTelemetry::default())),
        logs_cache: Arc::new(Mutex::new(HashMap::new())),
        logs_cache_last_invalidation_ms: Arc::new(AtomicI64::new(0)),
        client_session_upsert_min_interval: Duration::from_millis(
            client_session_upsert_min_interval_ms,
        ),
        client_session_last_upsert: Arc::new(Mutex::new(HashMap::new())),
        sse_tx: tokio::sync::broadcast::channel::<handlers::SseNotification>(64).0,
        sse_max_connections: Arc::new(Semaphore::new(
            parse_u32_env("GITGOV_SSE_MAX_CONNECTIONS", 50) as usize,
        )),
        sse_distributed_enabled,
        sse_distributed_channel: sse_distributed_channel.clone(),
        policy_check_block_scopes: policy_check_block_scopes.clone(),
    });

    if sse_distributed_enabled {
        spawn_distributed_sse_listener(Arc::clone(&state), database_url.clone());
    }

    if policy_check_block_scopes.is_empty() {
        tracing::info!("Policy check transport mode: advisory-only (default)");
    } else {
        let scopes = policy_check_block_scopes
            .iter()
            .map(|scope| format!("{}:{}", scope.org_pattern, scope.branch_pattern))
            .collect::<Vec<_>>()
            .join(", ");
        tracing::info!(
            scopes = %scopes,
            "Policy check transport mode: blocking for matching scopes"
        );
    }

    // Keep utility APIs exercised in non-test builds so strict linting does not
    // regress when these entry points are consumed by other binaries/tools.
    let _ = db::Database::get_github_events;
    let _ = db::Database::get_client_events;
    let _ = db::Database::reset_stale_jobs_safe;
    let _ = models::SignalType::as_str;
    let _ = models::SignalType::from_str;
    let _ = models::ConfidenceLevel::as_str;
    let _ = models::ConfidenceLevel::from_str;
    let _ = models::SignalStatus::as_str;
    let _ = models::SignalStatus::from_str;
    let _ = models::expand_login_aliases;

    let worker_id_for_log = worker_id_clone;

    // Audit trail retention policy (append-only tables are NOT deleted here).
    // Compliance guard: minimum 5 years for audit data.
    let configured_audit_retention_days =
        parse_i64_env("AUDIT_RETENTION_DAYS", MIN_AUDIT_RETENTION_DAYS);
    let effective_audit_retention_days =
        if configured_audit_retention_days < MIN_AUDIT_RETENTION_DAYS {
            tracing::warn!(
                configured = configured_audit_retention_days,
                min_days = MIN_AUDIT_RETENTION_DAYS,
                "AUDIT_RETENTION_DAYS below compliance minimum; clamping to min"
            );
            MIN_AUDIT_RETENTION_DAYS
        } else {
            configured_audit_retention_days
        };
    tracing::info!(
        audit_retention_days = effective_audit_retention_days,
        "Audit retention policy loaded (append-only audit tables)"
    );

    // TTL cleanup task — prunes stale client_sessions rows only.
    // Backward compatibility: DATA_RETENTION_DAYS still works as fallback.
    let db_ttl = Arc::clone(&db);
    let client_session_retention_days = parse_i64_env(
        "CLIENT_SESSION_RETENTION_DAYS",
        parse_i64_env("DATA_RETENTION_DAYS", 365),
    );
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(86_400)); // 24 h
        interval.tick().await; // skip first tick (fires immediately)
        loop {
            interval.tick().await;
            match db_ttl
                .delete_old_events(client_session_retention_days)
                .await
            {
                Ok(count) => tracing::info!(
                    deleted = count,
                    retention_days = client_session_retention_days,
                    "TTL cleanup: deleted stale client sessions"
                ),
                Err(e) => tracing::warn!(error = %e, "TTL cleanup failed"),
            }
        }
    });

    let admin_rate_limit_per_min = parse_u32_env("GITGOV_RATE_LIMIT_ADMIN_PER_MIN", 60);
    let logs_rate_limit_per_min =
        parse_u32_env("GITGOV_RATE_LIMIT_LOGS_PER_MIN", admin_rate_limit_per_min);
    let stats_rate_limit_per_min =
        parse_u32_env("GITGOV_RATE_LIMIT_STATS_PER_MIN", admin_rate_limit_per_min);
    let distributed_rate_limit_requested =
        parse_bool_env("GITGOV_RATE_LIMIT_DISTRIBUTED_DB", false);
    let distributed_rate_limit_enabled = if distributed_rate_limit_requested {
        match db.ensure_rate_limit_storage().await {
            Ok(()) => true,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Failed to initialize distributed rate limiter storage; falling back to in-memory limiter"
                );
                false
            }
        }
    } else {
        false
    };
    if distributed_rate_limit_enabled {
        let db_rate_limit_prune = Arc::clone(&db);
        let prune_interval_secs =
            parse_u32_env("GITGOV_RATE_LIMIT_DISTRIBUTED_PRUNE_INTERVAL_SECS", 300).max(30) as u64;
        let retention_secs =
            parse_u32_env("GITGOV_RATE_LIMIT_DISTRIBUTED_RETENTION_SECS", 3600).max(120) as u64;
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(prune_interval_secs));
            interval.tick().await;
            loop {
                interval.tick().await;
                match db_rate_limit_prune
                    .prune_rate_limit_counters(Duration::from_secs(retention_secs))
                    .await
                {
                    Ok(count) if count > 0 => {
                        tracing::debug!(
                            pruned = count,
                            retention_secs,
                            "Pruned distributed rate limiter counters"
                        );
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "Failed pruning distributed rate limiter counters"
                        );
                    }
                }
            }
        });
    }

    let make_rate_limiter = |name: &'static str, limit: u32, fail_open_on_internal_error: bool| {
        if distributed_rate_limit_enabled {
            Arc::new(RateLimiterState::DistributedDb(Arc::new(
                DistributedDbRateLimiter::new(
                    name,
                    limit,
                    Duration::from_secs(60),
                    fail_open_on_internal_error,
                    Arc::clone(&db),
                ),
            )))
        } else {
            Arc::new(RateLimiterState::InMemory(Arc::new(
                InMemoryRateLimiter::new(
                    name,
                    limit,
                    Duration::from_secs(60),
                    fail_open_on_internal_error,
                ),
            )))
        }
    };

    let events_rate_limit = make_rate_limiter(
        "events",
        parse_u32_env("GITGOV_RATE_LIMIT_EVENTS_PER_MIN", 240),
        true,
    );
    let audit_stream_rate_limit = make_rate_limiter(
        "audit_stream",
        parse_u32_env("GITGOV_RATE_LIMIT_AUDIT_STREAM_PER_MIN", 60),
        true,
    );
    let jenkins_rate_limit = make_rate_limiter(
        "jenkins_integrations",
        parse_u32_env("GITGOV_RATE_LIMIT_JENKINS_PER_MIN", 120),
        true,
    );
    let jira_rate_limit = make_rate_limiter(
        "jira_integrations",
        parse_u32_env("GITGOV_RATE_LIMIT_JIRA_PER_MIN", 120),
        true,
    );
    let github_webhook_rate_limit = make_rate_limiter(
        "github_webhook_public",
        parse_u32_env("GITGOV_RATE_LIMIT_GITHUB_WEBHOOK_PER_MIN", 240),
        true,
    );
    let org_invitation_public_rate_limit = make_rate_limiter(
        "org_invitation_public",
        parse_u32_env("GITGOV_RATE_LIMIT_ORG_INVITATION_PER_MIN", 90),
        true,
    );
    let admin_rate_limit = make_rate_limiter("admin_endpoints", admin_rate_limit_per_min, false);
    let logs_rate_limit = make_rate_limiter("logs_endpoints", logs_rate_limit_per_min, true);
    let stats_rate_limit = make_rate_limiter("stats_endpoints", stats_rate_limit_per_min, false);
    let chat_rate_limit = make_rate_limiter(
        "chat_endpoints",
        parse_u32_env("GITGOV_RATE_LIMIT_CHAT_PER_MIN", 40),
        true,
    );
    let events_body_limit_bytes = parse_usize_env("GITGOV_EVENTS_MAX_BODY_BYTES", 2 * 1024 * 1024);
    let jenkins_body_limit_bytes = parse_usize_env("GITGOV_JENKINS_MAX_BODY_BYTES", 256 * 1024);
    let jira_body_limit_bytes = parse_usize_env("GITGOV_JIRA_MAX_BODY_BYTES", 512 * 1024);
    let cors_allow_any = parse_bool_env("GITGOV_CORS_ALLOW_ANY", is_dev_env);
    let configured_cors_origins = std::env::var("GITGOV_CORS_ALLOW_ORIGINS").unwrap_or_default();
    let mut cors_origin_values = parse_cors_origins(&configured_cors_origins);
    if cors_origin_values.is_empty() && is_dev_env {
        cors_origin_values =
            parse_cors_origins("http://127.0.0.1:1420,http://localhost:1420,tauri://localhost");
    }
    if !cors_allow_any && cors_origin_values.is_empty() {
        tracing::error!(
            runtime_env = %runtime_env,
            "CORS strict mode enabled but no allowed origins configured; set GITGOV_CORS_ALLOW_ORIGINS"
        );
        std::process::exit(1);
    }
    let cors_origin_count = cors_origin_values.len();
    let cors_layer = if cors_allow_any {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(cors_origin_values))
            .allow_methods(Any)
            .allow_headers(Any)
    };

    tracing::info!(
        runtime_env = %runtime_env,
        rate_limit_mode = if distributed_rate_limit_enabled { "distributed_db" } else { "in_memory" },
        cors_allow_any,
        cors_origin_count = min(cors_origin_count, 50),
        events_per_min = events_rate_limit.limit(),
        audit_stream_per_min = audit_stream_rate_limit.limit(),
        jenkins_per_min = jenkins_rate_limit.limit(),
        jira_per_min = jira_rate_limit.limit(),
        github_webhook_public_per_min = github_webhook_rate_limit.limit(),
        org_invitation_public_per_min = org_invitation_public_rate_limit.limit(),
        events_body_limit_bytes,
        events_max_batch,
        jenkins_body_limit_bytes,
        jira_body_limit_bytes,
        admin_per_min = admin_rate_limit.limit(),
        logs_per_min = logs_rate_limit.limit(),
        stats_per_min = stats_rate_limit.limit(),
        chat_per_min = chat_rate_limit.limit(),
        chat_llm_max_concurrency,
        chat_llm_queue_timeout_ms,
        chat_llm_timeout_ms,
        stats_cache_ttl_ms,
        logs_cache_ttl_ms,
        org_lookup_cache_ttl_ms,
        repo_lookup_cache_ttl_ms,
        repo_upsert_min_interval_ms,
        cache_invalidation_min_interval_ms,
        stats_cache_invalidation_min_interval_ms,
        logs_cache_invalidation_min_interval_ms,
        logs_cache_stale_on_error_ms,
        client_session_upsert_min_interval_ms,
        logs_reject_offset_pagination,
        outbox_server_lease_enabled,
        outbox_server_lease_ttl_ms,
        sse_distributed_enabled,
        sse_distributed_channel = %sse_distributed_channel,
        "Rate limiting enabled for ingestion and control plane endpoints"
    );

    let app = routes::build_app(routes::RouteConfig {
        state: Arc::clone(&state),
        db: Arc::clone(&db),
        prometheus_handle,
        cors_layer,
        events_rate_limit,
        audit_stream_rate_limit,
        jenkins_rate_limit,
        jira_rate_limit,
        github_webhook_rate_limit,
        org_invitation_public_rate_limit,
        admin_rate_limit,
        logs_rate_limit,
        stats_rate_limit,
        chat_rate_limit,
        events_body_limit_bytes,
        jenkins_body_limit_bytes,
        jira_body_limit_bytes,
    });

    let addr: SocketAddr = std::env::var("GITGOV_SERVER_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
        .parse()
        .expect("Invalid server address");

    tracing::info!("GitGov Control Plane starting on {}", addr);
    tracing::info!("Using Supabase PostgreSQL database");
    tracing::info!("");
    tracing::info!(
        configured = state.alert_webhook_url.is_some(),
        "Generic alert webhook configured"
    );
    tracing::info!(
        drift_webhook_targets = state.drift_alert_webhook_urls.len(),
        "Dedicated drift alert webhook targets configured"
    );
    tracing::info!("");
    tracing::info!("Endpoints:");
    tracing::info!("  GET  /health                    - Health check (public)");
    tracing::info!("  GET  /health/detailed           - Detailed health (public)");
    tracing::info!("  POST /webhooks/github           - GitHub webhook (HMAC auth)");
    tracing::info!("  GET  /metrics                   - Prometheus metrics (public)");
    tracing::info!("  GET  /api-docs                  - Swagger UI (schema explorer, partial)");
    tracing::info!("  --- Authenticated endpoints ---");
    tracing::info!("  POST /events                    - Client events (auth)");
    tracing::info!("  POST /outbox/lease              - Outbox coordination lease (auth, opt-in)");
    tracing::info!("  GET  /outbox/lease/metrics      - Outbox lease telemetry (admin)");
    tracing::info!("  GET  /logs                      - Query events (auth, dev: own only)");
    tracing::info!("  GET  /pr-merges                 - PR merge evidence (admin)");
    tracing::info!("  GET  /stats                     - Statistics (admin)");
    tracing::info!("  GET  /stats/daily               - Daily commits/pushes series (admin)");
    tracing::info!("  GET  /dashboard                 - Dashboard (admin)");
    tracing::info!("  GET  /team/overview             - Team overview by developer (admin)");
    tracing::info!("  GET  /team/repos                - Team overview by repository (admin)");
    tracing::info!("  POST /integrations/jenkins      - Jenkins pipeline ingest (admin)");
    tracing::info!("  GET  /integrations/jenkins/status - Jenkins integration health (admin)");
    tracing::info!(
        "  GET  /integrations/jenkins/correlations - Commit->pipeline correlations (admin)"
    );
    tracing::info!("  GET  /integrations/correlations/v2 - Ticket->commit->pipeline view (admin)");
    tracing::info!("  POST /integrations/jira         - Jira webhook ingest (admin)");
    tracing::info!("  POST /webhooks/jira             - Jira signed webhook ingest (public, HMAC)");
    tracing::info!("  GET  /integrations/jira/status  - Jira integration health (admin)");
    tracing::info!("  GET  /integrations/jira/tickets/:ticket_id - Jira ticket detail (admin)");
    tracing::info!(
        "  POST /integrations/jira/correlate - Build commit<->ticket correlations (admin)"
    );
    tracing::info!("  GET  /integrations/jira/ticket-coverage - Ticket coverage metrics (admin)");
    tracing::info!("  GET  /compliance/:org           - Compliance (admin)");
    tracing::info!("  GET  /signals                   - Signals (auth)");
    tracing::info!("  POST /signals/:id               - Update signal (auth)");
    tracing::info!("  POST /signals/:id/confirm       - Confirm signal (admin)");
    tracing::info!("  POST /signals/detect/:org       - Trigger detection (admin)");
    tracing::info!("  --- Violation Decisions ---");
    tracing::info!("  GET  /violations/:id/decisions  - Get decision history (auth)");
    tracing::info!("  POST /violations/:id/decisions  - Add decision (admin)");
    tracing::info!("  GET  /policy/:repo              - Get policy (auth)");
    tracing::info!("  POST /policy/check              - Policy check (advisory + optional 409 block by scope, admin)");
    tracing::info!("  PUT  /policy/:repo/override     - Override policy (admin)");
    tracing::info!("  GET  /policy/:repo/history      - History (auth)");
    tracing::info!("  POST /policy/:repo/requests     - Create policy change request (auth)");
    tracing::info!("  GET  /policy/:repo/requests     - List policy change requests (auth)");
    tracing::info!("  POST /policy/requests/:id/approve - Approve policy change request (admin)");
    tracing::info!("  POST /policy/requests/:id/reject  - Reject policy change request (admin)");
    tracing::info!("  POST /policy/drift-events       - Ingest drift audit event (auth)");
    tracing::info!("  GET  /policy/drift-events       - List drift audit events (auth)");
    tracing::info!("  POST /export                    - Export (auth)");
    tracing::info!("  POST /orgs                      - Create/upsert org (admin)");
    tracing::info!("  POST /org-users                 - Create/update org user (admin)");
    tracing::info!("  GET  /org-users                 - List org users (admin)");
    tracing::info!("  PATCH /org-users/:id/status     - Activate/disable org user (admin)");
    tracing::info!("  POST /org-users/:id/api-key     - Issue API key for org user (admin)");
    tracing::info!("  POST /org-invitations           - Create org invitation (admin)");
    tracing::info!("  GET  /org-invitations           - List org invitations (admin)");
    tracing::info!("  POST /org-invitations/:id/resend - Regenerate invite token (admin)");
    tracing::info!("  POST /org-invitations/:id/revoke - Revoke invite (admin)");
    tracing::info!("  GET  /org-invitations/preview/:token - Preview invite (public)");
    tracing::info!("  POST /org-invitations/accept    - Accept invite and issue key (public)");
    tracing::info!("  POST /api-keys                  - Create API key (admin)");
    tracing::info!("  POST /audit-stream/github       - GitHub audit log stream (admin)");
    tracing::info!(
        "  (opt) JENKINS_WEBHOOK_SECRET    - Extra shared secret header x-gitgov-jenkins-secret"
    );
    tracing::info!("  (opt) JIRA_WEBHOOK_SECRET       - HMAC secret for signed Jira webhooks");
    tracing::info!(
        "  (opt) GITGOV_ALERT_WEBHOOK_URL  - Generic alert webhook (Slack/Discord/Teams)"
    );
    tracing::info!(
        "  (opt) GITGOV_DRIFT_ALERT_WEBHOOK_URLS - Dedicated drift alert webhooks (CSV)"
    );
    tracing::info!("  GET  /governance-events         - Query governance events (auth)");
    tracing::info!("  --- Job Queue Management ---");
    tracing::info!("  GET  /jobs/metrics              - Job queue metrics (admin)");
    tracing::info!("  GET  /jobs/dead                 - List dead jobs (admin)");
    tracing::info!("  POST /jobs/:id/retry            - Retry dead job (admin)");
    tracing::info!("");
    tracing::info!(
        worker_id = %worker_id_for_log,
        ttl_secs = JOB_WORKER_TTL_SECS,
        poll_interval_secs = JOB_POLL_INTERVAL_SECS,
        "Job worker configuration"
    );

    // TLS support: if GITGOV_TLS_CERT and GITGOV_TLS_KEY are set, serve HTTPS.
    let tls_cert = std::env::var("GITGOV_TLS_CERT").ok();
    let tls_key = std::env::var("GITGOV_TLS_KEY").ok();

    match (tls_cert, tls_key) {
        (Some(cert_path), Some(key_path)) => {
            let tls_config =
                axum_server::tls_rustls::RustlsConfig::from_pem_file(&cert_path, &key_path)
                    .await
                    .unwrap_or_else(|e| {
                        tracing::error!(
                            cert_path = %cert_path,
                            key_path = %key_path,
                            error = %e,
                            "Failed to load TLS certificate/key"
                        );
                        std::process::exit(1);
                    });

            tracing::info!(
                addr = %addr,
                cert = %cert_path,
                "Starting HTTPS server with TLS"
            );

            axum_server::bind_rustls(addr, tls_config)
                .serve(app.into_make_service())
                .await
                .unwrap_or_else(|e| {
                    tracing::error!("HTTPS server error: {}", e);
                });
        }
        (Some(_), None) | (None, Some(_)) => {
            tracing::error!(
                "Both GITGOV_TLS_CERT and GITGOV_TLS_KEY must be set for HTTPS. Only one was provided."
            );
            std::process::exit(1);
        }
        (None, None) => {
            if !is_dev_env {
                tracing::warn!(
                    "HTTPS is NOT enabled. Set GITGOV_TLS_CERT and GITGOV_TLS_KEY for production TLS. \
                     Use a reverse proxy (nginx/caddy) if terminating TLS externally."
                );
            }

            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to bind to address {}: {}", addr, e);
                    std::process::exit(1);
                });

            axum::serve(listener, app).await.unwrap_or_else(|e| {
                tracing::error!("Server error: {}", e);
            });
        }
    }
}
