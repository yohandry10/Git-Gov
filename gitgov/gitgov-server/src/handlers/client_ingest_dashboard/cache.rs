const GLOBAL_STATS_CACHE_KEY: &str = "__global__";
const MAX_STATS_CACHE_ENTRIES: usize = 256;
const MAX_LOGS_CACHE_ENTRIES: usize = 512;
const LOGS_OFFSET_DEPRECATION_NOTICE: &str =
    "The /logs `offset` query parameter is deprecated. Prefer keyset pagination with `before_created_at` and `before_id`.";

fn logs_deprecations_for_request(filter: &EventFilter) -> Option<Vec<String>> {
    (filter.offset > 0).then(|| vec![LOGS_OFFSET_DEPRECATION_NOTICE.to_string()])
}

fn should_reject_logs_offset(filter: &EventFilter, reject_offset_pagination: bool) -> bool {
    reject_offset_pagination && filter.offset > 0 && filter.before_created_at.is_none()
}

fn stats_cache_key(org_id: Option<&str>) -> String {
    org_id.unwrap_or(GLOBAL_STATS_CACHE_KEY).to_string()
}

fn get_cached_stats(state: &AppState, org_id: Option<&str>) -> Option<AuditStats> {
    if state.stats_cache_ttl.is_zero() {
        return None;
    }

    let now = Instant::now();
    let key = stats_cache_key(org_id);
    let mut cache = match state.stats_cache.lock() {
        Ok(guard) => guard,
        Err(_) => {
            tracing::warn!("Stats cache lock poisoned while reading; bypassing cache");
            return None;
        }
    };

    if let Some(entry) = cache.get(&key) {
        if entry.expires_at > now {
            return Some(entry.stats.clone());
        }
    }
    cache.remove(&key);
    None
}

fn put_cached_stats(state: &AppState, org_id: Option<&str>, stats: &AuditStats) {
    if state.stats_cache_ttl.is_zero() {
        return;
    }

    let key = stats_cache_key(org_id);
    let expires_at = Instant::now() + state.stats_cache_ttl;
    let mut cache = match state.stats_cache.lock() {
        Ok(guard) => guard,
        Err(_) => {
            tracing::warn!("Stats cache lock poisoned while writing; skipping cache write");
            return;
        }
    };

    cache.insert(
        key,
        StatsCacheEntry {
            stats: stats.clone(),
            expires_at,
        },
    );
    if cache.len() > MAX_STATS_CACHE_ENTRIES {
        cache.retain(|_, entry| entry.expires_at > Instant::now());
    }
}

fn invalidate_stats_cache(state: &AppState) {
    if state.stats_cache_ttl.is_zero() {
        return;
    }
    let min_interval = if state.stats_cache_invalidation_min_interval.is_zero() {
        state.cache_invalidation_min_interval
    } else {
        state.stats_cache_invalidation_min_interval
    };
    if !should_invalidate_cache(&state.stats_cache_last_invalidation_ms, min_interval) {
        return;
    }

    match state.stats_cache.lock() {
        Ok(mut cache) => {
            if !cache.is_empty() {
                cache.clear();
            }
        }
        Err(_) => {
            tracing::warn!("Stats cache lock poisoned while invalidating");
        }
    }
}

fn logs_cache_key(role: &UserRole, filter: &EventFilter) -> Option<String> {
    if filter.before_created_at.is_some() || filter.offset > 0 {
        // Cursor/offset pages are rarely repeated; avoid polluting cache.
        return None;
    }
    let role_scope = role.as_str();
    serde_json::to_string(filter)
        .ok()
        .map(|serialized| format!("{role_scope}|{serialized}"))
}

fn get_cached_logs(state: &AppState, key: &str) -> Option<Vec<CombinedEvent>> {
    if state.logs_cache_ttl.is_zero() {
        return None;
    }

    let now = Instant::now();
    let mut cache = match state.logs_cache.lock() {
        Ok(guard) => guard,
        Err(_) => {
            tracing::warn!("Logs cache lock poisoned while reading; bypassing cache");
            return None;
        }
    };

    if let Some(entry) = cache.get(key) {
        if entry.expires_at > now {
            return Some(entry.events.clone());
        }
    }
    cache.remove(key);
    None
}

fn get_cached_logs_on_error(state: &AppState, key: &str) -> Option<Vec<CombinedEvent>> {
    if state.logs_cache_ttl.is_zero() {
        return None;
    }
    let grace = state.logs_cache_stale_on_error;
    if grace.is_zero() {
        return None;
    }

    let now = Instant::now();
    let mut cache = match state.logs_cache.lock() {
        Ok(guard) => guard,
        Err(_) => {
            tracing::warn!("Logs cache lock poisoned while serving stale fallback");
            return None;
        }
    };

    if let Some(entry) = cache.get(key) {
        let stale_deadline = entry.expires_at + grace;
        if stale_deadline > now {
            return Some(entry.events.clone());
        }
    }

    cache.remove(key);
    None
}

fn put_cached_logs(state: &AppState, key: &str, events: &[CombinedEvent]) {
    if state.logs_cache_ttl.is_zero() {
        return;
    }

    let expires_at = Instant::now() + state.logs_cache_ttl;
    let mut cache = match state.logs_cache.lock() {
        Ok(guard) => guard,
        Err(_) => {
            tracing::warn!("Logs cache lock poisoned while writing; skipping cache write");
            return;
        }
    };

    cache.insert(
        key.to_string(),
        LogsCacheEntry {
            events: events.to_vec(),
            expires_at,
        },
    );

    if cache.len() > MAX_LOGS_CACHE_ENTRIES {
        cache.retain(|_, entry| entry.expires_at > Instant::now());
    }
}

fn invalidate_logs_cache(state: &AppState) {
    if state.logs_cache_ttl.is_zero() {
        return;
    }
    let min_interval = if state.logs_cache_invalidation_min_interval.is_zero() {
        state.cache_invalidation_min_interval
    } else {
        state.logs_cache_invalidation_min_interval
    };
    if !should_invalidate_cache(&state.logs_cache_last_invalidation_ms, min_interval) {
        return;
    }

    match state.logs_cache.lock() {
        Ok(mut cache) => {
            if !cache.is_empty() {
                cache.clear();
            }
        }
        Err(_) => {
            tracing::warn!("Logs cache lock poisoned while invalidating");
        }
    }
}

const MAX_ORG_LOOKUP_CACHE_ENTRIES: usize = 2_048;
const MAX_REPO_LOOKUP_CACHE_ENTRIES: usize = 8_192;

async fn resolve_org_id_with_cache(state: &AppState, org_name: &str) -> Option<String> {
    let cache_key = org_name.trim().to_ascii_lowercase();
    if cache_key.is_empty() {
        return None;
    }

    if let Some(cached) = get_cached_org_id(state, &cache_key) {
        return cached;
    }

    let resolved = state
        .db
        .get_org_by_login(org_name)
        .await
        .ok()
        .flatten()
        .map(|org| org.id);
    put_cached_org_id(state, cache_key, resolved.clone());
    resolved
}

fn get_cached_org_id(state: &AppState, cache_key: &str) -> Option<Option<String>> {
    if state.org_lookup_cache_ttl.is_zero() {
        return None;
    }

    let now = Instant::now();
    let mut cache = state.org_lookup_cache.lock().ok()?;
    let entry = cache.get(cache_key)?;
    if entry.expires_at <= now {
        cache.remove(cache_key);
        return None;
    }
    Some(entry.org_id.clone())
}

fn put_cached_org_id(state: &AppState, cache_key: String, org_id: Option<String>) {
    if state.org_lookup_cache_ttl.is_zero() {
        return;
    }

    let now = Instant::now();
    let mut cache = match state.org_lookup_cache.lock() {
        Ok(guard) => guard,
        Err(_) => {
            tracing::warn!("org_lookup_cache lock poisoned; skipping cache write");
            return;
        }
    };
    cache.insert(
        cache_key,
        OrgLookupCacheEntry {
            org_id,
            expires_at: now + state.org_lookup_cache_ttl,
        },
    );
    if cache.len() > MAX_ORG_LOOKUP_CACHE_ENTRIES {
        cache.retain(|_, entry| entry.expires_at > now);
    }
}

async fn resolve_repo_with_cache(state: &AppState, repo_full_name: &str) -> Option<Repo> {
    let cache_key = repo_full_name.trim().to_ascii_lowercase();
    if cache_key.is_empty() {
        return None;
    }

    if let Some(cached) = get_cached_repo(state, &cache_key) {
        return cached;
    }

    let resolved = state
        .db
        .get_repo_by_full_name(repo_full_name)
        .await
        .unwrap_or_default();
    put_cached_repo(state, cache_key, resolved.clone());
    resolved
}

fn get_cached_repo(state: &AppState, cache_key: &str) -> Option<Option<Repo>> {
    if state.repo_lookup_cache_ttl.is_zero() {
        return None;
    }

    let now = Instant::now();
    let mut cache = state.repo_lookup_cache.lock().ok()?;
    let entry = cache.get(cache_key)?;
    if entry.expires_at <= now {
        cache.remove(cache_key);
        return None;
    }
    Some(entry.repo.clone())
}

fn put_cached_repo(state: &AppState, cache_key: String, repo: Option<Repo>) {
    if state.repo_lookup_cache_ttl.is_zero() {
        return;
    }

    let now = Instant::now();
    let mut cache = match state.repo_lookup_cache.lock() {
        Ok(guard) => guard,
        Err(_) => {
            tracing::warn!("repo_lookup_cache lock poisoned; skipping cache write");
            return;
        }
    };
    cache.insert(
        cache_key,
        RepoLookupCacheEntry {
            repo,
            expires_at: now + state.repo_lookup_cache_ttl,
        },
    );
    if cache.len() > MAX_REPO_LOOKUP_CACHE_ENTRIES {
        cache.retain(|_, entry| entry.expires_at > now);
    }
}

const MAX_TRACKED_REPO_UPSERT_ATTEMPTS: usize = 8_192;

fn should_schedule_repo_upsert(state: &AppState, org_id: &str, repo_full_name: &str) -> bool {
    if state.repo_upsert_min_interval.is_zero() {
        return true;
    }
    let cache_key = format!(
        "{}:{}",
        org_id.trim(),
        repo_full_name.trim().to_ascii_lowercase()
    );
    if cache_key.ends_with(':') {
        return false;
    }

    let now = Instant::now();
    let mut cache = match state.repo_upsert_last_attempt.lock() {
        Ok(guard) => guard,
        Err(_) => {
            tracing::warn!("repo_upsert_last_attempt lock poisoned; bypassing debounce");
            return true;
        }
    };

    if let Some(last_attempt) = cache.get(&cache_key) {
        if now.saturating_duration_since(*last_attempt) < state.repo_upsert_min_interval {
            return false;
        }
    }
    cache.insert(cache_key, now);
    if cache.len() > MAX_TRACKED_REPO_UPSERT_ATTEMPTS {
        let stale_after = std::cmp::max(state.repo_upsert_min_interval, Duration::from_secs(120));
        cache.retain(|_, ts| now.saturating_duration_since(*ts) <= stale_after);
    }

    true
}

fn schedule_repo_upsert(
    state: Arc<AppState>,
    org_id: String,
    full_name: String,
    repo_name: String,
    event_uuid: String,
) {
    tokio::spawn(async move {
        match state
            .db
            .upsert_repo_by_full_name(Some(org_id.as_str()), &full_name, &repo_name, true)
            .await
        {
            Ok(repo_id) => {
                let repo = Repo {
                    id: repo_id,
                    org_id: Some(org_id),
                    github_id: None,
                    full_name: full_name.clone(),
                    name: repo_name,
                    private: true,
                    created_at: chrono::Utc::now().timestamp_millis(),
                };
                put_cached_repo(&state, full_name.to_ascii_lowercase(), Some(repo));
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    repo = %full_name,
                    event_uuid = %event_uuid,
                    "Background repo upsert from /events failed (non-fatal)"
                );
            }
        }
    });
}

fn should_invalidate_cache(last_invalidation_ms: &Arc<AtomicI64>, min_interval: Duration) -> bool {
    if min_interval.is_zero() {
        return true;
    }

    let min_interval_ms = min_interval.as_millis().min(i64::MAX as u128) as i64;
    let now_ms = chrono::Utc::now().timestamp_millis();
    loop {
        let previous_ms = last_invalidation_ms.load(Ordering::Acquire);
        if previous_ms > 0 && now_ms.saturating_sub(previous_ms) < min_interval_ms {
            return false;
        }
        if last_invalidation_ms
            .compare_exchange(previous_ms, now_ms, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            return true;
        }
    }
}

const MAX_TRACKED_CLIENT_SESSIONS: usize = 8_192;

fn should_upsert_client_session(state: &AppState, client_id: &str) -> bool {
    if state.client_session_upsert_min_interval.is_zero() {
        return true;
    }

    let now = Instant::now();
    let min_interval = state.client_session_upsert_min_interval;
    let mut cache = match state.client_session_last_upsert.lock() {
        Ok(guard) => guard,
        Err(_) => {
            tracing::warn!("client_session_last_upsert lock poisoned; bypassing debounce");
            return true;
        }
    };

    if let Some(last_seen) = cache.get(client_id) {
        if now.saturating_duration_since(*last_seen) < min_interval {
            return false;
        }
    }

    cache.insert(client_id.to_string(), now);
    if cache.len() > MAX_TRACKED_CLIENT_SESSIONS {
        let stale_after = std::cmp::max(min_interval, Duration::from_secs(120));
        cache.retain(|_, last_seen| now.saturating_duration_since(*last_seen) <= stale_after);
    }

    true
}
