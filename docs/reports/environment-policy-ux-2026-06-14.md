# KAN-86 Environment Policy UX

Updated: 2026-06-14

## Summary

KAN-86 adds a Desktop/admin Environment Policy Matrix for release governance. It turns the existing `environment_overrides` profile capability into a reviewable configuration surface and hardens the helper layer so product logic is not hand-assembled inside the panel.

## Changes

- Added `ReleaseGovernanceEnvironmentPolicyPanel` for the release governance environment matrix.
- Added helper functions for:
  - effective environment rows;
  - preserving overrides when the base mode changes;
  - adding an override with the next available common environment;
  - editing override environment/mode;
  - removing overrides and falling back to base policy.
- Updated `EnterpriseAdoptionPanel` to use the focused component and helpers.
- Added focused helper and component tests for production-stricter-than-staging behavior.

## Safety

- `record-only` remains the default release governance behavior.
- Blocking behavior still requires explicit `approval-required` or `quorum-required` policy.
- Non-`record-only` policy still requires the `formal-approval` module through existing profile validation.
- No secret values are read, stored, printed, or generated.
- No customer repository or provider configuration is mutated.

## Validation

Passed locally:

- `npm --prefix gitgov test -- --run src/test/components/dashboard-helpers.test.ts src/test/components/ReleaseGovernanceEnvironmentPolicyPanel.test.tsx`
- `npm --prefix gitgov run typecheck`
- `npm --prefix gitgov run lint`
- `npm --prefix gitgov test -- --run`
- `npm --prefix gitgov run build`
- `git diff --check`
- `.\scripts\security\publication_guard.ps1`

Notes:

- Full frontend suite passed with `360` tests.
- Build still reports the existing Vite large chunk warning.
- `EnterpriseAdoptionPanel.tsx` was reduced from `934` lines to `764` lines by extracting focused helpers and the release governance environment panel.

## GitHub Validation

- Issue: `#309 - KAN-86: Environment Policy UX`.
- PR: `#310 - product(KAN-86): add environment policy UX`.
- Merge commit: `b280570`.
- PR checks passed before merge.
- Post-merge `main` checks passed for `b280570`: `CI`, `Release Readiness Gate`, `Secret Scan`, `Public Naming Guard`, `Quality Gate Policy Matrix`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance`.
