# KAN-131 Multi-Repo Executive Governance Snapshot Export Report

Date: 2026-06-16

Issue: `#459`

## Summary

KAN-131 adds manual-first JSON snapshots for the filtered Executive Governance View.

Implemented locally:

- Supabase migration `v68` for `executive_governance_snapshots`.
- Backend snapshot create/list/get/download/archive routes.
- Hashable artifact schema `gitgov_executive_governance_snapshot.v1`.
- Tauri commands and client models.
- Control Plane store actions and Desktop snapshot panel.
- KAN-130 Tauri client filter propagation fix.
- Real Postgres integration coverage and focused frontend/store tests.

## Guardrails

Snapshots are read-only artifacts. They do not approve deployments, block releases, execute deploys,
mutate providers/repos, mutate source evidence, recalculate risk, create CAB/compliance artifacts
automatically, use AI/Agent Governance, or make compliance/legal/certification claims.

## Local Validation Status

Already passed during implementation:

- `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`
- `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`
- `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`
- `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml --no-run`
- Focused real Postgres test:
  `multi_repo_executive_governance_view_is_read_only_and_tenant_scoped`
- Local `v68` migration and `v68` postcheck with `ON_ERROR_STOP=1`
- `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`
- `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`49` tests)
- `npm --prefix gitgov run typecheck`
- `npm --prefix gitgov run lint`
- `npm --prefix gitgov test -- --run` (`386` tests)
- Focused frontend tests:
  `npm --prefix gitgov test -- --run src/test/useControlPlaneStore.test.ts src/test/components/MultiRepoExecutiveGovernancePanel.test.tsx` (`52` tests)
- `npm --prefix gitgov run build` with the pre-existing Vite large chunk warning
- `git diff --check`
- `scripts/security/publication_guard.ps1`

Remaining before merge: PR checks, production migration `v68`, Render deploy, and production
smoke.
