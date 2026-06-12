use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsString;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{Emitter, State};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};

use crate::control_plane::{CliCommandInput, ControlPlaneClient, ServerConfig};
use crate::models::AuditStatus;
use crate::outbox::{Outbox, OutboxEvent};

use super::server_commands::ServerConnectionConfig;

/// Safe-mode Git subcommands.
/// Admin-configurable in the future; hardcoded for MVP.
const DEFAULT_ALLOWED_GIT_SUBCOMMANDS: &[&str] = &[
    "branch",
    "log",
    "remote",
    "rev-parse",
    "status",
];
const SHELL_EXIT_MARKER_PREFIX: &str = "__GITGOV_EXIT__:";
const ENV_ENABLE_SHELL_COMMANDS: &str = "GITGOV_ENABLE_SHELL_COMMANDS";
const ENV_ENABLE_NATIVE_TERMINAL: &str = "GITGOV_ENABLE_NATIVE_TERMINAL";

/// Payload emitted per line of CLI output via Tauri event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliOutputEvent {
    pub command_id: String,
    pub line_type: String, // "stdout" | "stderr" | "system"
    pub text: String,
}

/// Payload emitted when CLI command finishes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliFinishedEvent {
    pub command_id: String,
    pub exit_code: i32,
}

/// Result returned to the frontend immediately when a command starts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliStartResult {
    pub command_id: String,
    pub allowed: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliExecuteRequest {
    pub command: String,
    pub cwd: String,
    #[serde(default)]
    pub user_login: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    pub origin: String,
    #[serde(default)]
    pub server_config: Option<ServerConnectionConfig>,
    #[serde(default)]
    pub execution_mode: CliExecutionMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliShellStartRequest {
    pub cwd: String,
    #[serde(default)]
    pub user_login: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub server_config: Option<ServerConnectionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliShellStartResult {
    pub session_id: String,
    pub shell: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliShellInputRequest {
    pub session_id: String,
    pub input: String,
    #[serde(default)]
    pub user_login: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub origin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliShellInputResult {
    pub command_id: String,
    pub accepted: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliShellStopResult {
    pub stopped: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliNativeTerminalStartRequest {
    pub cwd: String,
    #[serde(default)]
    pub cols: Option<u16>,
    #[serde(default)]
    pub rows: Option<u16>,
    #[serde(default)]
    pub shell: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliNativeTerminalStartResult {
    pub session_id: String,
    pub shell: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliNativeTerminalWriteRequest {
    pub session_id: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliNativeTerminalResizeRequest {
    pub session_id: String,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliNativeTerminalStopResult {
    pub stopped: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliNativeTerminalOutputEvent {
    pub session_id: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliNativeTerminalExitEvent {
    pub session_id: String,
    pub exit_code: i32,
}

#[derive(Debug)]
struct PendingShellCommand {
    command: String,
    origin: String,
    branch: String,
    user_login: String,
    cwd: String,
    repo_name: Option<String>,
    server_config: Option<ServerConnectionConfig>,
    started_at: Instant,
}

struct CliStartAuditInput<'a> {
    user_login: &'a str,
    branch: &'a str,
    command: &'a str,
    origin: &'a str,
    command_id: &'a str,
    execution_mode: &'a str,
    repo_name: Option<&'a str>,
}

#[derive(Debug)]
struct ShellSession {
    cwd: String,
    default_user_login: String,
    default_branch: String,
    repo_name: Option<String>,
    server_config: Option<ServerConnectionConfig>,
    stdin: Arc<Mutex<ChildStdin>>,
    child: Arc<Mutex<Child>>,
    active_command_id: Arc<Mutex<String>>,
    pending_commands: Arc<Mutex<HashMap<String, PendingShellCommand>>>,
}

struct NativeTerminalSession {
    master: Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    killer: Arc<Mutex<Box<dyn portable_pty::ChildKiller + Send + Sync>>>,
}

#[derive(Default)]
pub struct CliShellManager {
    sessions: Mutex<HashMap<String, ShellSession>>,
}

#[derive(Default)]
pub struct CliNativeTerminalManager {
    sessions: Arc<Mutex<HashMap<String, NativeTerminalSession>>>,
}
