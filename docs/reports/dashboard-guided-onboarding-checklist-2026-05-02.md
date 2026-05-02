# KAN-59 Dashboard Guided Onboarding Checklist

Updated: 2026-05-02

## Summary

KAN-59 adds a visible guided onboarding checklist to the Enterprise Adoption dashboard.

The checklist is generated from the current dashboard onboarding readiness report and the KAN-58 remediation plan. It shows complete, next, todo, and blocked stages with owner/action/validation details while preserving the existing JSON exports.

## Changes

| Area | Change |
| --- | --- |
| Dashboard helpers | Added `buildEnterpriseOnboardingGuide` and guide step/status types. |
| Dashboard UI | Added a `Guided checklist` section to `EnterpriseAdoptionPanel`. |
| Tests | Added helper tests for guide status ordering, next-step selection, config summary, ready-state behavior, and safety flags. |
| Documentation | Added design/report docs and updated the Enterprise Self-Service runbook and roadmap. |

## Safety

- No `.env` files are read.
- No provider tokens are read.
- No provider APIs are called.
- No secret values are printed.
- Secret names may be displayed, but values are never read or generated.
- No GitHub Actions variables or secrets are created.
- No customer repositories are mutated.
- No provider settings are mutated.
- No workflow dispatch occurs.
- Release blocking remains opt-in only.

## Validation

Local validation before PR:

| Command | Result |
| --- | --- |
| `npm test -- --run src/test/components/dashboard-helpers.test.ts` | PASS. `26` tests. |
| `npm run typecheck` | PASS. |
| `npm run lint` | PASS. |
| `npm test -- --run` | PASS. `25` test files, `294` tests. |
| `npm run build` | PASS with existing Vite large chunk warning. |
| `git diff --check` | PASS. |
| `.\scripts\security\publication_guard.ps1` | PASS. |

PR validation and post-merge evidence will be appended after merge.

## Current Status

KAN-59 implementation is in progress.
