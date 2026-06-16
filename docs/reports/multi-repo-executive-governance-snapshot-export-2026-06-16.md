# KAN-131 Multi-Repo Executive Governance Snapshot Export Report

Date: 2026-06-16

Issue: `#459`

## Summary

KAN-131 adds manual-first JSON snapshots for the filtered Executive Governance View.

Implemented:

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

## PR And Production Validation

KAN-131 shipped through:

- PR `#460` - initial snapshot export implementation, merged as `b3fe4561`.
- PR `#461` - archive contract hotfix, merged as `dee9d889`.
- PR `#462` - request deserialization hardening, merged as `44e2a492`.

Production validation:

- Supabase migration `v68` and `v68` postcheck passed with `ON_ERROR_STOP=1`.
- Final Render deploy `dep-d8olc88jo6nc73b94n4g` for commit `44e2a492` reached `live`.
- Final smoke passed against `https://gitgov-api.onrender.com`:
  - `/health=ok`.
  - Filtered executive view returned `repositories=1`, first repository `yohandry10/Git-Gov`,
    posture `review`.
  - Created snapshot `egs_06f228f93f184aeeb182e5932b98f4cc`.
  - Downloaded artifact hash
    `sha256:27e21be0854ecd8ad459551176f8de3aab6b487ec902896d7139923a9dcfb24c`.
  - Hash recomputation over the artifact preimage passed.
  - Archive with only `org_name` passed.
  - Download after archive returned HTTP `409`.
  - Source evidence counts were unchanged before/after snapshot operations:
    Deployment Gate authorizations `2`, Change Risk evaluations `6`, CAB packets `8`,
    CAB decision manifests `6`, Agent Governance evaluations `7`.
  - Snapshot table count increased by one, as expected for the created artifact.

The production smoke intentionally found a real archive DTO bug before completion: the first
deployed contract reused the create request shape and required `name` on archive, causing HTTP
`422`. PR `#461` introduced a dedicated archive DTO for backend and Tauri. PR `#462` added
backward-compatible deserialization so create still rejects a missing name through controlled
validation while archive requests with only org scope do not fail before the handler.
