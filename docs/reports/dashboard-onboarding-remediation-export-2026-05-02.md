# KAN-58 Dashboard Onboarding Remediation Export

Updated: 2026-05-02

## Summary

KAN-58 exposes the KAN-57 remediation plan from the Enterprise Adoption dashboard.

The dashboard now builds a remediation plan JSON from the current onboarding readiness report and adoption pack. It gives self-service users the same action list available from the CLI: priority, stage, owner, action, validation evidence, and placeholder-only GitHub Actions configuration commands.

## Changes

| Area | Change |
| --- | --- |
| Dashboard helpers | Added `buildEnterpriseOnboardingRemediationPlan` and `buildEnterpriseOnboardingRemediationPlanFilename`. |
| Dashboard UI | Added a `Plan` JSON download action in `EnterpriseAdoptionPanel`. |
| Tests | Added dashboard helper tests for remediation plan actions, placeholder commands, safety flags, and filename. |
| Documentation | Added design/report docs and updated the Enterprise Self-Service runbook and roadmap. |

## Safety

- No `.env` files are read.
- No provider tokens are read.
- No provider APIs are called.
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
| `npm test -- --run src/test/components/dashboard-helpers.test.ts` | PASS. `24` tests. |
| `npm run typecheck` | PASS. |
| `npm run lint` | PASS. |
| `npm test -- --run` | PASS. `25` test files, `292` tests. |
| `npm run build` | PASS with existing Vite large chunk warning. |
| `git diff --check` | PASS. |
| `.\scripts\security\publication_guard.ps1` | PASS. |

PR validation and post-merge evidence will be appended after merge.

## Current Status

KAN-58 implementation is in progress.

The dashboard can build and download a secret-safe onboarding remediation plan JSON without mutating repositories/providers or changing release-blocking defaults.
