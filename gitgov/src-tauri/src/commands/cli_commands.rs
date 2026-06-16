// Split facade for Desktop CLI command handlers.
// Live code is partitioned by responsibility under commands/cli_commands/.

include!("cli_commands/types.rs");
include!("cli_commands/helpers.rs");
include!("cli_commands/git_context.rs");
include!("cli_commands/shell_session.rs");
include!("cli_commands/native_terminal.rs");
include!("cli_commands/execute.rs");
include!("cli_commands/pipeline.rs");

// ============================================================================
// ARCHIVED ORIGINAL: src/commands/cli_commands.rs before SRP split.
// Keep temporarily as commented evidence until the split is reviewed and accepted.
// ============================================================================
// ===== PART 1 -> src/commands/cli_commands/types.rs lines 1-201 =====
// use serde::{Deserialize, Serialize};
// use std::collections::HashMap;
// use std::io::{BufRead, BufReader, Read, Write};
// use std::path::Path;
// use std::process::{Child, ChildStdin, Command, Stdio};
// use std::sync::{Arc, Mutex};
// use std::time::Instant;
// use tauri::{Emitter, State};
//
// use portable_pty::{native_pty_system, CommandBuilder, PtySize};
//
// use crate::control_plane::{CliCommandInput, ControlPlaneClient, ServerConfig};
// use crate::models::AuditStatus;
// use crate::outbox::{Outbox, OutboxEvent};
//
// use super::server_commands::ServerConnectionConfig;
//
// /// Whitelist of allowed command prefixes.
// /// Admin-configurable in the future; hardcoded for MVP.
// const DEFAULT_ALLOWED_PREFIXES: &[&str] = &["git", "gitgov"];
// const SHELL_EXIT_MARKER_PREFIX: &str = "__GITGOV_EXIT__:";
// const ENV_ENABLE_SHELL_COMMANDS: &str = "GITGOV_ENABLE_SHELL_COMMANDS";
// const ENV_ENABLE_NATIVE_TERMINAL: &str = "GITGOV_ENABLE_NATIVE_TERMINAL";
//
// /// Payload emitted per line of CLI output via Tauri event.
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CliOutputEvent {
//     pub command_id: String,
//     pub line_type: String, // "stdout" | "stderr" | "system"
//     pub text: String,
// }
//
// /// Payload emitted when CLI command finishes.
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CliFinishedEvent {
//     pub command_id: String,
//     pub exit_code: i32,
// }
//
// /// Result returned to the frontend immediately when a command starts.
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CliStartResult {
//     pub command_id: String,
//     pub allowed: bool,
//     pub error: Option<String>,
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CliExecuteRequest {
//     pub command: String,
//     pub cwd: String,
//     #[serde(default)]
//     pub user_login: Option<String>,
//     #[serde(default)]
//     pub branch: Option<String>,
//     pub origin: String,
//     #[serde(default)]
//     pub server_config: Option<ServerConnectionConfig>,
//     #[serde(default)]
//     pub execution_mode: CliExecutionMode,
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CliShellStartRequest {
//     pub cwd: String,
//     #[serde(default)]
//     pub user_login: Option<String>,
//     #[serde(default)]
//     pub branch: Option<String>,
//     #[serde(default)]
//     pub server_config: Option<ServerConnectionConfig>,
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CliShellStartResult {
//     pub session_id: String,
//     pub shell: String,
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CliShellInputRequest {
//     pub session_id: String,
//     pub input: String,
//     #[serde(default)]
//     pub user_login: Option<String>,
//     #[serde(default)]
//     pub branch: Option<String>,
//     #[serde(default)]
//     pub origin: Option<String>,
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CliShellInputResult {
//     pub command_id: String,
//     pub accepted: bool,
//     pub error: Option<String>,
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CliShellStopResult {
//     pub stopped: bool,
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CliNativeTerminalStartRequest {
//     pub cwd: String,
//     #[serde(default)]
//     pub cols: Option<u16>,
//     #[serde(default)]
//     pub rows: Option<u16>,
//     #[serde(default)]
//     pub shell: Option<String>,
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CliNativeTerminalStartResult {
//     pub session_id: String,
//     pub shell: String,
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CliNativeTerminalWriteRequest {
//     pub session_id: String,
//     pub data: String,
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CliNativeTerminalResizeRequest {
//     pub session_id: String,
//     pub cols: u16,
//     pub rows: u16,
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CliNativeTerminalStopResult {
//     pub stopped: bool,
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CliNativeTerminalOutputEvent {
//     pub session_id: String,
//     pub data: String,
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CliNativeTerminalExitEvent {
//     pub session_id: String,
//     pub exit_code: i32,
// }
//
// #[derive(Debug)]
// struct PendingShellCommand {
//     command: String,
//     origin: String,
//     branch: String,
//     user_login: String,
//     cwd: String,
//     repo_name: Option<String>,
//     server_config: Option<ServerConnectionConfig>,
//     started_at: Instant,
// }
//
// struct CliStartAuditInput<'a> {
//     user_login: &'a str,
//     branch: &'a str,
//     command: &'a str,
//     origin: &'a str,
//     command_id: &'a str,
//     execution_mode: &'a str,
//     repo_name: Option<&'a str>,
// }
//
// #[derive(Debug)]
// struct ShellSession {
//     cwd: String,
//     default_user_login: String,
//     default_branch: String,
//     repo_name: Option<String>,
//     server_config: Option<ServerConnectionConfig>,
//     stdin: Arc<Mutex<ChildStdin>>,
//     child: Arc<Mutex<Child>>,
//     active_command_id: Arc<Mutex<String>>,
//     pending_commands: Arc<Mutex<HashMap<String, PendingShellCommand>>>,
// }
//
// struct NativeTerminalSession {
//     master: Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>,
//     writer: Arc<Mutex<Box<dyn Write + Send>>>,
//     killer: Arc<Mutex<Box<dyn portable_pty::ChildKiller + Send + Sync>>>,
// }
//
// #[derive(Default)]
// pub struct CliShellManager {
//     sessions: Mutex<HashMap<String, ShellSession>>,
// }
//
// #[derive(Default)]
// pub struct CliNativeTerminalManager {
//     sessions: Arc<Mutex<HashMap<String, NativeTerminalSession>>>,
// }
//
// ===== PART 2 -> src/commands/cli_commands/helpers.rs lines 202-730 =====
// fn is_command_allowed(command: &str) -> bool {
//     let trimmed = command.trim();
//     DEFAULT_ALLOWED_PREFIXES
//         .iter()
//         .any(|prefix| trimmed == *prefix || trimmed.starts_with(&format!("{} ", prefix)))
// }
//
// fn split_command_line(command: &str) -> Result<Vec<String>, String> {
//     let mut args = Vec::new();
//     let mut current = String::new();
//     let mut quote: Option<char> = None;
//     let mut token_started = false;
//     let mut chars = command.trim().chars().peekable();
//
//     while let Some(ch) = chars.next() {
//         match quote {
//             Some(active_quote) => {
//                 if ch == active_quote {
//                     quote = None;
//                     token_started = true;
//                 } else if active_quote == '"' && ch == '\\' {
//                     match chars.peek().copied() {
//                         Some('"') | Some('\\') => {
//                             if let Some(next) = chars.next() {
//                                 current.push(next);
//                             }
//                         }
//                         _ => current.push(ch),
//                     }
//                     token_started = true;
//                 } else {
//                     current.push(ch);
//                     token_started = true;
//                 }
//             }
//             None => {
//                 if ch == '"' || ch == '\'' {
//                     quote = Some(ch);
//                     token_started = true;
//                 } else if ch.is_whitespace() {
//                     if token_started {
//                         args.push(std::mem::take(&mut current));
//                         token_started = false;
//                     }
//                 } else {
//                     current.push(ch);
//                     token_started = true;
//                 }
//             }
//         }
//     }
//
//     if let Some(active_quote) = quote {
//         return Err(format!("Unclosed {} quote in command", active_quote));
//     }
//     if token_started {
//         args.push(current);
//     }
//
//     Ok(args)
// }
//
// fn env_flag_enabled(var_name: &str, default_value: bool) -> bool {
//     let raw_value = match std::env::var(var_name) {
//         Ok(value) => value,
//         Err(_) => return default_value,
//     };
//
//     parse_env_flag_value(&raw_value, default_value)
// }
//
// fn parse_env_flag_value(raw_value: &str, default_value: bool) -> bool {
//     match raw_value.trim().to_ascii_lowercase().as_str() {
//         "1" | "true" | "yes" | "on" => true,
//         "0" | "false" | "no" | "off" => false,
//         _ => default_value,
//     }
// }
//
// fn shell_commands_enabled() -> bool {
//     env_flag_enabled(ENV_ENABLE_SHELL_COMMANDS, true)
// }
//
// fn native_terminal_enabled() -> bool {
//     env_flag_enabled(ENV_ENABLE_NATIVE_TERMINAL, true)
// }
//
// #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
// #[serde(rename_all = "snake_case")]
// pub enum CliExecutionMode {
//     #[default]
//     Safe,
//     Shell,
// }
//
// impl CliExecutionMode {
//     fn as_str(self) -> &'static str {
//         match self {
//             CliExecutionMode::Safe => "safe",
//             CliExecutionMode::Shell => "shell",
//         }
//     }
// }
//
// fn parse_shell_exit_marker(text: &str) -> Option<(String, i32)> {
//     let trimmed = text.trim();
//     let marker = trimmed.strip_prefix(SHELL_EXIT_MARKER_PREFIX)?;
//     let (command_id, exit_code) = marker.rsplit_once(':')?;
//     let parsed_exit = exit_code.trim().parse::<i32>().ok()?;
//     if command_id.trim().is_empty() {
//         return None;
//     }
//     Some((command_id.trim().to_string(), parsed_exit))
// }
//
// #[cfg(target_os = "windows")]
// fn wrap_shell_command(command: &str, command_id: &str) -> String {
//     format!(
//         "& {{ $global:LASTEXITCODE = $null; {} }}; $ggSucceeded = $?; $ggLastExit = $LASTEXITCODE; if ($null -ne $ggLastExit) {{ $ggEc = [int]$ggLastExit }} elseif ($ggSucceeded) {{ $ggEc = 0 }} else {{ $ggEc = 1 }}; Write-Output \"{}{}:$ggEc\"\n",
//         command, SHELL_EXIT_MARKER_PREFIX, command_id
//     )
// }
//
// #[cfg(not(target_os = "windows"))]
// fn wrap_shell_command(command: &str, command_id: &str) -> String {
//     format!(
//         "{{ {} ; }}; __gitgov_ec=$?; printf \"{}{}:%s\\n\" \"$__gitgov_ec\"\n",
//         command, SHELL_EXIT_MARKER_PREFIX, command_id
//     )
// }
//
// fn native_terminal_size(cols: Option<u16>, rows: Option<u16>) -> PtySize {
//     PtySize {
//         rows: rows.unwrap_or(30).max(5),
//         cols: cols.unwrap_or(120).max(20),
//         pixel_width: 0,
//         pixel_height: 0,
//     }
// }
//
// #[cfg(target_os = "windows")]
// fn build_native_terminal_command(shell: Option<&str>) -> (CommandBuilder, String) {
//     let requested = shell
//         .map(str::trim)
//         .filter(|s| !s.is_empty())
//         .map(|s| s.to_ascii_lowercase());
//
//     match requested.as_deref() {
//         Some("cmd") | Some("cmd.exe") => (CommandBuilder::new("cmd.exe"), "cmd".to_string()),
//         Some("pwsh") | Some("pwsh.exe") => {
//             let mut command = CommandBuilder::new("pwsh.exe");
//             command.arg("-NoLogo");
//             command.arg("-NoProfile");
//             (command, "pwsh".to_string())
//         }
//         _ => {
//             let mut command = CommandBuilder::new("powershell.exe");
//             command.arg("-NoLogo");
//             command.arg("-NoProfile");
//             (command, "powershell".to_string())
//         }
//     }
// }
//
// #[cfg(not(target_os = "windows"))]
// fn build_native_terminal_command(shell: Option<&str>) -> (CommandBuilder, String) {
//     let requested_shell = shell
//         .map(str::trim)
//         .filter(|s| !s.is_empty())
//         .map(ToOwned::to_owned)
//         .or_else(|| std::env::var("SHELL").ok().filter(|s| !s.trim().is_empty()))
//         .unwrap_or_else(|| "/bin/bash".to_string());
//
//     let mut command = CommandBuilder::new(&requested_shell);
//     command.arg("-i");
//
//     let label = Path::new(&requested_shell)
//         .file_name()
//         .and_then(|v| v.to_str())
//         .filter(|v| !v.trim().is_empty())
//         .unwrap_or("shell")
//         .to_string();
//
//     (command, label)
// }
//
// fn spawn_safe_child(command: &str, cwd: &str) -> Result<Child, String> {
//     let parts = split_command_line(command)?;
//     if parts.is_empty() {
//         return Err("Empty command".to_string());
//     }
//     let program = &parts[0];
//     let args = &parts[1..];
//     Command::new(program)
//         .args(args)
//         .current_dir(cwd)
//         .stdout(Stdio::piped())
//         .stderr(Stdio::piped())
//         .spawn()
//         .map_err(|e| format!("Failed to spawn '{}': {}", program, e))
// }
//
// #[cfg(target_os = "windows")]
// fn spawn_shell_child(command: &str, cwd: &str) -> Result<Child, String> {
//     Command::new("powershell")
//         .args(["-NoProfile", "-NonInteractive", "-Command", command])
//         .current_dir(cwd)
//         .stdout(Stdio::piped())
//         .stderr(Stdio::piped())
//         .spawn()
//         .map_err(|e| format!("Failed to spawn PowerShell: {}", e))
// }
//
// #[cfg(not(target_os = "windows"))]
// fn spawn_shell_child(command: &str, cwd: &str) -> Result<Child, String> {
//     let bash = Command::new("bash")
//         .args(["-lc", command])
//         .current_dir(cwd)
//         .stdout(Stdio::piped())
//         .stderr(Stdio::piped())
//         .spawn();
//     match bash {
//         Ok(child) => Ok(child),
//         Err(bash_err) => Command::new("sh")
//             .args(["-lc", command])
//             .current_dir(cwd)
//             .stdout(Stdio::piped())
//             .stderr(Stdio::piped())
//             .spawn()
//             .map_err(|sh_err| {
//                 format!(
//                     "Failed to spawn shell (bash error: {}; sh error: {})",
//                     bash_err, sh_err
//                 )
//             }),
//     }
// }
//
// #[cfg(target_os = "windows")]
// fn spawn_shell_session_child(cwd: &str) -> Result<(Child, &'static str), String> {
//     Command::new("powershell")
//         .args(["-NoLogo", "-NoProfile", "-NoExit"])
//         .current_dir(cwd)
//         .stdin(Stdio::piped())
//         .stdout(Stdio::piped())
//         .stderr(Stdio::piped())
//         .spawn()
//         .map(|child| (child, "powershell"))
//         .map_err(|e| format!("Failed to start PowerShell session: {}", e))
// }
//
// #[cfg(not(target_os = "windows"))]
// fn spawn_shell_session_child(cwd: &str) -> Result<(Child, &'static str), String> {
//     let bash = Command::new("bash")
//         .current_dir(cwd)
//         .stdin(Stdio::piped())
//         .stdout(Stdio::piped())
//         .stderr(Stdio::piped())
//         .spawn();
//     match bash {
//         Ok(child) => Ok((child, "bash")),
//         Err(bash_err) => Command::new("sh")
//             .current_dir(cwd)
//             .stdin(Stdio::piped())
//             .stdout(Stdio::piped())
//             .stderr(Stdio::piped())
//             .spawn()
//             .map(|child| (child, "sh"))
//             .map_err(|sh_err| {
//                 format!(
//                     "Failed to start shell session (bash error: {}; sh error: {})",
//                     bash_err, sh_err
//                 )
//             }),
//     }
// }
//
// fn resolve_user_login(explicit: Option<String>) -> String {
//     let value = explicit.unwrap_or_default().trim().to_string();
//     if !value.is_empty() {
//         return value;
//     }
//     super::auth_commands::load_current_user_session_login().unwrap_or_else(|| "unknown".to_string())
// }
//
// fn resolve_branch(explicit: Option<String>, cwd: &str) -> String {
//     let value = explicit.unwrap_or_default().trim().to_string();
//     if !value.is_empty() {
//         return value;
//     }
//
//     if let Ok(repo) = git2::Repository::open(cwd) {
//         if let Ok(head) = repo.head() {
//             if let Some(name) = head.shorthand() {
//                 let normalized = name.trim().to_string();
//                 if !normalized.is_empty() {
//                     return normalized;
//                 }
//             }
//         }
//     }
//
//     "unknown".to_string()
// }
//
// fn infer_repo_name_from_cwd(cwd: &str) -> Option<String> {
//     if let Ok(repo) = git2::Repository::open(cwd) {
//         return super::git_commands::infer_repo_full_name_pub(&repo);
//     }
//     None
// }
//
// fn emit_system_line(app: &tauri::AppHandle, command_id: &str, text: impl Into<String>) {
//     let _ = app.emit(
//         "gitgov:cli-output",
//         CliOutputEvent {
//             command_id: command_id.to_string(),
//             line_type: "system".to_string(),
//             text: text.into(),
//         },
//     );
// }
//
// fn redact_url_credentials(input: &str) -> String {
//     let mut output = input.to_string();
//     for scheme in ["https://", "http://"] {
//         let mut search_from = 0usize;
//         while let Some(relative_idx) = output[search_from..].find(scheme) {
//             let start = search_from + relative_idx + scheme.len();
//             let end = output[start..]
//                 .find(|c: char| c == '/' || c.is_whitespace())
//                 .map(|idx| start + idx)
//                 .unwrap_or(output.len());
//             if let Some(at_relative) = output[start..end].find('@') {
//                 let at = start + at_relative;
//                 output.replace_range(start..at, "[REDACTED]");
//                 search_from = start + "[REDACTED]@".len();
//             } else {
//                 search_from = end;
//             }
//         }
//     }
//     output
// }
//
// fn redact_after_marker_case_insensitive(input: String, marker: &str) -> String {
//     let mut output = input;
//     let marker_lower = marker.to_ascii_lowercase();
//     let mut search_from = 0usize;
//     loop {
//         let lower = output.to_ascii_lowercase();
//         let Some(relative_idx) = lower[search_from..].find(&marker_lower) else {
//             break;
//         };
//         let value_start = search_from + relative_idx + marker.len();
//         let value_end = output[value_start..]
//             .find(|c: char| {
//                 c.is_whitespace() || matches!(c, '"' | '\'' | '&' | ';' | ',' | ')' | ']' | '}')
//             })
//             .map(|idx| value_start + idx)
//             .unwrap_or(output.len());
//         if value_end > value_start {
//             output.replace_range(value_start..value_end, "[REDACTED]");
//             search_from = value_start + "[REDACTED]".len();
//         } else {
//             search_from = value_start;
//         }
//     }
//     output
// }
//
// fn redact_token_prefix(input: String, prefix: &str) -> String {
//     let mut output = input;
//     let mut search_from = 0usize;
//     loop {
//         let Some(relative_idx) = output[search_from..].find(prefix) else {
//             break;
//         };
//         let token_start = search_from + relative_idx;
//         let token_end = output[token_start..]
//             .find(|c: char| {
//                 c.is_whitespace() || matches!(c, '"' | '\'' | '&' | ';' | ',' | ')' | ']' | '}')
//             })
//             .map(|idx| token_start + idx)
//             .unwrap_or(output.len());
//         output.replace_range(token_start..token_end, "[REDACTED_SECRET]");
//         search_from = token_start + "[REDACTED_SECRET]".len();
//     }
//     output
// }
//
// fn redact_sensitive_cli_text(input: &str) -> String {
//     let mut output = redact_url_credentials(input);
//     for marker in [
//         "authorization: bearer ",
//         "bearer ",
//         "api_key=",
//         "apikey=",
//         "token=",
//         "password=",
//         "secret=",
//         "github_token=",
//         "gitgov_api_key=",
//         "sonar_token=",
//         "jira_api_token=",
//         "jenkins_api_token=",
//     ] {
//         output = redact_after_marker_case_insensitive(output, marker);
//     }
//     for prefix in [
//         "ghp_",
//         "gho_",
//         "ghu_",
//         "ghs_",
//         "github_pat_",
//         "glpat-",
//         "sk-",
//     ] {
//         output = redact_token_prefix(output, prefix);
//     }
//     output
// }
//
// fn redact_cli_preview_lines(lines: &[String]) -> Vec<String> {
//     lines
//         .iter()
//         .map(|line| redact_sensitive_cli_text(line))
//         .collect()
// }
//
// fn queue_cli_start_audit(outbox: &Arc<Outbox>, input: &CliStartAuditInput<'_>) {
//     let mut event = OutboxEvent::new(
//         "cli_command".to_string(),
//         input.user_login.to_string(),
//         Some(input.branch.to_string()),
//         AuditStatus::Success,
//     )
//     .with_metadata(serde_json::json!({
//         "command": redact_sensitive_cli_text(input.command),
//         "origin": input.origin,
//         "branch": input.branch,
//         "command_id": input.command_id,
//         "execution_mode": input.execution_mode,
//     }));
//
//     if let Some(full_name) = input.repo_name {
//         event = event.with_repo(full_name.to_string());
//         if let Some(org) = full_name.split('/').next().map(ToOwned::to_owned) {
//             event = event.with_org(org);
//         }
//     }
//     let _ = outbox.add(event);
// }
//
// fn queue_cli_completion_audit(
//     outbox: &Arc<Outbox>,
//     pending: &PendingShellCommand,
//     command_id: &str,
//     exit_code: i32,
//     stdout_preview: &[String],
//     stderr_preview: &[String],
// ) {
//     let audit_status = if exit_code == 0 {
//         AuditStatus::Success
//     } else {
//         AuditStatus::Failed
//     };
//     let duration_ms = pending.started_at.elapsed().as_millis() as i64;
//
//     let mut done_event = OutboxEvent::new(
//         "cli_command_completed".to_string(),
//         pending.user_login.clone(),
//         Some(pending.branch.clone()),
//         audit_status,
//     )
//     .with_metadata(serde_json::json!({
//         "command": redact_sensitive_cli_text(&pending.command),
//         "origin": pending.origin,
//         "branch": pending.branch,
//         "exit_code": exit_code,
//         "execution_mode": "shell",
//         "command_id": command_id,
//     }));
//
//     if let Some(full_name) = &pending.repo_name {
//         done_event = done_event.with_repo(full_name.clone());
//         if let Some(org) = full_name.split('/').next().map(ToOwned::to_owned) {
//             done_event = done_event.with_org(org);
//         }
//     } else if let Some(inferred_repo) = infer_repo_name_from_cwd(&pending.cwd) {
//         done_event = done_event.with_repo(inferred_repo.clone());
//         if let Some(org) = inferred_repo.split('/').next().map(ToOwned::to_owned) {
//             done_event = done_event.with_org(org);
//         }
//     }
//
//     let safe_stdout_preview = redact_cli_preview_lines(stdout_preview);
//     let safe_stderr_preview = redact_cli_preview_lines(stderr_preview);
//
//     let _ = outbox.add(done_event);
//     outbox.notify_flush();
//
//     if let Some(cfg) = &pending.server_config {
//         if !cfg.url.trim().is_empty() {
//             let client = ControlPlaneClient::new(ServerConfig {
//                 url: cfg.url.clone(),
//                 api_key: cfg.api_key.clone(),
//             });
//             let payload = CliCommandInput {
//                 command: redact_sensitive_cli_text(&pending.command),
//                 origin: pending.origin.clone(),
//                 branch: pending.branch.clone(),
//                 repo_name: pending.repo_name.clone(),
//                 exit_code: Some(exit_code),
//                 duration_ms: Some(duration_ms),
//                 metadata: serde_json::json!({
//                     "command_id": command_id,
//                     "execution_mode": "shell",
//                     "stdout_preview": safe_stdout_preview,
//                     "stderr_preview": safe_stderr_preview,
//                 }),
//             };
//             if let Err(e) = client.ingest_cli_command(&payload) {
//                 tracing::warn!(error = %e, "Shell command audit ingestion failed");
//             }
//         }
//     }
// }
//
// ===== PART 3 -> src/commands/cli_commands/shell_session.rs lines 731-1114 =====
// #[tauri::command]
// pub fn cmd_start_shell_session(
//     app: tauri::AppHandle,
//     request: CliShellStartRequest,
//     shell_manager: State<'_, CliShellManager>,
//     outbox: State<'_, Arc<Outbox>>,
// ) -> Result<CliShellStartResult, String> {
//     if !shell_commands_enabled() {
//         return Err(format!(
//             "Shell sessions are disabled by {}=false",
//             ENV_ENABLE_SHELL_COMMANDS
//         ));
//     }
//
//     let cwd = request.cwd.trim().to_string();
//     if cwd.is_empty() {
//         return Err("cwd is required".to_string());
//     }
//
//     // Keep a single active shell session to avoid terminal split-brain.
//     {
//         let mut sessions = shell_manager
//             .sessions
//             .lock()
//             .map_err(|_| "Shell session lock poisoned".to_string())?;
//         for (_, session) in sessions.drain() {
//             if let Ok(mut child) = session.child.lock() {
//                 let _ = child.kill();
//                 let _ = child.wait();
//             }
//         }
//     }
//
//     let (mut child, shell_name) = spawn_shell_session_child(&cwd)?;
//     let stdin = child
//         .stdin
//         .take()
//         .ok_or_else(|| "Failed to capture shell stdin".to_string())?;
//     let stdout = child
//         .stdout
//         .take()
//         .ok_or_else(|| "Failed to capture shell stdout".to_string())?;
//     let stderr = child
//         .stderr
//         .take()
//         .ok_or_else(|| "Failed to capture shell stderr".to_string())?;
//
//     let session_id = uuid::Uuid::new_v4().to_string();
//     let default_user_login = resolve_user_login(request.user_login);
//     let default_branch = resolve_branch(request.branch, &cwd);
//     let repo_name = infer_repo_name_from_cwd(&cwd);
//
//     let child_ref = Arc::new(Mutex::new(child));
//     let stdin_ref = Arc::new(Mutex::new(stdin));
//     let active_command_id = Arc::new(Mutex::new(session_id.clone()));
//     let pending_commands = Arc::new(Mutex::new(HashMap::<String, PendingShellCommand>::new()));
//
//     let session = ShellSession {
//         cwd: cwd.clone(),
//         default_user_login,
//         default_branch,
//         repo_name,
//         server_config: request.server_config,
//         stdin: Arc::clone(&stdin_ref),
//         child: Arc::clone(&child_ref),
//         active_command_id: Arc::clone(&active_command_id),
//         pending_commands: Arc::clone(&pending_commands),
//     };
//
//     {
//         let mut sessions = shell_manager
//             .sessions
//             .lock()
//             .map_err(|_| "Shell session lock poisoned".to_string())?;
//         sessions.insert(session_id.clone(), session);
//     }
//
//     let app_out = app.clone();
//     let active_command_id_out = Arc::clone(&active_command_id);
//     let session_id_out = session_id.clone();
//     let outbox_ref = Arc::clone(&outbox);
//     let stdout_preview_by_cmd = Arc::new(Mutex::new(HashMap::<String, Vec<String>>::new()));
//     let stderr_preview_by_cmd = Arc::new(Mutex::new(HashMap::<String, Vec<String>>::new()));
//     let stdout_preview_out = Arc::clone(&stdout_preview_by_cmd);
//     let stdout_preview_done = Arc::clone(&stdout_preview_by_cmd);
//     let stderr_preview_done = Arc::clone(&stderr_preview_by_cmd);
//     std::thread::spawn(move || {
//         let reader = BufReader::new(stdout);
//         for text in reader.lines().map_while(Result::ok) {
//             if let Some((marker_command_id, exit_code)) = parse_shell_exit_marker(&text) {
//                 let _ = app_out.emit(
//                     "gitgov:cli-finished",
//                     CliFinishedEvent {
//                         command_id: marker_command_id.clone(),
//                         exit_code,
//                     },
//                 );
//
//                 let pending = pending_commands
//                     .lock()
//                     .ok()
//                     .and_then(|mut m| m.remove(&marker_command_id));
//                 if let Some(pending_cmd) = pending {
//                     let stdout_preview = stdout_preview_done
//                         .lock()
//                         .ok()
//                         .and_then(|mut previews| previews.remove(&marker_command_id))
//                         .unwrap_or_default();
//                     let stderr_preview = stderr_preview_done
//                         .lock()
//                         .ok()
//                         .and_then(|mut previews| previews.remove(&marker_command_id))
//                         .unwrap_or_default();
//                     queue_cli_completion_audit(
//                         &outbox_ref,
//                         &pending_cmd,
//                         &marker_command_id,
//                         exit_code,
//                         &stdout_preview,
//                         &stderr_preview,
//                     );
//                 } else {
//                     let _ = stdout_preview_done
//                         .lock()
//                         .ok()
//                         .and_then(|mut previews| previews.remove(&marker_command_id));
//                     let _ = stderr_preview_done
//                         .lock()
//                         .ok()
//                         .and_then(|mut previews| previews.remove(&marker_command_id));
//                 }
//                 continue;
//             }
//
//             let command_id = active_command_id_out
//                 .lock()
//                 .map(|g| g.clone())
//                 .unwrap_or_else(|_| session_id_out.clone());
//
//             if let Ok(mut previews) = stdout_preview_out.lock() {
//                 let lines = previews.entry(command_id.clone()).or_default();
//                 lines.push(text.clone());
//                 if lines.len() > 20 {
//                     let overflow = lines.len() - 20;
//                     lines.drain(0..overflow);
//                 }
//             }
//
//             let _ = app_out.emit(
//                 "gitgov:cli-output",
//                 CliOutputEvent {
//                     command_id,
//                     line_type: "stdout".to_string(),
//                     text,
//                 },
//             );
//         }
//     });
//
//     let app_err = app.clone();
//     let active_command_id_err = Arc::clone(&active_command_id);
//     let session_id_err = session_id.clone();
//     let stderr_preview_err = Arc::clone(&stderr_preview_by_cmd);
//     std::thread::spawn(move || {
//         let reader = BufReader::new(stderr);
//         for text in reader.lines().map_while(Result::ok) {
//             if parse_shell_exit_marker(&text).is_some() {
//                 continue;
//             }
//             let command_id = active_command_id_err
//                 .lock()
//                 .map(|g| g.clone())
//                 .unwrap_or_else(|_| session_id_err.clone());
//             if let Ok(mut previews) = stderr_preview_err.lock() {
//                 let lines = previews.entry(command_id.clone()).or_default();
//                 lines.push(text.clone());
//                 if lines.len() > 20 {
//                     let overflow = lines.len() - 20;
//                     lines.drain(0..overflow);
//                 }
//             }
//             let _ = app_err.emit(
//                 "gitgov:cli-output",
//                 CliOutputEvent {
//                     command_id,
//                     line_type: "stderr".to_string(),
//                     text,
//                 },
//             );
//         }
//     });
//
//     emit_system_line(
//         &app,
//         &session_id,
//         format!("Shell session started ({}) in {}", shell_name, cwd),
//     );
//
//     Ok(CliShellStartResult {
//         session_id,
//         shell: shell_name.to_string(),
//     })
// }
//
// #[tauri::command]
// pub fn cmd_send_shell_input(
//     app: tauri::AppHandle,
//     request: CliShellInputRequest,
//     shell_manager: State<'_, CliShellManager>,
//     outbox: State<'_, Arc<Outbox>>,
// ) -> Result<CliShellInputResult, String> {
//     let input = request.input.trim().to_string();
//     if input.is_empty() {
//         return Ok(CliShellInputResult {
//             command_id: String::new(),
//             accepted: false,
//             error: Some("Empty input".to_string()),
//         });
//     }
//
//     let command_id = uuid::Uuid::new_v4().to_string();
//     let (
//         stdin_ref,
//         active_command_id,
//         pending_commands,
//         default_user_login,
//         default_branch,
//         repo_name,
//         server_config,
//         cwd,
//     ) = {
//         let sessions = shell_manager
//             .sessions
//             .lock()
//             .map_err(|_| "Shell session lock poisoned".to_string())?;
//         let session = sessions
//             .get(&request.session_id)
//             .ok_or_else(|| "Shell session not found".to_string())?;
//         (
//             Arc::clone(&session.stdin),
//             Arc::clone(&session.active_command_id),
//             Arc::clone(&session.pending_commands),
//             session.default_user_login.clone(),
//             session.default_branch.clone(),
//             session.repo_name.clone(),
//             session.server_config.clone(),
//             session.cwd.clone(),
//         )
//     };
//
//     {
//         let map = pending_commands
//             .lock()
//             .map_err(|_| "Shell pending command lock poisoned".to_string())?;
//         if !map.is_empty() {
//             return Ok(CliShellInputResult {
//                 command_id,
//                 accepted: false,
//                 error: Some(
//                     "Another shell command is still running; wait for it to finish before sending the next command."
//                         .to_string(),
//                 ),
//             });
//         }
//     }
//
//     let resolved_user_login = request
//         .user_login
//         .map(|v| v.trim().to_string())
//         .filter(|v| !v.is_empty())
//         .unwrap_or(default_user_login);
//     let resolved_branch = request
//         .branch
//         .map(|v| v.trim().to_string())
//         .filter(|v| !v.is_empty())
//         .unwrap_or(default_branch);
//     let origin = request
//         .origin
//         .map(|v| v.trim().to_string())
//         .filter(|v| !v.is_empty())
//         .unwrap_or_else(|| "manual_input".to_string());
//
//     let pending = PendingShellCommand {
//         command: input.clone(),
//         origin: origin.clone(),
//         branch: resolved_branch.clone(),
//         user_login: resolved_user_login.clone(),
//         cwd,
//         repo_name: repo_name.clone(),
//         server_config: server_config.clone(),
//         started_at: Instant::now(),
//     };
//
//     queue_cli_start_audit(
//         &outbox,
//         &CliStartAuditInput {
//             user_login: &resolved_user_login,
//             branch: &resolved_branch,
//             command: &input,
//             origin: &origin,
//             command_id: &command_id,
//             execution_mode: "shell",
//             repo_name: repo_name.as_deref(),
//         },
//     );
//
//     if let Ok(mut map) = pending_commands.lock() {
//         map.insert(command_id.clone(), pending);
//     }
//
//     if let Ok(mut active) = active_command_id.lock() {
//         *active = command_id.clone();
//     }
//
//     emit_system_line(&app, &command_id, format!("$ {}", input));
//
//     let wrapped = wrap_shell_command(&input, &command_id);
//     let write_result = stdin_ref
//         .lock()
//         .map_err(|_| "Shell stdin lock poisoned".to_string())
//         .and_then(|mut stdin| {
//             stdin
//                 .write_all(wrapped.as_bytes())
//                 .map_err(|e| format!("Failed to write to shell stdin: {}", e))?;
//             stdin
//                 .flush()
//                 .map_err(|e| format!("Failed to flush shell stdin: {}", e))
//         });
//
//     if let Err(e) = write_result {
//         if let Ok(mut map) = pending_commands.lock() {
//             map.remove(&command_id);
//         }
//         let _ = app.emit(
//             "gitgov:cli-finished",
//             CliFinishedEvent {
//                 command_id: command_id.clone(),
//                 exit_code: -1,
//             },
//         );
//         return Ok(CliShellInputResult {
//             command_id,
//             accepted: false,
//             error: Some(e),
//         });
//     }
//
//     Ok(CliShellInputResult {
//         command_id,
//         accepted: true,
//         error: None,
//     })
// }
//
// #[tauri::command]
// pub fn cmd_stop_shell_session(
//     app: tauri::AppHandle,
//     session_id: String,
//     shell_manager: State<'_, CliShellManager>,
// ) -> Result<CliShellStopResult, String> {
//     let session = {
//         let mut sessions = shell_manager
//             .sessions
//             .lock()
//             .map_err(|_| "Shell session lock poisoned".to_string())?;
//         sessions.remove(&session_id)
//     };
//
//     if let Some(session) = session {
//         if let Ok(mut stdin) = session.stdin.lock() {
//             let _ = stdin.write_all(b"exit\n");
//             let _ = stdin.flush();
//         }
//         if let Ok(mut child) = session.child.lock() {
//             let _ = child.kill();
//             let _ = child.wait();
//         }
//         emit_system_line(&app, &session_id, "Shell session stopped");
//         return Ok(CliShellStopResult { stopped: true });
//     }
//
//     Ok(CliShellStopResult { stopped: false })
// }
//
// ===== PART 4 -> src/commands/cli_commands/native_terminal.rs lines 1115-1320 =====
// #[tauri::command]
// pub fn cmd_start_native_terminal(
//     app: tauri::AppHandle,
//     request: CliNativeTerminalStartRequest,
//     native_manager: State<'_, CliNativeTerminalManager>,
// ) -> Result<CliNativeTerminalStartResult, String> {
//     if !native_terminal_enabled() {
//         return Err(format!(
//             "Native terminal is disabled by {}=false",
//             ENV_ENABLE_NATIVE_TERMINAL
//         ));
//     }
//
//     let cwd = request.cwd.trim();
//     if cwd.is_empty() {
//         return Err("cwd is required".to_string());
//     }
//     if !Path::new(cwd).exists() {
//         return Err(format!("cwd does not exist: {}", cwd));
//     }
//
//     let pty_system = native_pty_system();
//     let pair = pty_system
//         .openpty(native_terminal_size(request.cols, request.rows))
//         .map_err(|e| format!("Failed to open PTY: {}", e))?;
//
//     let (mut command, shell_name) = build_native_terminal_command(request.shell.as_deref());
//     command.cwd(cwd);
//     command.env("TERM", "xterm-256color");
//
//     let child = pair
//         .slave
//         .spawn_command(command)
//         .map_err(|e| format!("Failed to spawn native terminal shell: {}", e))?;
//     let mut child_killer = child.clone_killer();
//
//     let mut reader = match pair.master.try_clone_reader() {
//         Ok(reader) => reader,
//         Err(e) => {
//             let _ = child_killer.kill();
//             return Err(format!("Failed to clone PTY reader: {}", e));
//         }
//     };
//     let writer = match pair.master.take_writer() {
//         Ok(writer) => writer,
//         Err(e) => {
//             let _ = child_killer.kill();
//             return Err(format!("Failed to take PTY writer: {}", e));
//         }
//     };
//
//     let session_id = uuid::Uuid::new_v4().to_string();
//     let child_killer = Arc::new(Mutex::new(child_killer));
//
//     let session = NativeTerminalSession {
//         master: Arc::new(Mutex::new(pair.master)),
//         writer: Arc::new(Mutex::new(writer)),
//         killer: Arc::clone(&child_killer),
//     };
//
//     {
//         let mut sessions = native_manager
//             .sessions
//             .lock()
//             .map_err(|_| "Native terminal lock poisoned".to_string())?;
//         sessions.insert(session_id.clone(), session);
//     }
//
//     let app_output = app.clone();
//     let session_output = session_id.clone();
//     std::thread::spawn(move || {
//         let mut buffer = [0u8; 8192];
//         loop {
//             match reader.read(&mut buffer) {
//                 Ok(0) => break,
//                 Ok(read_bytes) => {
//                     let text = String::from_utf8_lossy(&buffer[..read_bytes]).to_string();
//                     let _ = app_output.emit(
//                         "gitgov:pty-output",
//                         CliNativeTerminalOutputEvent {
//                             session_id: session_output.clone(),
//                             data: text,
//                         },
//                     );
//                 }
//                 Err(_) => break,
//             }
//         }
//     });
//
//     let app_exit = app.clone();
//     let session_exit = session_id.clone();
//     let sessions_ref = Arc::clone(&native_manager.sessions);
//     std::thread::spawn(move || {
//         let mut child = child;
//         let exit_code = child
//             .wait()
//             .ok()
//             .map(|status| i32::try_from(status.exit_code()).unwrap_or(-1))
//             .unwrap_or(-1);
//
//         let _ = app_exit.emit(
//             "gitgov:pty-exit",
//             CliNativeTerminalExitEvent {
//                 session_id: session_exit.clone(),
//                 exit_code,
//             },
//         );
//
//         if let Ok(mut sessions) = sessions_ref.lock() {
//             sessions.remove(&session_exit);
//         }
//     });
//
//     Ok(CliNativeTerminalStartResult {
//         session_id,
//         shell: shell_name,
//     })
// }
//
// #[tauri::command]
// pub fn cmd_write_native_terminal(
//     request: CliNativeTerminalWriteRequest,
//     native_manager: State<'_, CliNativeTerminalManager>,
// ) -> Result<(), String> {
//     let writer = {
//         let sessions = native_manager
//             .sessions
//             .lock()
//             .map_err(|_| "Native terminal lock poisoned".to_string())?;
//         let session = sessions
//             .get(&request.session_id)
//             .ok_or_else(|| "Native terminal session not found".to_string())?;
//         Arc::clone(&session.writer)
//     };
//
//     let mut writer = writer
//         .lock()
//         .map_err(|_| "Native terminal writer lock poisoned".to_string())?;
//     writer
//         .write_all(request.data.as_bytes())
//         .map_err(|e| format!("Failed to write PTY input: {}", e))?;
//     writer
//         .flush()
//         .map_err(|e| format!("Failed to flush PTY input: {}", e))
// }
//
// #[tauri::command]
// pub fn cmd_resize_native_terminal(
//     request: CliNativeTerminalResizeRequest,
//     native_manager: State<'_, CliNativeTerminalManager>,
// ) -> Result<(), String> {
//     let master = {
//         let sessions = native_manager
//             .sessions
//             .lock()
//             .map_err(|_| "Native terminal lock poisoned".to_string())?;
//         let session = sessions
//             .get(&request.session_id)
//             .ok_or_else(|| "Native terminal session not found".to_string())?;
//         Arc::clone(&session.master)
//     };
//
//     let master = master
//         .lock()
//         .map_err(|_| "Native terminal master lock poisoned".to_string())?;
//     master
//         .resize(PtySize {
//             rows: request.rows.max(5),
//             cols: request.cols.max(20),
//             pixel_width: 0,
//             pixel_height: 0,
//         })
//         .map_err(|e| format!("Failed to resize PTY: {}", e))
// }
//
// #[tauri::command]
// pub fn cmd_stop_native_terminal(
//     session_id: String,
//     native_manager: State<'_, CliNativeTerminalManager>,
// ) -> Result<CliNativeTerminalStopResult, String> {
//     let session = {
//         let mut sessions = native_manager
//             .sessions
//             .lock()
//             .map_err(|_| "Native terminal lock poisoned".to_string())?;
//         sessions.remove(&session_id)
//     };
//
//     if let Some(session) = session {
//         if let Ok(mut killer) = session.killer.lock() {
//             let _ = killer.kill();
//         }
//         return Ok(CliNativeTerminalStopResult { stopped: true });
//     }
//
//     Ok(CliNativeTerminalStopResult { stopped: false })
// }
//
// /// Execute a CLI command with real-time streaming via Tauri events.
// ///
// /// The command runs in a background thread. Each line of stdout/stderr is emitted
// /// as a `gitgov:cli-output` event. When the command finishes, `gitgov:cli-finished`
// /// is emitted with the exit code.
// ///
// /// Returns immediately with the command_id (or an error if the command is not allowed).
// ===== PART 5 -> src/commands/cli_commands/execute.rs lines 1321-1599 =====
// #[tauri::command]
// pub fn cmd_execute_cli(
//     app: tauri::AppHandle,
//     request: CliExecuteRequest,
//     outbox: State<'_, Arc<Outbox>>,
// ) -> Result<CliStartResult, String> {
//     let command = request.command;
//     let cwd = request.cwd;
//     let origin = request.origin;
//     let user_login = request.user_login;
//     let branch = request.branch;
//     let server_config = request.server_config;
//     let execution_mode = request.execution_mode;
//
//     let command_id = uuid::Uuid::new_v4().to_string();
//     let resolved_user_login = resolve_user_login(user_login);
//     let resolved_branch = resolve_branch(branch, &cwd);
//
//     if execution_mode == CliExecutionMode::Shell && !shell_commands_enabled() {
//         let policy_error = format!(
//             "Shell command mode disabled by {} policy",
//             ENV_ENABLE_SHELL_COMMANDS
//         );
//         let _ = app.emit(
//             "gitgov:cli-output",
//             CliOutputEvent {
//                 command_id: command_id.clone(),
//                 line_type: "system".to_string(),
//                 text: policy_error.clone(),
//             },
//         );
//         let _ = app.emit(
//             "gitgov:cli-finished",
//             CliFinishedEvent {
//                 command_id: command_id.clone(),
//                 exit_code: -1,
//             },
//         );
//         return Ok(CliStartResult {
//             command_id,
//             allowed: false,
//             error: Some(policy_error),
//         });
//     }
//
//     // Validate against whitelist
//     if execution_mode == CliExecutionMode::Safe && !is_command_allowed(&command) {
//         let _ = app.emit(
//             "gitgov:cli-output",
//             CliOutputEvent {
//                 command_id: command_id.clone(),
//                 line_type: "system".to_string(),
//                 text: format!(
//                     "Command not allowed: '{}'. Allowed prefixes: {:?}",
//                     command, DEFAULT_ALLOWED_PREFIXES
//                 ),
//             },
//         );
//         let _ = app.emit(
//             "gitgov:cli-finished",
//             CliFinishedEvent {
//                 command_id: command_id.clone(),
//                 exit_code: -1,
//             },
//         );
//         return Ok(CliStartResult {
//             command_id,
//             allowed: false,
//             error: Some("Command not in whitelist".to_string()),
//         });
//     }
//
//     // Emit the command itself as a system line
//     let _ = app.emit(
//         "gitgov:cli-output",
//         CliOutputEvent {
//             command_id: command_id.clone(),
//             line_type: "system".to_string(),
//             text: format!("$ {}", redact_sensitive_cli_text(&command)),
//         },
//     );
//
//     // Spawn process
//     let mut child = match execution_mode {
//         CliExecutionMode::Safe => spawn_safe_child(&command, &cwd)?,
//         CliExecutionMode::Shell => spawn_shell_child(&command, &cwd)?,
//     };
//
//     let stdout = child.stdout.take();
//     let stderr = child.stderr.take();
//
//     let cmd_id = command_id.clone();
//     let app_clone = app.clone();
//
//     // Create audit event for the outbox
//     let mut audit_event = OutboxEvent::new(
//         "cli_command".to_string(),
//         resolved_user_login.clone(),
//         Some(resolved_branch.clone()),
//         AuditStatus::Success,
//     )
//     .with_metadata(serde_json::json!({
//         "command": redact_sensitive_cli_text(&command),
//         "origin": origin.clone(),
//         "branch": resolved_branch.clone(),
//         "command_id": command_id.clone(),
//         "execution_mode": execution_mode.as_str(),
//     }));
//
//     let mut repo_full_name: Option<String> = None;
//
//     // Try to infer repo name from cwd
//     if let Ok(repo) = git2::Repository::open(&cwd) {
//         if let Some(full_name) = super::git_commands::infer_repo_full_name_pub(&repo) {
//             repo_full_name = Some(full_name.clone());
//             audit_event = audit_event.with_repo(full_name.clone());
//             if let Some(org) = full_name.split('/').next().map(ToOwned::to_owned) {
//                 audit_event = audit_event.with_org(org);
//             }
//         }
//     }
//     let _ = outbox.add(audit_event);
//
//     // Background thread: stream stdout + stderr, then emit finished
//     let outbox_ref = Arc::clone(&outbox);
//     let user = resolved_user_login.clone();
//     let cmd_str = command.clone();
//     let origin_str = origin.clone();
//     let branch_str = resolved_branch.clone();
//     let execution_mode_str = execution_mode.as_str().to_string();
//     let cwd_str = cwd.clone();
//     let repo_name_for_audit = repo_full_name;
//     let cp_config_for_audit = server_config;
//     let command_started_at = Instant::now();
//
//     std::thread::spawn(move || {
//         let stdout_preview = Arc::new(Mutex::new(Vec::<String>::new()));
//         let stderr_preview = Arc::new(Mutex::new(Vec::<String>::new()));
//         let mut stream_handles = Vec::new();
//
//         // Stream stdout
//         if let Some(out) = stdout {
//             let app_stream = app_clone.clone();
//             let stream_command_id = cmd_id.clone();
//             let preview_ref = Arc::clone(&stdout_preview);
//             let reader = BufReader::new(out);
//             stream_handles.push(std::thread::spawn(move || {
//                 for text in reader.lines().map_while(Result::ok) {
//                     if let Ok(mut preview) = preview_ref.lock() {
//                         if preview.len() < 20 {
//                             preview.push(text.clone());
//                         }
//                     }
//                     let _ = app_stream.emit(
//                         "gitgov:cli-output",
//                         CliOutputEvent {
//                             command_id: stream_command_id.clone(),
//                             line_type: "stdout".to_string(),
//                             text,
//                         },
//                     );
//                 }
//             }));
//         }
//
//         // Stream stderr
//         if let Some(err) = stderr {
//             let app_stream = app_clone.clone();
//             let stream_command_id = cmd_id.clone();
//             let preview_ref = Arc::clone(&stderr_preview);
//             let reader = BufReader::new(err);
//             stream_handles.push(std::thread::spawn(move || {
//                 for text in reader.lines().map_while(Result::ok) {
//                     if let Ok(mut preview) = preview_ref.lock() {
//                         if preview.len() < 20 {
//                             preview.push(text.clone());
//                         }
//                     }
//                     let _ = app_stream.emit(
//                         "gitgov:cli-output",
//                         CliOutputEvent {
//                             command_id: stream_command_id.clone(),
//                             line_type: "stderr".to_string(),
//                             text,
//                         },
//                     );
//                 }
//             }));
//         }
//
//         // Wait for process to finish
//         let exit_code = match child.wait() {
//             Ok(status) => status.code().unwrap_or(-1),
//             Err(_) => -1,
//         };
//         for handle in stream_handles {
//             let _ = handle.join();
//         }
//         let elapsed_ms = command_started_at.elapsed().as_millis() as i64;
//         let stdout_preview = stdout_preview.lock().map(|v| v.clone()).unwrap_or_default();
//         let stderr_preview = stderr_preview.lock().map(|v| v.clone()).unwrap_or_default();
//         let safe_stdout_preview = redact_cli_preview_lines(&stdout_preview);
//         let safe_stderr_preview = redact_cli_preview_lines(&stderr_preview);
//
//         let _ = app_clone.emit(
//             "gitgov:cli-finished",
//             CliFinishedEvent {
//                 command_id: cmd_id.clone(),
//                 exit_code,
//             },
//         );
//
//         // Audit: record completion
//         let audit_status = if exit_code == 0 {
//             AuditStatus::Success
//         } else {
//             AuditStatus::Failed
//         };
//         let mut done_event = OutboxEvent::new(
//             "cli_command_completed".to_string(),
//             user.clone(),
//             Some(branch_str.clone()),
//             audit_status,
//         )
//         .with_metadata(serde_json::json!({
//             "command": redact_sensitive_cli_text(&cmd_str),
//             "origin": origin_str,
//             "branch": branch_str,
//             "exit_code": exit_code,
//             "execution_mode": execution_mode_str,
//             "command_id": cmd_id.clone(),
//         }));
//         if let Ok(repo) = git2::Repository::open(&cwd_str) {
//             if let Some(full_name) = super::git_commands::infer_repo_full_name_pub(&repo) {
//                 done_event = done_event.with_repo(full_name.clone());
//                 if let Some(org) = full_name.split('/').next().map(ToOwned::to_owned) {
//                     done_event = done_event.with_org(org);
//                 }
//             }
//         }
//         let _ = outbox_ref.add(done_event);
//         outbox_ref.notify_flush();
//
//         // Direct control-plane audit trail for CLI command history endpoint.
//         if let Some(cfg) = cp_config_for_audit {
//             if !cfg.url.trim().is_empty() {
//                 let client = ControlPlaneClient::new(ServerConfig {
//                     url: cfg.url,
//                     api_key: cfg.api_key,
//                 });
//                 let payload = CliCommandInput {
//                     command: redact_sensitive_cli_text(&cmd_str),
//                     origin: origin_str.clone(),
//                     branch: branch_str.clone(),
//                     repo_name: repo_name_for_audit.clone(),
//                     exit_code: Some(exit_code),
//                     duration_ms: Some(elapsed_ms),
//                     metadata: serde_json::json!({
//                         "command_id": cmd_id.clone(),
//                         "execution_mode": execution_mode_str,
//                         "stdout_preview": safe_stdout_preview,
//                         "stderr_preview": safe_stderr_preview,
//                     }),
//                 };
//                 if let Err(e) = client.ingest_cli_command(&payload) {
//                     tracing::warn!(error = %e, "CLI command audit ingestion failed");
//                 }
//             }
//         }
//     });
//
//     Ok(CliStartResult {
//         command_id,
//         allowed: true,
//         error: None,
//     })
// }
//
// /// Get the list of currently allowed command prefixes.
// ===== PART 6 -> src/commands/cli_commands/pipeline.rs lines 1600-1785 =====
// #[tauri::command]
// pub fn cmd_get_cli_whitelist() -> Vec<String> {
//     DEFAULT_ALLOWED_PREFIXES
//         .iter()
//         .map(|s| s.to_string())
//         .collect()
// }
//
// /// Build pipeline graph data for the current branch + develop/main.
// /// Returns commit history, branch relationships, and linked ticket/PR/pipeline info.
// #[tauri::command]
// pub fn cmd_get_pipeline_graph(
//     repo_path: String,
//     max_commits: Option<usize>,
// ) -> Result<serde_json::Value, String> {
//     let repo =
//         git2::Repository::open(&repo_path).map_err(|e| format!("Failed to open repo: {}", e))?;
//
//     let head = repo.head().map_err(|e| format!("No HEAD: {}", e))?;
//     let current_branch = head.shorthand().unwrap_or("HEAD").to_string();
//
//     let max = max_commits.unwrap_or(50);
//
//     // Walk current branch commits
//     let mut revwalk = repo
//         .revwalk()
//         .map_err(|e| format!("Revwalk error: {}", e))?;
//     revwalk
//         .push_head()
//         .map_err(|e| format!("Push head: {}", e))?;
//     revwalk
//         .set_sorting(git2::Sort::TIME)
//         .map_err(|e| format!("Sort error: {}", e))?;
//
//     let mut commits: Vec<serde_json::Value> = Vec::new();
//     for (i, oid) in revwalk.enumerate() {
//         if i >= max {
//             break;
//         }
//         if let Ok(oid) = oid {
//             if let Ok(commit) = repo.find_commit(oid) {
//                 let sha = oid.to_string();
//                 let short_sha = &sha[..7.min(sha.len())];
//                 let message = commit.message().unwrap_or("").to_string();
//                 let summary = commit.summary().unwrap_or("").to_string();
//                 let author = commit.author().name().unwrap_or("unknown").to_string();
//                 let time = commit.time().seconds();
//
//                 commits.push(serde_json::json!({
//                     "sha": sha,
//                     "short_sha": short_sha,
//                     "message": message.trim(),
//                     "summary": summary.trim(),
//                     "author": author,
//                     "timestamp": time,
//                     "branch": current_branch,
//                 }));
//             }
//         }
//     }
//
//     // Find target branches (develop, main, master)
//     let target_branches: Vec<String> = ["develop", "main", "master"]
//         .iter()
//         .filter(|name| {
//             repo.find_branch(name, git2::BranchType::Local).is_ok()
//                 || repo
//                     .find_branch(&format!("origin/{}", name), git2::BranchType::Remote)
//                     .is_ok()
//         })
//         .map(|s| s.to_string())
//         .collect();
//
//     Ok(serde_json::json!({
//         "current_branch": current_branch,
//         "target_branches": target_branches,
//         "commits": commits,
//     }))
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn allowed_commands() {
//         assert!(is_command_allowed("git status"));
//         assert!(is_command_allowed("git log --oneline"));
//         assert!(is_command_allowed("git remote -v"));
//         assert!(is_command_allowed("gitgov status"));
//         assert!(is_command_allowed("git"));
//     }
//
//     #[test]
//     fn blocked_commands() {
//         assert!(!is_command_allowed("rm -rf /"));
//         assert!(!is_command_allowed("npm install"));
//         assert!(!is_command_allowed("cargo build"));
//         assert!(!is_command_allowed("curl http://evil.com"));
//         assert!(!is_command_allowed(""));
//         assert!(!is_command_allowed("  "));
//     }
//
//     #[test]
//     fn split_command_line_preserves_quoted_arguments() {
//         let parts = split_command_line(r#"git commit -m "hello world""#).unwrap();
//
//         assert_eq!(parts, vec!["git", "commit", "-m", "hello world"]);
//     }
//
//     #[test]
//     fn split_command_line_preserves_windows_paths_inside_quotes() {
//         let parts = split_command_line(r#"git -C "C:\Users\PC\Desktop\Git Gov" status"#).unwrap();
//
//         assert_eq!(
//             parts,
//             vec!["git", "-C", r#"C:\Users\PC\Desktop\Git Gov"#, "status"]
//         );
//     }
//
//     #[test]
//     fn split_command_line_preserves_empty_quoted_argument() {
//         let parts = split_command_line(r#"git commit --allow-empty-message -m """#).unwrap();
//
//         assert_eq!(
//             parts,
//             vec!["git", "commit", "--allow-empty-message", "-m", ""]
//         );
//     }
//
//     #[test]
//     fn split_command_line_rejects_unclosed_quote() {
//         let error = split_command_line(r#"git commit -m "unfinished"#).unwrap_err();
//
//         assert!(error.contains("Unclosed"));
//     }
//
//     #[test]
//     fn parse_env_flag_value_handles_expected_variants() {
//         assert!(parse_env_flag_value("true", false));
//         assert!(parse_env_flag_value("  YES  ", false));
//         assert!(parse_env_flag_value("1", false));
//         assert!(!parse_env_flag_value("false", true));
//         assert!(!parse_env_flag_value("No", true));
//         assert!(!parse_env_flag_value("0", true));
//         assert!(parse_env_flag_value("invalid", true));
//         assert!(!parse_env_flag_value("invalid", false));
//     }
//
//     #[test]
//     fn parse_shell_exit_marker_accepts_valid_marker() {
//         let parsed = parse_shell_exit_marker("__GITGOV_EXIT__:abc-123:7");
//         assert_eq!(parsed, Some(("abc-123".to_string(), 7)));
//     }
//
//     #[test]
//     fn parse_shell_exit_marker_rejects_non_marker_output() {
//         assert_eq!(parse_shell_exit_marker("ordinary command output"), None);
//         assert_eq!(parse_shell_exit_marker("__GITGOV_EXIT__::0"), None);
//         assert_eq!(parse_shell_exit_marker("__GITGOV_EXIT__:abc:nope"), None);
//     }
//
//     #[test]
//     fn redacts_sensitive_cli_audit_text() {
//         let text = "git remote set-url origin https://token123@github.com/acme/repo.git && echo GITGOV_API_KEY=abc123 ghp_secretvalue";
//         let redacted = redact_sensitive_cli_text(text);
//
//         assert!(!redacted.contains("token123"));
//         assert!(!redacted.contains("abc123"));
//         assert!(!redacted.contains("ghp_secretvalue"));
//         assert!(redacted.contains("https://[REDACTED]@github.com/acme/repo.git"));
//         assert!(redacted.contains("GITGOV_API_KEY=[REDACTED]"));
//         assert!(redacted.contains("[REDACTED_SECRET]"));
//     }
//
//     #[cfg(target_os = "windows")]
//     #[test]
//     fn windows_shell_wrapper_captures_powershell_and_native_exit_status() {
//         let wrapped = wrap_shell_command("Write-Error 'boom'", "cmd-1");
//
//         assert!(wrapped.contains("$global:LASTEXITCODE = $null"));
//         assert!(wrapped.contains("$ggSucceeded = $?"));
//         assert!(wrapped.contains("$ggLastExit = $LASTEXITCODE"));
//         assert!(wrapped.contains("__GITGOV_EXIT__:cmd-1:$ggEc"));
//     }
// }
