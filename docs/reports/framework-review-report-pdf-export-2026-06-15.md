# KAN-111 Framework Review Report PDF Export Report

Updated: 2026-06-15
Ticket: `KAN-111`
Branch: `product/KAN-111-framework-review-report-pdf-export`
PR: `#388`
Merge commit: `97b1b94`

## Implemented

- Added Supabase migration/postcheck `v54` for append-only
  `compliance_framework_review_report_pdf_exports`.
- Added backend routes:
  - `POST /compliance/framework-review-reports/{report_id}/pdf-export`
  - `GET /compliance/framework-review-reports/{report_id}/pdf-export`
  - `GET /compliance/framework-review-reports/{report_id}/pdf-export/download`
- Added a deterministic, server-side PDF renderer for reviewed Framework Review Reports.
- Bound every PDF export to source report hash and provenance manifest hash.
- Persisted `pdf_artifact_hash`, content type, page count, creator, timestamps, and no-claim flags.
- Added Tauri models, client methods, and server commands for create/get/download.
- Added Desktop Governance Evidence Review PDF generation/download UI.
- Updated roadmap and architecture docs.

## Validation

- `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml`
- `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml`
- `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`
- `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`
- `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`
- `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`
- `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`
- `pnpm --dir gitgov typecheck`
- `TEST_DATABASE_URL=<ignored local value> cargo test --manifest-path gitgov/gitgov-server/Cargo.toml framework_review_report_exports_baseline_mapping_with_source_hashes_and_no_claims -- --nocapture`
  - Creates the real KAN-99/KAN-100/KAN-101/KAN-105/KAN-109/KAN-110 evidence chain.
  - Blocks PDF before manual `reviewed` status.
  - Blocks unassigned Auditor, Developer, and other tenant paths.
  - Creates PDF from a real reviewed report plus provenance manifest.
  - Downloads real `application/pdf` bytes.
  - Verifies `x-gitgov-artifact-hash` and SHA-256 of downloaded bytes.
  - Verifies PDF content contains no-claim language, report hash, manifest hash, and reviewer
    provenance.
  - Verifies source report artifact hash is unchanged.
  - Verifies no Agent Governance evaluations are created.
- `pnpm --dir gitgov test -- useControlPlaneStore.test.ts` (`34` passed)
- `TEST_DATABASE_URL=<ignored local value> cargo test --manifest-path gitgov/gitgov-server/Cargo.toml -- --test-threads=2` (`308` passed)
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`49` passed)
- `pnpm --dir gitgov lint`
- `pnpm --dir gitgov test` (`366` passed)
- `pnpm --dir gitgov build`
- Local `v54` migration and postcheck against ignored `DATABASE_URL`
- `git diff --check`
- `.\scripts\security\publication_guard.ps1`

Pending before completion:

- None.

## PR And Production Validation

- PR `#388` required checks passed and merged to `main` as `97b1b94`.
- Post-merge `main` checks passed for `97b1b94`: `CI`, `Release Readiness Gate`,
  `Quality Gate Policy Matrix`, `Secret Scan`, `Public Naming Guard`,
  `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance`.
- Render deploy `dep-d8nnnvgjo6nc73e4u030` for `97b1b94` reached `live`.
- Production `v54` migration and postcheck passed.
- Production smoke passed against real report `frr_ac4ee214bc051caee783485d5755d34a`:
  - `/health=ok`
  - report status was `reviewed`
  - created PDF export `frrpdf_f2bd8e2a866a9194889e05f59d0829b5`
  - PDF used manifest `frrm_a3135e737c43d92fbc3f8b56d19d0a0c`
  - source report hash was `sha256:7adb239ac4c00c67064ca39462f7bdce66898a818531b8f813b4cbb6cbee6a54`
  - PDF artifact hash was `sha256:f2bd8e2a866a9194889e05f59d0829b595cc1761af0d12ed8b1c1d85ffaa7e87`
  - downloaded PDF was `3930` bytes and started with `%PDF-1.4`
  - download `Content-Type` was `application/pdf`
  - `x-gitgov-artifact-hash` matched the stored PDF artifact hash
  - downloaded bytes SHA-256 matched the stored PDF artifact hash
  - PDF text contained no-claim language, report hash, manifest hash, and no-claim flags
  - source report `artifact_hash` stayed unchanged
