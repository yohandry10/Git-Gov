#[tauri::command]
pub fn cmd_start_shell_session(
    app: tauri::AppHandle,
    request: CliShellStartRequest,
    shell_manager: State<'_, CliShellManager>,
    outbox: State<'_, Arc<Outbox>>,
) -> Result<CliShellStartResult, String> {
    if !shell_commands_enabled() {
        return Err(format!(
            "Shell sessions are disabled by {}=false",
            ENV_ENABLE_SHELL_COMMANDS
        ));
    }

    let cwd = request.cwd.trim().to_string();
    if cwd.is_empty() {
        return Err("cwd is required".to_string());
    }

    // Keep a single active shell session to avoid terminal split-brain.
    {
        let mut sessions = shell_manager
            .sessions
            .lock()
            .map_err(|_| "Shell session lock poisoned".to_string())?;
        for (_, session) in sessions.drain() {
            if let Ok(mut child) = session.child.lock() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }

    let (mut child, shell_name) = spawn_shell_session_child(&cwd)?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Failed to capture shell stdin".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to capture shell stdout".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "Failed to capture shell stderr".to_string())?;

    let session_id = uuid::Uuid::new_v4().to_string();
    let default_user_login = resolve_user_login(request.user_login);
    let default_branch = resolve_branch(request.branch, &cwd);
    let repo_name = infer_repo_name_from_cwd(&cwd);

    let child_ref = Arc::new(Mutex::new(child));
    let stdin_ref = Arc::new(Mutex::new(stdin));
    let active_command_id = Arc::new(Mutex::new(session_id.clone()));
    let pending_commands = Arc::new(Mutex::new(HashMap::<String, PendingShellCommand>::new()));

    let session = ShellSession {
        cwd: cwd.clone(),
        default_user_login,
        default_branch,
        repo_name,
        server_config: request.server_config,
        stdin: Arc::clone(&stdin_ref),
        child: Arc::clone(&child_ref),
        active_command_id: Arc::clone(&active_command_id),
        pending_commands: Arc::clone(&pending_commands),
    };

    {
        let mut sessions = shell_manager
            .sessions
            .lock()
            .map_err(|_| "Shell session lock poisoned".to_string())?;
        sessions.insert(session_id.clone(), session);
    }

    let app_out = app.clone();
    let active_command_id_out = Arc::clone(&active_command_id);
    let session_id_out = session_id.clone();
    let outbox_ref = Arc::clone(&outbox);
    let stdout_preview_by_cmd = Arc::new(Mutex::new(HashMap::<String, Vec<String>>::new()));
    let stderr_preview_by_cmd = Arc::new(Mutex::new(HashMap::<String, Vec<String>>::new()));
    let stdout_preview_out = Arc::clone(&stdout_preview_by_cmd);
    let stdout_preview_done = Arc::clone(&stdout_preview_by_cmd);
    let stderr_preview_done = Arc::clone(&stderr_preview_by_cmd);
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for text in reader.lines().map_while(Result::ok) {
            if let Some((marker_command_id, exit_code)) = parse_shell_exit_marker(&text) {
                let _ = app_out.emit(
                    "gitgov:cli-finished",
                    CliFinishedEvent {
                        command_id: marker_command_id.clone(),
                        exit_code,
                    },
                );

                let pending = pending_commands
                    .lock()
                    .ok()
                    .and_then(|mut m| m.remove(&marker_command_id));
                if let Some(pending_cmd) = pending {
                    let stdout_preview = stdout_preview_done
                        .lock()
                        .ok()
                        .and_then(|mut previews| previews.remove(&marker_command_id))
                        .unwrap_or_default();
                    let stderr_preview = stderr_preview_done
                        .lock()
                        .ok()
                        .and_then(|mut previews| previews.remove(&marker_command_id))
                        .unwrap_or_default();
                    queue_cli_completion_audit(
                        &outbox_ref,
                        &pending_cmd,
                        &marker_command_id,
                        exit_code,
                        &stdout_preview,
                        &stderr_preview,
                    );
                } else {
                    let _ = stdout_preview_done
                        .lock()
                        .ok()
                        .and_then(|mut previews| previews.remove(&marker_command_id));
                    let _ = stderr_preview_done
                        .lock()
                        .ok()
                        .and_then(|mut previews| previews.remove(&marker_command_id));
                }
                continue;
            }

            let command_id = active_command_id_out
                .lock()
                .map(|g| g.clone())
                .unwrap_or_else(|_| session_id_out.clone());

            if let Ok(mut previews) = stdout_preview_out.lock() {
                let lines = previews.entry(command_id.clone()).or_default();
                lines.push(text.clone());
                if lines.len() > 20 {
                    let overflow = lines.len() - 20;
                    lines.drain(0..overflow);
                }
            }

            let _ = app_out.emit(
                "gitgov:cli-output",
                CliOutputEvent {
                    command_id,
                    line_type: "stdout".to_string(),
                    text,
                },
            );
        }
    });

    let app_err = app.clone();
    let active_command_id_err = Arc::clone(&active_command_id);
    let session_id_err = session_id.clone();
    let stderr_preview_err = Arc::clone(&stderr_preview_by_cmd);
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for text in reader.lines().map_while(Result::ok) {
            if parse_shell_exit_marker(&text).is_some() {
                continue;
            }
            let command_id = active_command_id_err
                .lock()
                .map(|g| g.clone())
                .unwrap_or_else(|_| session_id_err.clone());
            if let Ok(mut previews) = stderr_preview_err.lock() {
                let lines = previews.entry(command_id.clone()).or_default();
                lines.push(text.clone());
                if lines.len() > 20 {
                    let overflow = lines.len() - 20;
                    lines.drain(0..overflow);
                }
            }
            let _ = app_err.emit(
                "gitgov:cli-output",
                CliOutputEvent {
                    command_id,
                    line_type: "stderr".to_string(),
                    text,
                },
            );
        }
    });

    emit_system_line(
        &app,
        &session_id,
        format!("Shell session started ({}) in {}", shell_name, cwd),
    );

    Ok(CliShellStartResult {
        session_id,
        shell: shell_name.to_string(),
    })
}

#[tauri::command]
pub fn cmd_send_shell_input(
    app: tauri::AppHandle,
    request: CliShellInputRequest,
    shell_manager: State<'_, CliShellManager>,
    outbox: State<'_, Arc<Outbox>>,
) -> Result<CliShellInputResult, String> {
    let input = request.input.trim().to_string();
    if input.is_empty() {
        return Ok(CliShellInputResult {
            command_id: String::new(),
            accepted: false,
            error: Some("Empty input".to_string()),
        });
    }

    let command_id = uuid::Uuid::new_v4().to_string();
    let (
        stdin_ref,
        active_command_id,
        pending_commands,
        default_user_login,
        default_branch,
        repo_name,
        server_config,
        cwd,
    ) = {
        let sessions = shell_manager
            .sessions
            .lock()
            .map_err(|_| "Shell session lock poisoned".to_string())?;
        let session = sessions
            .get(&request.session_id)
            .ok_or_else(|| "Shell session not found".to_string())?;
        (
            Arc::clone(&session.stdin),
            Arc::clone(&session.active_command_id),
            Arc::clone(&session.pending_commands),
            session.default_user_login.clone(),
            session.default_branch.clone(),
            session.repo_name.clone(),
            session.server_config.clone(),
            session.cwd.clone(),
        )
    };

    {
        let map = pending_commands
            .lock()
            .map_err(|_| "Shell pending command lock poisoned".to_string())?;
        if !map.is_empty() {
            return Ok(CliShellInputResult {
                command_id,
                accepted: false,
                error: Some(
                    "Another shell command is still running; wait for it to finish before sending the next command."
                        .to_string(),
                ),
            });
        }
    }

    let resolved_user_login = request
        .user_login
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or(default_user_login);
    let resolved_branch = request
        .branch
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or(default_branch);
    let origin = request
        .origin
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "manual_input".to_string());

    let pending = PendingShellCommand {
        command: input.clone(),
        origin: origin.clone(),
        branch: resolved_branch.clone(),
        user_login: resolved_user_login.clone(),
        cwd,
        repo_name: repo_name.clone(),
        server_config: server_config.clone(),
        started_at: Instant::now(),
    };

    queue_cli_start_audit(
        &outbox,
        &CliStartAuditInput {
            user_login: &resolved_user_login,
            branch: &resolved_branch,
            command: &input,
            origin: &origin,
            command_id: &command_id,
            execution_mode: "shell",
            repo_name: repo_name.as_deref(),
        },
    );

    if let Ok(mut map) = pending_commands.lock() {
        map.insert(command_id.clone(), pending);
    }

    if let Ok(mut active) = active_command_id.lock() {
        *active = command_id.clone();
    }

    emit_system_line(&app, &command_id, format!("$ {}", input));

    let wrapped = wrap_shell_command(&input, &command_id);
    let write_result = stdin_ref
        .lock()
        .map_err(|_| "Shell stdin lock poisoned".to_string())
        .and_then(|mut stdin| {
            stdin
                .write_all(wrapped.as_bytes())
                .map_err(|e| format!("Failed to write to shell stdin: {}", e))?;
            stdin
                .flush()
                .map_err(|e| format!("Failed to flush shell stdin: {}", e))
        });

    if let Err(e) = write_result {
        if let Ok(mut map) = pending_commands.lock() {
            map.remove(&command_id);
        }
        let _ = app.emit(
            "gitgov:cli-finished",
            CliFinishedEvent {
                command_id: command_id.clone(),
                exit_code: -1,
            },
        );
        return Ok(CliShellInputResult {
            command_id,
            accepted: false,
            error: Some(e),
        });
    }

    Ok(CliShellInputResult {
        command_id,
        accepted: true,
        error: None,
    })
}

#[tauri::command]
pub fn cmd_stop_shell_session(
    app: tauri::AppHandle,
    session_id: String,
    shell_manager: State<'_, CliShellManager>,
) -> Result<CliShellStopResult, String> {
    let session = {
        let mut sessions = shell_manager
            .sessions
            .lock()
            .map_err(|_| "Shell session lock poisoned".to_string())?;
        sessions.remove(&session_id)
    };

    if let Some(session) = session {
        if let Ok(mut stdin) = session.stdin.lock() {
            let _ = stdin.write_all(b"exit\n");
            let _ = stdin.flush();
        }
        if let Ok(mut child) = session.child.lock() {
            let _ = child.kill();
            let _ = child.wait();
        }
        emit_system_line(&app, &session_id, "Shell session stopped");
        return Ok(CliShellStopResult { stopped: true });
    }

    Ok(CliShellStopResult { stopped: false })
}
