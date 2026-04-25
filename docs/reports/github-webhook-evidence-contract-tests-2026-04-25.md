# GitHub Webhook Evidence Contract Tests

Date: 2026-04-25

## Scope

Webhook evidence parsing for GitHub status-check and PR comment events.

## Change

- Refactored evidence extraction into pure functions for:
  - `check_run`
  - `check_suite`
  - `status`
  - `pull_request_review_comment`
- Added unit tests in `github_webhook_tests` to validate branch, SHA, status/conclusion, details URL, and PR review comment ticket-source metadata extraction.

## Validation

```powershell
cd gitgov\gitgov-server
cargo test github_webhook_tests
```

Result: `8 passed`.

## Operational Value

These tests protect the GitHub evidence contract without requiring database access, GitHub webhook redelivery, Render credentials, or provider tokens.
