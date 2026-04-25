# GitHub Evidence Stats Scope Fix

Date: 2026-04-25

## Problem

GitHub evidence report artifacts were fresh, but the latest executive/trend content reported `Sin evidencia` with `0/4 signals`.

Root cause:

- `scripts/control-plane/generate_github_evidence_report.ps1` reads `/stats.github_events.by_type`.
- The runtime-optimized `get_audit_stats` definitions from `supabase_schema_v18.sql` and `supabase_schema_v19.sql` returned `github_events.total=0`, `pushes_today=0`, `by_type={}`, and `active_repos=0`.
- Real GitHub webhook ingestion was working, but `/stats` hid that data from dashboard/report consumers.

## Fix

Added `gitgov/gitgov-server/supabase/supabase_schema_v22.sql`.

The migration restores GitHub stats in `get_audit_stats`:

- `github_events.total`
- `github_events.today`
- `github_events.pushes_today`
- `github_events.by_type`
- `active_repos`

The function keeps the v19 violation decision semantics intact.

Added `gitgov/gitgov-server/supabase/checks/v22_postcheck.sql` to verify that `/stats` GitHub counts match the `github_events` table.

## Expected Outcome

After applying `v22` in the target database:

- `GET /stats` should expose real GitHub event type counts.
- `github-evidence-report.yml` should generate non-empty GitHub evidence signal coverage when webhook evidence exists.
- `github-evidence-trend-report.yml` should stop reporting `0/4 signals` once it analyzes a fresh post-fix report artifact.

## Scope

This is a database stats/reporting fix. It does not change webhook ingestion, HMAC validation, PR merge materialization, Jira correlation, or branch protection behavior.

## Production Validation

The production database initially returned zeroed GitHub stats:

```json
{"today":0,"total":0,"by_type":{},"pushes_today":0}
```

`supabase_schema_v22.sql` was applied through `psql` using the ignored local `DATABASE_URL`.

`v22_postcheck.sql` passed all checks:

- `github_events.shape`: `PASS`
- `github_events.total_matches_table`: `PASS`
- `github_events.today_matches_table`: `PASS`
- `github_events.pushes_today_matches_table`: `PASS`
- `active_repos.is_number`: `PASS`

Post-migration production stats exposed real GitHub evidence:

```json
{
  "today": 2999,
  "total": 2999,
  "by_type": {
    "push": 110,
    "create": 37,
    "status": 148,
    "check_run": 1937,
    "check_suite": 599,
    "pull_request": 75,
    "issue_comment": 93
  },
  "pushes_today": 110
}
```

Local live report validation generated `docs/reports/github-evidence-executive-report-prod-v22-2026-04-25.md`:

- Status: `Parcial`
- Coverage: `3/4 signals`
- Missing signal: `Reviews`

GitHub-hosted artifact validation:

- GitHub Evidence Executive Report run `24942000355` passed; artifact `github-evidence-executive-report` ID `6643010178`.
- GitHub Evidence Artifact Monitor run `24942008460` passed; artifact `github-evidence-artifact-monitor` ID `6643012934`.
- GitHub Evidence Trend Report run `24942016196` passed; artifact `github-evidence-trend-report` ID `6643015713`.
- Trend artifact latest status: `Parcial`, latest coverage: `3/4 signals`, reports analyzed: `3`, coverage delta vs oldest report: `3` signals.

The previous `0/4 signals` implementation gap is closed. The remaining missing `Reviews` signal reflects the current event sample lacking `pull_request_review` events, not a stats ingestion failure.

## Review Signal Completion

PR `#71` was used to submit a real GitHub PR review event.

Post-review `/stats.github_events.by_type` included:

```json
{
  "pull_request_review": 1
}
```

Live report validation generated `docs/reports/github-evidence-executive-report-prod-review-v22-2026-04-25.md`:

- Status: `Completo`
- Coverage: `4/4 signals`
- Missing signals: `none`

This closes the GitHub executive evidence signal model for the observed production sample.
