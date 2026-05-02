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
| PR `#188` `Security Guard` | Passed |
| PR `#188` `Server Clippy + Check` | Passed |
| PR `#188` `Desktop Rust Clippy` | Passed |
| PR `#188` `Frontend Lint + Typecheck` | Passed |
| PR `#188` `Website Lint + Typecheck + Build` | Passed |
| PR `#188` `Workflow Lint` | Passed |
| PR `#188` `Validate quality_gates warn/block matrix` | Passed |
| PR `#188` `Sonar Scan + Quality Gate` | Passed |
| PR `#188` `Block internal-assistant markers in branch/commits` | Passed |
| PR `#188` Vercel preview | Passed |

Post-merge validation:

| Check | Result |
| --- | --- |
| Main merge commit | `78d4878` |
| `CI` | Passed, run `25247988131` |
| `Release Readiness Gate` | Passed, run `25247988122` |
| `Quality Gate Policy Matrix (Optional)` | Passed, run `25247988133` |
| `Secret Scan` | Passed, run `25247988128` |
| `Public Naming Guard` | Passed, run `25247988129` |
| `Governance Correlation Smoke (Optional)` | Passed, run `25247988126` |
| `Desktop Updater Readiness (Optional)` | Passed, run `25247988124` |
| `SonarQube Governance (Non-Blocking)` | Passed, run `25247988127` |

First workflow dispatch:

| Field | Result |
| --- | --- |
| Workflow | `Enterprise Route Auth Smoke Trend Enforcement Artifact Monitor` |
| Run | `25248025190` |
| Conclusion | Passed |
| Artifact | `enterprise-route-auth-smoke-trend-enforcement-artifact-monitor` |
| Artifact ID | `6761892441` |
| Artifact expiry | `2026-07-31T08:35:58Z` |
| Parsed result | `status=PASS`, source enforcement run `25247747284`, source enforcement artifact `6761818040`, source age `0.28h` |

## Current Status

KAN-67 is implemented, merged, workflow-validated, and awaiting Jira closure.
