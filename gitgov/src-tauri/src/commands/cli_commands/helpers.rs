fn is_command_allowed(command: &str) -> bool {
    let trimmed = command.trim();
    DEFAULT_ALLOWED_PREFIXES
        .iter()
        .any(|prefix| trimmed == *prefix || trimmed.starts_with(&format!("{} ", prefix)))
}

fn split_command_line(command: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut token_started = false;
    let mut chars = command.trim().chars().peekable();

    while let Some(ch) = chars.next() {
        match quote {
            Some(active_quote) => {
                if ch == active_quote {
                    quote = None;
                    token_started = true;
                } else if active_quote == '"' && ch == '\\' {
                    match chars.peek().copied() {
                        Some('"') | Some('\\') => {
                            if let Some(next) = chars.next() {
                                current.push(next);
                            }
                        }
                        _ => current.push(ch),
                    }
                    token_started = true;
                } else {
                    current.push(ch);
                    token_started = true;
                }
            }
            None => {
                if ch == '"' || ch == '\'' {
                    quote = Some(ch);
                    token_started = true;
                } else if ch.is_whitespace() {
                    if token_started {
                        args.push(std::mem::take(&mut current));
                        token_started = false;
                    }
                } else {
                    current.push(ch);
                    token_started = true;
                }
            }
        }
    }

    if let Some(active_quote) = quote {
        return Err(format!("Unclosed {} quote in command", active_quote));
    }
    if token_started {
        args.push(current);
    }

    Ok(args)
}

fn env_flag_enabled(var_name: &str, default_value: bool) -> bool {
    let raw_value = match std::env::var(var_name) {
        Ok(value) => value,
        Err(_) => return default_value,
    };

    parse_env_flag_value(&raw_value, default_value)
}

fn parse_env_flag_value(raw_value: &str, default_value: bool) -> bool {
    match raw_value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        _ => default_value,
    }
}

fn shell_commands_enabled() -> bool {
    env_flag_enabled(ENV_ENABLE_SHELL_COMMANDS, true)
}

fn native_terminal_enabled() -> bool {
    env_flag_enabled(ENV_ENABLE_NATIVE_TERMINAL, true)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CliExecutionMode {
    #[default]
    Safe,
    Shell,
}

impl CliExecutionMode {
    fn as_str(self) -> &'static str {
        match self {
            CliExecutionMode::Safe => "safe",
            CliExecutionMode::Shell => "shell",
        }
    }
}

fn parse_shell_exit_marker(text: &str) -> Option<(String, i32)> {
    let trimmed = text.trim();
    let marker = trimmed.strip_prefix(SHELL_EXIT_MARKER_PREFIX)?;
    let (command_id, exit_code) = marker.rsplit_once(':')?;
    let parsed_exit = exit_code.trim().parse::<i32>().ok()?;
    if command_id.trim().is_empty() {
        return None;
    }
    Some((command_id.trim().to_string(), parsed_exit))
}

#[cfg(target_os = "windows")]
fn wrap_shell_command(command: &str, command_id: &str) -> String {
    format!(
        "& {{ $global:LASTEXITCODE = $null; {} }}; $ggSucceeded = $?; $ggLastExit = $LASTEXITCODE; if ($null -ne $ggLastExit) {{ $ggEc = [int]$ggLastExit }} elseif ($ggSucceeded) {{ $ggEc = 0 }} else {{ $ggEc = 1 }}; Write-Output \"{}{}:$ggEc\"\n",
        command, SHELL_EXIT_MARKER_PREFIX, command_id
    )
}

#[cfg(not(target_os = "windows"))]
fn wrap_shell_command(command: &str, command_id: &str) -> String {
    format!(
        "{{ {} ; }}; __gitgov_ec=$?; printf \"{}{}:%s\\n\" \"$__gitgov_ec\"\n",
        command, SHELL_EXIT_MARKER_PREFIX, command_id
    )
}

fn native_terminal_size(cols: Option<u16>, rows: Option<u16>) -> PtySize {
    PtySize {
        rows: rows.unwrap_or(30).max(5),
        cols: cols.unwrap_or(120).max(20),
        pixel_width: 0,
        pixel_height: 0,
    }
}

#[cfg(target_os = "windows")]
fn build_native_terminal_command(shell: Option<&str>) -> (CommandBuilder, String) {
    let requested = shell
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_ascii_lowercase());

    match requested.as_deref() {
        Some("cmd") | Some("cmd.exe") => (CommandBuilder::new("cmd.exe"), "cmd".to_string()),
        Some("pwsh") | Some("pwsh.exe") => {
            let mut command = CommandBuilder::new("pwsh.exe");
            command.arg("-NoLogo");
            command.arg("-NoProfile");
            (command, "pwsh".to_string())
        }
        _ => {
            let mut command = CommandBuilder::new("powershell.exe");
            command.arg("-NoLogo");
            command.arg("-NoProfile");
            (command, "powershell".to_string())
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn build_native_terminal_command(shell: Option<&str>) -> (CommandBuilder, String) {
    let requested_shell = shell
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| std::env::var("SHELL").ok().filter(|s| !s.trim().is_empty()))
        .unwrap_or_else(|| "/bin/bash".to_string());

    let mut command = CommandBuilder::new(&requested_shell);
    command.arg("-i");

    let label = Path::new(&requested_shell)
        .file_name()
        .and_then(|v| v.to_str())
        .filter(|v| !v.trim().is_empty())
        .unwrap_or("shell")
        .to_string();

    (command, label)
}

fn spawn_safe_child(command: &str, cwd: &str) -> Result<Child, String> {
    let parts = split_command_line(command)?;
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }
    let program = &parts[0];
    let args = &parts[1..];
    Command::new(program)
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn '{}': {}", program, e))
}

#[cfg(target_os = "windows")]
fn spawn_shell_child(command: &str, cwd: &str) -> Result<Child, String> {
    Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", command])
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn PowerShell: {}", e))
}

#[cfg(not(target_os = "windows"))]
fn spawn_shell_child(command: &str, cwd: &str) -> Result<Child, String> {
    let bash = Command::new("bash")
        .args(["-lc", command])
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    match bash {
        Ok(child) => Ok(child),
        Err(bash_err) => Command::new("sh")
            .args(["-lc", command])
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|sh_err| {
                format!(
                    "Failed to spawn shell (bash error: {}; sh error: {})",
                    bash_err, sh_err
                )
            }),
    }
}

#[cfg(target_os = "windows")]
fn spawn_shell_session_child(cwd: &str) -> Result<(Child, &'static str), String> {
    Command::new("powershell")
        .args(["-NoLogo", "-NoProfile", "-NoExit"])
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map(|child| (child, "powershell"))
        .map_err(|e| format!("Failed to start PowerShell session: {}", e))
}

#[cfg(not(target_os = "windows"))]
fn spawn_shell_session_child(cwd: &str) -> Result<(Child, &'static str), String> {
    let bash = Command::new("bash")
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    match bash {
        Ok(child) => Ok((child, "bash")),
        Err(bash_err) => Command::new("sh")
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map(|child| (child, "sh"))
            .map_err(|sh_err| {
                format!(
                    "Failed to start shell session (bash error: {}; sh error: {})",
                    bash_err, sh_err
                )
            }),
    }
}

fn resolve_user_login(explicit: Option<String>) -> String {
    let value = explicit.unwrap_or_default().trim().to_string();
    if !value.is_empty() {
        return value;
    }
    super::auth_commands::load_current_user_session_login().unwrap_or_else(|| "unknown".to_string())
}

fn resolve_branch(explicit: Option<String>, cwd: &str) -> String {
    let value = explicit.unwrap_or_default().trim().to_string();
    if !value.is_empty() {
        return value;
    }

    if let Ok(repo) = git2::Repository::open(cwd) {
        if let Ok(head) = repo.head() {
            if let Some(name) = head.shorthand() {
                let normalized = name.trim().to_string();
                if !normalized.is_empty() {
                    return normalized;
                }
            }
        }
    }

    "unknown".to_string()
}

fn infer_repo_name_from_cwd(cwd: &str) -> Option<String> {
    if let Ok(repo) = git2::Repository::open(cwd) {
        return super::repo_event_context::infer_repo_full_name(&repo);
    }
    None
}

fn emit_system_line(app: &tauri::AppHandle, command_id: &str, text: impl Into<String>) {
    let _ = app.emit(
        "gitgov:cli-output",
        CliOutputEvent {
            command_id: command_id.to_string(),
            line_type: "system".to_string(),
            text: text.into(),
        },
    );
}

fn redact_url_credentials(input: &str) -> String {
    let mut output = input.to_string();
    for scheme in ["https://", "http://"] {
        let mut search_from = 0usize;
        while let Some(relative_idx) = output[search_from..].find(scheme) {
            let start = search_from + relative_idx + scheme.len();
            let end = output[start..]
                .find(|c: char| c == '/' || c.is_whitespace())
                .map(|idx| start + idx)
                .unwrap_or(output.len());
            if let Some(at_relative) = output[start..end].find('@') {
                let at = start + at_relative;
                output.replace_range(start..at, "[REDACTED]");
                search_from = start + "[REDACTED]@".len();
            } else {
                search_from = end;
            }
        }
    }
    output
}

fn redact_after_marker_case_insensitive(input: String, marker: &str) -> String {
    let mut output = input;
    let marker_lower = marker.to_ascii_lowercase();
    let mut search_from = 0usize;
    loop {
        let lower = output.to_ascii_lowercase();
        let Some(relative_idx) = lower[search_from..].find(&marker_lower) else {
            break;
        };
        let value_start = search_from + relative_idx + marker.len();
        let value_end = output[value_start..]
            .find(|c: char| {
                c.is_whitespace() || matches!(c, '"' | '\'' | '&' | ';' | ',' | ')' | ']' | '}')
            })
            .map(|idx| value_start + idx)
            .unwrap_or(output.len());
        if value_end > value_start {
            output.replace_range(value_start..value_end, "[REDACTED]");
            search_from = value_start + "[REDACTED]".len();
        } else {
            search_from = value_start;
        }
    }
    output
}

fn redact_token_prefix(input: String, prefix: &str) -> String {
    let mut output = input;
    let mut search_from = 0usize;
    while let Some(relative_idx) = output[search_from..].find(prefix) {
        let token_start = search_from + relative_idx;
        let token_end = output[token_start..]
            .find(|c: char| {
                c.is_whitespace() || matches!(c, '"' | '\'' | '&' | ';' | ',' | ')' | ']' | '}')
            })
            .map(|idx| token_start + idx)
            .unwrap_or(output.len());
        output.replace_range(token_start..token_end, "[REDACTED_SECRET]");
        search_from = token_start + "[REDACTED_SECRET]".len();
    }
    output
}

fn redact_sensitive_cli_text(input: &str) -> String {
    let mut output = redact_url_credentials(input);
    for marker in [
        "authorization: bearer ",
        "bearer ",
        "api_key=",
        "apikey=",
        "token=",
        "password=",
        "secret=",
        "github_token=",
        "gitgov_api_key=",
        "sonar_token=",
        "jira_api_token=",
        "jenkins_api_token=",
    ] {
        output = redact_after_marker_case_insensitive(output, marker);
    }
    for prefix in [
        "ghp_",
        "gho_",
        "ghu_",
        "ghs_",
        "github_pat_",
        "glpat-",
        "sk-",
    ] {
        output = redact_token_prefix(output, prefix);
    }
    output
}

fn redact_cli_preview_lines(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .map(|line| redact_sensitive_cli_text(line))
        .collect()
}

fn queue_cli_start_audit(outbox: &Arc<Outbox>, input: &CliStartAuditInput<'_>) {
    let mut event = OutboxEvent::new(
        "cli_command".to_string(),
        input.user_login.to_string(),
        Some(input.branch.to_string()),
        AuditStatus::Success,
    )
    .with_metadata(serde_json::json!({
        "command": redact_sensitive_cli_text(input.command),
        "origin": input.origin,
        "branch": input.branch,
        "command_id": input.command_id,
        "execution_mode": input.execution_mode,
    }));

    if let Some(full_name) = input.repo_name {
        event = event.with_repo(full_name.to_string());
        if let Some(org) = super::repo_event_context::infer_org_name_from_full_name(full_name) {
            event = event.with_org(org);
        }
    }
    let _ = outbox.add(event);
}

fn queue_cli_completion_audit(
    outbox: &Arc<Outbox>,
    pending: &PendingShellCommand,
    command_id: &str,
    exit_code: i32,
    stdout_preview: &[String],
    stderr_preview: &[String],
) {
    let audit_status = if exit_code == 0 {
        AuditStatus::Success
    } else {
        AuditStatus::Failed
    };
    let duration_ms = pending.started_at.elapsed().as_millis() as i64;

    let mut done_event = OutboxEvent::new(
        "cli_command_completed".to_string(),
        pending.user_login.clone(),
        Some(pending.branch.clone()),
        audit_status,
    )
    .with_metadata(serde_json::json!({
        "command": redact_sensitive_cli_text(&pending.command),
        "origin": pending.origin,
        "branch": pending.branch,
        "exit_code": exit_code,
        "execution_mode": "shell",
        "command_id": command_id,
    }));

    let effective_repo_name = pending
        .repo_name
        .clone()
        .or_else(|| infer_repo_name_from_cwd(&pending.cwd));

    if let Some(full_name) = &effective_repo_name {
        done_event = done_event.with_repo(full_name.clone());
        if let Some(org) = super::repo_event_context::infer_org_name_from_full_name(full_name) {
            done_event = done_event.with_org(org);
        }
    }

    let safe_stdout_preview = redact_cli_preview_lines(stdout_preview);
    let safe_stderr_preview = redact_cli_preview_lines(stderr_preview);

    let _ = outbox.add(done_event);
    outbox.notify_flush();

    if let Some(cfg) = &pending.server_config {
        if !cfg.url.trim().is_empty() {
            let client = ControlPlaneClient::new(ServerConfig {
                url: cfg.url.clone(),
                api_key: cfg.api_key.clone(),
            });
            let payload = CliCommandInput {
                org_name: effective_repo_name
                    .as_deref()
                    .and_then(super::repo_event_context::infer_org_name_from_full_name),
                command: redact_sensitive_cli_text(&pending.command),
                origin: pending.origin.clone(),
                branch: pending.branch.clone(),
                repo_name: effective_repo_name.clone(),
                exit_code: Some(exit_code),
                duration_ms: Some(duration_ms),
                metadata: serde_json::json!({
                    "command_id": command_id,
                    "execution_mode": "shell",
                    "stdout_preview": safe_stdout_preview,
                    "stderr_preview": safe_stderr_preview,
                }),
            };
            if let Err(e) = client.ingest_cli_command(&payload) {
                tracing::warn!(error = %e, "Shell command audit ingestion failed");
            }
        }
    }
}
