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
    let remote = repo.find_remote("origin").ok()?;
    parse_github_remote_full_name(remote.url()?.trim())
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

fn is_valid_repo_full_name_part(part: &str) -> bool {
    part.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}

#[cfg(test)]
mod tests {
    use super::{infer_org_name_from_full_name, parse_github_remote_full_name, RepoEventContext};
    use crate::models::AuditStatus;
    use crate::outbox::OutboxEvent;

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
