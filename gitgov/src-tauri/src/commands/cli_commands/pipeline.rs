/// Get the list of currently allowed command prefixes.
#[tauri::command]
pub fn cmd_get_cli_whitelist() -> Vec<String> {
    DEFAULT_ALLOWED_PREFIXES
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Build pipeline graph data for the current branch + develop/main.
/// Returns commit history, branch relationships, and linked ticket/PR/pipeline info.
#[tauri::command]
pub fn cmd_get_pipeline_graph(
    repo_path: String,
    max_commits: Option<usize>,
) -> Result<serde_json::Value, String> {
    let repo =
        git2::Repository::open(&repo_path).map_err(|e| format!("Failed to open repo: {}", e))?;

    let head = repo.head().map_err(|e| format!("No HEAD: {}", e))?;
    let current_branch = head.shorthand().unwrap_or("HEAD").to_string();

    let max = max_commits.unwrap_or(50);

    // Walk current branch commits
    let mut revwalk = repo
        .revwalk()
        .map_err(|e| format!("Revwalk error: {}", e))?;
    revwalk
        .push_head()
        .map_err(|e| format!("Push head: {}", e))?;
    revwalk
        .set_sorting(git2::Sort::TIME)
        .map_err(|e| format!("Sort error: {}", e))?;

    let mut commits: Vec<serde_json::Value> = Vec::new();
    for (i, oid) in revwalk.enumerate() {
        if i >= max {
            break;
        }
        if let Ok(oid) = oid {
            if let Ok(commit) = repo.find_commit(oid) {
                let sha = oid.to_string();
                let short_sha = &sha[..7.min(sha.len())];
                let message = commit.message().unwrap_or("").to_string();
                let summary = commit.summary().unwrap_or("").to_string();
                let author = commit.author().name().unwrap_or("unknown").to_string();
                let time = commit.time().seconds();

                commits.push(serde_json::json!({
                    "sha": sha,
                    "short_sha": short_sha,
                    "message": message.trim(),
                    "summary": summary.trim(),
                    "author": author,
                    "timestamp": time,
                    "branch": current_branch,
                }));
            }
        }
    }

    // Find target branches (develop, main, master)
    let target_branches: Vec<String> = ["develop", "main", "master"]
        .iter()
        .filter(|name| {
            repo.find_branch(name, git2::BranchType::Local).is_ok()
                || repo
                    .find_branch(&format!("origin/{}", name), git2::BranchType::Remote)
                    .is_ok()
        })
        .map(|s| s.to_string())
        .collect();

    Ok(serde_json::json!({
        "current_branch": current_branch,
        "target_branches": target_branches,
        "commits": commits,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_commands() {
        assert!(is_command_allowed("git status"));
        assert!(is_command_allowed("git log --oneline"));
        assert!(is_command_allowed("git remote -v"));
        assert!(is_command_allowed("gitgov status"));
        assert!(is_command_allowed("git"));
    }

    #[test]
    fn blocked_commands() {
        assert!(!is_command_allowed("rm -rf /"));
        assert!(!is_command_allowed("npm install"));
        assert!(!is_command_allowed("cargo build"));
        assert!(!is_command_allowed("curl http://evil.com"));
        assert!(!is_command_allowed(""));
        assert!(!is_command_allowed("  "));
    }

    #[test]
    fn split_command_line_preserves_quoted_arguments() {
        let parts = split_command_line(r#"git commit -m "hello world""#).unwrap();

        assert_eq!(parts, vec!["git", "commit", "-m", "hello world"]);
    }

    #[test]
    fn split_command_line_preserves_windows_paths_inside_quotes() {
        let parts = split_command_line(r#"git -C "C:\Users\PC\Desktop\Git Gov" status"#).unwrap();

        assert_eq!(
            parts,
            vec!["git", "-C", r#"C:\Users\PC\Desktop\Git Gov"#, "status"]
        );
    }

    #[test]
    fn split_command_line_preserves_empty_quoted_argument() {
        let parts = split_command_line(r#"git commit --allow-empty-message -m """#).unwrap();

        assert_eq!(
            parts,
            vec!["git", "commit", "--allow-empty-message", "-m", ""]
        );
    }

    #[test]
    fn split_command_line_rejects_unclosed_quote() {
        let error = split_command_line(r#"git commit -m "unfinished"#).unwrap_err();

        assert!(error.contains("Unclosed"));
    }

    #[test]
    fn parse_env_flag_value_handles_expected_variants() {
        assert!(parse_env_flag_value("true", false));
        assert!(parse_env_flag_value("  YES  ", false));
        assert!(parse_env_flag_value("1", false));
        assert!(!parse_env_flag_value("false", true));
        assert!(!parse_env_flag_value("No", true));
        assert!(!parse_env_flag_value("0", true));
        assert!(parse_env_flag_value("invalid", true));
        assert!(!parse_env_flag_value("invalid", false));
    }

    #[test]
    fn parse_shell_exit_marker_accepts_valid_marker() {
        let parsed = parse_shell_exit_marker("__GITGOV_EXIT__:abc-123:7");
        assert_eq!(parsed, Some(("abc-123".to_string(), 7)));
    }

    #[test]
    fn parse_shell_exit_marker_rejects_non_marker_output() {
        assert_eq!(parse_shell_exit_marker("ordinary command output"), None);
        assert_eq!(parse_shell_exit_marker("__GITGOV_EXIT__::0"), None);
        assert_eq!(parse_shell_exit_marker("__GITGOV_EXIT__:abc:nope"), None);
    }

    #[test]
    fn redacts_sensitive_cli_audit_text() {
        let text = "git remote set-url origin https://token123@github.com/acme/repo.git && echo GITGOV_API_KEY=abc123 ghp_secretvalue";
        let redacted = redact_sensitive_cli_text(text);

        assert!(!redacted.contains("token123"));
        assert!(!redacted.contains("abc123"));
        assert!(!redacted.contains("ghp_secretvalue"));
        assert!(redacted.contains("https://[REDACTED]@github.com/acme/repo.git"));
        assert!(redacted.contains("GITGOV_API_KEY=[REDACTED]"));
        assert!(redacted.contains("[REDACTED_SECRET]"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_shell_wrapper_captures_powershell_and_native_exit_status() {
        let wrapped = wrap_shell_command("Write-Error 'boom'", "cmd-1");

        assert!(wrapped.contains("$global:LASTEXITCODE = $null"));
        assert!(wrapped.contains("$ggSucceeded = $?"));
        assert!(wrapped.contains("$ggLastExit = $LASTEXITCODE"));
        assert!(wrapped.contains("__GITGOV_EXIT__:cmd-1:$ggEc"));
    }
}
