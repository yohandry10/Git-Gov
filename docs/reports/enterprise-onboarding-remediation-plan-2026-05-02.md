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

PR validation and post-merge evidence will be appended after merge.

## Current Status

KAN-57 implementation is in progress.

The local remediation plan converts the ExampleCo readiness report into actionable next steps without reading secret values, mutating repositories/providers, or changing release-blocking defaults.
