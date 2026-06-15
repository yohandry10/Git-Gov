# KAN-108 Tenant Auditor RBAC

Date: 2026-06-14
Branch: `product/KAN-108-tenant-auditor-rbac`
Issue: GitHub `#377`

## Implemented

- Added tenant `Auditor` role to backend `UserRole`.
- Added strict role parsing for `Auditor` in org users, org invitations, and API key creation.
- Added Supabase migration/postcheck `v51` for `api_keys`, `org_users`, and `org_invitations` role constraints.
- Added `require_compliance_reviewer` for Admin/Auditor access.
- Allowed Auditor read/download/review access only on compliance evidence review surfaces:
  - control framework list/get;
  - compliance evidence export get/download;
  - evidence mapping get;
  - review package get/download;
  - framework review report list/get/download/review.
- Kept creation and configuration surfaces Admin-only.
- Added Desktop Admin onboarding role options and API key role badge styling for Auditor.
- Updated architecture and roadmap docs.

## Non-Goals

KAN-108 does not add signed manifests, PDF/DOCX exports, official regulatory mappings, compliance scores, certification claims, BYOM, MCP, chatbot behavior, OPA/Rego execution, provider mutation, policy mutation, deployment gate mutation, Agent Governance dependency, SSO/SCIM, or granular permission matrices.

## Validation

Focused validation completed during implementation:

- `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`
- `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`
- `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`
- `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`
- `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`49` passed)
- `pnpm --dir gitgov typecheck`
- `TEST_DATABASE_URL=<local ignored DATABASE_URL> cargo test --manifest-path gitgov/gitgov-server/Cargo.toml compliance_framework_review_reports -- --nocapture` (`2` passed)
- `pnpm --dir gitgov exec vitest run src/test/useControlPlaneStore.test.ts` (`34` passed)
- `pnpm --dir gitgov exec vitest run` (`366` passed)
- `pnpm --dir gitgov lint`
- `pnpm --dir gitgov build`
- `TEST_DATABASE_URL=<local ignored DATABASE_URL> cargo test --manifest-path gitgov/gitgov-server/Cargo.toml -- --test-threads=2` (`308` passed)
- `psql --dbname=<local ignored DATABASE_URL without pgbouncer> -v ON_ERROR_STOP=1 -f gitgov/gitgov-server/supabase/supabase_schema_v51.sql`
- `psql --dbname=<local ignored DATABASE_URL without pgbouncer> -v ON_ERROR_STOP=1 -f gitgov/gitgov-server/supabase/supabase_schema_v51_postcheck.sql`
- `git diff --check`

- `.\scripts\security\publication_guard.ps1`

Publication and production validation:

- PR `#378` passed required checks and merged to `main` as `98cf543`.
- Post-merge checks passed for `98cf543`: `CI`, `Release Readiness Gate`, `Quality Gate Policy Matrix`, `Secret Scan`, `Public Naming Guard`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance`.
- Render deploy `dep-d8nk7hm8bjmc73f0sfng` for `98cf543` reached `live`.
- Production `v51` migration and postcheck passed.
- Production smoke passed with a temporary Auditor API key: evidence/report read and downloads returned `200`, report review returned `200`, report artifact hash stayed unchanged, no-claim flags stayed intact, Admin-only mutation surfaces returned `403`, and the temporary Auditor key was revoked.
