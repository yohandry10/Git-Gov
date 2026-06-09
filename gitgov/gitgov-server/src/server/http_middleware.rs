use axum::{
    body::Body,
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
pub(crate) async fn security_headers(req: Request<Body>, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    // Modern browsers ignore X-XSS-Protection; value "0" disables legacy filter
    // to avoid introducing new vulnerabilities in older browsers.
    headers.insert("x-xss-protection", HeaderValue::from_static("0"));
    headers.insert(
        "referrer-policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    response
}

/// Middleware that records per-request HTTP duration and status as Prometheus
/// histograms/counters.  The path is normalized to avoid high-cardinality label
/// explosion (UUIDs and numeric IDs are replaced with placeholders).
pub(crate) async fn request_metrics_middleware(req: Request<Body>, next: Next) -> Response {
    let method = req.method().to_string();
    let raw_path = req.uri().path().to_string();
    let start = Instant::now();
    let response = next.run(req).await;
    let duration = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    // Normalize path to avoid high-cardinality labels (UUIDs, numeric IDs).
    let path = normalize_metrics_path(&raw_path);

    metrics::histogram!("gitgov_http_request_duration_seconds",
        "method" => method, "path" => path, "status" => status
    )
    .record(duration);

    response
}

/// Replace UUID segments and numeric IDs with `{id}` to keep label cardinality bounded.
fn normalize_metrics_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for segment in path.split('/') {
        out.push('/');
        if segment.is_empty() {
            continue;
        }
        // UUID-like: 8-4-4-4-12 hex chars
        let is_uuid =
            segment.len() == 36 && segment.chars().all(|c| c.is_ascii_hexdigit() || c == '-');
        // Pure numeric
        let is_numeric = !segment.is_empty() && segment.chars().all(|c| c.is_ascii_digit());
        if is_uuid || is_numeric {
            out.push_str("{id}");
        } else {
            out.push_str(segment);
        }
    }
    if out.is_empty() {
        "/".to_string()
    } else {
        out
    }
}
