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

## Operational Note

The live report path depends on `/stats.github_events.by_type` for the API key scope being queried. If `/stats` returns an empty `by_type`, the report correctly shows `Sin evidencia`; operators should then verify API-key scope and GitHub webhook ingestion visibility before treating it as a product data gap.

The GitHub Actions workflow is intentionally optional. It skips without failing when `GITGOV_URL` or `GITGOV_API_KEY` is not configured, matching the existing optional governance-report pattern.

The artifact monitor uses GitHub Actions metadata, not GitGov provider credentials. It requires only a GitHub token with Actions read access and fails when the latest successful report run has no fresh `github-evidence-executive-report` artifact.
