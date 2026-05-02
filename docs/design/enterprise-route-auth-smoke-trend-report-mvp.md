# Enterprise Route Auth Smoke Trend Report MVP

Updated: 2026-05-02

## Goal

Track whether Enterprise route auth smoke results stay stable across recent workflow runs.

This adds historical visibility on top of:

- KAN-62: the smoke workflow itself.
- KAN-63: freshness monitoring for the latest smoke artifact.

## Scope

- Parse recent successful `enterprise-route-auth-smoke.yml` workflow runs.
- Download the latest `enterprise-route-auth-smoke-*` artifact from each run.
- Read `enterprise-route-auth-smoke.json`.
- Produce:
  - `out/enterprise-route-auth-smoke-trend-report.md`
  - `out/enterprise-route-auth-smoke-trend-report.json`
- Upload a single workflow artifact:
  - `enterprise-route-auth-smoke-trend-report`

## Data Model

Per parsed report:

- `workflow_run_id`
- `workflow_run_url`
- `workflow_created_at`
- `artifact_id`
- `artifact_name`
- `artifact_created_at`
- `checked_at_utc`
- `gitgov_url`
- `org_name`
- `repository_full_name`
- `release_id`
- `environment`
- `status`
- `total_checks`
- `passed_checks`
- `failed_checks`
- `anonymous_checks`
- `authenticated_checks`

Summary fields:

- `reports_analyzed`
- `successful_runs_scanned`
- `latest_successful_run_id`
- `latest_successful_run_artifact_status`
- `latest_status`
- `latest_total_checks`
- `latest_passed_checks`
- `latest_failed_checks`
- `passed_delta_vs_oldest`
- `failed_delta_vs_oldest`
- `runs_passed`
- `runs_failed`
- `runs_skipped`
- `trend_direction`
- `skipped_artifacts`

## Trend Rules

- `stable`: latest and oldest parsed reports have the same failure count.
- `improving`: latest report has fewer failed checks than the oldest parsed report.
- `declining`: latest report has more failed checks than the oldest parsed report.
- `failing`: the latest parsed report still contains failed checks, even if there is not yet a downward historical delta.

## Safety

- Read GitHub Actions artifacts only.
- Do not read `.env` files.
- Do not read or print provider secrets.
- Do not call GitGov protected routes directly.
- Do not mutate GitHub settings, workflows, branch protection, or repository secrets.
- Do not make release blocking the default.

## Scheduling

- Source workflow runs every Monday.
- Artifact freshness monitor runs every Tuesday.
- Trend report runs every Wednesday at `15:17 UTC`.

This keeps the auth-smoke evidence cadence ordered and predictable.
