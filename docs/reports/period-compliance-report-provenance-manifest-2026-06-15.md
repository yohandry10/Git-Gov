# Period Compliance Report Provenance Manifest

Date: 2026-06-15
Ticket: `KAN-116`

## Decision

After KAN-113 JSON reports, KAN-114 PDF exports, and KAN-115 retention/custody history, the next
enterprise slice is a Period Compliance Report provenance manifest. The goal is to give Admins and
Auditors a single append-only JSON artifact that binds the period report hash, source evidence
hashes, PDF export hashes, retention state, and custody/access-log summary.

This remains manual-first. It does not add a scheduler, DOCX/formal regulatory template, official
regulatory wording, compliance score, certification, legal attestation, AI summary, cryptographic
KMS signing, or Agent Governance dependency.

## Implemented

- Added Supabase migration/postcheck `v58`.
- Added `compliance_period_report_manifests` with:
  `manifest_id`, `period_report_id`, `manifest_hash`, `previous_manifest_hash`,
  `signature_algorithm`, redacted payload JSON, and no-claim constraints.
- Extended `compliance_period_report_access_log` to include `manifest_created`,
  `manifest_downloaded`, and `manifest` artifact type.
- Added backend routes:
  `POST /compliance/period-reports/{period_report_id}/provenance-manifests` and
  `GET /compliance/period-reports/{period_report_id}/provenance-manifests/{manifest_id}`.
- Manifest artifact schema:
  `gitgov_period_compliance_report_provenance_manifest.v1`.
- Signature/hash-chain label:
  `sha256-period-report-provenance-manifest-v1`.
- The manifest binds period JSON hash, source hashes/manifests, PDF exports, retention metadata,
  access-log summary, no-claim flags, and source-artifact non-mutation metadata.
- Added Tauri DTOs, client methods, and commands.
- Added Desktop store actions and state for create/download.
- Added `CompliancePeriodReportProvenancePanel` under the existing Period Compliance Report panel.

## Validation

Local validation run during implementation:

- `cargo fmt --manifest-path gitgov\gitgov-server\Cargo.toml`
- `cargo fmt --manifest-path gitgov\src-tauri\Cargo.toml`
- `cargo check --manifest-path gitgov\gitgov-server\Cargo.toml`
- `cargo check --manifest-path gitgov\src-tauri\Cargo.toml`
- `cargo clippy --manifest-path gitgov\gitgov-server\Cargo.toml -- -D warnings`
- `cargo clippy --manifest-path gitgov\src-tauri\Cargo.toml -- -D warnings`
- `cargo test --manifest-path gitgov\src-tauri\Cargo.toml` (`49` passed)
- `cargo test --manifest-path gitgov\gitgov-server\Cargo.toml` (`310` passed)
- Focused backend test:
  `period_compliance_report_aggregates_reviewed_reports_without_claims`
- `pnpm --dir gitgov typecheck`
- `pnpm --dir gitgov lint`
- `pnpm --dir gitgov exec vitest run src/test/useControlPlaneStore.test.ts` (`35` passed)
- `pnpm --dir gitgov exec vitest run` (`367` passed)
- `pnpm --dir gitgov build`
- Local Docker Postgres migration chain `v55` through `v58` plus postchecks.

The focused Period Compliance Report integration coverage now verifies:

- Period JSON generation from reviewed source reports.
- PDF export creation and download.
- Manifest creation by an authorized Auditor.
- Unassigned Auditor and other-tenant access denial.
- Manifest schema/version, signature algorithm, source period hash, PDF hash binding, no-claim flags,
  and Agent Governance non-dependency.
- Manifest download returns the stored hash-chain payload.
- Second manifest records `previous_manifest_hash` equal to the first manifest hash.
- Manifest create/download actions are present in custody access log.
- Source period report artifact hash remains unchanged.

## Future Work

- KMS-backed or customer-key cryptographic signatures.
- Formal report/DOCX templates after customer validation of the JSON/PDF/manifest chain.
- Scheduled period report generation.
- Legal hold and physical deletion workflow after separate legal/security design.

## Production Validation

- PR `#403` merged to `main` as `81fe9aa`.
- Render deploy `dep-d8nsqo8jo6nc73e7qjpg` for `81fe9aa` reached `live`.
- Production `v58` migration and postcheck passed after removing the local-only `pgbouncer`
  parameter from the `psql` connection string.
- Production smoke used the KAN-115 temporary Period Compliance Report
  `cpr_d02adc7f1f3d4389bb612f0be1c9a7d1`.
- First manifest:
  `cprm_271f881c821d21735981a38cdae84552`,
  `sha256:02485b69a43db63e2afb644d7424afe802e508dcc2a51b05615c71c6f632abbd`.
- Downloaded manifest returned schema
  `gitgov_period_compliance_report_provenance_manifest.v1` and matching hash-chain manifest hash.
- Second manifest:
  `cprm_eb0888d6a47f7433c4395ca6c0634ba9`; its `previous_manifest_hash` matched the first
  manifest hash.
- Manifest artifact reported `pdf_export_count=1`.
- Access log contained `manifest_created`, `manifest_downloaded`, `viewed`, `archived`,
  `retention_updated`, `downloaded_pdf`, and `downloaded_json`.
