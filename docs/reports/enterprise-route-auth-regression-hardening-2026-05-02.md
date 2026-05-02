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
| `cargo test enterprise_admin_routes_enforce_auth_and_org_scope -- --nocapture` with Docker-backed `TEST_DATABASE_URL` | Passed against temporary Postgres on local port `55433` |
| `cargo test` in `gitgov/gitgov-server` | Passed, `193` tests |
| `git diff --check` | Passed |
| `.\scripts\security\publication_guard.ps1` | Passed |

Notes:

- Docker Desktop was restarted locally for DB-backed validation.
- The existing `gitgov-db` container was running on `5433`, but the host also had a local `postgres` process listening on that port, so the DB-backed test used an isolated temporary Postgres container on `55433`.
- Temporary `pg_hba.conf` changes tested on the persistent `gitgov-db` container were restored before continuing.

PR validation:

| Check | Result |
| --- | --- |
| PR `#176` `Security Guard` | Passed |
| PR `#176` `Server Clippy + Check` | Passed |
| PR `#176` `Desktop Rust Clippy` | Passed |
| PR `#176` `Frontend Lint + Typecheck` | Passed |
| PR `#176` `Website Lint + Typecheck + Build` | Passed |
| PR `#176` `Workflow Lint` | Passed |
| PR `#176` `Validate quality_gates warn/block matrix` | Passed |
| PR `#176` `Sonar Scan + Quality Gate` | Passed |
| PR `#176` `Block internal-assistant markers in branch/commits` | Passed |
| PR `#176` Vercel preview | Passed |

Post-merge validation:

| Check | Result |
| --- | --- |
| Main merge commit | `6483c53` |
| `CI` | Passed, run `25245741318` |
| `Release Readiness Gate` | Passed, run `25245741327` |
| `Quality Gate Policy Matrix (Optional)` | Passed, run `25245741326` |
| `Secret Scan` | Passed, run `25245741329` |
| `Public Naming Guard` | Passed, run `25245741320` |
| `Governance Correlation Smoke (Optional)` | Passed, run `25245741322` |
| `Desktop Updater Readiness (Optional)` | Passed, run `25245741328` |
| `SonarQube Governance (Non-Blocking)` | Passed, run `25245741313` |

Production validation:

| Probe | Result |
| --- | --- |
| Render deploy | `dep-d7qph6vlk1mc73d5lni0` reached `live` for commit `6483c53` |
| `GET /health` | `200` |
| Anonymous `GET /enterprise/adoption-profile?org_name=yohandry10` | `401` |
| Authenticated `GET /enterprise/adoption-profile?org_name=yohandry10` | `200` |
| Authenticated `GET /enterprise/onboarding-checklist-tracking?org_name=yohandry10` | `200` |

No database migration was needed for KAN-61.

## Current Status

KAN-61 is implemented, merged, deployed, and smoke-validated in production.
