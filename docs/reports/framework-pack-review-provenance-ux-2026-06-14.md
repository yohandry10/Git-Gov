# KAN-104 Framework Pack Review And Provenance UX

Date: 2026-06-14
Issue: GitHub `#365`
Branch: `product/KAN-104-framework-pack-review-provenance-ux`

## Implemented

- Added Supabase migration/postcheck `v47` for framework pack review provenance.
- Added `PATCH /compliance/framework-packs/{framework_pack_id}/review`.
- Changed new customer packs to start as `needs_review`.
- Migrated old KAN-103 statuses to `needs_review`/`reviewed`.
- Hid unreviewed customer frameworks from the mapping selector API.
- Blocked customer framework mappings until the pack is `reviewed`.
- Revalidated current pack status before creating review packages.
- Added review provenance metadata to review package artifacts.
- Added Tauri command/client/model support.
- Added Governance Evidence Review UI for pack review/provenance.
- Corrected the frontend import flow so imported-but-unreviewed packs are not auto-selected for mapping.

## Explicit Non-Scope

KAN-104 does not create official regulatory mappings, certification claims, compliance scores, OPA/Rego execution, Policy-as-Code enforcement, provider mutation, Action Center automation, MCP/chatbot behavior, BYOM routing, or Agent Governance dependency.

## Local Validation

- `cargo check` in `gitgov/gitgov-server`
- `cargo fmt --check` in `gitgov/gitgov-server`
- `cargo clippy -- -D warnings` in `gitgov/gitgov-server`
- `TEST_DATABASE_URL=<local Postgres> cargo test compliance_framework_packs -- --nocapture` in `gitgov/gitgov-server`
- `TEST_DATABASE_URL=<local Postgres> cargo test` in `gitgov/gitgov-server` (`305` tests)
- `supabase_schema_v47.sql` plus `supabase_schema_v47_postcheck.sql` in an isolated local Postgres schema seeded with `customer_review_required` and `customer_reviewed`
- `pnpm --dir gitgov typecheck`
- `pnpm --dir gitgov lint`
- `pnpm --dir gitgov test src/test/useControlPlaneStore.test.ts src/test/components/ComplianceEvidenceFlowPanel.test.tsx`
- `pnpm --dir gitgov test` (`366` tests)
- `pnpm --dir gitgov build` with the existing large chunk warning
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`
- `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`49` tests)
- `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`
- `git diff --check`
- `.\scripts\security\publication_guard.ps1`

The focused backend suite includes real state-transition checks for pre-review mapping denial, reviewed mapping success, negative status denial, package revalidation after rejection, no-claim provenance, tenant isolation, and non-admin/agent-key denial.
