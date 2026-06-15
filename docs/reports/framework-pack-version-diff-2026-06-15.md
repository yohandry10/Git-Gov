# KAN-112 Framework Pack Versioning And Diff Report

Updated: 2026-06-15

## Summary

KAN-112 adds a manual-first framework pack diff before any official regulatory mapping work. Admins can compare two customer-provided versions of the same original framework pack and inspect added, removed, changed, and unchanged controls.

## Implemented

- Backend model and DB loader for diff source packs, including `raw_pack_redacted`.
- Admin-only `GET /compliance/framework-packs/diff`.
- Same-tenant and same-original-framework enforcement.
- No-claim invariant enforcement for both packs.
- Deterministic control comparison by normalized `control_id`.
- Tauri model, client method, and command.
- Desktop Governance Evidence Review diff panel for customer-provided framework packs.
- Store state/action for loading and displaying the latest diff.
- Roadmap and architecture docs.

## Product Guardrails

- The endpoint is read-only.
- The diff is not persisted as an artifact in this MVP.
- The diff does not create compliance, regulatory, certification, or official mapping claims.
- The diff does not create Agent Governance evaluations.
- The diff does not make customer frameworks GitGov-owned.

## Validation Status

Local validation passed on branch `product/KAN-112-framework-pack-version-diff`.

Evidence:

- Backend Rust `cargo fmt --check`, `cargo check`, and `cargo clippy -- -D warnings`.
- Focused real Postgres integration test:

```text
customer_framework_pack_diff_compares_real_versions_without_claims
```

- Full backend Postgres suite: `309` passed.
- Tauri `cargo fmt --check`, `cargo check`, `cargo clippy -- -D warnings`, and tests: `49` passed.
- Frontend `pnpm --dir gitgov typecheck`.
- Focused frontend store test: `35` passed.
- Full frontend Vitest run: `367` passed.
- Frontend lint and build passed.
- `git diff --check` passed.
- `scripts/security/publication_guard.ps1` passed.
- PR `#391` checks passed.
- Post-merge `main` checks for `5499f78` passed:
  - `CI`
  - `Release Readiness Gate`
  - `Quality Gate Policy Matrix`
  - `Secret Scan`
  - `Public Naming Guard`
  - `Governance Correlation Smoke`
  - `Desktop Updater Readiness`
  - `SonarQube Governance`
- Render deploy `dep-d8noflu8bjmc73f2u2rg` reached `live`.
- Production smoke passed:
  - `/health=ok`
  - original framework: `bank_release_controls_kan112_20260615001152`
  - base pack: `cfp_8181d41d2bb39ed54af8050056fbb7eb`
  - target pack: `cfp_fdec1d243cad05936aee96a678ca35e1`
  - summary: `added=1`, `removed=1`, `changed=1`, `unchanged=2`
  - no-claim flags remained `false/false/false/false` with `requires_auditor_review=true`
