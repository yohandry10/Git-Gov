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

PR validation:

| Check | Result |
| --- | --- |
| PR `#184` `Security Guard` | Passed |
| PR `#184` `Server Clippy + Check` | Passed |
| PR `#184` `Desktop Rust Clippy` | Passed |
| PR `#184` `Frontend Lint + Typecheck` | Passed |
| PR `#184` `Website Lint + Typecheck + Build` | Passed |
| PR `#184` `Workflow Lint` | Passed |
| PR `#184` `Validate quality_gates warn/block matrix` | Passed |
| PR `#184` `Sonar Scan + Quality Gate` | Passed |
| PR `#184` `Block internal-assistant markers in branch/commits` | Passed |
| PR `#184` Vercel preview | Passed |

Post-merge validation:

| Check | Result |
| --- | --- |
| Main merge commit | `8bd9cf0` |
| `CI` | Passed, run `25247484224` |
| `Release Readiness Gate` | Passed, run `25247484227` |
| `Quality Gate Policy Matrix (Optional)` | Passed, run `25247484230` |
| `Secret Scan` | Passed, run `25247484222` |
| `Public Naming Guard` | Passed, run `25247484226` |
| `Governance Correlation Smoke (Optional)` | Passed, run `25247484223` |
| `Desktop Updater Readiness (Optional)` | Passed, run `25247484225` |
| `SonarQube Governance (Non-Blocking)` | Passed, run `25247484231` |

First workflow dispatch:

| Field | Result |
| --- | --- |
| Workflow | `Enterprise Route Auth Smoke Trend Artifact Monitor` |
| Run | `25247519159` |
| Conclusion | Passed |
| Artifact | `enterprise-route-auth-smoke-trend-artifact-monitor` |
| Artifact ID | `6761758944` |
| Artifact expiry | `2026-07-31T08:05:56Z` |
| Parsed result | `status=PASS`, source trend run `25247310737`, source trend artifact `6761702022`, source age `0.2h` |

## Current Status

KAN-65 is implemented, merged, workflow-validated, and documented.
