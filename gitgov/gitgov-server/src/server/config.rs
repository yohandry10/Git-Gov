use axum::http::HeaderValue;
use clap::Parser;

use crate::handlers::PolicyCheckBlockingScope;
#[derive(Parser, Debug)]
#[command(name = "gitgov-server", about = "GitGov Control Plane")]
pub(crate) struct Args {
    #[arg(
        long,
        help = "Print bootstrap admin key to stdout (use for initial setup)"
    )]
    pub(crate) print_bootstrap_key: bool,
}

pub(crate) const JOB_WORKER_TTL_SECS: u64 = 300;
pub(crate) const JOB_POLL_INTERVAL_SECS: u64 = 5;
pub(crate) const JOB_ERROR_BACKOFF_SECS: u64 = 10;
pub(crate) const MIN_AUDIT_RETENTION_DAYS: i64 = 365 * 5;
pub(crate) const SIMULATE_RATE_LIMIT_INTERNAL_ERROR_ENV: &str =
    "GITGOV_SIMULATE_RATE_LIMIT_INTERNAL_ERROR";
pub(crate) const SIMULATE_RATE_LIMIT_INTERNAL_ERROR_FOR_ENV: &str =
    "GITGOV_SIMULATE_RATE_LIMIT_INTERNAL_ERROR_FOR";
pub(crate) const SSE_DISTRIBUTED_CHANNEL_DEFAULT: &str = "gitgov_sse_events";
pub(crate) const SSE_LISTENER_BACKOFF_START_SECS: u64 = 1;
pub(crate) const SSE_LISTENER_BACKOFF_MAX_SECS: u64 = 30;

pub(crate) fn parse_u32_env(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(default)
}

pub(crate) fn parse_usize_env(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(default)
}

pub(crate) fn parse_bool_env(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(default)
}

pub(crate) fn should_simulate_rate_limiter_internal_error(limiter_name: &str) -> bool {
    if !cfg!(debug_assertions) {
        return false;
    }
    if !parse_bool_env(SIMULATE_RATE_LIMIT_INTERNAL_ERROR_ENV, false) {
        return false;
    }

    let raw_targets = std::env::var(SIMULATE_RATE_LIMIT_INTERNAL_ERROR_FOR_ENV).unwrap_or_default();
    let trimmed = raw_targets.trim();
    if trimmed.is_empty() {
        return true;
    }

    raw_targets.split(',').any(|item| {
        let target = item.trim();
        !target.is_empty() && (target.eq_ignore_ascii_case("all") || target == limiter_name)
    })
}

pub(crate) fn parse_i64_env(key: &str, default: i64) -> i64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(default)
}

pub(crate) fn parse_csv_env(key: &str) -> Vec<String> {
    std::env::var(key)
        .ok()
        .map(|raw| {
            raw.split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub(crate) fn parse_policy_check_block_scopes_env(key: &str) -> Vec<PolicyCheckBlockingScope> {
    std::env::var(key)
        .ok()
        .map(|raw| {
            raw.split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .filter_map(|entry| {
                    let mut parts = entry.splitn(2, ':');
                    let org_pattern = parts
                        .next()
                        .map(str::trim)
                        .filter(|v| !v.is_empty())
                        .map(ToOwned::to_owned)?;
                    let branch_pattern = parts
                        .next()
                        .map(str::trim)
                        .filter(|v| !v.is_empty())
                        .unwrap_or("*")
                        .to_string();
                    Some(PolicyCheckBlockingScope::new(org_pattern, branch_pattern))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub(crate) fn parse_runtime_env() -> (String, bool, bool) {
    let runtime_env_explicit = std::env::var("GITGOV_ENV").is_ok();
    let default_env = if cfg!(debug_assertions) {
        "dev"
    } else {
        "prod"
    };
    let runtime_env = std::env::var("GITGOV_ENV")
        .unwrap_or_else(|_| default_env.to_string())
        .trim()
        .to_ascii_lowercase();
    let is_dev_env = matches!(
        runtime_env.as_str(),
        "dev" | "development" | "local" | "test"
    );
    (runtime_env, is_dev_env, runtime_env_explicit)
}

pub(crate) fn parse_cors_origins(input: &str) -> Vec<HeaderValue> {
    input
        .split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect()
}
