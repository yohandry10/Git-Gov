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

Additional validation will be recorded after PR checks and first workflow dispatch.

## Current Status

Implementation is ready for PR validation.
