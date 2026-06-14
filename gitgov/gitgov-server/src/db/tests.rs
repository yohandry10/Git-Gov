use super::*;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

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
    simulate_auth_db_failure: Option<String>,
    simulate_auth_db_failure_flag_file: Option<String>,
}

impl EnvGuard {
    fn apply(simulate_auth_db_failure: &str, flag_file: Option<&std::path::Path>) -> Self {
        let guard = Self {
            simulate_auth_db_failure: std::env::var(SIMULATE_AUTH_DB_FAILURE_ENV).ok(),
            simulate_auth_db_failure_flag_file: std::env::var(
                SIMULATE_AUTH_DB_FAILURE_FLAG_FILE_ENV,
            )
            .ok(),
        };
        set_env_var(SIMULATE_AUTH_DB_FAILURE_ENV, simulate_auth_db_failure);
        match flag_file {
            Some(path) => set_env_var(
                SIMULATE_AUTH_DB_FAILURE_FLAG_FILE_ENV,
                &path.as_os_str().to_string_lossy(),
            ),
            None => remove_env_var(SIMULATE_AUTH_DB_FAILURE_FLAG_FILE_ENV),
        }
        guard
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        set_or_clear_env(
            SIMULATE_AUTH_DB_FAILURE_ENV,
            self.simulate_auth_db_failure.as_deref(),
        );
        set_or_clear_env(
            SIMULATE_AUTH_DB_FAILURE_FLAG_FILE_ENV,
            self.simulate_auth_db_failure_flag_file.as_deref(),
        );
    }
}

struct TempFileGuard {
    path: std::path::PathBuf,
}

impl TempFileGuard {
    fn create() -> Self {
        let path = std::env::temp_dir().join(format!(
            "gitgov-auth-db-failpoint-{}.flag",
            uuid::Uuid::new_v4()
        ));
        std::fs::write(&path, b"1").expect("failed to create temp failpoint flag file");
        Self { path }
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[test]
fn auth_db_failure_simulation_enabled_reads_bool_env() {
    let _env_lock = env_lock().lock().expect("env lock poisoned");
    let _env_guard = EnvGuard::apply("true", None);
    assert!(auth_db_failure_simulation_enabled());
}

#[test]
fn auth_db_failure_simulation_enabled_reads_flag_file() {
    let _env_lock = env_lock().lock().expect("env lock poisoned");
    let flag = TempFileGuard::create();
    let _env_guard = EnvGuard::apply("false", Some(&flag.path));
    assert!(auth_db_failure_simulation_enabled());
}

fn build_test_db(
    auth_cache_ttl_secs: u64,
    auth_cache_stale_max_secs: u64,
    auth_stale_fail_closed_after: u32,
) -> Database {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://gitgov:gitgov@127.0.0.1/gitgov")
        .expect("failed to build lazy pg pool for auth cache tests");
    Database {
        pool,
        auth_cache: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        auth_cache_ttl: Duration::from_secs(auth_cache_ttl_secs),
        auth_cache_stale_max: Duration::from_secs(auth_cache_stale_max_secs),
        auth_cache_max_entries: 64,
        auth_db_failure_streak: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
        auth_stale_fail_closed_after,
    }
}

#[tokio::test]
async fn expired_fresh_cache_entry_remains_available_for_stale_lookup() {
    let db = build_test_db(1, 120, 0);
    db.put_cached_api_key_auth(
        "k",
        Some(ApiKeyAuthContext {
            client_id: "admin".to_string(),
            role: UserRole::Admin,
            org_id: Some("org1".to_string()),
            platform_principal_id: None,
            is_platform_founder: false,
            principal_type: "human".to_string(),
            scopes: Vec::new(),
            agent_key_id: None,
            agent_display_name: None,
        }),
    );

    {
        let mut cache = db.auth_cache.lock().expect("auth cache poisoned");
        let entry = cache.get_mut("k").expect("missing cached entry");
        entry.cached_at = Instant::now() - Duration::from_secs(2);
    }

    assert!(db.get_cached_api_key_auth("k").is_none());

    let stale = db
        .get_stale_cached_api_key_auth("k")
        .expect("stale auth cache should be available");
    assert_eq!(stale.0.client_id, "admin");
    assert!(stale.1 >= 1);
}

#[tokio::test]
async fn stale_cache_entry_older_than_max_age_is_evicted() {
    let db = build_test_db(1, 2, 0);
    db.put_cached_api_key_auth(
        "k",
        Some(ApiKeyAuthContext {
            client_id: "admin".to_string(),
            role: UserRole::Admin,
            org_id: Some("org1".to_string()),
            platform_principal_id: None,
            is_platform_founder: false,
            principal_type: "human".to_string(),
            scopes: Vec::new(),
            agent_key_id: None,
            agent_display_name: None,
        }),
    );

    {
        let mut cache = db.auth_cache.lock().expect("auth cache poisoned");
        let entry = cache.get_mut("k").expect("missing cached entry");
        entry.cached_at = Instant::now() - Duration::from_secs(3);
    }

    assert!(db.get_cached_api_key_auth("k").is_none());
    assert!(db.get_stale_cached_api_key_auth("k").is_none());
    let cache = db.auth_cache.lock().expect("auth cache poisoned");
    assert!(cache.get("k").is_none());
}

#[tokio::test]
async fn auth_db_failure_threshold_trips_fail_closed_mode() {
    let db = build_test_db(1, 120, 3);
    let (streak1, fail_closed1) = db.note_auth_db_failure();
    let (streak2, fail_closed2) = db.note_auth_db_failure();
    let (streak3, fail_closed3) = db.note_auth_db_failure();

    assert_eq!(streak1, 1);
    assert_eq!(streak2, 2);
    assert_eq!(streak3, 3);
    assert!(!fail_closed1);
    assert!(!fail_closed2);
    assert!(fail_closed3);
}

#[tokio::test]
async fn auth_db_failure_threshold_zero_keeps_stale_enabled() {
    let db = build_test_db(1, 120, 0);
    for _ in 0..5 {
        let (_, fail_closed) = db.note_auth_db_failure();
        assert!(!fail_closed);
    }
}

#[tokio::test]
async fn auth_db_failure_streak_resets_after_success_signal() {
    let db = build_test_db(1, 120, 2);
    let (_, fail_closed1) = db.note_auth_db_failure();
    assert!(!fail_closed1);

    db.reset_auth_db_failure_streak();

    let (streak_after_reset, fail_closed_after_reset) = db.note_auth_db_failure();
    assert_eq!(streak_after_reset, 1);
    assert!(!fail_closed_after_reset);
}
