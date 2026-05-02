# KAN-61 Enterprise Route Auth Regression Hardening

Updated: 2026-05-02

## Summary

KAN-61 adds regression hardening for Enterprise admin routes after KAN-60 introduced persisted guided checklist tracking.

The goal is to make route access mistakes harder to reintroduce:

- anonymous callers must receive `401`.
- non-admin callers must receive `403`.
- global admin keys must provide `org_name`.
- org-scoped admin keys must not cross tenant boundaries.
- valid scoped admin access must continue to work.

## Scope

Covered route classes:

- `GET/PUT /enterprise/adoption-profile`.
- `GET/PUT /enterprise/onboarding-checklist-tracking`.
- `GET /enterprise/release-approvals`.
- `GET /enterprise/release-governance/evaluate`.

Hardening changes:

- Treat all `/enterprise/*` routes as sensitive admin paths for stale-auth-cache fail-closed behavior.
- Extend the integration test harness with minimal Enterprise tables.
- Add a DB-backed integration matrix for Enterprise auth and org-scope outcomes.

## Safety

- No provider APIs are called.
- No `.env` files are read by the code change.
- No provider secrets are read, printed, stored, or mutated.
- No customer repositories are mutated.
- No GitHub Actions variables or secrets are created.
- No workflow dispatch occurs.
- No branch protection is changed.
- No release blocking default is changed.
- No database migration is needed; this is code/test hardening only.

## Validation

Local validation before PR creation:

| Command | Result |
| --- | --- |
| `cargo fmt` in `gitgov/gitgov-server` | Passed |
| `cargo check` in `gitgov/gitgov-server` | Passed |
| `cargo clippy -- -D warnings` in `gitgov/gitgov-server` | Passed |
| `cargo test enterprise_admin_routes_enforce_auth_and_org_scope -- --nocapture` in `gitgov/gitgov-server` | Passed |
| `cargo test` in `gitgov/gitgov-server` | Passed, `193` tests |
| `git diff --check` | Passed |

Full publication guard and post-merge validation will be recorded after PR validation.

## Current Status

KAN-61 implementation is ready for PR validation.
