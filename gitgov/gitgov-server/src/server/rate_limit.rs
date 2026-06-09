use axum::{
    body::Body,
    extract::State,
    http::{header::RETRY_AFTER, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use sha2::Digest;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::server::config::should_simulate_rate_limiter_internal_error;
use crate::{auth, db};
pub(crate) struct RateBucket {
    pub(crate) window_start: Instant,
    pub(crate) count: u32,
}

#[derive(Clone)]
pub(crate) struct InMemoryRateLimiter {
    name: &'static str,
    limit: u32,
    window: Duration,
    fail_open_on_lock_poison: bool,
    pub(crate) buckets: Arc<Mutex<HashMap<String, RateBucket>>>,
}

#[derive(Debug)]
pub(crate) struct RateLimitDecision {
    pub(crate) allowed: bool,
    pub(crate) retry_after_secs: u64,
    pub(crate) internal_error: bool,
}

#[derive(Clone)]
pub(crate) struct DistributedDbRateLimiter {
    name: &'static str,
    limit: u32,
    window: Duration,
    fail_open_on_db_error: bool,
    db: Arc<db::Database>,
    pub(crate) denied_until_cache: Arc<Mutex<HashMap<String, Instant>>>,
    pub(crate) denied_until_cache_max_entries: usize,
}

#[derive(Clone)]
pub(crate) enum RateLimiterState {
    InMemory(Arc<InMemoryRateLimiter>),
    DistributedDb(Arc<DistributedDbRateLimiter>),
}

impl InMemoryRateLimiter {
    pub(crate) fn new(
        name: &'static str,
        limit: u32,
        window: Duration,
        fail_open_on_lock_poison: bool,
    ) -> Self {
        Self {
            name,
            limit,
            window,
            fail_open_on_lock_poison,
            buckets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub(crate) fn check(&self, key: &str) -> RateLimitDecision {
        if self.limit == 0 {
            return RateLimitDecision {
                allowed: true,
                retry_after_secs: 0,
                internal_error: false,
            };
        }

        if should_simulate_rate_limiter_internal_error(self.name) {
            if self.fail_open_on_lock_poison {
                tracing::warn!(
                    limiter = self.name,
                    "Simulating rate limiter internal error (debug failpoint, fail-open)"
                );
                return RateLimitDecision {
                    allowed: true,
                    retry_after_secs: 0,
                    internal_error: false,
                };
            }
            tracing::warn!(
                limiter = self.name,
                "Simulating rate limiter internal error (debug failpoint, fail-closed)"
            );
            return RateLimitDecision {
                allowed: false,
                retry_after_secs: 1,
                internal_error: true,
            };
        }

        let now = Instant::now();
        let mut buckets = match self.buckets.lock() {
            Ok(guard) => guard,
            Err(_) => {
                let mode = if self.fail_open_on_lock_poison {
                    "fail-open"
                } else {
                    "fail-closed"
                };
                tracing::warn!(limiter = self.name, mode, "Rate limiter lock poisoned");
                if self.fail_open_on_lock_poison {
                    return RateLimitDecision {
                        allowed: true,
                        retry_after_secs: 0,
                        internal_error: false,
                    };
                }
                return RateLimitDecision {
                    allowed: false,
                    retry_after_secs: 1,
                    internal_error: true,
                };
            }
        };

        // Opportunistic cleanup to prevent unbounded growth.
        if buckets.len() > 10_000 {
            let stale_after = self.window + self.window;
            buckets.retain(|_, bucket| now.duration_since(bucket.window_start) <= stale_after);
        }

        let bucket = buckets.entry(key.to_string()).or_insert(RateBucket {
            window_start: now,
            count: 0,
        });

        if now.duration_since(bucket.window_start) >= self.window {
            bucket.window_start = now;
            bucket.count = 0;
        }

        if bucket.count >= self.limit {
            let elapsed = now.duration_since(bucket.window_start);
            let retry_after = self.window.saturating_sub(elapsed).as_secs().max(1);
            return RateLimitDecision {
                allowed: false,
                retry_after_secs: retry_after,
                internal_error: false,
            };
        }

        bucket.count += 1;
        RateLimitDecision {
            allowed: true,
            retry_after_secs: 0,
            internal_error: false,
        }
    }
}

impl DistributedDbRateLimiter {
    pub(crate) fn new(
        name: &'static str,
        limit: u32,
        window: Duration,
        fail_open_on_db_error: bool,
        db: Arc<db::Database>,
    ) -> Self {
        Self {
            name,
            limit,
            window,
            fail_open_on_db_error,
            db,
            denied_until_cache: Arc::new(Mutex::new(HashMap::new())),
            denied_until_cache_max_entries: 16_384,
        }
    }

    pub(crate) async fn check(&self, key: &str) -> RateLimitDecision {
        if self.limit == 0 {
            return RateLimitDecision {
                allowed: true,
                retry_after_secs: 0,
                internal_error: false,
            };
        }

        if should_simulate_rate_limiter_internal_error(self.name) {
            if self.fail_open_on_db_error {
                tracing::warn!(
                    limiter = self.name,
                    "Simulating distributed rate limiter internal error (debug failpoint, fail-open)"
                );
                return RateLimitDecision {
                    allowed: true,
                    retry_after_secs: 0,
                    internal_error: false,
                };
            }
            tracing::warn!(
                limiter = self.name,
                "Simulating distributed rate limiter internal error (debug failpoint, fail-closed)"
            );
            return RateLimitDecision {
                allowed: false,
                retry_after_secs: 1,
                internal_error: true,
            };
        }

        if let Some(retry_after_secs) = get_cached_denied_retry_secs(&self.denied_until_cache, key)
        {
            return RateLimitDecision {
                allowed: false,
                retry_after_secs,
                internal_error: false,
            };
        }

        match self
            .db
            .check_distributed_rate_limit(self.name, key, self.limit, self.window)
            .await
        {
            Ok(result) => {
                if result.allowed {
                    clear_cached_denied_key(&self.denied_until_cache, key);
                } else {
                    put_cached_denied_retry_secs(
                        &self.denied_until_cache,
                        key,
                        result.retry_after_secs,
                        self.denied_until_cache_max_entries,
                    );
                }
                RateLimitDecision {
                    allowed: result.allowed,
                    retry_after_secs: result.retry_after_secs,
                    internal_error: false,
                }
            }
            Err(e) => {
                let mode = if self.fail_open_on_db_error {
                    "fail-open"
                } else {
                    "fail-closed"
                };
                tracing::warn!(
                    limiter = self.name,
                    mode,
                    error = %e,
                    "Distributed rate limiter DB check failed"
                );
                if self.fail_open_on_db_error {
                    RateLimitDecision {
                        allowed: true,
                        retry_after_secs: 0,
                        internal_error: false,
                    }
                } else {
                    RateLimitDecision {
                        allowed: false,
                        retry_after_secs: 1,
                        internal_error: true,
                    }
                }
            }
        }
    }
}

pub(crate) fn get_cached_denied_retry_secs(
    cache: &Mutex<HashMap<String, Instant>>,
    key: &str,
) -> Option<u64> {
    let now = Instant::now();
    let mut guard = cache.lock().ok()?;
    let denied_until = guard.get(key).copied()?;
    if denied_until <= now {
        guard.remove(key);
        return None;
    }
    Some(denied_until.duration_since(now).as_secs().max(1))
}

pub(crate) fn put_cached_denied_retry_secs(
    cache: &Mutex<HashMap<String, Instant>>,
    key: &str,
    retry_after_secs: u64,
    max_entries: usize,
) {
    if retry_after_secs == 0 {
        return;
    }
    let denied_until = Instant::now() + Duration::from_secs(retry_after_secs.max(1));
    let mut guard = match cache.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    guard.insert(key.to_string(), denied_until);
    if guard.len() > max_entries {
        let now = Instant::now();
        guard.retain(|_, expires_at| *expires_at > now);
        if guard.len() > max_entries {
            let overflow = guard.len().saturating_sub(max_entries);
            let stale_keys = guard.keys().take(overflow).cloned().collect::<Vec<_>>();
            for stale_key in stale_keys {
                guard.remove(&stale_key);
            }
        }
    }
}

pub(crate) fn clear_cached_denied_key(cache: &Mutex<HashMap<String, Instant>>, key: &str) {
    if let Ok(mut guard) = cache.lock() {
        guard.remove(key);
    }
}

impl RateLimiterState {
    pub(crate) async fn check(&self, key: &str) -> RateLimitDecision {
        match self {
            Self::InMemory(limiter) => limiter.check(key),
            Self::DistributedDb(limiter) => limiter.check(key).await,
        }
    }

    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::InMemory(limiter) => limiter.name,
            Self::DistributedDb(limiter) => limiter.name,
        }
    }

    pub(crate) fn limit(&self) -> u32 {
        match self {
            Self::InMemory(limiter) => limiter.limit,
            Self::DistributedDb(limiter) => limiter.limit,
        }
    }
}

/// Build rate-limit key from the authenticated user identity.
/// Priority: authenticated identity (scoped by org when available) > auth token hash + IP.
/// This keeps authenticated rate limiting stable across IP changes and avoids
/// cross-tenant collisions when different orgs share the same login string.
pub(crate) fn rate_limit_key_from_request(req: &Request<Body>) -> String {
    // If auth middleware has already run, use the authenticated user identity.
    // For multi-tenant isolation, scope by org_id when available.
    if let Some(auth_user) = req.extensions().get::<auth::AuthUser>() {
        if let Some(org_id) = auth_user
            .org_id
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            return format!("org:{}:user:{}", org_id, auth_user.client_id);
        }
        return format!("user:{}", auth_user.client_id);
    }

    // Fallback for unauthenticated routes: IP + token hash (original behavior)
    let headers = req.headers();
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .map(str::trim)
                .filter(|v| !v.is_empty())
        })
        .unwrap_or("unknown");

    let auth_fingerprint = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .map(|auth| {
            let digest = sha2::Sha256::digest(auth.as_bytes());
            format!("{:x}", digest)[..12].to_string()
        })
        .unwrap_or_else(|| "noauth".to_string());

    format!("{}:{}", ip, auth_fingerprint)
}

pub(crate) async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiterState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let key = rate_limit_key_from_request(&req);
    let decision = limiter.check(&key).await;

    if decision.allowed {
        return next.run(req).await;
    }

    metrics::counter!("gitgov_rate_limited_total", "limiter" => limiter.name().to_string())
        .increment(1);

    if decision.internal_error {
        tracing::error!(
            limiter = limiter.name(),
            key = %key,
            "Rate limiter unavailable (internal error)"
        );
        let mut response = (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(serde_json::json!({
                "error": "Rate limiter temporarily unavailable",
                "code": "RATE_LIMITER_UNAVAILABLE",
                "retry_after_seconds": decision.retry_after_secs
            })),
        )
            .into_response();
        if let Ok(value) = HeaderValue::from_str(&decision.retry_after_secs.to_string()) {
            response.headers_mut().insert(RETRY_AFTER, value);
        }
        return response;
    }

    tracing::warn!(
        limiter = limiter.name(),
        key = %key,
        retry_after_secs = decision.retry_after_secs,
        "Request rate limited"
    );

    let mut response = (
        StatusCode::TOO_MANY_REQUESTS,
        axum::Json(serde_json::json!({
            "error": "Too many requests",
            "code": "RATE_LIMITED",
            "retry_after_seconds": decision.retry_after_secs
        })),
    )
        .into_response();

    if let Ok(value) = HeaderValue::from_str(&decision.retry_after_secs.to_string()) {
        response.headers_mut().insert(RETRY_AFTER, value);
    }

    response
}
