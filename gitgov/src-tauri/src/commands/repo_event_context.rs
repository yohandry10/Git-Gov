use crate::git::{get_current_branch, get_head_commit_hash};
use crate::outbox::OutboxEvent;
use git2::Repository;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepoEventContext {
    pub repo_full_name: Option<String>,
    pub org_name: Option<String>,
    pub branch: Option<String>,
    pub head_commit_sha: Option<String>,
}

impl RepoEventContext {
    pub fn apply_to_event(
        &self,
        mut event: OutboxEvent,
        include_head_commit_sha: bool,
    ) -> OutboxEvent {
        if event.branch.is_none() {
            event.branch = self.branch.clone();
        }
        if include_head_commit_sha && event.commit_sha.is_none() {
            event.commit_sha = self.head_commit_sha.clone();
        }
        if let Some(full_name) = &self.repo_full_name {
            event = event.with_repo(full_name.clone());
        }
        if let Some(org) = &self.org_name {
            event = event.with_org(org.clone());
        }

        event
    }
}

pub fn resolve_repo_event_context(repo: &Repository) -> RepoEventContext {
    let repo_full_name = infer_repo_full_name(repo);
    let org_name = repo_full_name
        .as_deref()
        .and_then(infer_org_name_from_full_name);

    RepoEventContext {
        repo_full_name,
        org_name,
        branch: get_current_branch(repo).ok(),
        head_commit_sha: get_head_commit_hash(repo).ok(),
    }
}

pub fn infer_repo_full_name(repo: &Repository) -> Option<String> {
    let mut remote_names = Vec::new();
    let upstream_ref = repo
        .head()
        .ok()
        .and_then(|head| head.resolve().ok())
        .and_then(|head| head.shorthand().map(str::to_string))
        .and_then(|branch_name| repo.find_branch(&branch_name, git2::BranchType::Local).ok())
        .and_then(|branch| branch.upstream().ok())
        .and_then(|upstream| upstream.name().ok().flatten().map(str::to_string));

    if let Some(upstream) = upstream_ref {
        if let Some(remote_name) = upstream_remote_name(&upstream) {
            remote_names.push(remote_name.to_string());
        }
    }

    if !remote_names.iter().any(|existing| existing == "origin") {
        remote_names.push("origin".to_string());
    }

    for remote_name in &remote_names {
        let Ok(remote) = repo.find_remote(remote_name) else {
            continue;
        };
        if let Some(url) = remote.url().map(str::trim).filter(|url| !url.is_empty()) {
            if let Some(full_name) = parse_github_remote_full_name(url) {
                return Some(full_name);
            }
        }
    }

    let mut parseable_fallbacks = Vec::new();
    if let Ok(remotes) = repo.remotes() {
        for name in remotes.iter().flatten() {
            if remote_names.iter().any(|existing| existing == name) {
                continue;
            }
            let Ok(remote) = repo.find_remote(name) else {
                continue;
            };
            let Some(url) = remote.url().map(str::trim).filter(|url| !url.is_empty()) else {
                continue;
            };
            if let Some(full_name) = parse_github_remote_full_name(url) {
                if !parseable_fallbacks.contains(&full_name) {
                    parseable_fallbacks.push(full_name);
                }
            }
        }
    }

    if parseable_fallbacks.len() == 1 {
        return parseable_fallbacks.pop();
    }

    None
}

pub fn infer_org_name_from_full_name(repo_full_name: &str) -> Option<String> {
    let mut parts = repo_full_name.trim().split('/');
    let owner = parts.next()?.trim();
    let repo = parts.next()?.trim();

    if owner.is_empty()
        || repo.is_empty()
        || parts.next().is_some()
        || !is_valid_repo_full_name_part(owner)
        || !is_valid_repo_full_name_part(repo)
    {
        return None;
    }

    Some(owner.to_string())
}

pub fn parse_github_remote_full_name(raw_url: &str) -> Option<String> {
    let value = raw_url.trim();
    if value.is_empty() {
        return None;
    }

    if let Some(rest) = value.strip_prefix("git@github.com:") {
        return normalize_repo_path(rest);
    }

    let parsed = reqwest::Url::parse(value).ok()?;
    let host = parsed.host_str()?.to_ascii_lowercase();
    if host != "github.com" {
        return None;
    }

    match parsed.scheme() {
        "https" | "http" | "ssh" => normalize_repo_path(parsed.path().trim_start_matches('/')),
        _ => None,
    }
}

fn normalize_repo_path(path: &str) -> Option<String> {
    let clean_path = path.trim().trim_matches('/');
    let mut parts = clean_path.split('/');
    let owner = parts.next()?.trim();
    let repo = parts.next()?.trim().trim_end_matches(".git");

    if owner.is_empty()
        || repo.is_empty()
        || parts.next().is_some()
        || !is_valid_repo_full_name_part(owner)
        || !is_valid_repo_full_name_part(repo)
    {
        return None;
    }

    Some(format!("{owner}/{repo}"))
}

fn upstream_remote_name(upstream_ref: &str) -> Option<&str> {
    let rest = upstream_ref.strip_prefix("refs/remotes/")?;
    let (remote, branch) = rest.split_once('/')?;
    (!remote.trim().is_empty() && !branch.trim().is_empty()).then_some(remote)
}

fn is_valid_repo_full_name_part(part: &str) -> bool {
    part.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}

#[cfg(test)]
mod tests {
    use super::{
        infer_org_name_from_full_name, infer_repo_full_name, parse_github_remote_full_name,
        upstream_remote_name, RepoEventContext,
    };
    use crate::models::AuditStatus;
    use crate::outbox::OutboxEvent;
    use git2::Repository;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn parses_common_github_remote_urls() {
        assert_eq!(
            parse_github_remote_full_name("https://github.com/acme/repo.git").as_deref(),
            Some("acme/repo")
        );
        assert_eq!(
            parse_github_remote_full_name("git@github.com:acme/repo.git").as_deref(),
            Some("acme/repo")
        );
        assert_eq!(
            parse_github_remote_full_name("ssh://git@github.com/acme/repo.git").as_deref(),
            Some("acme/repo")
        );
        assert_eq!(
            parse_github_remote_full_name("https://github.com/acme/repo/").as_deref(),
            Some("acme/repo")
        );
        assert_eq!(
            parse_github_remote_full_name("git+ssh://git@github.com/acme/repo.git"),
            None
        );
    }

    #[test]
    fn rejects_non_github_or_incomplete_remote_urls() {
        assert_eq!(
            parse_github_remote_full_name("https://gitlab.com/acme/repo.git"),
            None
        );
        assert_eq!(
            parse_github_remote_full_name("https://github.com/acme"),
            None
        );
        assert_eq!(
            parse_github_remote_full_name("https://github.com/acme/repo/extra"),
            None
        );
        assert_eq!(
            parse_github_remote_full_name("https://github.com/acme/repo with spaces.git"),
            None
        );
        assert_eq!(
            parse_github_remote_full_name("git@github.com:acme/repo:bad.git"),
            None
        );
    }

    #[test]
    fn derives_org_only_from_exact_repo_full_name() {
        assert_eq!(
            infer_org_name_from_full_name("acme/repo").as_deref(),
            Some("acme")
        );
        assert_eq!(infer_org_name_from_full_name("repo"), None);
        assert_eq!(infer_org_name_from_full_name("acme/repo/extra"), None);
        assert_eq!(infer_org_name_from_full_name("acme/repo with spaces"), None);
        assert_eq!(infer_org_name_from_full_name("/repo"), None);
    }

    #[test]
    fn extracts_upstream_remote_name_from_remote_tracking_ref() {
        assert_eq!(
            upstream_remote_name("refs/remotes/upstream/main"),
            Some("upstream")
        );
        assert_eq!(
            upstream_remote_name("refs/remotes/origin/feature/KAN-77"),
            Some("origin")
        );
        assert_eq!(upstream_remote_name("refs/heads/main"), None);
        assert_eq!(upstream_remote_name("refs/remotes/origin"), None);
    }

    fn temp_repo_path(test_name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "gitgov-repo-event-context-{}-{}",
            test_name,
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&path).expect("create temp repo dir");
        path
    }

    #[test]
    fn infer_repo_full_name_prefers_origin_when_no_upstream() {
        let path = temp_repo_path("origin");
        let repo = Repository::init(&path).expect("init repo");
        repo.remote("origin", "https://github.com/acme/origin.git")
            .expect("add origin");
        repo.remote("fork", "https://github.com/other/fork.git")
            .expect("add fork");

        assert_eq!(infer_repo_full_name(&repo).as_deref(), Some("acme/origin"));

        fs::remove_dir_all(path).ok();
    }

    #[test]
    fn infer_repo_full_name_uses_single_parseable_fallback_remote() {
        let path = temp_repo_path("single-fallback");
        let repo = Repository::init(&path).expect("init repo");
        repo.remote("origin", "https://gitlab.com/acme/repo.git")
            .expect("add origin");
        repo.remote("github", "git@github.com:acme/repo.git")
            .expect("add github remote");

        assert_eq!(infer_repo_full_name(&repo).as_deref(), Some("acme/repo"));

        fs::remove_dir_all(path).ok();
    }

    #[test]
    fn infer_repo_full_name_rejects_ambiguous_fallback_remotes() {
        let path = temp_repo_path("ambiguous-fallback");
        let repo = Repository::init(&path).expect("init repo");
        repo.remote("origin", "https://gitlab.com/acme/repo.git")
            .expect("add origin");
        repo.remote("github-a", "git@github.com:acme/repo-a.git")
            .expect("add github-a");
        repo.remote("github-b", "git@github.com:acme/repo-b.git")
            .expect("add github-b");

        assert_eq!(infer_repo_full_name(&repo), None);

        fs::remove_dir_all(path).ok();
    }

    #[test]
    fn applies_repo_context_to_push_event_with_head_sha() {
        let context = RepoEventContext {
            repo_full_name: Some("acme/repo".to_string()),
            org_name: Some("acme".to_string()),
            branch: Some("main".to_string()),
            head_commit_sha: Some("f1d2d2f924e986ac86fdf7b36c94bcdf32beec15".to_string()),
        };
        let event = OutboxEvent::new(
            "successful_push".to_string(),
            "alice".to_string(),
            None,
            AuditStatus::Success,
        );

        let event = context.apply_to_event(event, true);

        assert_eq!(event.repo_full_name.as_deref(), Some("acme/repo"));
        assert_eq!(event.org_name.as_deref(), Some("acme"));
        assert_eq!(event.branch.as_deref(), Some("main"));
        assert_eq!(
            event.commit_sha.as_deref(),
            Some("f1d2d2f924e986ac86fdf7b36c94bcdf32beec15")
        );
    }
}
