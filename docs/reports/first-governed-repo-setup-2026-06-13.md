# First Governed Repo Setup Implementation Report

Date: 2026-06-13

Ticket: `KAN-80`

## Summary

Implemented the First Governed Repo Setup MVP as the first concrete slice of Deployment Gates 0.1.

This adds a persistent, idempotent setup run per organization, an Admin-only API, Desktop/Tauri
commands, Control Plane store actions, and a `Governance > Adoption` panel that prepares one
repository for advisory deployment-gate simulation.

## Product Shape

The implementation follows the product-lead decision:

```text
No broad integration wizard. Build the minimum setup that makes Deployment Gates demonstrable,
explainable, and sellable.
```

The setup captures:

- first governed repository.
- default branch.
- setup goal.
- policy preset.
- evidence providers.
- governance modules.
- policy/workflow preview acknowledgement.
- normalized baseline readiness.
- Action Center gaps.
- CTA into release/gate simulation.

## Main Files

- `gitgov/gitgov-server/supabase/supabase_schema_v32.sql`
- `gitgov/gitgov-server/src/handlers/first_governed_repo_setup.rs`
- `gitgov/gitgov-server/src/db/enterprise.rs`
- `gitgov/gitgov-server/src/models/enterprise.rs`
- `gitgov/src-tauri/src/control_plane/server/models/enterprise.rs`
- `gitgov/src-tauri/src/control_plane/server/client/enterprise.rs`
- `gitgov/src-tauri/src/commands/server_commands.rs`
- `gitgov/src/components/control_plane/FirstGovernedRepoSetupPanel.tsx`
- `gitgov/src/components/control_plane/dashboard-helpers/first-governed-repo-setup.ts`
- `gitgov/src/store/useControlPlaneStore/actions/enterprise.ts`
- `gitgov/src/store/useControlPlaneStore/types.ts`
- `gitgov/src/store/useControlPlaneStore/state.ts`

## Security And Business Logic

The backend is the source of truth. It validates and normalizes the setup before persistence.

Covered controls:

- Admin-only access.
- org scope enforcement.
- global Admin keys require `org_name`.
- selected providers must include GitHub.
- unsupported provider/module/goal/preset/status values are rejected.
- baseline JSON has a size cap.
- baseline rejects secret-looking keys or values.
- `completed` status requires `baseline_ready`.
- audit metadata is secret-safe.
- upsert preserves `run_id` for idempotency.

## Validation Status

Completed before this report was written:

- `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`
- `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml first_governed_repo_setup -- --nocapture`
- `$env:TEST_DATABASE_URL='postgresql://gitgov:gitgov_dev_password@127.0.0.1:5433/gitgov'; cargo test --manifest-path gitgov/gitgov-server/Cargo.toml first_governed_repo_setup -- --nocapture`
- `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml`
- `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml`
- `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`
- `npm --prefix gitgov test -- --run src/test/components/dashboard-helpers.test.ts`
- `npm --prefix gitgov test -- --run`
- `npm --prefix gitgov run typecheck`
- `npm --prefix gitgov run lint -- --quiet`
- `npm --prefix gitgov run build`
- `git diff --check`
- `.\scripts\security\publication_guard.ps1`

The focused KAN-80 integration test was also run with an explicit local Postgres
`TEST_DATABASE_URL` on `127.0.0.1:5433`, covering Admin authorization, Developer rejection,
secret-looking baseline rejection, idempotent `run_id` preservation, baseline normalization, GET
readback, and audit-log insertion.

## Residual Work

Next ticket after this slice should be Deployment Gates Advisory 0.1:

- stable deployment gate evaluation/authorization API.
- advisory response contract.
- UI history for gate simulation attempts.
- per-environment advisory policy.
- later, explicit customer opt-in for blocking mode.

Do not fold Slack, universal OAuth, OPA/Rego execution, marketplace connectors, bulk onboarding, or
hard enforcement into this MVP.
