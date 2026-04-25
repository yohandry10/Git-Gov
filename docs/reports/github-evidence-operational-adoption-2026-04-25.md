# GitHub Evidence Operational Adoption

Date: 2026-04-25

## Scope

Operational adoption run for the GitHub evidence reporting path documented in `docs/runbooks/github-evidence-operations.md`.

This validates:

- manual executive report workflow
- artifact freshness monitor workflow
- multi-run trend workflow
- local artifact freshness check
- local trend generation
- data-quality escalation when coverage remains incomplete

## GitHub Actions Runs

| Workflow | Run | Result | Artifact | Artifact ID |
|---|---:|---|---|---:|
| `github-evidence-report.yml` | `24941348198` | `success` | `github-evidence-executive-report` | `6642829154` |
| `github-evidence-artifact-monitor.yml` | `24941358185` | `success` | `github-evidence-artifact-monitor` | `6642831722` |
| `github-evidence-trend-report.yml` | `24941363195` | `success` | `github-evidence-trend-report` | `6642833188` |

All three workflows ran on `main` commit `65613b0`.

## Local Runbook Validation

Artifact freshness monitor:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/control-plane/validate_github_evidence_report_artifact.ps1 `
  -Repository "yohandry10/Git-Gov" `
  -WorkflowFile "github-evidence-report.yml" `
  -ArtifactName "github-evidence-executive-report" `
  -MaxAgeHours 192 `
  -OutputPath "out/github-evidence-operational-monitor-2026-04-25.json"
```

Result:

- Status: `PASS`
- Latest report workflow run: `24941348198`
- Latest report artifact ID: `6642829154`
- Artifact expired: `false`
- Artifact age at validation: `0.02h`

Trend report:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/control-plane/generate_github_evidence_trend_report.ps1 `
  -Repository "yohandry10/Git-Gov" `
  -WorkflowFile "github-evidence-report.yml" `
  -ArtifactName "github-evidence-executive-report" `
  -MaxReports 12 `
  -OutputMarkdownPath "out/github-evidence-operational-trend-2026-04-25.md" `
  -OutputJsonPath "out/github-evidence-operational-trend-2026-04-25.json"
```

Result:

- Reports analyzed: `2`
- Latest status: `Sin evidencia`
- Latest coverage: `0/4 signals`
- Coverage delta vs oldest report: `0`
- Complete reports: `0`

## Data-Quality Follow-Up

The artifact path is operational and fresh, but the executive evidence content still reports `Sin evidencia` / `0/4 signals`.

Missing signal families:

- PR lifecycle
- Reviews
- PR comments
- Checks/status

This is a data-quality/scope follow-up, not an artifact freshness or workflow failure.

Jira follow-up created:

- `KAN-7` - Operational follow-up: GitHub evidence report stats scope returns 0/4 signals

Investigation target:

- Verify API-key scope or `/stats.github_events.by_type` tenant visibility against already validated GitHub webhook deliveries to Render.

## Operational Decision

The GitHub evidence operations path is adopted:

- report artifact generation works
- artifact freshness monitoring works
- trend reporting works
- local runbook validation works
- data-quality escalation is tracked in Jira

Dashboard-local snapshots remain an operator review action because they are stored in browser `localStorage` under `gitgov.dashboard.github_evidence_trend` and are intentionally not generated from CI or backend tokens.
