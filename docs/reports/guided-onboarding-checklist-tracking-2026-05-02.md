# KAN-60 Guided Onboarding Checklist Tracking

Updated: 2026-05-02

## Summary

KAN-60 persists guided onboarding checklist tracking metadata per organization.

Admins can save checklist stage tracking state, owner, target date, external reference, and notes from the Enterprise Adoption dashboard. The tracking layer is separate from calculated readiness and does not change evidence, readiness score, policy evaluation, or release gate defaults.

## Changes

| Area | Change |
| --- | --- |
| Backend | Added admin `GET/PUT /enterprise/onboarding-checklist-tracking`. |
| Database | Added `enterprise_onboarding_checklist_tracking` with migration `v25` and postcheck. |
| Auth | Added stale-auth-cache fail-closed handling for the new sensitive admin route. |
| Tauri | Added server client methods and commands for checklist tracking. |
| Dashboard store | Added load/save state and actions for checklist tracking. |
| Dashboard UI | Added per-stage tracking controls inside `Guided checklist`. |
| Tests | Added backend validation tests and dashboard helper tests. |
| Documentation | Added design/report docs and will update runbook/roadmap/context. |

## Safety

- No `.env` files are read.
- No provider tokens are read.
- No provider APIs are called.
- No secret values are printed.
- Backend validation rejects common secret-looking values in tracking text fields.
- No GitHub Actions variables or secrets are created.
- No customer repositories are mutated.
- No provider settings are mutated.
- No workflow dispatch occurs.
- Release blocking remains opt-in only.

## Validation

Local validation before PR creation:

| Command | Result |
| --- | --- |
| `cargo fmt` in `gitgov/gitgov-server` | Passed |
| `cargo check` in `gitgov/gitgov-server` | Passed |
| `cargo clippy -- -D warnings` in `gitgov/gitgov-server` | Passed |
| `cargo test` in `gitgov/gitgov-server` | Passed, `192` tests |
| `cargo fmt` in `gitgov/src-tauri` | Passed |
| `cargo check` in `gitgov/src-tauri` | Passed |
| `cargo clippy -- -D warnings` in `gitgov/src-tauri` | Passed |
| `cargo test` in `gitgov/src-tauri` | Passed, `23` tests |
| `npm test -- --run src/test/components/dashboard-helpers.test.ts` in `gitgov` | Passed, `28` tests |
| `npm run typecheck` in `gitgov` | Passed |
| `npm run lint` in `gitgov` | Passed |
| `npm test -- --run` in `gitgov` | Passed, `25` files and `296` tests |
| `npm run build` in `gitgov` | Passed with existing Vite large chunk warning |
| `git diff --check` | Passed |
| `.\scripts\security\publication_guard.ps1` | Passed |

Post-merge and production validation will be recorded after PR merge and database migration `v25`.

## Current Status

KAN-60 implementation is ready for PR validation.
