# KAN-63 Enterprise Route Auth Smoke Artifact Monitor

Updated: 2026-05-02

## Summary

KAN-63 adds a freshness monitor for KAN-62 Enterprise Route Auth Smoke artifacts.

## Scope

Implemented:

- `.github/workflows/enterprise-route-auth-smoke-artifact-monitor.yml`.
- `docs/design/enterprise-route-auth-smoke-artifact-monitor-mvp.md`.
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
- No customer repository, provider, branch protection, route smoke, or workflow dispatch mutation is performed.
- No release governance default is changed.
- No Render deploy or database migration is required.

## Validation

Local validation:

| Command | Result |
| --- | --- |
| `.\scripts\control-plane\validate_github_evidence_report_artifact.ps1 -Repository yohandry10/Git-Gov -WorkflowFile enterprise-route-auth-smoke.yml -ArtifactNamePrefix enterprise-route-auth-smoke- -MaxAgeHours 192 -OutputPath out\enterprise-route-auth-smoke-artifact-monitor.json` | Passed |
| `git diff --check` | Passed |
| `.\scripts\security\publication_guard.ps1` | Passed |

Parsed local monitor output:

| Field | Result |
| --- | --- |
| Status | `PASS` |
| Workflow run | `25246304135` |
| Artifact | `enterprise-route-auth-smoke-25246304135` |
| Artifact ID | `6761394808` |
| Expired | `false` |
| Max age | `192h` |
| Observed age | `0.15h` |

Additional validation will be recorded after PR checks and first workflow dispatch.

## Current Status

Implementation is ready for PR validation.
