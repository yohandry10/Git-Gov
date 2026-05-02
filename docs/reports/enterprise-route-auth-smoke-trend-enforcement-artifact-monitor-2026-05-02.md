# KAN-67 Enterprise Route Auth Smoke Trend Enforcement Artifact Monitor

Updated: 2026-05-02

## Summary

KAN-67 adds a freshness monitor for KAN-66 Enterprise Route Auth Smoke Trend Enforcement artifacts.

## Scope

Implemented:

- `.github/workflows/enterprise-route-auth-smoke-trend-enforcement-artifact-monitor.yml`.
- `docs/design/enterprise-route-auth-smoke-trend-enforcement-artifact-monitor-mvp.md`.
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
- No customer repository, provider, branch protection, trend generation, trend enforcement, or workflow dispatch mutation is performed.
- No release governance default is changed.
- No Render deploy or database migration is required.

## Validation

Local validation:

| Command | Result |
| --- | --- |
| `.\scripts\control-plane\validate_github_evidence_report_artifact.ps1 -Repository yohandry10/Git-Gov -WorkflowFile enterprise-route-auth-smoke-trend-enforcement.yml -ArtifactName enterprise-route-auth-smoke-trend-enforcement -MaxAgeHours 192 -OutputPath out\enterprise-route-auth-smoke-trend-enforcement-artifact-monitor.json` | Passed |
| `git diff --check` | Passed |
| `.\scripts\security\publication_guard.ps1` | Passed |

Parsed local monitor output:

| Field | Result |
| --- | --- |
| Status | `PASS` |
| Workflow run | `25247747284` |
| Artifact | `enterprise-route-auth-smoke-trend-enforcement` |
| Artifact ID | `6761818040` |
| Expired | `false` |
| Max age | `192h` |
| Observed age | `0.17h` |

PR validation:

| Check | Result |
| --- | --- |
| PR `#TBD` `Security Guard` | Pending |
| PR `#TBD` `Server Clippy + Check` | Pending |
| PR `#TBD` `Desktop Rust Clippy` | Pending |
| PR `#TBD` `Frontend Lint + Typecheck` | Pending |
| PR `#TBD` `Website Lint + Typecheck + Build` | Pending |
| PR `#TBD` `Workflow Lint` | Pending |
| PR `#TBD` `Validate quality_gates warn/block matrix` | Pending |
| PR `#TBD` `Sonar Scan + Quality Gate` | Pending |
| PR `#TBD` `Block internal-assistant markers in branch/commits` | Pending |
| PR `#TBD` Vercel preview | Pending |

Post-merge validation:

| Check | Result |
| --- | --- |
| Main merge commit | Pending |
| `CI` | Pending |
| `Release Readiness Gate` | Pending |

First workflow dispatch:

| Field | Result |
| --- | --- |
| Workflow | `Enterprise Route Auth Smoke Trend Enforcement Artifact Monitor` |
| Run | Pending |
| Conclusion | Pending |
| Artifact | `enterprise-route-auth-smoke-trend-enforcement-artifact-monitor` |
| Artifact ID | Pending |
| Artifact expiry | Pending |
| Parsed result | Pending |

## Current Status

KAN-67 is implemented locally and awaiting validation, PR checks, merge, first workflow dispatch, and Jira closure.
