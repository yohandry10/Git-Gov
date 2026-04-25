# GitHub Evidence Operations Runbook

Date: 2026-04-25

## Purpose

Operate the GitHub evidence reporting path after implementation:

- dashboard executive coverage
- dashboard local trend snapshots
- audit export packaging
- Markdown report artifact
- artifact freshness monitor
- multi-run artifact trend report

This runbook is operational memory. It must not contain API keys, webhook secrets, or raw provider tokens.

Latest operational baseline:

- 2026-04-25: `docs/reports/github-evidence-operational-adoption-2026-04-25.md`
- Workflows and local monitor/trend scripts passed.
- Data-quality follow-up `KAN-7` tracks the remaining `Sin evidencia` / `0/4 signals` report-content issue.

## Evidence Model

GitHub evidence is considered complete when all four families are present:

| Signal | Source events |
|---|---|
| PR lifecycle | `pull_request` |
| Reviews | `pull_request_review` |
| PR comments | `pull_request_review_comment`, PR-linked `issue_comment` |
| Checks/status | `check_run`, `check_suite`, `status` |

Status mapping:

| Status | Meaning |
|---|---|
| `Completo` | All four signal families are present. |
| `Parcial` | At least one, but not all, signal families are present. |
| `Sin evidencia` | No signal family is present in the observed stats/report. |

## Dashboard Procedure

Use this when reviewing GitHub evidence from the admin dashboard.

1. Open the Control Plane dashboard as an admin.
2. Confirm `GitHub por Tipo` shows the executive coverage badge and `n/4 señales`.
3. Review missing signal labels. If any are missing, verify repository webhook events and recent repository activity.
4. In `Trend evidencia GitHub`, click `Capturar` after validating the current dashboard state.
5. Confirm the widget shows:
   - current coverage
   - delta versus oldest local snapshot
   - number of local points
   - latest missing signals

Storage note:

- Dashboard snapshots are browser-local.
- Key: `gitgov.dashboard.github_evidence_trend`.
- This path does not read GitHub Actions artifacts and does not require a GitHub token in the frontend.

Acceptance criteria:

- Operator can explain the current GitHub evidence status from the dashboard.
- Operator captures a snapshot after each release/readiness review.
- Missing signals are treated as data-quality follow-up, not automatically as code failure.

## Manual Markdown Report

Use this when a standalone executive report is needed from the Control Plane `/stats` endpoint.

```powershell
powershell -ExecutionPolicy Bypass -File scripts/control-plane/generate_github_evidence_report.ps1 `
  -GitGovUrl "https://gitgov-api.onrender.com" `
  -ApiKey $env:GITGOV_API_KEY `
  -OrgName "yohandry10" `
  -OutputPath "out/github-evidence-executive-report.md"
```

Expected output:

- Markdown report.
- No provider secrets.
- `Status`, `Coverage`, and `Missing signals`.

If the report returns `Sin evidencia` while GitHub webhooks are known to be active:

1. Check API key scope/tenant visibility.
2. Confirm `/stats.github_events.by_type` is populated for the queried scope.
3. Confirm GitHub webhook deliveries are returning HTTP `200`.

## Artifact Freshness Monitor

Use this to verify the latest scheduled/manual report artifact is still available and recent.

```powershell
$env:GITHUB_TOKEN = & C:\Users\PC\Tools\gh\bin\gh.exe auth token
powershell -ExecutionPolicy Bypass -File scripts/control-plane/validate_github_evidence_report_artifact.ps1 `
  -Repository "yohandry10/Git-Gov" `
  -WorkflowFile "github-evidence-report.yml" `
  -ArtifactName "github-evidence-executive-report" `
  -MaxAgeHours 192 `
  -OutputPath "out/github-evidence-artifact-monitor.json"
Remove-Item Env:\GITHUB_TOKEN
```

Acceptance criteria:

- Script exits `0`.
- JSON status is `PASS`.
- Artifact is not expired.
- Artifact age is within the configured freshness window.

GitHub Actions workflow:

- `.github/workflows/github-evidence-artifact-monitor.yml`
- Manual dispatch plus Tuesday `14:07 UTC`.
- Artifact: `github-evidence-artifact-monitor`.

## Multi-Run Trend Report

Use this to compare recent report artifacts over time.

```powershell
$env:GITHUB_TOKEN = & C:\Users\PC\Tools\gh\bin\gh.exe auth token
powershell -ExecutionPolicy Bypass -File scripts/control-plane/generate_github_evidence_trend_report.ps1 `
  -Repository "yohandry10/Git-Gov" `
  -WorkflowFile "github-evidence-report.yml" `
  -ArtifactName "github-evidence-executive-report" `
  -MaxReports 12 `
  -OutputMarkdownPath "out/github-evidence-trend-report.md" `
  -OutputJsonPath "out/github-evidence-trend-report.json"
Remove-Item Env:\GITHUB_TOKEN
```

Acceptance criteria:

- Script exits `0`.
- Markdown and JSON outputs are created.
- `reports_analyzed` is greater than or equal to `1`.
- `coverage_delta_vs_oldest` is reviewed for regressions.

GitHub Actions workflow:

- `.github/workflows/github-evidence-trend-report.yml`
- Manual dispatch plus Tuesday `14:17 UTC`.
- Artifact: `github-evidence-trend-report`.

## Weekly Operating Cadence

Every release or weekly review:

1. Capture one dashboard local snapshot with `Capturar`.
2. Confirm the latest `github-evidence-executive-report` artifact exists.
3. Confirm the freshness monitor artifact is green.
4. Review the trend artifact for coverage regression.
5. If coverage is `Parcial` or `Sin evidencia`, open a data-quality follow-up with:
   - missing signal family
   - expected webhook event
   - latest relevant GitHub Actions run ID
   - latest webhook delivery status if available

## Escalation

Treat these as operational issues:

- latest report artifact missing
- report artifact expired
- trend report cannot parse any report artifact
- dashboard and artifact report disagree on coverage because they query different scopes

Treat these as data-quality issues:

- missing PR lifecycle evidence
- missing review evidence
- missing PR comment evidence
- missing check/status evidence

Treat these as configuration issues:

- `GITGOV_URL` missing in GitHub Actions variables
- `GITGOV_API_KEY` missing in GitHub Actions secrets
- GitHub webhook events not selected for PR/review/comment/check/status coverage
