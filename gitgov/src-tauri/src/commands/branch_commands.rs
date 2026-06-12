use super::repo_event_context::resolve_repo_event_context;
use crate::audit::AuditDatabase;
use crate::config::{load_config, validate_branch_name};
use crate::git::{
    checkout_branch, create_branch, get_branch_sync_status, get_current_branch,
    get_pending_push_preview, list_branches, open_repository, BranchInfo, BranchSyncStatus,
    PendingPushPreview,
};
use crate::models::{AuditAction, AuditLogEntry, AuditStatus};
use crate::outbox::{Outbox, OutboxEvent};
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchActorInput {
    pub developer_login: String,
    pub is_admin: bool,
    pub user_group: Option<String>,
}

fn to_command_error(e: impl std::fmt::Display, code: &str) -> String {
    serde_json::to_string(&serde_json::json!({
        "code": code,
        "message": e.to_string()
    }))
    .unwrap_or_else(|_| format!("{{\"code\":\"{}\",\"message\":\"{}\"}}", code, e))
}

fn trigger_flush(outbox: &Arc<Outbox>) {
    outbox.notify_flush();
}

#[tauri::command]
pub fn cmd_list_branches(repo_path: String) -> Result<Vec<BranchInfo>, String> {
    let repo = open_repository(&repo_path).map_err(|e| to_command_error(e, "GIT_ERROR"))?;

    let branches = list_branches(&repo).map_err(|e| to_command_error(e, "GIT_ERROR"))?;

    Ok(branches)
}

#[tauri::command]
pub fn cmd_get_current_branch(repo_path: String) -> Result<String, String> {
    let repo = open_repository(&repo_path).map_err(|e| to_command_error(e, "GIT_ERROR"))?;

    let branch = get_current_branch(&repo).map_err(|e| to_command_error(e, "GIT_ERROR"))?;

    Ok(branch)
}

#[tauri::command]
pub fn cmd_get_branch_sync_status(
    repo_path: String,
    branch: Option<String>,
) -> Result<BranchSyncStatus, String> {
    let repo = open_repository(&repo_path).map_err(|e| to_command_error(e, "GIT_ERROR"))?;

    let status = get_branch_sync_status(&repo, branch.as_deref())
        .map_err(|e| to_command_error(e, "GIT_ERROR"))?;

    Ok(status)
}

#[tauri::command]
pub fn cmd_get_pending_push_preview(
    repo_path: String,
    branch: Option<String>,
) -> Result<PendingPushPreview, String> {
    let repo = open_repository(&repo_path).map_err(|e| to_command_error(e, "GIT_ERROR"))?;

    let preview = get_pending_push_preview(&repo, branch.as_deref())
        .map_err(|e| to_command_error(e, "GIT_ERROR"))?;

    Ok(preview)
}

#[tauri::command]
pub fn cmd_create_branch(
    repo_path: String,
    name: String,
    from_branch: String,
    actor: BranchActorInput,
    audit_db: State<'_, Arc<AuditDatabase>>,
    outbox: State<'_, Arc<Outbox>>,
) -> Result<(), String> {
    use crate::models::AuthenticatedUser;

    let repo = open_repository(&repo_path).map_err(|e| to_command_error(e, "GIT_ERROR"))?;
    let repo_context = resolve_repo_event_context(&repo);

    let config = load_config(&repo_path);

    if let Ok(cfg) = &config {
        let user = AuthenticatedUser {
            id: None,
            login: actor.developer_login.clone(),
            name: actor.developer_login.clone(),
            avatar_url: String::new(),
            group: actor.user_group,
            is_admin: actor.is_admin,
        };

        let validation = validate_branch_name(&name, cfg, &user);

        if let crate::config::ValidationResult::Blocked(reason) = validation {
            let mut blocked_event = OutboxEvent::new(
                "blocked_branch".to_string(),
                actor.developer_login.clone(),
                Some(name.clone()),
                AuditStatus::Blocked,
            )
            .with_reason(reason.clone())
            .with_metadata(serde_json::json!({ "from_branch": from_branch }));
            blocked_event = repo_context.apply_to_event(blocked_event, false);

            let _ = outbox.add(blocked_event);

            let entry = AuditLogEntry {
                id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now().timestamp_millis(),
                developer_login: actor.developer_login.clone(),
                developer_name: actor.developer_login.clone(),
                action: AuditAction::BlockedBranch,
                branch: name.clone(),
                files: vec![],
                commit_hash: None,
                status: AuditStatus::Blocked,
                reason: Some(reason.clone()),
            };
            let _ = audit_db.insert(&entry);

            trigger_flush(&outbox);

            return Err(to_command_error(reason, "BLOCKED"));
        }
    }

    match create_branch(&repo, &name, &from_branch) {
        Ok(()) => {
            let mut success_event = OutboxEvent::new(
                "create_branch".to_string(),
                actor.developer_login.clone(),
                Some(name.clone()),
                AuditStatus::Success,
            )
            .with_metadata(serde_json::json!({ "from_branch": from_branch }));
            success_event = repo_context.apply_to_event(success_event, false);

            let _ = outbox.add(success_event);

            let entry = AuditLogEntry {
                id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now().timestamp_millis(),
                developer_login: actor.developer_login,
                developer_name: String::new(),
                action: AuditAction::BranchCreate,
                branch: name,
                files: vec![],
                commit_hash: None,
                status: AuditStatus::Success,
                reason: None,
            };
            let _ = audit_db.insert(&entry);

            trigger_flush(&outbox);

            Ok(())
        }
        Err(e) => {
            let mut failed_event = OutboxEvent::new(
                "create_branch".to_string(),
                actor.developer_login,
                Some(name),
                AuditStatus::Failed,
            )
            .with_reason(e.to_string())
            .with_metadata(serde_json::json!({ "from_branch": from_branch }));
            failed_event = repo_context.apply_to_event(failed_event, false);

            let _ = outbox.add(failed_event);

            trigger_flush(&outbox);

            Err(to_command_error(e, "GIT_ERROR"))
        }
    }
}

#[tauri::command]
pub fn cmd_checkout_branch(
    repo_path: String,
    name: String,
    developer_login: String,
    outbox: State<'_, Arc<Outbox>>,
) -> Result<(), String> {
    let repo = open_repository(&repo_path).map_err(|e| to_command_error(e, "GIT_ERROR"))?;
    let before_context = resolve_repo_event_context(&repo);

    match checkout_branch(&repo, &name) {
        Ok(()) => {
            let after_context = resolve_repo_event_context(&repo);
            let mut event = OutboxEvent::new(
                "checkout_branch".to_string(),
                developer_login,
                Some(name.clone()),
                AuditStatus::Success,
            )
            .with_metadata(serde_json::json!({
                "from_branch": before_context.branch,
                "to_branch": name
            }));
            event = after_context.apply_to_event(event, true);
            let _ = outbox.add(event);
            trigger_flush(&outbox);
            Ok(())
        }
        Err(e) => {
            let mut event = OutboxEvent::new(
                "checkout_branch".to_string(),
                developer_login,
                Some(name.clone()),
                AuditStatus::Failed,
            )
            .with_reason(e.to_string())
            .with_metadata(serde_json::json!({
                "from_branch": before_context.branch,
                "to_branch": name
            }));
            event = before_context.apply_to_event(event, true);
            let _ = outbox.add(event);
            trigger_flush(&outbox);
            Err(to_command_error(e, "GIT_ERROR"))
        }
    }
}
