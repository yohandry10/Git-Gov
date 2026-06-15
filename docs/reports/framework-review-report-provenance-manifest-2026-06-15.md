# KAN-110 Framework Review Report Provenance Manifest Report

Updated: 2026-06-15
Ticket: `KAN-110`
Branch: `product/KAN-110-reviewed-report-provenance-manifest`
PR: `#384`
Merge commit: `a7ab2e5`

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

## PR And Production Validation

- PR `#384` required checks passed and merged to `main` as `a7ab2e5`.
- Post-merge `main` checks passed for `a7ab2e5`: `CI`, `Release Readiness Gate`,
  `Quality Gate Policy Matrix`, `Secret Scan`, `Public Naming Guard`,
  `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance`.
- Render deploy `dep-d8nmoluq1p3s738cqdl0` for `a7ab2e5` reached `live`.
- Production `v53` migration and postcheck passed.
- Production smoke passed against real report `frr_ac4ee214bc051caee783485d5755d34a`:
  - `/health=200`
  - report status was `reviewed`
  - created manifest `frrm_ed5a356ad89f406b90e236756655183c`
  - created manifest `frrm_a3135e737c43d92fbc3f8b56d19d0a0c`
  - second manifest `previous_manifest_hash` matched the first manifest hash
  - downloaded manifest hash-chain matched the stored manifest
  - schema was `gitgov_framework_review_report_provenance_manifest.v1`
  - signature algorithm was `sha256-provenance-manifest-v1`
  - source report `artifact_hash` stayed unchanged
  - no-claim flags stayed valid
  - `agent_governance_required=false`
  - `source_report_artifact_mutated=false`
