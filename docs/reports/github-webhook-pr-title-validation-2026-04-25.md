# GitHub Webhook and PR-Title Traceability Validation

Date: 2026-04-25

## Scope

This report records the production validation work completed for GitGov GitHub webhook ingestion and Jira ticket traceability through merged pull request titles.

The validation used Jira ticket `KAN-4` and the production backend:

- Repository: `yohandry10/Git-Gov`
- Backend: `https://gitgov-api.onrender.com`
- Render service: `gitgov-api`
- GitHub webhook ID: `610772988`
- Jira project: `KAN`

No secret values are stored in this report.

## Access Configured

- GitHub CLI is authenticated as `yohandry10` and can inspect/administer the repository.
- Render API access is available locally through ignored env key `RENDER_API_KEY`.
- GitGov admin API access is available locally through ignored env key `GITGOV_API_KEY`.
- Jira Cloud API access is available locally through ignored env keys `JIRA_BASE_URL`, `JIRA_EMAIL`, `JIRA_API_TOKEN`, and `JIRA_PROJECT_KEY`.
- Local SonarQube and Jenkins API access remain available through ignored env files.

## Webhook Configuration

The GitHub repository webhook targets:

```text
https://gitgov-api.onrender.com/webhooks/github
```

Configured event families:

- `push`
- `create`
- `pull_request`
- `pull_request_review`
- `pull_request_review_comment`
- `issue_comment`
- `check_run`
- `check_suite`
- `status`

The webhook uses HMAC validation with `GITHUB_WEBHOOK_SECRET`, configured in both GitHub and Render.

## Fixes Completed

- Merged PR titles containing Jira ticket IDs now create commit-ticket correlations for merge/head SHAs.
- `POST /integrations/jira/correlate` scans recent merged PR titles as a backfill path.
- Duplicate or redelivered `pull_request` events for merged PRs no longer return before PR merge materialization and title correlation.
- GitHub organization upsert now resolves existing org rows by `login` before inserting/updating by `github_id`.
- PR-title correlations now use the allowed production source value `pr_title`.

## Production Validation

Observed production validation results:

- Real GitHub webhook deliveries returned HTTP `200`.
- A merged PR webhook redelivery returned `processed=true`.
- `pull_request_merges` contained at least `2` production records after validation.
- Jira backfill scanned `2` merged PRs and created `2` correlations.
- `KAN-4` commit-ticket correlations existed for validated merge/head SHAs using `source=pr_title`.

The validated correlation source must stay as:

```text
pr_title
```

Production DB constraints allow these source values:

- `branch_name`
- `commit_message`
- `pr_title`
- `manual`

## Original Remaining Gap

GitHub webhook ingestion is not the current blocker.

The remaining readiness blocker observed during the original validation was ticket coverage/readiness query semantics. After PR-title correlations existed, the live ticket coverage endpoint still reported:

- covered-universe commits: `3`
- commits with ticket: `1`
- coverage: `33.33%`

If merged PR evidence should count directly toward release readiness, update the ticket coverage/readiness aggregation so correlated PR merge evidence is included in the denominator used by that endpoint.

## Follow-up Fix

Implemented on 2026-04-25:

- `GET /integrations/jira/ticket-coverage` now builds its commit universe from both `client_events(event_type='commit')` and `pull_request_merges`.
- For PR merge evidence, the endpoint uses `merge_commit_sha` from webhook payload first and falls back to `head_sha`.
- Missing-commit output now includes a `source` field so operators can distinguish `client_event` from `pull_request_merge` evidence.
- Regression test: `ticket_coverage_counts_pr_merge_commit_without_client_event`.

## Next Step

After this change is deployed to Render, re-run Jira correlation and validate production ticket coverage for `yohandry10/Git-Gov` on `main`.
