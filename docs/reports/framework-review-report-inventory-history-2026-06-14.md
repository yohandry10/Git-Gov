# KAN-106 Framework Review Report Inventory History

Date: 2026-06-14
Branch: `product/KAN-106-framework-report-inventory`
Issue: GitHub `#371`

## Implemented

- Added Admin-only `GET /compliance/framework-review-reports` list behavior beside the existing KAN-105 create route.
- Added metadata-only list response with `items`, `count`, and effective `limit`.
- Added filters for `framework_id`, `mapping_id`, and `review_package_id`.
- Added validation and limit clamping for list queries.
- Added DB query scoped by `org_id`, ordered by newest report first, without selecting `payload_json_redacted`.
- Added Supabase migration/postcheck `v49` with report inventory indexes.
- Added Tauri model/client/command support for listing framework review reports.
- Added Desktop Governance Evidence Review history UI with:
  - `History` load action.
  - metadata cards for recent reports.
  - selected historical JSON download.
- Extended store state/actions and focused tests.

## Non-Goals

KAN-106 does not create official regulatory mappings, auditor approval workflow, certification claims, compliance scores, PDF/DOCX output, OPA/Rego execution, provider mutation, policy mutation, BYOM, MCP, chatbot behavior, LLM-generated summaries, or Agent Governance dependency.

## Validation

Initial focused validation completed during implementation:

- `pnpm --dir gitgov exec vitest run src/test/useControlPlaneStore.test.ts` (`34` passed)
- `pnpm --dir gitgov typecheck`
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`
- `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`
- `TEST_DATABASE_URL=<local ignored DATABASE_URL> cargo test --manifest-path gitgov/gitgov-server/Cargo.toml compliance_framework_review_reports -- --nocapture` (`2` passed)

Final validation:

- PR `#372` checks passed.
- Merged to `main` as `56ec538`.
- Post-merge `main` checks passed: `CI`, `Release Readiness Gate`, `Quality Gate Policy Matrix`, `Secret Scan`, `Public Naming Guard`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance`.
- Render deploy `dep-d8nisuv7f7vs73fnnnq0` reached `live`.
- Production smoke passed:
  - `/health=ok`
  - authenticated `/stats=200`
  - framework report list returned `200`, `count=2`, effective `limit=100`
  - list metadata did not include payload/artifact fields
  - historical download returned schema `gitgov_framework_review_report.v1` with `10` controls
  - invalid `mapping_id=bad` returned `400`
