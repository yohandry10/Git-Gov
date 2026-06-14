# KAN-107 Framework Review Report Review Workflow

Date: 2026-06-14
Branch: `product/KAN-107-framework-report-review`
Issue: GitHub `#374`
PR: GitHub `#375`
Merged commit: `bd4583a`
Render deploy: `dep-d8njkhsvikkc73alv9l0`

## Implemented

- Added `PATCH /compliance/framework-review-reports/{report_id}/review`.
- Added `review_status`, `reviewed_by_user_id`, `reviewed_at`, and `review_notes_safe` metadata through migration `v50`.
- Added metadata-only review state to report get/list responses.
- Added audit event `compliance_framework_review_report.reviewed`.
- Added Tauri model/client/command support.
- Added Desktop Governance Evidence Review controls for manual review status and safe notes.
- Updated roadmap and architecture docs.

## Non-Goals

KAN-107 does not add a tenant Auditor role, assignment workflow, multi-review comments, signed manifests, PDF/DOCX, official regulatory mappings, compliance scores, certification claims, BYOM, MCP, chatbot behavior, OPA/Rego execution, provider mutation, policy mutation, or Agent Governance dependency.

## Validation

Validation completed during implementation:

- `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`
- `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`
- `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`
- `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`
- `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`49` passed)
- `pnpm --dir gitgov exec vitest run src/test/useControlPlaneStore.test.ts` (`34` passed)
- `pnpm --dir gitgov typecheck`
- `pnpm --dir gitgov exec vitest run` (`366` passed)
- `pnpm --dir gitgov lint`
- `pnpm --dir gitgov build`
- `TEST_DATABASE_URL=<local ignored DATABASE_URL> cargo test --manifest-path gitgov/gitgov-server/Cargo.toml compliance_framework_review_reports -- --nocapture` (`2` passed)
- `TEST_DATABASE_URL=<local ignored DATABASE_URL> cargo test --manifest-path gitgov/gitgov-server/Cargo.toml -- --test-threads=2` (`307` passed)
- `psql --dbname=<local ignored DATABASE_URL without pgbouncer> -v ON_ERROR_STOP=1 -f gitgov/gitgov-server/supabase/supabase_schema_v50.sql`
- `psql --dbname=<local ignored DATABASE_URL without pgbouncer> -v ON_ERROR_STOP=1 -f gitgov/gitgov-server/supabase/supabase_schema_v50_postcheck.sql`
- `git diff --check`
- `.\scripts\security\publication_guard.ps1`

PR checks and post-merge `main` checks passed. Render deployed commit `bd4583a` and deploy `dep-d8njkhsvikkc73alv9l0` reached `live`.

Production smoke passed:

- `/health=ok`
- Authenticated `/stats=200`
- Listed Framework Review Reports for `org_name=yohandry10` and `framework_id=gitgov_release_governance_baseline_v1`.
- Reviewed report `frr_ac4ee214bc051caee783485d5755d34a` as `needs_changes` with a safe production-smoke note.
- Verified reviewer provenance was present.
- Verified `artifact_hash` was unchanged after review.
- Verified list metadata reflected `needs_changes`.
- Verified no-claim flags stayed intact.
- Verified invalid `review_status=approved` returned `400`.
