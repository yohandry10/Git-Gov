use serde_json::json;

/// Send a Slack-compatible webhook alert (fire-and-forget).
/// Errors are logged as warnings but never propagate — the caller must not await this in the hot path.
pub async fn send_alert(client: &reqwest::Client, webhook_url: &str, text: String) {
    let payload = json!({ "text": text });
    match client.post(webhook_url).json(&payload).send().await {
        Ok(resp) if resp.status().is_success() => {
            tracing::debug!("Alert webhook delivered");
        }
        Ok(resp) => {
            tracing::warn!(
                status = %resp.status(),
                "Alert webhook returned non-success status"
            );
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to deliver alert webhook");
        }
    }
}

pub fn format_blocked_push_alert(actor: &str, repo: &str, branch: &str) -> String {
    format!(
        ":no_entry: *Blocked Push* — `{actor}` intentó hacer push a `{branch}` en `{repo}`. \
         El push fue bloqueado por política de gobernanza.",
        actor = actor,
        branch = branch,
        repo = repo
    )
}

pub fn format_signal_confirmed_alert(signal_type: &str, actor: &str, repo: Option<&str>) -> String {
    let repo_part = repo.map(|r| format!(" en `{r}`")).unwrap_or_default();
    format!(
        ":warning: *Signal Confirmada* — Tipo: `{signal_type}` | Actor: `{actor}`{repo_part}. \
         Revisar el dashboard de GitGov para más detalles.",
        signal_type = signal_type,
        actor = actor,
        repo_part = repo_part
    )
}

pub fn format_critical_policy_drift_alert(
    actor: &str,
    repo: &str,
    drift_count: i64,
    critical_count: i64,
) -> String {
    format!(
        ":rotating_light: *Policy Drift Crítico* — `{actor}` detectó drift crítico en `{repo}`. \
         Drift total: `{drift_count}` | Drift crítico: `{critical_count}`. \
         Revisar y resolver desde Pipeline Drift Detection (sync/push policy).",
        actor = actor,
        repo = repo,
        drift_count = drift_count,
        critical_count = critical_count
    )
}

pub fn format_quality_gate_policy_alert(
    actor: &str,
    repo: &str,
    branch: &str,
    commit_sha: &str,
    job_name: &str,
    gate_status: &str,
    enforcement: &str,
) -> String {
    format!(
        ":triangular_flag_on_post: *Quality Gate no verde* — Actor `{actor}` en `{repo}` (`{branch}`) \
         commit `{commit}` | job `{job}` | status `{status}` | enforcement `{enforcement}`.",
        actor = actor,
        repo = repo,
        branch = branch,
        commit = commit_sha,
        job = job_name,
        status = gate_status,
        enforcement = enforcement
    )
}

#[cfg(test)]
mod tests {
    use super::{
        format_blocked_push_alert, format_critical_policy_drift_alert,
        format_quality_gate_policy_alert, format_signal_confirmed_alert,
    };

    #[test]
    fn blocked_push_alert_contains_actor_repo_branch() {
        let text = format_blocked_push_alert("alice", "org/repo", "main");
        assert!(text.contains("alice"));
        assert!(text.contains("org/repo"));
        assert!(text.contains("main"));
    }

    #[test]
    fn signal_confirmed_alert_includes_optional_repo() {
        let with_repo =
            format_signal_confirmed_alert("policy_violation", "alice", Some("org/repo"));
        assert!(with_repo.contains("policy_violation"));
        assert!(with_repo.contains("alice"));
        assert!(with_repo.contains("org/repo"));

        let without_repo = format_signal_confirmed_alert("policy_violation", "alice", None);
        assert!(without_repo.contains("policy_violation"));
        assert!(without_repo.contains("alice"));
    }

    #[test]
    fn critical_policy_drift_alert_contains_counts() {
        let text = format_critical_policy_drift_alert("alice", "org/repo", 5, 2);
        assert!(text.contains("alice"));
        assert!(text.contains("org/repo"));
        assert!(text.contains("5"));
        assert!(text.contains("2"));
    }

    #[test]
    fn quality_gate_alert_contains_all_key_fields() {
        let text = format_quality_gate_policy_alert(
            "jenkins",
            "org/repo",
            "main",
            "abc1234",
            "sonar-governance",
            "failure",
            "block",
        );
        assert!(text.contains("jenkins"));
        assert!(text.contains("org/repo"));
        assert!(text.contains("main"));
        assert!(text.contains("abc1234"));
        assert!(text.contains("sonar-governance"));
        assert!(text.contains("failure"));
        assert!(text.contains("block"));
    }
}
