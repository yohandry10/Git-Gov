# KAN-65 Enterprise Route Auth Smoke Trend Artifact Monitor

Updated: 2026-05-02

## Summary

KAN-65 adds a freshness monitor for KAN-64 Enterprise Route Auth Smoke Trend artifacts.

## Scope

Implemented:

- `.github/workflows/enterprise-route-auth-smoke-trend-artifact-monitor.yml`.
- `docs/design/enterprise-route-auth-smoke-trend-artifact-monitor-mvp.md`.
- Runbook update in `docs/runbooks/enterprise-self-service-adoption.md`.

The implementation reuses the existing shared validator:

```text
scripts/control-plane/validate_github_evidence_report_artifact.ps1
```

## Safety

- No `.env` files are read.
- No provider token values are read or printed.
- No GitGov API key is read or printed.
- GitHub artifact metadata is read only through the GitHub token available to the caller.
- No GitHub Actions variables or secrets are created.
- No customer repository, provider, branch protection, trend generation, or workflow dispatch mutation is performed.
- No release governance default is changed.
- No Render deploy or database migration is required.

## Validation

Local validation:

| Command | Result |
| --- | --- |
| `.\scripts\control-plane\validate_github_evidence_report_artifact.ps1 -Repository yohandry10/Git-Gov -WorkflowFile enterprise-route-auth-smoke-trend-report.yml -ArtifactNamePrefix enterprise-route-auth-smoke-trend-report -MaxAgeHours 192 -OutputPath out\enterprise-route-auth-smoke-trend-artifact-monitor.json` | Passed |
| `git diff --check` | Passed |
| `.\scripts\security\publication_guard.ps1` | Passed |

PR, post-merge, and first workflow dispatch validation will be added after implementation merge.

Parsed local monitor output:

| Field | Result |
| --- | --- |
| Status | `PASS` |
| Workflow run | `25247310737` |
| Artifact | `enterprise-route-auth-smoke-trend-report` |
| Artifact ID | `6761702022` |
| Expired | `false` |
| Max age | `192h` |
| Observed age | `0.13h` |

## Current Status

KAN-65 is in progress.
