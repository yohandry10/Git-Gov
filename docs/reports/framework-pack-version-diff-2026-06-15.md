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
- `git diff --check` and `scripts/security/publication_guard.ps1` are required before commit.
- PR checks, post-merge Render deployment, and production smoke are required before closing KAN-112.
