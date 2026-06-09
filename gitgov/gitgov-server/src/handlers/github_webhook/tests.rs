#[cfg(test)]
mod github_webhook_tests {
    use super::{
        extract_check_run_evidence, extract_check_suite_evidence, extract_commit_status_evidence,
        extract_pull_request_review_comment_evidence, merged_pr_ticket_targets,
    };
    use serde_json::json;

    #[test]
    fn merged_pr_ticket_targets_prefers_merge_commit_then_head() {
        let targets = merged_pr_ticket_targets(Some("head-sha"), Some("merge-sha"));

        assert_eq!(
            targets,
            vec![
                ("pr_title", "merge-sha"),
                ("pr_title", "head-sha")
            ]
        );
    }

    #[test]
    fn merged_pr_ticket_targets_deduplicates_same_head_and_merge_commit() {
        let targets = merged_pr_ticket_targets(Some("ABCDEF"), Some("abcdef"));

        assert_eq!(targets, vec![("pr_title", "abcdef")]);
    }

    #[test]
    fn check_run_evidence_prefers_nested_suite_branch() {
        let payload = json!({
            "action": "completed",
            "check_run": {
                "status": "completed",
                "conclusion": "success",
                "head_sha": "abc123",
                "head_branch": "fallback-branch",
                "details_url": "https://github.com/example/actions/runs/1",
                "check_suite": {
                    "head_branch": "main"
                }
            }
        });

        let evidence = extract_check_run_evidence(&payload);

        assert_eq!(evidence.action, "completed");
        assert_eq!(evidence.status, "completed");
        assert_eq!(evidence.conclusion.as_deref(), Some("success"));
        assert_eq!(evidence.after_sha.as_deref(), Some("abc123"));
        assert_eq!(evidence.ref_name.as_deref(), Some("main"));
        assert_eq!(
            evidence.details_url.as_deref(),
            Some("https://github.com/example/actions/runs/1")
        );
    }

    #[test]
    fn check_run_evidence_falls_back_to_run_branch() {
        let payload = json!({
            "action": "rerequested",
            "check_run": {
                "status": "queued",
                "head_sha": "def456",
                "head_branch": "feature/KAN-4"
            }
        });

        let evidence = extract_check_run_evidence(&payload);

        assert_eq!(evidence.action, "rerequested");
        assert_eq!(evidence.status, "queued");
        assert_eq!(evidence.conclusion, None);
        assert_eq!(evidence.after_sha.as_deref(), Some("def456"));
        assert_eq!(evidence.ref_name.as_deref(), Some("feature/KAN-4"));
    }

    #[test]
    fn check_suite_evidence_extracts_branch_and_sha() {
        let payload = json!({
            "action": "completed",
            "check_suite": {
                "status": "completed",
                "conclusion": "failure",
                "head_sha": "suite-sha",
                "head_branch": "main"
            }
        });

        let evidence = extract_check_suite_evidence(&payload);

        assert_eq!(evidence.action, "completed");
        assert_eq!(evidence.status, "completed");
        assert_eq!(evidence.conclusion.as_deref(), Some("failure"));
        assert_eq!(evidence.after_sha.as_deref(), Some("suite-sha"));
        assert_eq!(evidence.ref_name.as_deref(), Some("main"));
    }

    #[test]
    fn commit_status_evidence_uses_first_branch() {
        let payload = json!({
            "state": "success",
            "context": "ci/build",
            "description": "Build passed",
            "target_url": "https://ci.example/run/42",
            "sha": "status-sha",
            "branches": [
                { "name": "main" },
                { "name": "release" }
            ]
        });

        let evidence = extract_commit_status_evidence(&payload);

        assert_eq!(evidence.state_name, "success");
        assert_eq!(evidence.context.as_deref(), Some("ci/build"));
        assert_eq!(evidence.description.as_deref(), Some("Build passed"));
        assert_eq!(evidence.target_url.as_deref(), Some("https://ci.example/run/42"));
        assert_eq!(evidence.after_sha.as_deref(), Some("status-sha"));
        assert_eq!(evidence.ref_name.as_deref(), Some("main"));
    }

    #[test]
    fn review_comment_evidence_prefers_comment_commit_sha() {
        let payload = json!({
            "action": "created",
            "pull_request": {
                "number": 47,
                "title": "KAN-4 harden traceability",
                "base": { "ref": "main" },
                "head": { "sha": "head-sha" }
            },
            "comment": {
                "commit_id": "comment-sha",
                "body": "Follow-up for KAN-4"
            }
        });

        let evidence = extract_pull_request_review_comment_evidence(&payload);

        assert_eq!(evidence.action, "created");
        assert_eq!(evidence.pr_number, 47);
        assert_eq!(evidence.pr_title.as_deref(), Some("KAN-4 harden traceability"));
        assert_eq!(evidence.base_branch.as_deref(), Some("main"));
        assert_eq!(evidence.head_sha.as_deref(), Some("head-sha"));
        assert_eq!(evidence.comment_commit_sha.as_deref(), Some("comment-sha"));
        assert_eq!(evidence.comment_body.as_deref(), Some("Follow-up for KAN-4"));
        assert_eq!(evidence.commit_sha.as_deref(), Some("comment-sha"));
    }

    #[test]
    fn review_comment_evidence_falls_back_to_head_sha() {
        let payload = json!({
            "pull_request": {
                "number": 6,
                "head": { "sha": "head-only-sha" }
            },
            "comment": {
                "body": "KAN-6 release evidence"
            }
        });

        let evidence = extract_pull_request_review_comment_evidence(&payload);

        assert_eq!(evidence.action, "unknown");
        assert_eq!(evidence.pr_number, 6);
        assert_eq!(evidence.comment_commit_sha, None);
        assert_eq!(evidence.commit_sha.as_deref(), Some("head-only-sha"));
    }
}
