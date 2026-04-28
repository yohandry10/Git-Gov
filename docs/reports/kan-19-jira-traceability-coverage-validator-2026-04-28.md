# KAN-19 Jira Traceability Coverage Validator

Date: 2026-04-28

## Purpose

Add a focused operational validator for Jira ticket coverage so operators do not need to run the full release readiness gate just to inspect traceability health.

## Result

Added:

- `scripts/control-plane/validate_jira_traceability_coverage.ps1`
- `docs/runbooks/jira-traceability-coverage.md`

## Behavior

The validator:

- loads ignored local env files by default
- uses `GITGOV_API_KEY` without printing it
- optionally refreshes Jira/PR correlations through `POST /integrations/jira/correlate`
- queries `GET /integrations/jira/ticket-coverage`
- supports `-MinCoverage` for threshold enforcement
- emits JSON evidence through stdout and optional `-OutputPath`

## Validation

Read-only validation:

```powershell
.\scripts\control-plane\validate_jira_traceability_coverage.ps1
```

Latest production validation:

```powershell
.\scripts\control-plane\validate_jira_traceability_coverage.ps1 -RefreshCorrelations -MinCoverage 50
```

Result:

- `ok=true`
- `scanned_commits=3`
- `scanned_prs=53`
- `correlations_created=0`
- `total_commits=56`
- `commits_with_ticket=54`
- `coverage_percentage=96.43`
- `commits_without_ticket=2`

Correlation refresh plus threshold:

```powershell
.\scripts\control-plane\validate_jira_traceability_coverage.ps1 -RefreshCorrelations -MinCoverage 50
```

## Remaining Work

No platform code is required. Keep using Jira IDs in branches, commits, PR titles, and PR comments so coverage remains healthy as new merges land.
