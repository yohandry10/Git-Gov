# KAN-36 Provider Connection Validation

Updated: 2026-04-30

## Summary

KAN-36 adds direct provider connection validation for Enterprise Self-Service onboarding.

This complements KAN-32 provider health. KAN-32 answers "has GitGov observed evidence from this provider?" KAN-36 answers "can the setup credentials reach this provider right now?"

## Changes

- Added `scripts/control-plane/validate_enterprise_provider_connections.ps1`.
- Reads the adoption profile provider list by default.
- Supports explicit provider and target overrides for local/customer validation.
- Produces sanitized JSON reports.
- Supports strict mode by default and non-blocking `-ReportOnly` mode.
- Supports GitHub, Jira, Jenkins, SonarQube, Render, and Vercel checks.

## PR

- PR: `#123` - `product(KAN-36): add provider connection validation`.
- Merge commit: `8c075a4`.

## Safety

The validator:

- does not print token values.
- does not write token values.
- does not mutate provider settings.
- does not mutate repositories.
- does not install webhooks.
- does not create GitHub Actions variables or secrets.
- reports required config names, not secret values.

## Local Validation

Validated from the repository root.

GitHub and Jira ready-path validation:

```powershell
.\scripts\control-plane\validate_enterprise_provider_connections.ps1 -Providers github,jira -RepositoryFullName yohandry10/Git-Gov -JiraProjectKey KAN -OutputPath out\KAN-36-provider-connections-github-jira.json
```

Result:

- status `ready`.
- `2` ready checks.
- `0` missing config.
- `0` failed checks.

Report-only profile validation:

```powershell
.\scripts\control-plane\validate_enterprise_provider_connections.ps1 -ProfilePath docs\examples\enterprise-adoption-profile.example.json -RepositoryFullName yohandry10/Git-Gov -JiraProjectKey KAN -ReportOnly -OutputPath out\KAN-36-provider-connections-report-only.json
```

Result:

- exited successfully because `-ReportOnly` was used.
- GitHub and Jira were `ready`.
- Jenkins and SonarQube were `failed` in this local session because their local services were not reachable.
- no secret values were printed.

Missing-config validation:

```powershell
.\scripts\control-plane\validate_enterprise_provider_connections.ps1 -Providers vercel -ReportOnly -OutputPath out\KAN-36-provider-connections-missing-config.json
```

Result:

- status `missing-config`.
- missing config listed as `VERCEL_TOKEN`.
- no token value was printed.

Strict-mode failure validation:

- running the Vercel check without `-ReportOnly` exited non-zero as expected when `VERCEL_TOKEN` was not configured.

Repository guardrails:

- `git diff --check` passed.
- `.\scripts\security\publication_guard.ps1` passed.
- targeted secret-pattern scan over KAN-36 files returned no committed secret-like assignments.

## Post-Merge Validation

Post-merge `main` checks passed for commit `8c075a4`:

- `CI` run `25192626074`.
- `Release Readiness Gate` run `25192626059`.
- `Quality Gate Policy Matrix (Optional)` run `25192626048`.
- `Secret Scan` run `25192626067`.
- `Public Naming Guard` run `25192626061`.
- `SonarQube Governance (Non-Blocking)` run `25192626079`.
- `Governance Correlation Smoke (Optional)` run `25192626054`.
- `Desktop Updater Readiness (Optional)` run `25192626050`.

## Remaining Product Work Before AI SDK

- Formal enterprise release approval.
- Optional dashboard wrapper for direct provider connection reports.
- Optional provider setup automation after explicit customer authorization.

Vercel AI SDK Copilot remains pending until onboarding and approval evidence are complete enough to explain a full adoption state.
