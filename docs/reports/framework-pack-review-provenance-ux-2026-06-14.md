# KAN-104 Framework Pack Review And Provenance UX

Date: 2026-06-14
Issue: GitHub `#365`
Branch: `product/KAN-104-framework-pack-review-provenance-ux`
PR: `#366`
Merge: `e34433a`

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

## Production Validation

- Production `v47` migration and postcheck passed.
- Render deploy `dep-d8ngum19rddc739qhvog` for `e34433a` reached `live`.
- `/health` returned `ok`.
- Authenticated `/stats` returned HTTP `200`.
- Runtime customer framework pack `cfp_3fb36bb89a583956dc1f1e775654354a` started as `needs_review`.
- The unreviewed framework was hidden from `GET /compliance/control-frameworks`.
- Pre-review mapping returned `409 framework_pack_not_reviewed`.
- After review, the framework became listable and mapping `cem_93a966d7e26b4728a5a28f534019a5fc` was created.
- Review package `crp_e117bebd4154f647be447fbec5fe4ec9` was created with artifact hash `sha256:dee159d526ec05cb38e677688d5eceb0c7e1f021fea62584f50816810caace93`.
- The package artifact preserved `review_status=reviewed`, `framework_pack_id`, `pack_hash`, and no-claim flags.
- After rejecting the pack, creating another package from the old mapping returned `409 framework_pack_rejected`.
- Smoke packs `cfp_3fb36bb89a583956dc1f1e775654354a` and `cfp_d4d194979ebc0c59fd073cb3884b59ce` were archived after validation.
