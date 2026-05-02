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

PR validation:

| Check | Result |
| --- | --- |
| PR `#180` `Security Guard` | Passed |
| PR `#180` `Server Clippy + Check` | Passed |
| PR `#180` `Desktop Rust Clippy` | Passed |
| PR `#180` `Frontend Lint + Typecheck` | Passed |
| PR `#180` `Website Lint + Typecheck + Build` | Passed |
| PR `#180` `Workflow Lint` | Passed |
| PR `#180` `Validate quality_gates warn/block matrix` | Passed |
| PR `#180` `Sonar Scan + Quality Gate` | Passed |
| PR `#180` `Block internal-assistant markers in branch/commits` | Passed |
| PR `#180` Vercel preview | Passed |

Post-merge validation:

| Check | Result |
| --- | --- |
| Main merge commit | `4342947` |
| `CI` | Passed, run `25246990171` |
| `Release Readiness Gate` | Passed, run `25246990161` |
| `Quality Gate Policy Matrix (Optional)` | Passed, run `25246990166` |
| `Secret Scan` | Passed, run `25246990197` |
| `Public Naming Guard` | Passed, run `25246990188` |
| `Governance Correlation Smoke (Optional)` | Passed, run `25246990170` |
| `Desktop Updater Readiness (Optional)` | Passed, run `25246990174` |
| `SonarQube Governance (Non-Blocking)` | Passed, run `25246990176` |

First workflow dispatch:

| Field | Result |
| --- | --- |
| Workflow | `Enterprise Route Auth Smoke Artifact Monitor` |
| Run | `25247025700` |
| Conclusion | Passed |
| Artifact | `enterprise-route-auth-smoke-artifact-monitor` |
| Artifact ID | `6761616364` |
| Artifact expiry | `2026-07-31T07:37:28Z` |
| Parsed result | `status=PASS`, source run `25246304135`, source artifact `6761394808`, source age `0.68h` |

No Render deploy or database migration was needed because KAN-63 adds only workflow and documentation changes.

## Current Status

KAN-63 is implemented, merged, workflow-validated, and documented.
