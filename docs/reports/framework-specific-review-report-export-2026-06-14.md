# KAN-105 Framework-specific Review Report Export

Date: 2026-06-14
Issue: GitHub `#368`
Branch: `product/KAN-105-framework-review-report-export`

## Implemented

- Added Supabase migration/postcheck `v48`.
- Added `compliance_framework_review_reports`.
- Added backend routes for create, metadata, and JSON download:
  - `POST /compliance/framework-review-reports`
  - `GET /compliance/framework-review-reports/{report_id}`
  - `GET /compliance/framework-review-reports/{report_id}/download`
- Added framework-specific artifact schema `gitgov_framework_review_report.v1`.
- Bound reports to both `mapping_id` and `review_package_id`.
- Preserved evidence export hash, mapping hash, review package hash, framework owner/source/review provenance, pack hash, control statuses, evidence refs, and missing evidence.
- Reused KAN-104 current-pack review gate for customer-owned frameworks.
- Added Tauri client/model/command support.
- Added Desktop Governance Evidence Review report panel.
- Added store actions and focused frontend tests for create/download behavior.

## Explicit Non-Scope

KAN-105 does not create official regulatory mappings, certification claims, compliance scores, PDF/DOCX export, OPA/Rego execution, enforcement, provider mutation, Action Center writes, BYOM, MCP, chatbot, LLM summaries, or Agent Governance dependency.

## Local Validation

- `cargo check` in `gitgov/gitgov-server`
- `TEST_DATABASE_URL=<local Postgres> cargo test compliance_framework_review_reports -- --nocapture` in `gitgov/gitgov-server` (`2` passed)
- `TEST_DATABASE_URL=<local Postgres> cargo test evidence_mapping_enforces_admin_tenant_scope_and_framework_limits -- --nocapture` in `gitgov/gitgov-server` (`1` passed)
- `TEST_DATABASE_URL=<local Postgres> cargo test -- --test-threads=2` in `gitgov/gitgov-server` (`307` passed)
- `cargo fmt --check` in `gitgov/gitgov-server`
- `cargo clippy -- -D warnings` in `gitgov/gitgov-server`
- `pnpm --dir gitgov typecheck`
- `pnpm --dir gitgov exec vitest run src/test/useControlPlaneStore.test.ts` (`34` passed)
- `pnpm --dir gitgov test` (`366` passed)
- `pnpm --dir gitgov lint`
- `pnpm --dir gitgov build`
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`
- `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`
- `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`49` passed)
- Local `v48` migration and postcheck against the ignored local `DATABASE_URL`
- `git diff --check`

## Real Test Coverage

- Baseline GitGov framework:
  - Deployment Gate authorization seeded in PostgreSQL.
  - KAN-99 export created through HTTP.
  - KAN-100 mapping created through HTTP.
  - KAN-101 review package created through HTTP.
  - KAN-105 framework report created through HTTP.
  - Downloaded JSON hash matches persisted `artifact_hash`.
  - Artifact includes 10 controls, source hashes, missing evidence, no raw payload, and no-claim flags.
  - Agent Governance evaluation count does not change.
- Customer-owned reviewed framework:
  - Customer pack imported through HTTP.
  - Pack marked `reviewed`.
  - Mapping, review package, and framework report generated.
  - Artifact preserves `owner_type=customer`, `source=customer_provided`, `pack_hash`, and `review_status=reviewed`.
  - After rejecting the pack, new report generation returns `409 framework_pack_rejected`.
