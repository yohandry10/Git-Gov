use super::config::{
    should_simulate_rate_limiter_internal_error, SIMULATE_RATE_LIMIT_INTERNAL_ERROR_ENV,
    SIMULATE_RATE_LIMIT_INTERNAL_ERROR_FOR_ENV,
};
use super::rate_limit::{
    get_cached_denied_retry_secs, put_cached_denied_retry_secs, rate_limit_key_from_request,
    InMemoryRateLimiter,
};
use axum::{
    body::Body,
    http::{HeaderValue, Request, StatusCode},
    middleware,
    middleware::Next,
    response::Response,
    routing::get,
    Router,
};
use sha2::Digest;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tower::ServiceExt;

use crate::auth;

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn set_env_var(key: &str, value: &str) {
    #[allow(unused_unsafe)]
    unsafe {
        std::env::set_var(key, value);
    }
}

fn remove_env_var(key: &str) {
    #[allow(unused_unsafe)]
    unsafe {
        std::env::remove_var(key);
    }
}

fn set_or_clear_env(key: &str, value: Option<&str>) {
    match value {
        Some(v) => set_env_var(key, v),
        None => remove_env_var(key),
    }
}

struct EnvGuard {
    simulate_internal_error: Option<String>,
    simulate_internal_error_for: Option<String>,
}

impl EnvGuard {
    fn apply(simulate_internal_error: &str, simulate_internal_error_for: Option<&str>) -> Self {
        let guard = Self {
            simulate_internal_error: std::env::var(SIMULATE_RATE_LIMIT_INTERNAL_ERROR_ENV).ok(),
            simulate_internal_error_for: std::env::var(SIMULATE_RATE_LIMIT_INTERNAL_ERROR_FOR_ENV)
                .ok(),
        };
        set_env_var(
            SIMULATE_RATE_LIMIT_INTERNAL_ERROR_ENV,
            simulate_internal_error,
        );
        match simulate_internal_error_for {
            Some(value) => set_env_var(SIMULATE_RATE_LIMIT_INTERNAL_ERROR_FOR_ENV, value),
            None => remove_env_var(SIMULATE_RATE_LIMIT_INTERNAL_ERROR_FOR_ENV),
        }
        guard
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        set_or_clear_env(
            SIMULATE_RATE_LIMIT_INTERNAL_ERROR_ENV,
            self.simulate_internal_error.as_deref(),
        );
        set_or_clear_env(
            SIMULATE_RATE_LIMIT_INTERNAL_ERROR_FOR_ENV,
            self.simulate_internal_error_for.as_deref(),
        );
    }
}

fn poison_limiter_lock(limiter: &InMemoryRateLimiter) {
    let buckets = Arc::clone(&limiter.buckets);
    let _ = std::thread::spawn(move || {
        let _guard = buckets.lock().expect("lock buckets");
        panic!("intentional poison for test");
    })
    .join();
}

#[test]
fn rate_limiter_fail_open_allows_when_lock_is_poisoned() {
    let limiter = InMemoryRateLimiter::new("test_fail_open", 10, Duration::from_secs(60), true);
    poison_limiter_lock(&limiter);

    let decision = limiter.check("k");
    assert!(decision.allowed);
    assert!(!decision.internal_error);
}

#[test]
fn rate_limiter_fail_closed_blocks_when_lock_is_poisoned() {
    let limiter = InMemoryRateLimiter::new("test_fail_closed", 10, Duration::from_secs(60), false);
    poison_limiter_lock(&limiter);

    let decision = limiter.check("k");
    assert!(!decision.allowed);
    assert!(decision.internal_error);
    assert_eq!(decision.retry_after_secs, 1);
}

#[test]
fn rate_limiter_failpoint_applies_to_selected_limiter() {
    let _env_lock = env_lock().lock().expect("env lock poisoned");
    let _env_guard = EnvGuard::apply("true", Some("admin_endpoints"));

    assert!(should_simulate_rate_limiter_internal_error(
        "admin_endpoints"
    ));
    assert!(!should_simulate_rate_limiter_internal_error("events"));
}

#[test]
fn rate_limiter_failpoint_fail_closed_returns_internal_error() {
    let _env_lock = env_lock().lock().expect("env lock poisoned");
    let _env_guard = EnvGuard::apply("true", Some("admin_endpoints"));

    let limiter = InMemoryRateLimiter::new("admin_endpoints", 10, Duration::from_secs(60), false);
    let decision = limiter.check("k");
    assert!(!decision.allowed);
    assert!(decision.internal_error);
    assert_eq!(decision.retry_after_secs, 1);
}

#[test]
fn distributed_denied_cache_returns_retry_window() {
    let cache = Mutex::new(HashMap::new());
    put_cached_denied_retry_secs(&cache, "k1", 3, 128);
    let retry_after = get_cached_denied_retry_secs(&cache, "k1").expect("expected cached deny");
    assert!((1..=3).contains(&retry_after));
}

#[test]
fn distributed_denied_cache_evicted_when_expired() {
    let cache = Mutex::new(HashMap::new());
    {
        let mut guard = cache.lock().expect("cache lock");
        guard.insert("k2".to_string(), Instant::now() - Duration::from_secs(1));
    }
    assert!(get_cached_denied_retry_secs(&cache, "k2").is_none());
    let guard = cache.lock().expect("cache lock");
    assert!(!guard.contains_key("k2"));
}

#[test]
fn rate_limit_key_prefers_authenticated_identity_scoped_by_org() {
    let mut req = Request::builder()
        .uri("/stats")
        .body(Body::empty())
        .expect("request");
    req.extensions_mut().insert(auth::AuthUser {
        client_id: "andres".to_string(),
        role: crate::models::UserRole::Admin,
        org_id: Some("org-123".to_string()),
    });

    let key = rate_limit_key_from_request(&req);
    assert_eq!(key, "org:org-123:user:andres");
}

#[test]
fn rate_limit_key_uses_client_identity_when_org_missing() {
    let mut req = Request::builder()
        .uri("/stats")
        .body(Body::empty())
        .expect("request");
    req.extensions_mut().insert(auth::AuthUser {
        client_id: "andres".to_string(),
        role: crate::models::UserRole::Developer,
        org_id: None,
    });

    let key = rate_limit_key_from_request(&req);
    assert_eq!(key, "user:andres");
}

#[test]
fn rate_limit_key_fallback_matches_ip_and_auth_fingerprint() {
    let req = Request::builder()
        .uri("/health")
        .header("x-real-ip", "10.20.30.40")
        .header("authorization", "Bearer test-token")
        .body(Body::empty())
        .expect("request");

    let digest = sha2::Sha256::digest("Bearer test-token".as_bytes());
    let expected_fingerprint = format!("{:x}", digest)[..12].to_string();
    let key = rate_limit_key_from_request(&req);
    assert_eq!(key, format!("10.20.30.40:{}", expected_fingerprint));
}

async fn inject_test_auth(mut req: Request<Body>, next: Next) -> Response {
    req.extensions_mut().insert(auth::AuthUser {
        client_id: "test-user".to_string(),
        role: crate::models::UserRole::Admin,
        org_id: Some("test-org".to_string()),
    });
    next.run(req).await
}

async fn attach_rate_limit_key_header(req: Request<Body>, next: Next) -> Response {
    let key = rate_limit_key_from_request(&req);
    let mut response = next.run(req).await;
    let value = HeaderValue::from_str(&key).expect("valid header value");
    response.headers_mut().insert("x-rate-limit-key", value);
    response
}

#[tokio::test]
async fn auth_layer_populates_identity_before_route_level_rate_limit_key() {
    let app = Router::new()
        .route(
            "/probe",
            get(|| async { StatusCode::OK })
                .layer(middleware::from_fn(attach_rate_limit_key_header)),
        )
        .layer(middleware::from_fn(inject_test_auth));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/probe")
                .method("GET")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    let key = response
        .headers()
        .get("x-rate-limit-key")
        .and_then(|h| h.to_str().ok())
        .expect("x-rate-limit-key header");
    assert_eq!(key, "org:test-org:user:test-user");
}
