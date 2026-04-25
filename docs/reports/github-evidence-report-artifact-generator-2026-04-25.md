# GitHub Evidence Report Artifact Generator

Date: 2026-04-25

## Scope

Standalone executive reporting for GitHub evidence outside the dashboard download flow.

## Change

- Added `scripts/control-plane/generate_github_evidence_report.ps1`.
- The script generates a Markdown report from either:
  - live Control Plane `/stats`
  - an offline stats JSON fixture
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

## Operational Note

The live report path depends on `/stats.github_events.by_type` for the API key scope being queried. If `/stats` returns an empty `by_type`, the report correctly shows `Sin evidencia`; operators should then verify API-key scope and GitHub webhook ingestion visibility before treating it as a product data gap.
