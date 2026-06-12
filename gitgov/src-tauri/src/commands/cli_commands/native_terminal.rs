#[tauri::command]
pub fn cmd_start_native_terminal(
    app: tauri::AppHandle,
    request: CliNativeTerminalStartRequest,
    native_manager: State<'_, CliNativeTerminalManager>,
) -> Result<CliNativeTerminalStartResult, String> {
    if !native_terminal_enabled() {
        return Err(format!(
            "Native terminal is disabled by {}=false",
            ENV_ENABLE_NATIVE_TERMINAL
        ));
    }

    let cwd = request.cwd.trim();
    if cwd.is_empty() {
        return Err("cwd is required".to_string());
    }
    if !Path::new(cwd).exists() {
        return Err(format!("cwd does not exist: {}", cwd));
    }

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(native_terminal_size(request.cols, request.rows))
        .map_err(|e| format!("Failed to open PTY: {}", e))?;

    let (mut command, shell_name) = build_native_terminal_command(request.shell.as_deref());
    command.cwd(cwd);
    apply_sanitized_pty_env(&mut command);
    command.env("TERM", "xterm-256color");

    let child = pair
        .slave
        .spawn_command(command)
        .map_err(|e| format!("Failed to spawn native terminal shell: {}", e))?;
    let mut child_killer = child.clone_killer();

    let mut reader = match pair.master.try_clone_reader() {
        Ok(reader) => reader,
        Err(e) => {
            let _ = child_killer.kill();
            return Err(format!("Failed to clone PTY reader: {}", e));
        }
    };
    let writer = match pair.master.take_writer() {
        Ok(writer) => writer,
        Err(e) => {
            let _ = child_killer.kill();
            return Err(format!("Failed to take PTY writer: {}", e));
        }
    };

    let session_id = uuid::Uuid::new_v4().to_string();
    let child_killer = Arc::new(Mutex::new(child_killer));

    let session = NativeTerminalSession {
        master: Arc::new(Mutex::new(pair.master)),
        writer: Arc::new(Mutex::new(writer)),
        killer: Arc::clone(&child_killer),
    };

    {
        let mut sessions = native_manager
            .sessions
            .lock()
            .map_err(|_| "Native terminal lock poisoned".to_string())?;
        sessions.insert(session_id.clone(), session);
    }

    let app_output = app.clone();
    let session_output = session_id.clone();
    std::thread::spawn(move || {
        let mut buffer = [0u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(read_bytes) => {
                    let text = String::from_utf8_lossy(&buffer[..read_bytes]).to_string();
                    let _ = app_output.emit(
                        "gitgov:pty-output",
                        CliNativeTerminalOutputEvent {
                            session_id: session_output.clone(),
                            data: text,
                        },
                    );
                }
                Err(_) => break,
            }
        }
    });

    let app_exit = app.clone();
    let session_exit = session_id.clone();
    let sessions_ref = Arc::clone(&native_manager.sessions);
    std::thread::spawn(move || {
        let mut child = child;
        let exit_code = child
            .wait()
            .ok()
            .map(|status| i32::try_from(status.exit_code()).unwrap_or(-1))
            .unwrap_or(-1);

        let _ = app_exit.emit(
            "gitgov:pty-exit",
            CliNativeTerminalExitEvent {
                session_id: session_exit.clone(),
                exit_code,
            },
        );

        if let Ok(mut sessions) = sessions_ref.lock() {
            sessions.remove(&session_exit);
        }
    });

    Ok(CliNativeTerminalStartResult {
        session_id,
        shell: shell_name,
    })
}

#[tauri::command]
pub fn cmd_write_native_terminal(
    request: CliNativeTerminalWriteRequest,
    native_manager: State<'_, CliNativeTerminalManager>,
) -> Result<(), String> {
    let writer = {
        let sessions = native_manager
            .sessions
            .lock()
            .map_err(|_| "Native terminal lock poisoned".to_string())?;
        let session = sessions
            .get(&request.session_id)
            .ok_or_else(|| "Native terminal session not found".to_string())?;
        Arc::clone(&session.writer)
    };

    let mut writer = writer
        .lock()
        .map_err(|_| "Native terminal writer lock poisoned".to_string())?;
    writer
        .write_all(request.data.as_bytes())
        .map_err(|e| format!("Failed to write PTY input: {}", e))?;
    writer
        .flush()
        .map_err(|e| format!("Failed to flush PTY input: {}", e))
}

#[tauri::command]
pub fn cmd_resize_native_terminal(
    request: CliNativeTerminalResizeRequest,
    native_manager: State<'_, CliNativeTerminalManager>,
) -> Result<(), String> {
    let master = {
        let sessions = native_manager
            .sessions
            .lock()
            .map_err(|_| "Native terminal lock poisoned".to_string())?;
        let session = sessions
            .get(&request.session_id)
            .ok_or_else(|| "Native terminal session not found".to_string())?;
        Arc::clone(&session.master)
    };

    let master = master
        .lock()
        .map_err(|_| "Native terminal master lock poisoned".to_string())?;
    master
        .resize(PtySize {
            rows: request.rows.max(5),
            cols: request.cols.max(20),
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("Failed to resize PTY: {}", e))
}

#[tauri::command]
pub fn cmd_stop_native_terminal(
    session_id: String,
    native_manager: State<'_, CliNativeTerminalManager>,
) -> Result<CliNativeTerminalStopResult, String> {
    let session = {
        let mut sessions = native_manager
            .sessions
            .lock()
            .map_err(|_| "Native terminal lock poisoned".to_string())?;
        sessions.remove(&session_id)
    };

    if let Some(session) = session {
        if let Ok(mut killer) = session.killer.lock() {
            let _ = killer.kill();
        }
        return Ok(CliNativeTerminalStopResult { stopped: true });
    }

    Ok(CliNativeTerminalStopResult { stopped: false })
}
