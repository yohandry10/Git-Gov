fn native_terminal_git_context(cwd: &str, command: Option<&str>) -> CliNativeTerminalGitContextResult {
    let effective_cwd = resolve_terminal_cwd_after_command(Path::new(cwd), command);
    let detected_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);

    let Ok(repo) = git2::Repository::discover(&effective_cwd) else {
        return CliNativeTerminalGitContextResult {
            cwd: effective_cwd.to_string_lossy().to_string(),
            is_git_repo: false,
            is_detached: false,
            repo_name: None,
            branch: None,
            commit_short: None,
            detected_at_ms,
        };
    };

    let repo_name = repo
        .workdir()
        .or_else(|| repo.path().parent())
        .and_then(|path| path.file_name())
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned);

    let head = repo.head().ok();
    let is_detached = repo.head_detached().unwrap_or(false);
    let branch = if is_detached {
        None
    } else {
        head.as_ref()
            .and_then(|head| head.shorthand())
            .filter(|value| !value.trim().is_empty())
            .map(ToOwned::to_owned)
    };
    let commit_short = head.as_ref().and_then(|head| head.target()).map(|oid| {
        let value = oid.to_string();
        value[..7.min(value.len())].to_string()
    });

    CliNativeTerminalGitContextResult {
        cwd: effective_cwd.to_string_lossy().to_string(),
        is_git_repo: true,
        is_detached,
        repo_name,
        branch,
        commit_short,
        detected_at_ms,
    }
}

fn resolve_terminal_cwd_after_command(current_cwd: &Path, command: Option<&str>) -> PathBuf {
    let fallback = canonical_or_original(current_cwd);
    let Some(command) = command.and_then(extract_directory_change_target) else {
        return fallback;
    };

    let target = Path::new(&command);
    let candidate = if target.is_absolute() {
        PathBuf::from(target)
    } else {
        fallback.join(target)
    };

    if candidate.is_dir() {
        canonical_or_original(&candidate)
    } else {
        fallback
    }
}

fn canonical_or_original(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn extract_directory_change_target(command: &str) -> Option<String> {
    let trimmed = command.trim();
    if trimmed.is_empty() || contains_shell_control_operator(trimmed) {
        return None;
    }

    let parts = split_command_line(trimmed).ok()?;
    if parts.is_empty() {
        return None;
    }

    let verb = parts[0].trim().to_ascii_lowercase();
    let is_cd = matches!(verb.as_str(), "cd" | "chdir" | "sl" | "set-location");
    if !is_cd || parts.len() != 2 {
        return None;
    }

    let target = parts[1].trim();
    if target.is_empty() || target == "-" || target.starts_with('-') {
        return None;
    }

    Some(target.to_string())
}

fn contains_shell_control_operator(command: &str) -> bool {
    command.contains("&&")
        || command.contains("||")
        || command.contains(';')
        || command.contains('|')
}
