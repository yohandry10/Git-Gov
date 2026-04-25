# GitHub Evidence Report Artifact Generator

Date: 2026-04-25

## Scope

Standalone executive reporting for GitHub evidence outside the dashboard download flow.

## Change

- Added `scripts/control-plane/generate_github_evidence_report.ps1`.
- The script generates a Markdown report from either:
  - live Control Plane `/stats`
  - an offline stats JSON fixture
- Added `.github/workflows/github-evidence-report.yml` to generate and upload the report as an optional artifact.
- Added `scripts/control-plane/validate_github_evidence_report_artifact.ps1` to validate operational freshness of the uploaded report artifact.
- Added `.github/workflows/github-evidence-artifact-monitor.yml` to run the artifact freshness check manually or weekly.
- Added `scripts/control-plane/generate_github_evidence_trend_report.ps1` to build Markdown/JSON trend history from uploaded report artifacts.
- Added `.github/workflows/github-evidence-trend-report.yml` to run the trend report manually or weekly.
- The signal model matches the dashboard/export package:
  - PR lifecycle
  - reviews
  - PR comments
  - checks/status

## Validation

Token-free fixture validation passed:

```powershell
.\scripts\control-plane\generate_github_evidence_report.ps1 `
  -StatsJsonPath "$env:TEMP\gitgov-github-evidence-stats.json" `
  -OutputPath "$env:TEMP\gitgov-github-evidence-report.md" `
  -OrgName yohandry10
```

Expected fixture result:

- Status: `Completo`
- Coverage: `4/4 signals`
- No missing signals

GitHub-hosted workflow validation passed:

- Run: `24939329055`
- Event: `workflow_dispatch`
- Commit: `3935c21`
- Job: `Generate GitHub evidence report`
- Artifact: `github-evidence-executive-report`
- Artifact upload: successful

Artifact monitor live validation passed:

```powershell
.\scripts\control-plane\validate_github_evidence_report_artifact.ps1 `
  -Repository yohandry10/Git-Gov `
  -WorkflowFile github-evidence-report.yml `
  -ArtifactName github-evidence-executive-report `
  -MaxAgeHours 192 `
  -OutputPath "$env:TEMP\github-evidence-artifact-monitor.json"
```

Observed result:

- Status: `PASS`
- Report workflow run: `24939329055`
- Artifact ID: `6642253304`
- Artifact expired: `false`
- Freshness window: `192h`

GitHub-hosted artifact monitor validation passed:

- Run: `24939815276`
- Event: `workflow_dispatch`
- Job: `Validate GitHub evidence report artifact`
- Artifact: `github-evidence-artifact-monitor`
- Artifact ID: `6642391452`
- Artifact expired: `false`
- Artifact upload: successful

Trend report live validation passed:

```powershell
.\scripts\control-plane\generate_github_evidence_trend_report.ps1 `
  -Repository yohandry10/Git-Gov `
  -WorkflowFile github-evidence-report.yml `
  -ArtifactName github-evidence-executive-report `
  -MaxReports 12 `
  -OutputMarkdownPath out\github-evidence-trend-report.md `
  -OutputJsonPath out\github-evidence-trend-report.json
```

Observed result:

- Reports analyzed: `1`
- Latest report workflow run: `24939329055`
- Latest coverage: `0/4 signals`
- Output files: Markdown + JSON trend report

GitHub-hosted trend workflow validation passed:

- Run: `24940027811`
- Event: `workflow_dispatch`
- Commit: `a58ae81`
- Job: `Generate GitHub evidence trend report`
- Artifact: `github-evidence-trend-report`
- Artifact ID: `6642453325`
- Artifact expired: `false`
- Artifact upload: successful

Post-merge validation for the trend workflow rollout passed:

- Commit: `a58ae81`
- CI: `24940024455`
- Quality Gate Policy Matrix: `24940024458`
- Release Readiness Gate: `24940024457`

## Operational Note

The live report path depends on `/stats.github_events.by_type` for the API key scope being queried. If `/stats` returns an empty `by_type`, the report correctly shows `Sin evidencia`; operators should then verify API-key scope and GitHub webhook ingestion visibility before treating it as a product data gap.

The GitHub Actions workflow is intentionally optional. It skips without failing when `GITGOV_URL` or `GITGOV_API_KEY` is not configured, matching the existing optional governance-report pattern.

The artifact monitor uses GitHub Actions metadata, not GitGov provider credentials. It requires only a GitHub token with Actions read access and fails when the latest successful report run has no fresh `github-evidence-executive-report` artifact.

The trend report also uses GitHub Actions artifact metadata and artifact contents only. It does not query GitGov directly and does not expose provider secrets. If the latest report says `Sin evidencia`, use the existing `/stats.github_events.by_type` visibility note to investigate API key scope or webhook visibility.

Dashboard trend productization:

- `GitHubEvidenceTrendWidget` provides an in-dashboard local trend view.
- Operators explicitly click `Capturar` to store a browser-local snapshot of the current GitHub evidence coverage.
- Snapshots live in `localStorage` under `gitgov.dashboard.github_evidence_trend`.
- This UI path intentionally does not read GitHub Actions artifacts because that would require exposing GitHub token access to the frontend.
- Post-merge validation for the dashboard trend widget passed on `main` commit `74a51a5` with CI run `24940280762`, Quality Gate Policy Matrix run `24940280775`, and Release Readiness Gate run `24940280751`.

Operational adoption:

- `docs/runbooks/github-evidence-operations.md` defines the dashboard capture procedure, manual Markdown report generation, artifact freshness monitor, multi-run trend report, weekly cadence, and escalation criteria.
- This runbook is the source of truth for operating the GitHub evidence reporting path without exposing GitHub, GitGov, Render, Jenkins, Sonar, or Jira secrets.
- Post-merge validation for the runbook rollout passed on `main` commit `7577f90`: CI `24940874607`, Quality Gate Policy Matrix `24940874602`, Release Readiness Gate `24940874616`, Secret Scan `24940874599`, SonarQube Governance `24940874600`, Public Naming Guard `24940874603`, Governance Correlation Smoke `24940874611`, and Desktop Updater Readiness `24940874597`.
