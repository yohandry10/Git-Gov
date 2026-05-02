# KAN-57 Enterprise Onboarding Remediation Plan

Updated: 2026-05-02

## Summary

KAN-57 adds a secret-safe remediation plan generator for Enterprise Self-Service Onboarding.

The new script consumes KAN-52 readiness JSON and produces a prioritized action plan for the customer/operator. It explains which onboarding stages need action, who should own each action, how to validate the fix, and which GitHub Actions variable/secret names must be configured using placeholder-only commands.

## Changes

| Area | Change |
| --- | --- |
| Script | Added `scripts/control-plane/generate_enterprise_onboarding_remediation_plan.ps1`. |
| Documentation | Added design/report docs and updated the Enterprise Self-Service runbook and roadmap. |

## Plan Policy

| Setting | Value |
| --- | --- |
| Source input | `enterprise-onboarding-readiness.json` |
| Optional source | `enterprise-adoption-pack.json` |
| Output Markdown | `enterprise-onboarding-remediation-plan.md` |
| Output JSON | `enterprise-onboarding-remediation-plan.json` |
| Default mode | advisory/non-mutating |
| Strict option | `-FailOnBlocked` |
| Release blocking default | `false` |

## Safety

- No `.env` files are read.
- No provider tokens are read.
- No secret values are printed.
- GitHub Actions secret names may be listed, but values are never read or generated.
- Placeholder commands use `<value>` only.
- No GitHub Actions variables or secrets are created.
- No customer repositories are mutated.
- No provider settings are mutated.
- No branch protection is changed.
- No workflow dispatch occurs.
- Release blocking remains opt-in only.

## Validation

Local validation before PR:

| Command | Result |
| --- | --- |
| PowerShell parser check for `generate_enterprise_onboarding_remediation_plan.ps1` | PASS. |
| `.\scripts\control-plane\generate_enterprise_onboarding_readiness_report.ps1 -ProfilePath docs\examples\enterprise-adoption-profile.example.json -OutputDir out\KAN-57-onboarding-readiness -ReportOnly` | PASS. Produced readiness `needs-action`, score `75`, `3` ready stages, `3` needs-action stages, `0` blocked stages. |
| `.\scripts\control-plane\generate_enterprise_onboarding_remediation_plan.ps1 -ReadinessPath out\KAN-57-onboarding-readiness\enterprise-onboarding-readiness.json -OutputDir out\KAN-57-onboarding-remediation-plan` | PASS. Produced remediation status `needs-action`, `3` actions, `3` variable names, and `2` secret names with placeholder-only commands. |
| Generated output scan for `Authorization`, `Bearer`, `ATATT`, `vck_`, `gho_`, `JIRA_API_TOKEN=`, `GITGOV_API_KEY=`, and `SONAR_TOKEN=` | PASS. No matches. |
| `git diff --check` | PASS. |
| `.\scripts\security\publication_guard.ps1` | PASS. |

PR `#168` merged on `main` as `dca7e0b`.

PR checks passed before merge:

- `Security Guard`: passed.
- `Server Clippy + Check`: passed.
- `Desktop Rust Clippy`: passed.
- `Frontend Lint + Typecheck`: passed.
- `Website Lint + Typecheck + Build`: passed.
- `Workflow Lint`: passed.
- `Validate quality_gates warn/block matrix`: passed.
- `Sonar Scan + Quality Gate`: passed.
- `Block internal-assistant markers in branch/commits`: passed.
- `Vercel`: passed.
- `Vercel Preview Comments`: passed.

Post-merge validation for commit `dca7e0b` passed:

- `CI` - run `25243574261`.
- `Release Readiness Gate` - run `25243574245`.
- `Quality Gate Policy Matrix (Optional)` - run `25243574251`.
- `Secret Scan` - run `25243574256`.
- `Public Naming Guard` - run `25243574262`.
- `Governance Correlation Smoke (Optional)` - run `25243574244`.
- `Desktop Updater Readiness (Optional)` - run `25243574236`.
- `SonarQube Governance (Non-Blocking)` - run `25243577058`.

## Current Status

KAN-57 implementation is complete and merged through PR `#168`.

The remediation plan generator converts the ExampleCo readiness report into actionable next steps without reading secret values, mutating repositories/providers, or changing release-blocking defaults.
