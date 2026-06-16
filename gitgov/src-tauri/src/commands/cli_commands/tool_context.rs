const TOOL_SCAN_MAX_ENTRIES: usize = 200;
const TOOL_SCAN_MAX_DEPTH: usize = 2;
const TOOL_SCAN_IGNORED_DIRS: &[&str] = &[
    ".git",
    ".next",
    ".terraform",
    "build",
    "dist",
    "node_modules",
    "target",
];

#[derive(Default)]
struct NativeTerminalToolSignals {
    terraform_config: bool,
    terraform_lock: bool,
    docker_compose: bool,
    helm_chart: bool,
    helm_templates_dir: bool,
    kubernetes_kustomization: bool,
    kubernetes_manifests_dir: bool,
    scan_limited: bool,
    entries_seen: usize,
}

fn native_terminal_tool_context(
    cwd: &str,
    command: Option<&str>,
) -> CliNativeTerminalToolContextResult {
    let effective_cwd = resolve_terminal_cwd_after_command(Path::new(cwd), command);
    let detected_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);

    let cwd_kind = if git2::Repository::discover(&effective_cwd).is_ok() {
        "git_repo"
    } else if effective_cwd.is_dir() {
        "non_git"
    } else {
        "unknown"
    };

    let mut signals = NativeTerminalToolSignals::default();
    collect_tool_signals(&effective_cwd, 0, &mut signals);

    CliNativeTerminalToolContextResult {
        cwd_kind: cwd_kind.to_string(),
        tools: build_tool_detections(&signals),
        scan_limited: signals.scan_limited,
        secrets_read: false,
        network_used: false,
        detected_at_ms,
    }
}

fn collect_tool_signals(path: &Path, depth: usize, signals: &mut NativeTerminalToolSignals) {
    if signals.scan_limited || depth > TOOL_SCAN_MAX_DEPTH {
        return;
    }

    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };

    for entry in entries.flatten() {
        if signals.entries_seen >= TOOL_SCAN_MAX_ENTRIES {
            signals.scan_limited = true;
            return;
        }
        signals.entries_seen += 1;

        let file_name = entry.file_name().to_string_lossy().to_string();
        let normalized = file_name.to_ascii_lowercase();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        if file_type.is_dir() {
            detect_safe_tool_directory(&normalized, signals);
            if should_descend_tool_directory(&normalized, depth) {
                collect_tool_signals(&entry.path(), depth + 1, signals);
            }
            continue;
        }

        if file_type.is_file() {
            detect_safe_tool_file(&normalized, signals);
        }
    }
}

fn detect_safe_tool_file(file_name: &str, signals: &mut NativeTerminalToolSignals) {
    if file_name.ends_with(".tf") || file_name.ends_with(".tfvars.example") {
        signals.terraform_config = true;
    }
    if file_name == ".terraform.lock.hcl" {
        signals.terraform_lock = true;
    }
    if matches!(
        file_name,
        "compose.yaml" | "compose.yml" | "docker-compose.yml" | "docker-compose.yaml"
    ) {
        signals.docker_compose = true;
    }
    if file_name == "chart.yaml" {
        signals.helm_chart = true;
    }
    if matches!(file_name, "kustomization.yaml" | "kustomization.yml") {
        signals.kubernetes_kustomization = true;
    }
}

fn detect_safe_tool_directory(dir_name: &str, signals: &mut NativeTerminalToolSignals) {
    if dir_name == "templates" {
        signals.helm_templates_dir = true;
    }
    if dir_name == "manifests" {
        signals.kubernetes_manifests_dir = true;
    }
}

fn should_descend_tool_directory(dir_name: &str, depth: usize) -> bool {
    depth < TOOL_SCAN_MAX_DEPTH && !TOOL_SCAN_IGNORED_DIRS.contains(&dir_name)
}

fn build_tool_detections(
    signals: &NativeTerminalToolSignals,
) -> Vec<CliNativeTerminalToolDetection> {
    let terraform_detected = signals.terraform_config || signals.terraform_lock;
    let docker_detected = signals.docker_compose;
    let helm_detected = signals.helm_chart || signals.helm_templates_dir;
    let kubernetes_detected =
        signals.kubernetes_kustomization || signals.kubernetes_manifests_dir;

    vec![
        tool_detection(
            "terraform",
            terraform_detected,
            if signals.terraform_config {
                "terraform_files_present"
            } else if signals.terraform_lock {
                "terraform_lockfile_present"
            } else {
                "not_detected"
            },
            &["terraform-fmt-check", "terraform-validate"],
        ),
        tool_detection(
            "docker-compose",
            docker_detected,
            if signals.docker_compose {
                "docker_compose_file_present"
            } else {
                "not_detected"
            },
            &["docker-compose-services", "docker-compose-check"],
        ),
        tool_detection(
            "helm",
            helm_detected,
            if signals.helm_chart {
                "helm_chart_metadata_present"
            } else if signals.helm_templates_dir {
                "helm_templates_directory_present"
            } else {
                "not_detected"
            },
            &["helm-lint-local"],
        ),
        tool_detection(
            "kubernetes",
            kubernetes_detected,
            if signals.kubernetes_kustomization {
                "kubernetes_kustomization_present"
            } else if signals.kubernetes_manifests_dir {
                "kubernetes_manifests_directory_present"
            } else {
                "not_detected"
            },
            &["kubectl-current-context", "kubectl-list-contexts"],
        ),
    ]
}

fn tool_detection(
    tool: &str,
    detected: bool,
    reason: &str,
    safe_command_ids: &[&str],
) -> CliNativeTerminalToolDetection {
    CliNativeTerminalToolDetection {
        tool: tool.to_string(),
        detected,
        confidence: if detected { "high" } else { "none" }.to_string(),
        reason: reason.to_string(),
        safe_command_ids: safe_command_ids.iter().map(|value| value.to_string()).collect(),
    }
}
