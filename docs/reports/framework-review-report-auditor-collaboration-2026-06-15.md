# Framework Review Report Auditor Collaboration Report

Date: 2026-06-15
Ticket: `KAN-109`
Issue: GitHub `#380`

## Implemented

- Added persisted `compliance_framework_review_report_assignments` and
  `compliance_framework_review_report_comments` through Supabase migration `v52`.
- Added backend collaboration endpoints for assignment replacement, assignment listing,
  assigned-to-me report history, comment creation, and comment listing.
- Added assignment-aware authorization: Admins retain control; assigned Auditors can collaborate;
  unassigned same-tenant Auditors are blocked once active assignments exist.
- Extended Desktop/Tauri models, client methods, commands, Zustand state/actions, and Governance
  Evidence Review UI controls.
- Kept the implementation manual-first and metadata-only. Report artifacts, hashes, claims, policy,
  Deployment Gates, and Agent Governance are not changed.

## Validation

- `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`
- `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`
- `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`
- Full backend Postgres suite:
  `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml -- --test-threads=2` (`308`
  passed)
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`
- `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`
- `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`49` passed)
- `pnpm --dir gitgov typecheck`
- `pnpm --dir gitgov test src/test/useControlPlaneStore.test.ts` (`34` tests)
- `pnpm --dir gitgov test` (`366` passed)
- `pnpm --dir gitgov lint`
- `pnpm --dir gitgov build`
- Local `supabase_schema_v52.sql` plus `supabase_schema_v52_postcheck.sql` through ignored
  `DATABASE_URL`
- `git diff --check`
- `.\scripts\security\publication_guard.ps1`
- Real Postgres focused test:
  `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml framework_review_report_exports_baseline_mapping_with_source_hashes_and_no_claims -- --nocapture`

The focused Postgres test builds the real KAN-99 to KAN-105 evidence chain, assigns one Auditor,
verifies assigned-to-me filtering with framework/mapping/package filters, blocks a same-tenant
unassigned Auditor, blocks another tenant, creates a safe comment, rejects secret-like text, updates
review metadata as the assigned Auditor, checks audit rows, preserves the artifact hash and no-claim
flags, and confirms Agent Governance evaluations are unchanged.

## Merge

Completed by PR `#381`, merged to `main` as `cee4594`.

## Production Validation

- Post-merge `main` checks passed for `cee4594`: `CI`, `Release Readiness Gate`, `Quality Gate
  Policy Matrix`, `Secret Scan`, `Public Naming Guard`, `Governance Correlation Smoke`, `Desktop
  Updater Readiness`, and `SonarQube Governance`.
- Render deploy `dep-d8nlhi6q1p3s738c7f10` for `cee4594` reached `live`.
- Production `supabase_schema_v52.sql` plus `supabase_schema_v52_postcheck.sql` passed.
- Production smoke passed on report `frr_ac4ee214bc051caee783485d5755d34a`:
  - `/health=200`
  - authenticated `/stats=200`
  - temporary Auditor A assigned to the report
  - Auditor A `assigned-to-me` returned the report
  - temporary Auditor B `assigned-to-me` did not return the report
  - Auditor A comment returned `201`
  - Auditor B comment and review returned `403 auditor_not_assigned`
  - Auditor A review returned `200`
  - report `artifact_hash` stayed unchanged
  - `compliance_claim=false`, `regulatory_claim=false`, `certification=false`, and
    `requires_auditor_review=true` stayed unchanged
  - active assignments were cleared after smoke
  - all `kan109-prod-*` temporary API keys were revoked (`activeTemp=0`)
