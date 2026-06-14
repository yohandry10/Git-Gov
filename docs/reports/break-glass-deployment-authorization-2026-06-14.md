# KAN-87 Break-glass Deployment Authorization

Updated: 2026-06-14

## Summary

KAN-87 adds audited break-glass authorization for Deployment Gates. A blocked deployment can now be authorized by explicit exception only when the evaluated policy is truly blocking and the request carries a reason.

Traceability:

- GitHub issue: `#312 - KAN-87: Break-glass Deployment Authorization`.

## Changes

- Extended `POST /deployment-gates/authorize` with optional `break_glass`.
- Added persisted break-glass evidence fields to `deployment_gate_authorizations`.
- Added `decision=break_glass`.
- Kept `blocking=true` and `would_block=true` on break-glass records to preserve the original policy result.
- Added backend validation for reason, authorizing actor, expiry, and misuse against non-blocking policy.
- Added admin audit metadata for break-glass usage.
- Added Desktop/Tauri/store fields for break-glass history.
- Updated `DeploymentGateHistoryPanel` to show break-glass usage, reason, authorizer, expiry, and original blockers.
- Updated generated release governance gate artifact output to include break-glass fields returned by the API.

## Validation So Far

Passed locally:

- `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml`
- `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`
- `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`
- `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`
- `$env:TEST_DATABASE_URL='postgresql://gitgov:gitgov_dev_password@127.0.0.1:5433/gitgov'; cargo test --manifest-path gitgov/gitgov-server/Cargo.toml deployment_gate -- --nocapture`
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`
- `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml`
- `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`
- `npm --prefix gitgov run typecheck`
- `npm --prefix gitgov run lint`
- `npm --prefix gitgov test -- --run src/test/components/DeploymentGateHistoryPanel.test.tsx src/test/useControlPlaneStore.test.ts src/test/components/dashboard-helpers.test.ts`
- `npm --prefix gitgov test -- --run`
- `npm --prefix gitgov run build`
- `git diff --check`
- `.\scripts\security\publication_guard.ps1`

Focused real coverage:

- blocked policy without break-glass returns `decision=blocked`;
- blocked policy with valid break-glass returns `decision=break_glass`, `approved=true`, `blocking=true`, `would_block=true`;
- break-glass request against non-blocking policy is rejected and does not persist history;
- Developer keys remain forbidden;
- scoped Admin keys remain tenant-bound;
- ticket/evidence packet mismatch still rejects the request;
- Desktop history renders break-glass evidence explicitly.

Migration validation:

- `supabase_schema_v36.sql` was applied through the ignored local `DATABASE_URL`.
- Local SQL checks verified the four break-glass columns and the `decision` constraint containing `break_glass`.

Notes:

- Full backend suite passed with `262` tests.
- Full frontend suite passed with `361` tests.
- Tauri suite passed with `49` tests.
- Build still reports the existing Vite large chunk warning.

## Remaining Validation

- GitHub PR and post-merge checks.
