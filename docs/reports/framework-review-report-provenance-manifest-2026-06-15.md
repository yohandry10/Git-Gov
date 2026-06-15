# KAN-110 Framework Review Report Provenance Manifest Report

Updated: 2026-06-15
Ticket: `KAN-110`
Branch: `product/KAN-110-reviewed-report-provenance-manifest`

## Implemented

- Added append-only table `compliance_framework_review_report_manifests` through Supabase migration
  `v53` and postcheck.
- Added backend route `POST /compliance/framework-review-reports/{report_id}/provenance-manifests`.
- Added backend route `GET /compliance/framework-review-reports/{report_id}/provenance-manifests/{manifest_id}`.
- Added manifest payload schema `gitgov_framework_review_report_provenance_manifest.v1`.
- Added hash-chain fields: `manifest_hash` and `previous_manifest_hash`.
- Added no-claim constraints and audit metadata confirming no report artifact mutation and no Agent
  Governance dependency.
- Added Tauri models/client/command support.
- Added Desktop Governance Evidence Review manifest generation/download control.

## Validation

- `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`
- `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`
- `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`
- `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`
- `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`
- `pnpm --dir gitgov typecheck`
- `pnpm --dir gitgov lint`
- `TEST_DATABASE_URL=<ignored local value> cargo test --manifest-path gitgov/gitgov-server/Cargo.toml framework_review_report_exports_baseline_mapping_with_source_hashes_and_no_claims -- --nocapture`
  - Creates real evidence export, evidence mapping, review package, Framework Review Report,
    Auditor assignment, safe comment, manual review transitions, and provenance manifests.
  - Confirms manifest is blocked before `reviewed`.
  - Confirms unassigned Auditor and Developer are blocked.
  - Confirms first manifest has no previous hash and second manifest chains to the first.
  - Confirms report artifact hash and no-claim flags remain unchanged.
  - Confirms no Agent Governance evaluations are created.
- `pnpm --dir gitgov test -- useControlPlaneStore.test.ts` (`34` passed)
- `TEST_DATABASE_URL=<ignored local value> cargo test --manifest-path gitgov/gitgov-server/Cargo.toml -- --test-threads=2` (`308` passed)
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`49` passed)
- `pnpm --dir gitgov test` (`366` passed)
- `pnpm --dir gitgov build`
- Local `v53` migration and postcheck against ignored `DATABASE_URL`
- `git diff --check`
- `.\scripts\security\publication_guard.ps1`

## Remaining Before Merge

- Open PR, wait for required checks, merge.
- Apply `v53` in production during deployment validation.
- Smoke production with a reviewed report after merge and Render deploy.
