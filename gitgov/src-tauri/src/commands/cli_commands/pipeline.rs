/// Get the list of currently allowed command prefixes.
#[tauri::command]
pub fn cmd_get_cli_whitelist() -> Vec<String> {
    safe_mode_allowed_command_descriptions()
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
        assert!(is_command_allowed("git status --porcelain=v2 --branch"));
        assert!(is_command_allowed("git log --oneline --max-count=10"));
        assert!(is_command_allowed("git remote -v"));
        assert!(is_command_allowed("git --no-pager log --oneline"));
        assert!(is_command_allowed("git rev-parse --abbrev-ref HEAD"));
        assert!(is_command_allowed("gitgov status"));
        assert!(is_command_allowed("git"));
    }

    #[test]
    fn blocked_commands() {
        assert!(!is_command_allowed("rm -rf /"));
        assert!(!is_command_allowed("npm install"));
        assert!(!is_command_allowed("cargo build"));
        assert!(!is_command_allowed("curl http://evil.com"));
        assert!(!is_command_allowed("git clone https://example.com/acme/repo"));
        assert!(!is_command_allowed("git -c core.sshCommand=calc fetch"));
        assert!(!is_command_allowed("git --config-env core.sshCommand=SSH_CMD status"));
        assert!(!is_command_allowed("git --exec-path=C:\\Temp status"));
        assert!(!is_command_allowed("git --git-dir C:\\Temp\\.git status"));
        assert!(!is_command_allowed("git -C . status"));
        assert!(!is_command_allowed("git -C"));
        assert!(!is_command_allowed("git diff --no-index C:\\Temp\\a C:\\Temp\\b"));
        assert!(!is_command_allowed("git show HEAD:C:\\Temp\\secret.txt"));
        assert!(!is_command_allowed("git branch -D main"));
        assert!(!is_command_allowed("git remote add origin https://example.com/acme/repo"));
        assert!(!is_command_allowed("git log --max-count="));
        assert!(!is_command_allowed("git log --max-count=abc"));
        assert!(!is_command_allowed("git log -n"));
        assert!(!is_command_allowed("git log -nabc"));
        assert!(!is_command_allowed("git rev-parse --git-dir"));
        assert!(!is_command_allowed("git status C:\\Temp"));
        assert!(!is_command_allowed(""));
        assert!(!is_command_allowed("  "));
    }

    #[test]
    fn cli_whitelist_reports_concrete_safe_mode_commands() {
        let whitelist = cmd_get_cli_whitelist();

        assert!(whitelist.contains(&"git status".to_string()));
        assert!(whitelist.contains(&"git log".to_string()));
        assert!(whitelist.contains(&"gitgov <command>".to_string()));
        assert_ne!(whitelist, vec!["git".to_string(), "gitgov".to_string()]);
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
    fn native_terminal_is_enabled_by_default_and_respects_explicit_false() {
        let original = std::env::var_os(ENV_ENABLE_NATIVE_TERMINAL);
        std::env::remove_var(ENV_ENABLE_NATIVE_TERMINAL);

        assert!(native_terminal_enabled());

        std::env::set_var(ENV_ENABLE_NATIVE_TERMINAL, "false");
        assert!(!native_terminal_enabled());

        if let Some(value) = original {
            std::env::set_var(ENV_ENABLE_NATIVE_TERMINAL, value);
        } else {
            std::env::remove_var(ENV_ENABLE_NATIVE_TERMINAL);
        }
    }

    #[test]
    fn child_environment_classification_removes_secret_like_keys() {
        for key in [
            "GITGOV_API_KEY",
            "github_token",
            "jira_api_token",
            "jenkins_webhook_secret",
            "database_url",
            "client_private_key",
            "service_password",
            "cloud_credentials",
            "aws_access_key_id",
            "npm_config__auth",
            "ssh_auth_sock",
            "ssh_askpass",
            "session_cookie",
            "git_config_global",
            "git_external_diff",
            "git_ssh_command",
            "git_askpass",
            "p4passwd",
        ] {
            assert!(is_sensitive_child_env_key(key), "{key} should be scrubbed");
        }
    }

    #[test]
    fn child_environment_classification_preserves_operational_keys() {
        for key in [
            "PATH",
            "SystemRoot",
            "HOME",
            "USERPROFILE",
            "GITGOV_ENABLE_NATIVE_TERMINAL",
            "GITGOV_ENABLE_SHELL_COMMANDS",
        ] {
            assert!(
                !is_sensitive_child_env_key(key),
                "{key} should be preserved"
            );
        }
    }

    #[test]
    fn safe_program_resolution_rejects_path_components() {
        assert!(resolve_safe_program("../git").is_err());
        assert!(resolve_safe_program("tools/git").is_err());
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

    #[test]
    fn native_terminal_git_context_reports_non_git_directory_without_running_git() {
        let temp = tempfile::tempdir().expect("tempdir");
        let context = native_terminal_git_context(temp.path().to_str().unwrap(), None);

        assert!(!context.is_git_repo);
        assert!(!context.is_detached);
        assert_eq!(context.repo_name, None);
        assert_eq!(context.branch, None);
        assert_eq!(context.commit_short, None);
        assert!(!context.cwd.is_empty());
    }

    #[test]
    fn native_terminal_git_context_reports_repo_branch_and_commit() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo = git2::Repository::init(temp.path()).expect("repo init");
        let file_path = temp.path().join("README.md");
        std::fs::write(&file_path, "hello").expect("write file");

        let mut index = repo.index().expect("index");
        index
            .add_path(std::path::Path::new("README.md"))
            .expect("add file");
        let tree_oid = index.write_tree().expect("write tree");
        let tree = repo.find_tree(tree_oid).expect("tree");
        let signature = git2::Signature::now("GitGov Test", "test@example.com").expect("signature");
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "KAN-133 test commit",
            &tree,
            &[],
        )
        .expect("commit");

        let context = native_terminal_git_context(temp.path().to_str().unwrap(), None);

        assert!(context.is_git_repo);
        assert!(!context.is_detached);
        assert_eq!(context.repo_name.as_deref(), temp.path().file_name().and_then(|v| v.to_str()));
        assert!(matches!(context.branch.as_deref(), Some("main") | Some("master")));
        assert_eq!(context.commit_short.as_ref().map(String::len), Some(7));
    }

    #[test]
    fn terminal_cd_resolution_updates_only_for_simple_existing_directory_changes() {
        let temp = tempfile::tempdir().expect("tempdir");
        let nested = temp.path().join("nested repo");
        std::fs::create_dir(&nested).expect("nested dir");

        let resolved = resolve_terminal_cwd_after_command(temp.path(), Some(r#"cd "nested repo""#));
        assert_eq!(resolved, nested.canonicalize().expect("canonical nested"));

        let ignored = resolve_terminal_cwd_after_command(temp.path(), Some("cd nested repo && git status"));
        assert_eq!(ignored, temp.path().canonicalize().expect("canonical temp"));

        let missing = resolve_terminal_cwd_after_command(temp.path(), Some("cd missing"));
        assert_eq!(missing, temp.path().canonicalize().expect("canonical temp"));
    }

    fn detected_tool_names(context: &CliNativeTerminalToolContextResult) -> std::collections::BTreeSet<String> {
        context
            .tools
            .iter()
            .filter(|tool| tool.detected)
            .map(|tool| tool.tool.clone())
            .collect()
    }

    fn tool_detection<'a>(
        context: &'a CliNativeTerminalToolContextResult,
        name: &str,
    ) -> &'a CliNativeTerminalToolDetection {
        context
            .tools
            .iter()
            .find(|tool| tool.tool == name)
            .expect("tool detection")
    }

    #[test]
    fn native_terminal_tool_context_detects_safe_local_tool_signals_only() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::write(temp.path().join("main.tf"), "resource \"x\" \"y\" {}").expect("tf");
        std::fs::write(temp.path().join("docker-compose.yml"), "services: {}").expect("compose");
        std::fs::write(temp.path().join("Chart.yaml"), "apiVersion: v2").expect("chart");
        std::fs::write(temp.path().join("kustomization.yaml"), "resources: []").expect("kustomize");

        let context = native_terminal_tool_context(temp.path().to_str().unwrap(), None);
        let detected = detected_tool_names(&context);

        assert_eq!(context.cwd_kind, "non_git");
        assert!(detected.contains("terraform"));
        assert!(detected.contains("docker-compose"));
        assert!(detected.contains("helm"));
        assert!(detected.contains("kubernetes"));
        assert!(!context.scan_limited);
        assert!(!context.secrets_read);
        assert!(!context.network_used);
        assert_eq!(
            tool_detection(&context, "terraform").safe_command_ids,
            vec!["terraform-fmt-check".to_string(), "terraform-validate".to_string()]
        );
        assert_eq!(
            tool_detection(&context, "docker-compose").reason,
            "docker_compose_file_present"
        );
    }

    #[test]
    fn native_terminal_tool_context_uses_lockfiles_and_safe_directory_names() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::write(temp.path().join(".terraform.lock.hcl"), "# lock").expect("lock");
        std::fs::create_dir(temp.path().join("templates")).expect("templates");
        std::fs::create_dir(temp.path().join("manifests")).expect("manifests");

        let context = native_terminal_tool_context(temp.path().to_str().unwrap(), None);
        let detected = detected_tool_names(&context);

        assert!(detected.contains("terraform"));
        assert!(detected.contains("helm"));
        assert!(detected.contains("kubernetes"));
        assert_eq!(
            tool_detection(&context, "terraform").reason,
            "terraform_lockfile_present"
        );
        assert_eq!(
            tool_detection(&context, "helm").reason,
            "helm_templates_directory_present"
        );
    }

    #[test]
    fn native_terminal_tool_context_does_not_treat_secret_files_as_detection_or_output() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::write(temp.path().join("prod.tfvars"), "token = \"secret\"").expect("tfvars");
        std::fs::write(temp.path().join("terraform.tfstate"), "{\"secret\":\"value\"}").expect("tfstate");
        std::fs::write(temp.path().join(".env"), "TOKEN=secret").expect("env");
        std::fs::write(temp.path().join("secret-values.yaml"), "password: secret").expect("secret");

        let context = native_terminal_tool_context(temp.path().to_str().unwrap(), None);
        let serialized = serde_json::to_string(&context).expect("serialize context");

        assert!(detected_tool_names(&context).is_empty());
        assert!(!context.secrets_read);
        assert!(!serialized.contains(temp.path().to_string_lossy().as_ref()));
        assert!(!serialized.contains("prod.tfvars"));
        assert!(!serialized.contains("terraform.tfstate"));
        assert!(!serialized.contains("secret-values.yaml"));
        assert!(!serialized.contains("TOKEN"));
        assert!(!serialized.contains("password"));
    }

    #[test]
    fn native_terminal_tool_context_ignores_heavy_or_sensitive_directories() {
        let temp = tempfile::tempdir().expect("tempdir");
        for dir in [".terraform", "node_modules", ".git", "target", "dist", "build", ".next"] {
            let nested = temp.path().join(dir);
            std::fs::create_dir(&nested).expect("ignored dir");
            std::fs::write(nested.join("main.tf"), "resource \"x\" \"y\" {}").expect("ignored file");
            std::fs::write(nested.join("docker-compose.yml"), "services: {}").expect("ignored compose");
        }

        let context = native_terminal_tool_context(temp.path().to_str().unwrap(), None);

        assert!(detected_tool_names(&context).is_empty());
        assert!(!context.scan_limited);
    }

    #[test]
    fn native_terminal_tool_context_limits_scanning_and_marks_limited() {
        let temp = tempfile::tempdir().expect("tempdir");
        for index in 0..=TOOL_SCAN_MAX_ENTRIES {
            std::fs::write(temp.path().join(format!("file-{index}.txt")), "x").expect("file");
        }

        let context = native_terminal_tool_context(temp.path().to_str().unwrap(), None);

        assert!(context.scan_limited);
        assert!(!context.secrets_read);
        assert!(!context.network_used);
    }

    #[test]
    fn native_terminal_tool_context_resolves_simple_cd_without_exposing_paths() {
        let temp = tempfile::tempdir().expect("tempdir");
        let nested = temp.path().join("infra");
        std::fs::create_dir(&nested).expect("nested");
        std::fs::write(nested.join("main.tf"), "resource \"x\" \"y\" {}").expect("tf");

        let context = native_terminal_tool_context(temp.path().to_str().unwrap(), Some("cd infra"));
        let serialized = serde_json::to_string(&context).expect("serialize context");

        assert!(detected_tool_names(&context).contains("terraform"));
        assert!(!serialized.contains(temp.path().to_string_lossy().as_ref()));
        assert!(!serialized.contains("infra"));
    }
}
