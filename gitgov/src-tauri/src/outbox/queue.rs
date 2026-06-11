use crate::models::{AuditAction, AuditStatus};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

const OUTBOX_BATCH_SIZE: usize = 100;
const RETRY_BASE_DELAY_MS: i64 = 1_000;
const RETRY_MAX_DELAY_MS: i64 = 60_000;
const RETRY_RATE_LIMIT_FLOOR_MS: i64 = 5_000;
const RETRY_CLIENT_ERROR_FLOOR_MS: i64 = 30_000;
const DEFAULT_FLUSH_INTERVAL_JITTER_MAX_MS: u64 = 5_000;
const DEFAULT_GLOBAL_COORD_ENABLED: bool = false;
const DEFAULT_GLOBAL_COORD_WINDOW_MS: u64 = 20_000;
const DEFAULT_GLOBAL_COORD_MAX_DEFERRAL_MS: u64 = 1_600;
const DEFAULT_SERVER_LEASE_ENABLED: bool = false;
const DEFAULT_SERVER_LEASE_TTL_MS: u64 = 2_000;
const DEFAULT_SERVER_LEASE_SCOPE: &str = "global";
const DEAD_LETTER_REASON_MAX_CHARS: usize = 240;

#[derive(Debug, Error)]
pub enum OutboxError {
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
}

#[derive(Debug, Clone, Copy)]
enum RetryClass {
    Network,
    Http5xx,
    RateLimited,
    HttpOther,
}

#[derive(Debug, Clone, Copy)]
struct RetryDirective {
    class: RetryClass,
    retry_after_ms: Option<i64>,
    status_code: Option<u16>,
}

impl RetryDirective {
    fn default_backoff() -> Self {
        Self {
            class: RetryClass::Network,
            retry_after_ms: None,
            status_code: None,
        }
    }

    fn rate_limited(retry_after_ms: Option<i64>, status_code: u16) -> Self {
        Self {
            class: RetryClass::RateLimited,
            retry_after_ms,
            status_code: Some(status_code),
        }
    }

    fn http_5xx(status_code: u16) -> Self {
        Self {
            class: RetryClass::Http5xx,
            retry_after_ms: None,
            status_code: Some(status_code),
        }
    }

    fn http_other(status_code: u16) -> Self {
        Self {
            class: RetryClass::HttpOther,
            retry_after_ms: None,
            status_code: Some(status_code),
        }
    }

    fn network() -> Self {
        Self {
            class: RetryClass::Network,
            retry_after_ms: None,
            status_code: None,
        }
    }

    fn class_name(&self) -> &'static str {
        match self.class {
            RetryClass::Network => "network",
            RetryClass::Http5xx => "http_5xx",
            RetryClass::RateLimited => "http_429",
            RetryClass::HttpOther => "http_other",
        }
    }
}

#[derive(Debug)]
struct SendBatchFailure {
    error: OutboxError,
    retry: RetryDirective,
}

#[derive(Debug, Clone, Copy)]
struct ServerLeaseDecision {
    granted: bool,
    wait_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEvent {
    pub event_uuid: String,
    pub event_type: String,
    pub user_login: String,
    pub user_name: Option<String>,
    pub branch: Option<String>,
    pub commit_sha: Option<String>,
    pub files: Vec<String>,
    pub status: String,
    pub reason: Option<String>,
    pub repo_full_name: Option<String>,
    pub org_name: Option<String>,
    pub timestamp: i64,
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub sent: bool,
    #[serde(default)]
    pub attempts: u32,
    #[serde(default)]
    pub last_attempt: Option<i64>,
    #[serde(default)]
    pub next_attempt_at: Option<i64>,
    #[serde(default)]
    pub dead_lettered: bool,
    #[serde(default)]
    pub dead_lettered_at: Option<i64>,
    #[serde(default)]
    pub dead_letter_reason: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
}

impl OutboxEvent {
    pub fn new(
        event_type: String,
        user_login: String,
        branch: Option<String>,
        status: AuditStatus,
    ) -> Self {
        Self {
            event_uuid: Uuid::new_v4().to_string(),
            event_type,
            user_login,
            user_name: None,
            branch,
            commit_sha: None,
            files: vec![],
            status: status.as_str().to_string(),
            reason: None,
            repo_full_name: None,
            org_name: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
            metadata: None,
            sent: false,
            attempts: 0,
            last_attempt: None,
            next_attempt_at: None,
            dead_lettered: false,
            dead_lettered_at: None,
            dead_letter_reason: None,
            last_error: None,
        }
    }

    pub fn from_audit_action(action: &AuditAction) -> String {
        match action {
            AuditAction::Push => "successful_push",
            AuditAction::BranchCreate => "create_branch",
            AuditAction::StageFile => "stage_files",
            AuditAction::Commit => "commit",
            AuditAction::BlockedPush => "blocked_push",
            AuditAction::BlockedBranch => "blocked_branch",
        }
        .to_string()
    }

    pub fn with_user_name(mut self, name: String) -> Self {
        self.user_name = Some(name);
        self
    }

    pub fn with_commit_sha(mut self, sha: String) -> Self {
        self.commit_sha = Some(sha);
        self
    }

    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.files = files;
        self
    }

    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }

    pub fn with_repo(mut self, full_name: String) -> Self {
        self.repo_full_name = Some(full_name);
        self
    }

    pub fn with_org(mut self, org: String) -> Self {
        self.org_name = Some(org);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Worker control for clean shutdown
struct WorkerControl {
    shutdown: AtomicBool,
    trigger: Condvar,
}

/// Outbox implementation with CRASH-SAFE persistence and NO nested locks.
///
/// CRASH-SAFETY: Uses atomic rename pattern:
///   1. Write complete file to .tmp
///   2. Sync to disk
///   3. Atomic rename to final location
///
/// On crash: Either old file exists (complete) or new file exists (complete)
///
/// LOCK DISCIPLINE: Never hold two locks simultaneously.
/// Pattern: snapshot → release → work → update → release → persist
pub struct Outbox {
    path: PathBuf,
    events: Arc<Mutex<Vec<OutboxEvent>>>,
    file_lock: Arc<Mutex<()>>,
    worker_control: Arc<WorkerControl>,
    server_url: Arc<Mutex<Option<String>>>,
    api_key: Arc<Mutex<Option<String>>>,
    http_client: reqwest::blocking::Client,
    max_retries: u32,
    flush_interval_jitter_max_ms: u64,
    global_coord_enabled: bool,
    global_coord_window_ms: u64,
    global_coord_max_deferral_ms: u64,
    server_lease_enabled: bool,
    server_lease_ttl_ms: u64,
    server_lease_scope: String,
}

impl Outbox {
    pub fn new(app_data_dir: &std::path::Path) -> Result<Self, OutboxError> {
        let path = app_data_dir.join("outbox.jsonl");
        let events = Self::load_events(&path)?;
        let flush_interval_jitter_max_ms = std::env::var("GITGOV_OUTBOX_FLUSH_JITTER_MAX_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DEFAULT_FLUSH_INTERVAL_JITTER_MAX_MS)
            .min(60_000);
        let global_coord_enabled = std::env::var("GITGOV_OUTBOX_GLOBAL_COORD_ENABLED")
            .ok()
            .map(|v| {
                matches!(
                    v.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(DEFAULT_GLOBAL_COORD_ENABLED);
        let global_coord_window_ms = std::env::var("GITGOV_OUTBOX_GLOBAL_COORD_WINDOW_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DEFAULT_GLOBAL_COORD_WINDOW_MS)
            .clamp(5_000, 300_000);
        let global_coord_max_deferral_ms =
            std::env::var("GITGOV_OUTBOX_GLOBAL_COORD_MAX_DEFERRAL_MS")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(DEFAULT_GLOBAL_COORD_MAX_DEFERRAL_MS)
                .min(global_coord_window_ms.saturating_sub(1));
        let server_lease_enabled = std::env::var("GITGOV_OUTBOX_SERVER_LEASE_ENABLED")
            .ok()
            .map(|v| {
                matches!(
                    v.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(DEFAULT_SERVER_LEASE_ENABLED);
        let server_lease_ttl_ms = std::env::var("GITGOV_OUTBOX_SERVER_LEASE_TTL_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DEFAULT_SERVER_LEASE_TTL_MS)
            .clamp(1_000, 60_000);
        let server_lease_scope = std::env::var("GITGOV_OUTBOX_SERVER_LEASE_SCOPE")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| DEFAULT_SERVER_LEASE_SCOPE.to_string());
        let http_client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                OutboxError::NetworkError(format!("Failed to initialize HTTP client: {}", e))
            })?;

        tracing::info!(
            flush_interval_jitter_max_ms,
            global_coord_enabled,
            global_coord_window_ms,
            global_coord_max_deferral_ms,
            server_lease_enabled,
            server_lease_ttl_ms,
            server_lease_scope = %server_lease_scope,
            "Outbox coordination config loaded"
        );

        Ok(Self {
            path,
            events: Arc::new(Mutex::new(events)),
            file_lock: Arc::new(Mutex::new(())),
            worker_control: Arc::new(WorkerControl {
                shutdown: AtomicBool::new(false),
                trigger: Condvar::new(),
            }),
            server_url: Arc::new(Mutex::new(None)),
            api_key: Arc::new(Mutex::new(None)),
            http_client,
            max_retries: 5,
            flush_interval_jitter_max_ms,
            global_coord_enabled,
            global_coord_window_ms,
            global_coord_max_deferral_ms,
            server_lease_enabled,
            server_lease_ttl_ms,
            server_lease_scope,
        })
    }

    pub fn with_server(self, url: String, api_key: Option<String>) -> Self {
        if let Ok(mut server_url) = self.server_url.lock() {
            *server_url = Some(url);
        }
        if let Ok(mut key) = self.api_key.lock() {
            *key = api_key;
        }
        self
    }

    pub fn set_server_config(&self, server_url: Option<String>, api_key: Option<String>) {
        if let Ok(mut url_guard) = self.server_url.lock() {
            *url_guard = server_url;
        }
        if let Ok(mut key_guard) = self.api_key.lock() {
            *key_guard = api_key;
        }
        // Wake worker so it can flush promptly with the new config.
        self.worker_control.trigger.notify_all();
    }

    fn load_events(path: &PathBuf) -> Result<Vec<OutboxEvent>, OutboxError> {
        if !path.exists() {
            return Ok(vec![]);
        }

        let file = File::open(path).map_err(|e| OutboxError::IoError(e.to_string()))?;

        let reader = BufReader::new(file);
        let mut events = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| OutboxError::IoError(e.to_string()))?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<OutboxEvent>(&line) {
                Ok(event) if !event.sent => events.push(event),
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("Failed to parse outbox event: {}", e);
                }
            }
        }

        Ok(events)
    }

    /// CRASH-SAFE persist using atomic rename pattern.
    ///
    /// Pattern:
    ///   1. Take events snapshot (events lock only)
    ///   2. Write to .tmp file
    ///   3. Sync to disk (ensures data on storage)
    ///   4. Atomic rename .tmp → final (or safe Windows fallback)
    ///   5. Cleanup backup if exists
    fn persist(&self) -> Result<(), OutboxError> {
        // Step 1: Snapshot events (only events lock, NO file_lock)
        let snapshot: Vec<String> = {
            let events = self
                .events
                .lock()
                .map_err(|_| OutboxError::IoError("Events lock poisoned".to_string()))?;

            events
                .iter()
                .map(serde_json::to_string)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| OutboxError::SerializationError(e.to_string()))?
        };
        // events lock released here

        // Step 2-5: Write atomically (only file_lock, NO events lock)
        {
            let _file_guard = self
                .file_lock
                .lock()
                .map_err(|_| OutboxError::IoError("File lock poisoned".to_string()))?;

            let tmp_path = self.path.with_extension("jsonl.tmp");

            // Write to temp file
            {
                let mut tmp_file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&tmp_path)
                    .map_err(|e| {
                        OutboxError::IoError(format!("Failed to create temp file: {}", e))
                    })?;

                for json in snapshot.iter() {
                    writeln!(tmp_file, "{}", json).map_err(|e| {
                        OutboxError::IoError(format!("Failed to write temp file: {}", e))
                    })?;
                }

                // CRITICAL: Sync to disk before rename
                tmp_file.sync_all().map_err(|e| {
                    OutboxError::IoError(format!("Failed to sync temp file: {}", e))
                })?;
            }

            // Atomic replace strategy (cross-platform safe)
            // On Unix: rename is atomic
            // On Windows: need to backup original first
            #[cfg(unix)]
            {
                std::fs::rename(&tmp_path, &self.path)
                    .map_err(|e| OutboxError::IoError(format!("Failed to rename: {}", e)))?;
            }

            #[cfg(windows)]
            {
                let bak_path = self.path.with_extension("jsonl.bak");
                // Windows: backup original, rename tmp, delete backup
                if self.path.exists() {
                    let _ = std::fs::rename(&self.path, &bak_path);
                }

                if let Err(e) = std::fs::rename(&tmp_path, &self.path) {
                    // Rollback: restore backup
                    if bak_path.exists() {
                        let _ = std::fs::rename(&bak_path, &self.path);
                    }
                    return Err(OutboxError::IoError(format!(
                        "Failed to rename on Windows: {}",
                        e
                    )));
                }

                // Cleanup backup
                if bak_path.exists() {
                    let _ = std::fs::remove_file(&bak_path);
                }
            }

            tracing::trace!("Persisted {} events to outbox", snapshot.len());
        }
        // file_lock released here

        Ok(())
    }

    /// Add event to outbox. NEVER holds both locks simultaneously.
    pub fn add(&self, event: OutboxEvent) -> Result<String, OutboxError> {
        let event_uuid = event.event_uuid.clone();

        // Step 1: Add to memory (only events lock)
        {
            let mut events = self
                .events
                .lock()
                .map_err(|_| OutboxError::IoError("Events lock poisoned".to_string()))?;
            events.push(event);
        }

        // Step 2: Persist (separate lock discipline)
        if let Err(e) = self.persist() {
            tracing::error!(
                "Failed to persist outbox event: {}. Event will be lost if app crashes.",
                e
            );
        }

        // Step 3: Wake up worker if waiting
        self.worker_control.trigger.notify_all();

        Ok(event_uuid)
    }

    /// Notify background worker to flush pending events promptly.
    pub fn notify_flush(&self) {
        self.worker_control.trigger.notify_all();
    }

    pub fn get_pending_count(&self) -> usize {
        self.events
            .lock()
            .map(|e| e.iter().filter(|ev| !ev.sent && !ev.dead_lettered).count())
            .unwrap_or(0)
    }

    pub fn get_status(&self) -> OutboxStatus {
        let Ok(events) = self.events.lock() else {
            return OutboxStatus {
                pending_count: 0,
                retry_scheduled_count: 0,
                dead_letter_count: 0,
                max_attempts: self.max_retries,
                next_attempt_at: None,
                last_error: Some("Outbox events lock poisoned".to_string()),
                last_dead_letter_at: None,
            };
        };

        let mut pending_count = 0usize;
        let mut retry_scheduled_count = 0usize;
        let mut dead_letter_count = 0usize;
        let mut next_attempt_at: Option<i64> = None;
        let mut last_error: Option<(i64, String)> = None;
        let mut last_dead_letter_at: Option<i64> = None;

        for event in events.iter() {
            if event.sent {
                continue;
            }

            if event.dead_lettered {
                dead_letter_count += 1;
                if let Some(dead_lettered_at) = event.dead_lettered_at {
                    last_dead_letter_at = Some(
                        last_dead_letter_at
                            .map_or(dead_lettered_at, |current| current.max(dead_lettered_at)),
                    );
                }
                if let Some(reason) = event
                    .dead_letter_reason
                    .as_ref()
                    .or(event.last_error.as_ref())
                {
                    let ts = event
                        .dead_lettered_at
                        .or(event.last_attempt)
                        .unwrap_or(event.timestamp);
                    if last_error
                        .as_ref()
                        .is_none_or(|(current, _)| ts >= *current)
                    {
                        last_error = Some((ts, reason.clone()));
                    }
                }
                continue;
            }

            pending_count += 1;
            if let Some(next_ms) = event.next_attempt_at {
                retry_scheduled_count += 1;
                next_attempt_at =
                    Some(next_attempt_at.map_or(next_ms, |current| current.min(next_ms)));
            }
            if let Some(error) = event.last_error.as_ref() {
                let ts = event.last_attempt.unwrap_or(event.timestamp);
                if last_error
                    .as_ref()
                    .is_none_or(|(current, _)| ts >= *current)
                {
                    last_error = Some((ts, error.clone()));
                }
            }
        }

        OutboxStatus {
            pending_count,
            retry_scheduled_count,
            dead_letter_count,
            max_attempts: self.max_retries,
            next_attempt_at,
            last_error: last_error.map(|(_, error)| error),
            last_dead_letter_at,
        }
    }

    /// Flush pending events. NEVER holds both locks simultaneously.
    pub fn flush(&self) -> Result<FlushResult, OutboxError> {
        let server_url = match self.server_url.lock() {
            Ok(guard) => match &*guard {
                Some(url) => url.clone(),
                None => {
                    return Err(OutboxError::NetworkError(
                        "Server URL not configured".to_string(),
                    ))
                }
            },
            Err(_) => return Err(OutboxError::IoError("Server URL lock poisoned".to_string())),
        };

        let api_key = match self.api_key.lock() {
            Ok(guard) => guard.clone(),
            Err(_) => return Err(OutboxError::IoError("API key lock poisoned".to_string())),
        };

        // Step 1: Snapshot pending events that are ready for retry.
        let now_ms = chrono::Utc::now().timestamp_millis();
        let events_to_send: Vec<OutboxEvent> = {
            let events = self
                .events
                .lock()
                .map_err(|_| OutboxError::IoError("Events lock poisoned".to_string()))?;
            events
                .iter()
                .filter(|e| Self::is_event_ready_for_retry(e, now_ms))
                .cloned()
                .collect()
        };

        if events_to_send.is_empty() {
            return Ok(FlushResult {
                sent: 0,
                duplicates: 0,
                failed: 0,
            });
        }

        let mut sent_count = 0usize;
        let mut dup_count = 0usize;
        let mut failed_count = 0usize;

        // Step 2-4: Build chunks, send, and apply response
        for chunk in events_to_send.chunks(OUTBOX_BATCH_SIZE) {
            let batch = Self::build_batch(chunk);
            let response = match self.send_batch(&server_url, api_key.as_deref(), &batch) {
                Ok(response) => response,
                Err(failure) => {
                    tracing::warn!(
                        retry_class = failure.retry.class_name(),
                        status_code = ?failure.retry.status_code,
                        retry_after_ms = ?failure.retry.retry_after_ms,
                        "Outbox flush chunk failed; scheduling retry"
                    );
                    if let Ok(mut events) = self.events.lock() {
                        let error = failure.error.to_string();
                        Self::mark_chunk_retry(
                            &mut events,
                            chunk,
                            self.max_retries,
                            failure.retry,
                            Some(error.as_str()),
                        );
                    }
                    // Persist progress of previously applied chunks before returning.
                    if let Err(persist_error) = self.persist() {
                        tracing::error!(
                            "Failed to persist outbox after partial flush error: {}",
                            persist_error
                        );
                    }
                    return Err(failure.error);
                }
            };

            sent_count += response.accepted.len();
            dup_count += response.duplicates.len();
            failed_count += response.errors.len();

            let mut events = self
                .events
                .lock()
                .map_err(|_| OutboxError::IoError("Events lock poisoned".to_string()))?;
            Self::apply_batch_response(&mut events, &response, self.max_retries);
        }

        // Step 5: Persist final state
        self.persist()?;

        Ok(FlushResult {
            sent: sent_count,
            duplicates: dup_count,
            failed: failed_count,
        })
    }

    fn send_batch(
        &self,
        server_url: &str,
        api_key: Option<&str>,
        batch: &ClientEventBatch,
    ) -> Result<ClientEventResponse, SendBatchFailure> {
        Self::send_batch_with_client(&self.http_client, server_url, api_key, batch)
    }

    fn send_batch_with_client(
        http_client: &reqwest::blocking::Client,
        server_url: &str,
        api_key: Option<&str>,
        batch: &ClientEventBatch,
    ) -> Result<ClientEventResponse, SendBatchFailure> {
        let url = format!("{}/events", server_url);

        let mut request = http_client.post(&url).json(batch);
        if let Some(api_key) = api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().map_err(|e| SendBatchFailure {
            error: OutboxError::NetworkError(e.to_string()),
            retry: RetryDirective::network(),
        })?;

        let status = response.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let retry_after_ms = Self::parse_retry_after_ms(response.headers());
            let retry = if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                RetryDirective::rate_limited(retry_after_ms, status_code)
            } else if status.is_server_error() {
                RetryDirective::http_5xx(status_code)
            } else {
                RetryDirective::http_other(status_code)
            };

            return Err(SendBatchFailure {
                error: OutboxError::NetworkError(format!("Server returned status: {}", status)),
                retry,
            });
        }

        response.json().map_err(|e| SendBatchFailure {
            error: OutboxError::SerializationError(e.to_string()),
            retry: RetryDirective::network(),
        })
    }

    fn parse_retry_after_ms(headers: &reqwest::header::HeaderMap) -> Option<i64> {
        let value = headers.get(reqwest::header::RETRY_AFTER)?;
        let raw = value.to_str().ok()?.trim();
        if raw.is_empty() {
            return None;
        }

        if let Ok(seconds) = raw.parse::<u64>() {
            return Some((seconds.saturating_mul(1_000)).min(i64::MAX as u64) as i64);
        }

        let date = chrono::DateTime::parse_from_rfc2822(raw).ok()?;
        let now = chrono::Utc::now();
        let retry_after = date
            .with_timezone(&chrono::Utc)
            .signed_duration_since(now)
            .num_milliseconds();
        if retry_after <= 0 {
            return None;
        }
        Some(retry_after)
    }

    fn build_batch(events: &[OutboxEvent]) -> ClientEventBatch {
        ClientEventBatch {
            events: events
                .iter()
                .map(|e| ClientEventInput {
                    event_uuid: e.event_uuid.clone(),
                    event_type: e.event_type.clone(),
                    org_name: e.org_name.clone(),
                    repo_full_name: e.repo_full_name.clone(),
                    user_login: e.user_login.clone(),
                    user_name: e.user_name.clone(),
                    branch: e.branch.clone(),
                    commit_sha: e.commit_sha.clone(),
                    files: e.files.clone(),
                    status: e.status.clone(),
                    reason: e.reason.clone(),
                    metadata: e.metadata.clone(),
                    timestamp: Some(e.timestamp),
                })
                .collect(),
            client_id: None,
            client_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }
    }

    /// Start background flush worker with CLEAN SHUTDOWN support.
    ///
    /// Returns JoinHandle for clean termination.
    /// Uses Condvar for fast wake-up on shutdown (no 60s wait).
    pub fn start_background_flush(&self, interval_secs: u64) -> std::thread::JoinHandle<()> {
        let events = Arc::clone(&self.events);
        let file_lock = Arc::clone(&self.file_lock);
        let worker_control = Arc::clone(&self.worker_control);
        let path = self.path.clone();
        let server_url = Arc::clone(&self.server_url);
        let api_key = Arc::clone(&self.api_key);
        let http_client = self.http_client.clone();
        let max_retries = self.max_retries;
        let flush_interval_jitter_max_ms = self.flush_interval_jitter_max_ms;
        let global_coord_enabled = self.global_coord_enabled;
        let global_coord_window_ms = self.global_coord_window_ms;
        let global_coord_max_deferral_ms = self.global_coord_max_deferral_ms;
        let server_lease_enabled = self.server_lease_enabled;
        let server_lease_ttl_ms = self.server_lease_ttl_ms;
        let server_lease_scope = self.server_lease_scope.clone();

        std::thread::spawn(move || {
            let mut last_flush = std::time::Instant::now();
            let base_interval = Duration::from_secs(interval_secs);
            let schedule_jitter_ms =
                Self::stable_worker_jitter_ms(&path, flush_interval_jitter_max_ms);
            let interval = base_interval + Duration::from_millis(schedule_jitter_ms);

            tracing::info!(
                base_interval_secs = interval_secs,
                schedule_jitter_ms,
                effective_interval_ms = interval.as_millis() as u64,
                "Outbox worker periodic flush interval configured"
            );

            loop {
                // Wait with timeout, wakeable by shutdown signal
                let events_lock = events.lock().unwrap();
                let wait_for = interval
                    .checked_sub(last_flush.elapsed())
                    .unwrap_or(Duration::from_secs(0));
                let (lock, wait_result) = worker_control
                    .trigger
                    .wait_timeout(events_lock, wait_for)
                    .unwrap();
                drop(lock);

                // Check shutdown
                if worker_control.shutdown.load(Ordering::Relaxed) {
                    tracing::info!("Outbox worker shutting down gracefully");
                    return;
                }

                let was_notified = !wait_result.timed_out();
                let periodic_due = last_flush.elapsed() >= interval;
                if !was_notified && !periodic_due {
                    continue;
                }

                // Snapshot pending events that are ready for retry.
                let now_ms = chrono::Utc::now().timestamp_millis();
                let events_to_send: Vec<OutboxEvent> = {
                    let events_lock = events.lock().unwrap();
                    events_lock
                        .iter()
                        .filter(|e| Self::is_event_ready_for_retry(e, now_ms))
                        .cloned()
                        .collect()
                };

                if events_to_send.is_empty() {
                    if periodic_due {
                        last_flush = std::time::Instant::now();
                    }
                    continue;
                }

                // Mark flush attempt to avoid tight loops on repeated wake-ups.
                last_flush = std::time::Instant::now();

                // Build and send batch
                let url_opt = server_url.lock().ok().and_then(|g| (*g).clone());
                let api_key_opt = api_key.lock().ok().and_then(|g| (*g).clone());

                if server_lease_enabled {
                    if let (Some(url), Some(api_key)) = (url_opt.as_deref(), api_key_opt.as_deref())
                    {
                        let holder = Self::global_coordination_identity(Some(api_key), &path);
                        match Self::try_acquire_server_flush_lease(
                            &http_client,
                            url,
                            api_key,
                            server_lease_scope.as_str(),
                            holder.as_str(),
                            server_lease_ttl_ms,
                        ) {
                            Ok(decision) if !decision.granted && decision.wait_ms > 0 => {
                                tracing::debug!(
                                    wait_ms = decision.wait_ms,
                                    lease_ttl_ms = server_lease_ttl_ms,
                                    "Outbox server lease not granted yet; waiting before flush"
                                );
                                let events_lock = events.lock().unwrap();
                                let (lock, _wait_result) = worker_control
                                    .trigger
                                    .wait_timeout(
                                        events_lock,
                                        Duration::from_millis(decision.wait_ms),
                                    )
                                    .unwrap();
                                drop(lock);
                                if worker_control.shutdown.load(Ordering::Relaxed) {
                                    tracing::info!("Outbox worker shutting down gracefully");
                                    return;
                                }
                                continue;
                            }
                            Ok(_) => {}
                            Err(e) => {
                                tracing::warn!(
                                    error = %e,
                                    "Outbox server lease request failed; continuing fail-open"
                                );
                            }
                        }
                    }
                }

                if global_coord_enabled {
                    let identity_seed =
                        Self::global_coordination_identity(api_key_opt.as_deref(), &path);
                    let now_ms = chrono::Utc::now().timestamp_millis();
                    let delay_ms = Self::global_coordination_wait_ms(
                        identity_seed.as_str(),
                        now_ms,
                        global_coord_window_ms,
                        global_coord_max_deferral_ms,
                    );
                    if delay_ms > 0 {
                        tracing::debug!(
                            delay_ms,
                            global_coord_window_ms,
                            global_coord_max_deferral_ms,
                            "Outbox global coordination deferring flush window"
                        );
                        let events_lock = events.lock().unwrap();
                        let (lock, _wait_result) = worker_control
                            .trigger
                            .wait_timeout(events_lock, Duration::from_millis(delay_ms))
                            .unwrap();
                        drop(lock);
                        if worker_control.shutdown.load(Ordering::Relaxed) {
                            tracing::info!("Outbox worker shutting down gracefully");
                            return;
                        }
                        continue;
                    }
                }

                if let Some(ref url) = url_opt {
                    for chunk in events_to_send.chunks(OUTBOX_BATCH_SIZE) {
                        let batch = Self::build_batch(chunk);
                        match Self::send_batch_with_client(
                            &http_client,
                            url,
                            api_key_opt.as_deref(),
                            &batch,
                        ) {
                            Ok(batch_response) => {
                                // Update state and persist atomically
                                let snapshot_for_persist: Vec<String> = {
                                    let mut events_lock = events.lock().unwrap();
                                    Self::apply_batch_response(
                                        &mut events_lock,
                                        &batch_response,
                                        max_retries,
                                    );

                                    events_lock
                                        .iter()
                                        .filter_map(|e| serde_json::to_string(e).ok())
                                        .collect()
                                };

                                // Atomic persist
                                if let Err(e) = Self::atomic_persist_static(
                                    &path,
                                    &file_lock,
                                    &snapshot_for_persist,
                                ) {
                                    tracing::error!("Failed to persist outbox: {}", e);
                                }
                            }
                            Err(failure) => {
                                let error = failure.error.to_string();
                                tracing::warn!(
                                    url = %format!("{}/events", url),
                                    retry_class = failure.retry.class_name(),
                                    status_code = ?failure.retry.status_code,
                                    retry_after_ms = ?failure.retry.retry_after_ms,
                                    error = %failure.error,
                                    "Outbox flush failed; scheduling retry"
                                );
                                let snapshot_for_persist: Vec<String> = {
                                    let mut events_lock = events.lock().unwrap();
                                    Self::mark_chunk_retry(
                                        &mut events_lock,
                                        chunk,
                                        max_retries,
                                        failure.retry,
                                        Some(error.as_str()),
                                    );
                                    events_lock
                                        .iter()
                                        .filter_map(|e| serde_json::to_string(e).ok())
                                        .collect()
                                };
                                if let Err(pe) = Self::atomic_persist_static(
                                    &path,
                                    &file_lock,
                                    &snapshot_for_persist,
                                ) {
                                    tracing::error!("Failed to persist outbox: {}", pe);
                                }
                                break;
                            }
                        }
                    }
                }
            }
        })
    }

    fn apply_batch_response(
        events: &mut Vec<OutboxEvent>,
        batch_response: &ClientEventResponse,
        max_retries: u32,
    ) {
        let accepted: HashSet<&str> = batch_response.accepted.iter().map(String::as_str).collect();
        let duplicates: HashSet<&str> = batch_response
            .duplicates
            .iter()
            .map(String::as_str)
            .collect();
        let errors: HashMap<&str, &str> = batch_response
            .errors
            .iter()
            .map(|e| (e.event_uuid.as_str(), e.error.as_str()))
            .collect();
        let now_ms = chrono::Utc::now().timestamp_millis();

        for event in events.iter_mut() {
            let event_uuid = event.event_uuid.as_str();
            if accepted.contains(event_uuid) || duplicates.contains(event_uuid) {
                event.sent = true;
                event.next_attempt_at = None;
                event.last_error = None;
            } else if let Some(error) = errors.get(event_uuid) {
                Self::mark_event_retry(
                    event,
                    now_ms,
                    RetryDirective::default_backoff(),
                    max_retries,
                    Some(*error),
                );
            }
        }

        events.retain(|e| !e.sent);
    }

    fn is_event_ready_for_retry(event: &OutboxEvent, now_ms: i64) -> bool {
        if event.sent || event.dead_lettered {
            return false;
        }
        match event.next_attempt_at {
            Some(next_ms) => now_ms >= next_ms,
            None => true,
        }
    }

    fn mark_chunk_retry(
        events: &mut Vec<OutboxEvent>,
        chunk: &[OutboxEvent],
        max_retries: u32,
        retry: RetryDirective,
        last_error: Option<&str>,
    ) {
        let now_ms = chrono::Utc::now().timestamp_millis();
        let chunk_uuids: HashSet<&str> = chunk.iter().map(|e| e.event_uuid.as_str()).collect();
        for event in events.iter_mut() {
            if chunk_uuids.contains(event.event_uuid.as_str()) {
                Self::mark_event_retry(event, now_ms, retry, max_retries, last_error);
            }
        }
        events.retain(|e| !e.sent);
    }

    fn mark_event_retry(
        event: &mut OutboxEvent,
        now_ms: i64,
        retry: RetryDirective,
        max_retries: u32,
        last_error: Option<&str>,
    ) {
        event.attempts = event.attempts.saturating_add(1);
        event.last_attempt = Some(now_ms);
        if let Some(error) = last_error {
            event.last_error = Some(Self::truncate_dead_letter_reason(error));
        }

        if event.attempts >= max_retries {
            event.dead_lettered = true;
            event.dead_lettered_at = Some(now_ms);
            event.dead_letter_reason =
                Some(Self::build_dead_letter_reason(event, retry, last_error));
            event.next_attempt_at = None;
            return;
        }

        let retry_delay_ms =
            Self::compute_retry_delay_ms(event.attempts, event.event_uuid.as_str(), retry);
        event.next_attempt_at = Some(now_ms + retry_delay_ms);
    }

    fn build_dead_letter_reason(
        event: &OutboxEvent,
        retry: RetryDirective,
        last_error: Option<&str>,
    ) -> String {
        let mut reason = format!(
            "retry_exhausted:{}:attempts={}",
            retry.class_name(),
            event.attempts
        );
        if let Some(status_code) = retry.status_code {
            reason.push_str(&format!("; status={}", status_code));
        }
        if let Some(error) = last_error {
            reason.push_str("; error=");
            reason.push_str(&Self::truncate_dead_letter_reason(error));
        }
        Self::truncate_dead_letter_reason(&reason)
    }

    fn truncate_dead_letter_reason(raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.chars().count() <= DEAD_LETTER_REASON_MAX_CHARS {
            return trimmed.to_string();
        }

        let mut truncated = trimmed
            .chars()
            .take(DEAD_LETTER_REASON_MAX_CHARS.saturating_sub(3))
            .collect::<String>();
        truncated.push_str("...");
        truncated
    }

    fn compute_retry_delay_ms(attempt: u32, event_uuid: &str, retry: RetryDirective) -> i64 {
        let exp = attempt.saturating_sub(1).min(6);
        let core_delay = (RETRY_BASE_DELAY_MS.saturating_mul(1_i64 << exp)).min(RETRY_MAX_DELAY_MS);
        let floor_delay = match retry.class {
            RetryClass::RateLimited => retry
                .retry_after_ms
                .unwrap_or(RETRY_RATE_LIMIT_FLOOR_MS)
                .max(RETRY_RATE_LIMIT_FLOOR_MS),
            RetryClass::HttpOther => RETRY_CLIENT_ERROR_FLOOR_MS,
            RetryClass::Network | RetryClass::Http5xx => 0,
        };
        let delay_without_jitter = core_delay.max(floor_delay);
        let jitter_cap = (delay_without_jitter / 5).clamp(50, 5_000);
        delay_without_jitter + Self::stable_jitter_ms(event_uuid, attempt, jitter_cap)
    }

    fn stable_jitter_ms(event_uuid: &str, attempt: u32, max_jitter_ms: i64) -> i64 {
        if max_jitter_ms <= 0 {
            return 0;
        }
        let mut hasher = DefaultHasher::new();
        event_uuid.hash(&mut hasher);
        attempt.hash(&mut hasher);
        let value = hasher.finish();
        (value % max_jitter_ms as u64) as i64
    }

    fn stable_worker_jitter_ms(path: &Path, max_jitter_ms: u64) -> u64 {
        if max_jitter_ms == 0 {
            return 0;
        }
        let mut hasher = DefaultHasher::new();
        path.to_string_lossy().hash(&mut hasher);
        std::process::id().hash(&mut hasher);
        hasher.finish() % max_jitter_ms
    }

    fn global_coordination_identity(api_key: Option<&str>, path: &Path) -> String {
        if let Some(key) = api_key {
            let trimmed = key.trim();
            if !trimmed.is_empty() {
                let mut hasher = DefaultHasher::new();
                trimmed.hash(&mut hasher);
                return format!("api-key-hash:{:016x}", hasher.finish());
            }
        }

        let hostname = std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "unknown-host".to_string());
        format!("path:{}|host:{}", path.to_string_lossy(), hostname)
    }

    fn global_coordination_wait_ms(
        identity_seed: &str,
        now_ms: i64,
        window_ms: u64,
        max_deferral_ms: u64,
    ) -> u64 {
        if identity_seed.trim().is_empty() || window_ms == 0 || max_deferral_ms == 0 {
            return 0;
        }
        let now = now_ms.max(0) as u64;
        let window_index = now / window_ms;
        let elapsed_in_window = now % window_ms;

        let mut hasher = DefaultHasher::new();
        identity_seed.hash(&mut hasher);
        window_index.hash(&mut hasher);
        let target_offset = hasher.finish() % (max_deferral_ms + 1);

        target_offset.saturating_sub(elapsed_in_window)
    }

    fn try_acquire_server_flush_lease(
        http_client: &reqwest::blocking::Client,
        server_url: &str,
        api_key: &str,
        scope: &str,
        holder: &str,
        lease_ttl_ms: u64,
    ) -> Result<ServerLeaseDecision, OutboxError> {
        #[derive(Serialize)]
        struct LeaseRequest<'a> {
            scope: &'a str,
            holder: &'a str,
            lease_ttl_ms: u64,
            max_wait_ms: u64,
        }

        #[derive(Deserialize)]
        struct LeaseResponse {
            granted: bool,
            #[serde(default)]
            wait_ms: u64,
        }

        let url = format!("{}/outbox/lease", server_url);
        let response = http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&LeaseRequest {
                scope,
                holder,
                lease_ttl_ms,
                max_wait_ms: lease_ttl_ms,
            })
            .send()
            .map_err(|e| OutboxError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(OutboxError::NetworkError(format!(
                "Lease endpoint returned status: {}",
                response.status()
            )));
        }

        let parsed: LeaseResponse = response
            .json()
            .map_err(|e| OutboxError::SerializationError(e.to_string()))?;
        Ok(ServerLeaseDecision {
            granted: parsed.granted,
            wait_ms: parsed.wait_ms.min(120_000),
        })
    }

    /// Signal shutdown to worker
    pub fn signal_shutdown(&self) {
        self.worker_control.shutdown.store(true, Ordering::Relaxed);
        self.worker_control.trigger.notify_all();
    }

    /// Static atomic persist for use in worker
    fn atomic_persist_static(
        path: &PathBuf,
        file_lock: &Arc<Mutex<()>>,
        snapshot: &[String],
    ) -> Result<(), OutboxError> {
        let _file_guard = file_lock
            .lock()
            .map_err(|_| OutboxError::IoError("File lock poisoned".to_string()))?;

        let tmp_path = path.with_extension("jsonl.tmp");

        // Write to temp
        {
            let mut tmp_file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&tmp_path)
                .map_err(|e| OutboxError::IoError(format!("Temp file: {}", e)))?;

            for json in snapshot.iter() {
                writeln!(tmp_file, "{}", json)
                    .map_err(|e| OutboxError::IoError(format!("Write: {}", e)))?;
            }

            tmp_file
                .sync_all()
                .map_err(|e| OutboxError::IoError(format!("Sync: {}", e)))?;
        }

        // Atomic replace
        #[cfg(unix)]
        {
            std::fs::rename(&tmp_path, path)
                .map_err(|e| OutboxError::IoError(format!("Rename: {}", e)))?;
        }

        #[cfg(windows)]
        {
            let bak_path = path.with_extension("jsonl.bak");
            if path.exists() {
                let _ = std::fs::rename(path, &bak_path);
            }

            if let Err(e) = std::fs::rename(&tmp_path, path) {
                if bak_path.exists() {
                    let _ = std::fs::rename(&bak_path, path);
                }
                return Err(OutboxError::IoError(format!("Windows rename: {}", e)));
            }

            if bak_path.exists() {
                let _ = std::fs::remove_file(&bak_path);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientEventBatch {
    pub events: Vec<ClientEventInput>,
    pub client_id: Option<String>,
    pub client_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientEventInput {
    pub event_uuid: String,
    pub event_type: String,
    pub org_name: Option<String>,
    pub repo_full_name: Option<String>,
    pub user_login: String,
    pub user_name: Option<String>,
    pub branch: Option<String>,
    pub commit_sha: Option<String>,
    pub files: Vec<String>,
    pub status: String,
    pub reason: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientEventResponse {
    pub accepted: Vec<String>,
    pub duplicates: Vec<String>,
    pub errors: Vec<EventError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventError {
    pub event_uuid: String,
    pub error: String,
}

#[derive(Debug)]
pub struct FlushResult {
    pub sent: usize,
    pub duplicates: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutboxStatus {
    pub pending_count: usize,
    pub retry_scheduled_count: usize,
    pub dead_letter_count: usize,
    pub max_attempts: u32,
    pub next_attempt_at: Option<i64>,
    pub last_error: Option<String>,
    pub last_dead_letter_at: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_retry_after_ms_supports_seconds_header() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::RETRY_AFTER,
            reqwest::header::HeaderValue::from_static("120"),
        );

        assert_eq!(Outbox::parse_retry_after_ms(&headers), Some(120_000));
    }

    #[test]
    fn compute_retry_delay_rate_limited_honors_retry_after_floor() {
        let retry = RetryDirective::rate_limited(Some(45_000), 429);
        let delay = Outbox::compute_retry_delay_ms(1, "event-uuid-1", retry);
        assert!(delay >= 45_000);
    }

    #[test]
    fn compute_retry_delay_grows_for_http_5xx_attempts() {
        let retry = RetryDirective::http_5xx(503);
        let delay_attempt_1 = Outbox::compute_retry_delay_ms(1, "event-uuid-2", retry);
        let delay_attempt_2 = Outbox::compute_retry_delay_ms(2, "event-uuid-2", retry);
        assert!(delay_attempt_2 > delay_attempt_1);
    }

    #[test]
    fn worker_jitter_is_stable_and_bounded() {
        let path = PathBuf::from("C:/tmp/gitgov/outbox.jsonl");
        let max = 5_000;
        let jitter1 = Outbox::stable_worker_jitter_ms(&path, max);
        let jitter2 = Outbox::stable_worker_jitter_ms(&path, max);
        assert_eq!(jitter1, jitter2);
        assert!(jitter1 < max);
    }

    #[test]
    fn global_coordination_wait_is_stable_and_bounded() {
        let identity = "api-key:test-key";
        let now_ms = 123_456_789_i64;
        let wait1 = Outbox::global_coordination_wait_ms(identity, now_ms, 60_000, 15_000);
        let wait2 = Outbox::global_coordination_wait_ms(identity, now_ms, 60_000, 15_000);
        assert_eq!(wait1, wait2);
        assert!(wait1 <= 15_000);
    }

    #[test]
    fn global_coordination_wait_is_zero_when_slot_already_elapsed() {
        let identity = "api-key:test-key";
        let window = 10_000_u64;
        let max_deferral = 2_000_u64;
        let mut found = false;
        for elapsed in 0..window {
            let now_ms = (42 * window + elapsed) as i64;
            let wait = Outbox::global_coordination_wait_ms(identity, now_ms, window, max_deferral);
            if wait == 0 {
                found = true;
                break;
            }
        }
        assert!(found);
    }

    #[test]
    fn global_coordination_identity_does_not_expose_raw_api_key() {
        let path = PathBuf::from("C:/tmp/gitgov/outbox.jsonl");
        let identity = Outbox::global_coordination_identity(Some("secret-key-123"), &path);
        assert!(identity.starts_with("api-key-hash:"));
        assert!(!identity.contains("secret-key-123"));
    }

    #[test]
    fn event_moves_to_dead_letter_after_max_retries() {
        let mut event = OutboxEvent::new(
            "attempt_push".to_string(),
            "alice".to_string(),
            Some("main".to_string()),
            AuditStatus::Failed,
        );
        let now_ms = 1_700_000_000_000;

        for offset in 0..5 {
            Outbox::mark_event_retry(
                &mut event,
                now_ms + offset,
                RetryDirective::http_other(400),
                5,
                Some("server rejected event"),
            );
        }

        assert_eq!(event.attempts, 5);
        assert!(event.dead_lettered);
        assert_eq!(event.dead_lettered_at, Some(now_ms + 4));
        assert!(event
            .dead_letter_reason
            .as_deref()
            .unwrap_or_default()
            .contains("status=400"));
        assert!(!Outbox::is_event_ready_for_retry(&event, now_ms + 10_000));
    }

    #[test]
    fn get_status_counts_pending_scheduled_and_dead_letter_events() {
        let test_dir = std::env::temp_dir().join(format!("gitgov-outbox-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&test_dir).expect("create temp dir");
        let outbox = Outbox::new(&test_dir).expect("create outbox");

        let pending = OutboxEvent::new(
            "commit".to_string(),
            "alice".to_string(),
            Some("main".to_string()),
            AuditStatus::Success,
        );
        let mut scheduled = OutboxEvent::new(
            "push_failed".to_string(),
            "alice".to_string(),
            Some("main".to_string()),
            AuditStatus::Failed,
        );
        scheduled.next_attempt_at = Some(1_700_000_010_000);
        scheduled.last_attempt = Some(1_700_000_005_000);
        scheduled.last_error = Some("network timeout".to_string());

        let mut dead = OutboxEvent::new(
            "cli_command".to_string(),
            "alice".to_string(),
            Some("main".to_string()),
            AuditStatus::Failed,
        );
        dead.dead_lettered = true;
        dead.dead_lettered_at = Some(1_700_000_020_000);
        dead.dead_letter_reason = Some("retry_exhausted:http_other:attempts=5".to_string());

        outbox.add(pending).expect("add pending");
        outbox.add(scheduled).expect("add scheduled");
        outbox.add(dead).expect("add dead");

        let status = outbox.get_status();
        assert_eq!(status.pending_count, 2);
        assert_eq!(status.retry_scheduled_count, 1);
        assert_eq!(status.dead_letter_count, 1);
        assert_eq!(status.next_attempt_at, Some(1_700_000_010_000));
        assert_eq!(status.last_dead_letter_at, Some(1_700_000_020_000));
        assert!(status
            .last_error
            .as_deref()
            .unwrap_or_default()
            .contains("retry_exhausted"));

        std::fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn batch_response_errors_dead_letter_after_final_attempt() {
        let mut event = OutboxEvent::new(
            "attempt_push".to_string(),
            "alice".to_string(),
            Some("main".to_string()),
            AuditStatus::Failed,
        );
        event.attempts = 4;
        let event_uuid = event.event_uuid.clone();
        let response = ClientEventResponse {
            accepted: vec![],
            duplicates: vec![],
            errors: vec![EventError {
                event_uuid,
                error: "unknown event_type".to_string(),
            }],
        };
        let mut events = vec![event];

        Outbox::apply_batch_response(&mut events, &response, 5);

        assert_eq!(events.len(), 1);
        assert!(events[0].dead_lettered);
        assert_eq!(events[0].next_attempt_at, None);
        assert!(events[0]
            .last_error
            .as_deref()
            .unwrap_or_default()
            .contains("unknown event_type"));
    }
}
