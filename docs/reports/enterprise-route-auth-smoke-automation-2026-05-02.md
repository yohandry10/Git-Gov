# KAN-62 Enterprise Route Auth Smoke Automation

Updated: 2026-05-02

## Summary

KAN-62 adds a repeatable smoke check for Enterprise route auth behavior after KAN-61 hardened `/enterprise/*` stale-auth-cache handling and integration coverage.

## Scope

Implemented:

- `scripts/control-plane/validate_enterprise_route_auth_smoke.ps1`.
- `.github/workflows/enterprise-route-auth-smoke.yml`.
- `docs/design/enterprise-route-auth-smoke-automation-mvp.md`.
- Runbook update in `docs/runbooks/enterprise-self-service-adoption.md`.

## Safety

- No `.env` file values are printed.
- No API key, Authorization header, provider token, or response body is written to artifacts.
- No provider APIs are mutated.
- No customer repositories are mutated.
- No GitHub Actions variables or secrets are created.
- No workflow dispatch against customer repositories occurs.
- No branch protection or release blocking default is changed.
- No database migration is required.

## Validation

Local validation:

| Command | Result |
| --- | --- |
| `.\scripts\control-plane\validate_enterprise_route_auth_smoke.ps1 -GitGovUrl https://gitgov-api.onrender.com -OrgName yohandry10 -RepoFullName yohandry10/Git-Gov -ReleaseId KAN-62-local-smoke -Environment production -OutputDir out\enterprise-route-auth-smoke` | Passed |
| `.\scripts\control-plane\validate_enterprise_route_auth_smoke.ps1 ... -AllowMissingApiKey` with `GITGOV_API_KEY` unset | Passed, wrote `skipped` evidence |
| `git diff --check` | Passed |
| `.\scripts\security\publication_guard.ps1` | Passed |

Production probe result from the strict local run:

| Check | Expected | Actual |
| --- | --- | --- |
| `health_public` | `200` | `200` |
| `adoption_profile_anonymous` | `401` | `401` |
| `onboarding_checklist_tracking_anonymous` | `401` | `401` |
| `release_approvals_anonymous` | `401` | `401` |
| `release_governance_evaluate_anonymous` | `401` | `401` |
| `adoption_profile_authenticated` | `200` | `200` |
| `onboarding_checklist_tracking_authenticated` | `200` | `200` |
| `release_approvals_authenticated` | `200` | `200` |
| `release_governance_evaluate_authenticated` | `200` | `200` |

PR validation:

| Check | Result |
| --- | --- |
| PR `#178` `Security Guard` | Passed |
| PR `#178` `Server Clippy + Check` | Passed |
| PR `#178` `Desktop Rust Clippy` | Passed |
| PR `#178` `Frontend Lint + Typecheck` | Passed |
| PR `#178` `Website Lint + Typecheck + Build` | Passed |
| PR `#178` `Workflow Lint` | Passed |
| PR `#178` `Validate quality_gates warn/block matrix` | Passed |
| PR `#178` `Sonar Scan + Quality Gate` | Passed |
| PR `#178` `Block internal-assistant markers in branch/commits` | Passed |
| PR `#178` Vercel preview | Passed |

Post-merge validation:

| Check | Result |
| --- | --- |
| Main merge commit | `e86c6bc` |
| `CI` | Passed, run `25246267909` |
| `Release Readiness Gate` | Passed, run `25246267897` |
| `Quality Gate Policy Matrix (Optional)` | Passed, run `25246267908` |
| `Secret Scan` | Passed, run `25246267900` |
| `Public Naming Guard` | Passed, run `25246267906` |
| `Governance Correlation Smoke (Optional)` | Passed, run `25246267918` |
| `Desktop Updater Readiness (Optional)` | Passed, run `25246267904` |
| `SonarQube Governance (Non-Blocking)` | Passed, run `25246267912` |

First workflow dispatch:

| Field | Result |
| --- | --- |
| Workflow | `Enterprise Route Auth Smoke` |
| Run | `25246304135` |
| Conclusion | Passed |
| Artifact | `enterprise-route-auth-smoke-25246304135` |
| Artifact ID | `6761394808` |
| Artifact expiry | `2026-07-31T06:56:47Z` |
| Parsed result | `status=passed`, `checks=9`, `failures=0` |

No Render deploy or database migration was needed because KAN-62 changes only workflow, script, and docs.

## Current Status

KAN-62 is implemented, merged, workflow-validated, and documented.
