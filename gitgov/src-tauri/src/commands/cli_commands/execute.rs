/// Execute a CLI command with real-time streaming via Tauri events.
///
/// The command runs in a background thread. Each line of stdout/stderr is emitted
/// as a `gitgov:cli-output` event. When the command finishes, `gitgov:cli-finished`
/// is emitted with the exit code.
///
/// Returns immediately with the command_id (or an error if the command is not allowed).
#[tauri::command]
pub fn cmd_execute_cli(
    app: tauri::AppHandle,
    request: CliExecuteRequest,
    outbox: State<'_, Arc<Outbox>>,
) -> Result<CliStartResult, String> {
    let command = request.command;
    let cwd = request.cwd;
    let origin = request.origin;
    let user_login = request.user_login;
    let branch = request.branch;
    let server_config = request.server_config;
    let execution_mode = request.execution_mode;

    let command_id = uuid::Uuid::new_v4().to_string();
    let resolved_user_login = resolve_user_login(user_login);
    let resolved_branch = resolve_branch(branch, &cwd);

    if execution_mode == CliExecutionMode::Shell && !shell_commands_enabled() {
        let policy_error = format!(
            "Shell command mode disabled by {} policy",
            ENV_ENABLE_SHELL_COMMANDS
        );
        let _ = app.emit(
            "gitgov:cli-output",
            CliOutputEvent {
                command_id: command_id.clone(),
                line_type: "system".to_string(),
                text: policy_error.clone(),
            },
        );
        let _ = app.emit(
            "gitgov:cli-finished",
            CliFinishedEvent {
                command_id: command_id.clone(),
                exit_code: -1,
            },
        );
        return Ok(CliStartResult {
            command_id,
            allowed: false,
            error: Some(policy_error),
        });
    }

    // Validate against whitelist
    if execution_mode == CliExecutionMode::Safe && !is_command_allowed(&command) {
        let _ = app.emit(
            "gitgov:cli-output",
            CliOutputEvent {
                command_id: command_id.clone(),
                line_type: "system".to_string(),
                text: format!(
                    "Command not allowed: '{}'. Allowed prefixes: {:?}",
                    command, DEFAULT_ALLOWED_PREFIXES
                ),
            },
        );
        let _ = app.emit(
            "gitgov:cli-finished",
            CliFinishedEvent {
                command_id: command_id.clone(),
                exit_code: -1,
            },
        );
        return Ok(CliStartResult {
            command_id,
            allowed: false,
            error: Some("Command not in whitelist".to_string()),
        });
    }

    // Emit the command itself as a system line
    let _ = app.emit(
        "gitgov:cli-output",
        CliOutputEvent {
            command_id: command_id.clone(),
            line_type: "system".to_string(),
            text: format!("$ {}", redact_sensitive_cli_text(&command)),
        },
    );

    // Spawn process
    let mut child = match execution_mode {
        CliExecutionMode::Safe => spawn_safe_child(&command, &cwd)?,
        CliExecutionMode::Shell => spawn_shell_child(&command, &cwd)?,
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let cmd_id = command_id.clone();
    let app_clone = app.clone();

    // Create audit event for the outbox
    let mut audit_event = OutboxEvent::new(
        "cli_command".to_string(),
        resolved_user_login.clone(),
        Some(resolved_branch.clone()),
        AuditStatus::Success,
    )
    .with_metadata(serde_json::json!({
        "command": redact_sensitive_cli_text(&command),
        "origin": origin.clone(),
        "branch": resolved_branch.clone(),
        "command_id": command_id.clone(),
        "execution_mode": execution_mode.as_str(),
    }));

    let mut repo_full_name: Option<String> = None;

    // Try to infer repo name from cwd
    if let Ok(repo) = git2::Repository::open(&cwd) {
        let repo_context = super::repo_event_context::resolve_repo_event_context(&repo);
        repo_full_name = repo_context.repo_full_name.clone();
        audit_event = repo_context.apply_to_event(audit_event, false);
    }
    let _ = outbox.add(audit_event);

    // Background thread: stream stdout + stderr, then emit finished
    let outbox_ref = Arc::clone(&outbox);
    let user = resolved_user_login.clone();
    let cmd_str = command.clone();
    let origin_str = origin.clone();
    let branch_str = resolved_branch.clone();
    let execution_mode_str = execution_mode.as_str().to_string();
    let cwd_str = cwd.clone();
    let repo_name_for_audit = repo_full_name;
    let cp_config_for_audit = server_config;
    let command_started_at = Instant::now();

    std::thread::spawn(move || {
        let stdout_preview = Arc::new(Mutex::new(Vec::<String>::new()));
        let stderr_preview = Arc::new(Mutex::new(Vec::<String>::new()));
        let mut stream_handles = Vec::new();

        // Stream stdout
        if let Some(out) = stdout {
            let app_stream = app_clone.clone();
            let stream_command_id = cmd_id.clone();
            let preview_ref = Arc::clone(&stdout_preview);
            let reader = BufReader::new(out);
            stream_handles.push(std::thread::spawn(move || {
                for text in reader.lines().map_while(Result::ok) {
                    if let Ok(mut preview) = preview_ref.lock() {
                        if preview.len() < 20 {
                            preview.push(text.clone());
                        }
                    }
                    let _ = app_stream.emit(
                        "gitgov:cli-output",
                        CliOutputEvent {
                            command_id: stream_command_id.clone(),
                            line_type: "stdout".to_string(),
                            text,
                        },
                    );
                }
            }));
        }

        // Stream stderr
        if let Some(err) = stderr {
            let app_stream = app_clone.clone();
            let stream_command_id = cmd_id.clone();
            let preview_ref = Arc::clone(&stderr_preview);
            let reader = BufReader::new(err);
            stream_handles.push(std::thread::spawn(move || {
                for text in reader.lines().map_while(Result::ok) {
                    if let Ok(mut preview) = preview_ref.lock() {
                        if preview.len() < 20 {
                            preview.push(text.clone());
                        }
                    }
                    let _ = app_stream.emit(
                        "gitgov:cli-output",
                        CliOutputEvent {
                            command_id: stream_command_id.clone(),
                            line_type: "stderr".to_string(),
                            text,
                        },
                    );
                }
            }));
        }

        // Wait for process to finish
        let exit_code = match child.wait() {
            Ok(status) => status.code().unwrap_or(-1),
            Err(_) => -1,
        };
        for handle in stream_handles {
            let _ = handle.join();
        }
        let elapsed_ms = command_started_at.elapsed().as_millis() as i64;
        let stdout_preview = stdout_preview.lock().map(|v| v.clone()).unwrap_or_default();
        let stderr_preview = stderr_preview.lock().map(|v| v.clone()).unwrap_or_default();
        let safe_stdout_preview = redact_cli_preview_lines(&stdout_preview);
        let safe_stderr_preview = redact_cli_preview_lines(&stderr_preview);

        let _ = app_clone.emit(
            "gitgov:cli-finished",
            CliFinishedEvent {
                command_id: cmd_id.clone(),
                exit_code,
            },
        );

        // Audit: record completion
        let audit_status = if exit_code == 0 {
            AuditStatus::Success
        } else {
            AuditStatus::Failed
        };
        let mut done_event = OutboxEvent::new(
            "cli_command_completed".to_string(),
            user.clone(),
            Some(branch_str.clone()),
            audit_status,
        )
        .with_metadata(serde_json::json!({
            "command": redact_sensitive_cli_text(&cmd_str),
            "origin": origin_str,
            "branch": branch_str,
            "exit_code": exit_code,
            "execution_mode": execution_mode_str,
            "command_id": cmd_id.clone(),
        }));
        let mut completion_repo_name = repo_name_for_audit.clone();
        if let Ok(repo) = git2::Repository::open(&cwd_str) {
            let repo_context = super::repo_event_context::resolve_repo_event_context(&repo);
            if repo_context.repo_full_name.is_some() {
                completion_repo_name = repo_context.repo_full_name.clone();
            }
            done_event = repo_context.apply_to_event(done_event, false);
        }
        let _ = outbox_ref.add(done_event);
        outbox_ref.notify_flush();

        // Direct control-plane audit trail for CLI command history endpoint.
        if let Some(cfg) = cp_config_for_audit {
            if !cfg.url.trim().is_empty() {
                let client = ControlPlaneClient::new(ServerConfig {
                    url: cfg.url,
                    api_key: cfg.api_key,
                });
                let payload = CliCommandInput {
                    org_name: completion_repo_name
                        .as_deref()
                        .and_then(super::repo_event_context::infer_org_name_from_full_name),
                    command: redact_sensitive_cli_text(&cmd_str),
                    origin: origin_str.clone(),
                    branch: branch_str.clone(),
                    repo_name: completion_repo_name.clone(),
                    exit_code: Some(exit_code),
                    duration_ms: Some(elapsed_ms),
                    metadata: serde_json::json!({
                        "command_id": cmd_id.clone(),
                        "execution_mode": execution_mode_str,
                        "stdout_preview": safe_stdout_preview,
                        "stderr_preview": safe_stderr_preview,
                    }),
                };
                if let Err(e) = client.ingest_cli_command(&payload) {
                    tracing::warn!(error = %e, "CLI command audit ingestion failed");
                }
            }
        }
    });

    Ok(CliStartResult {
        command_id,
        allowed: true,
        error: None,
    })
}
