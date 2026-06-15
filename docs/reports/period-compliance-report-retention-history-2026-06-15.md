# Period Compliance Report Retention And Export History

Date: 2026-06-15
Ticket: `KAN-115`

## Decision

After KAN-113 and KAN-114, the next enterprise slice is retention and export custody for Period
Compliance Reports. Banks and regulated customers need to know whether a generated report is active,
expired, or archived; who accessed it; which JSON/PDF artifact was downloaded; and whether an Admin
changed retention after creation.

This slice stays manual-first. It does not add a scheduler, DOCX templates, official regulatory
wording, certification, compliance scoring, AI summaries, Agent Governance dependency, or physical
deletion workflow.

## Implemented

- Added Supabase migration `v57` with retention metadata on `compliance_period_reports`:
  `retention_status`, `retention_until`, `download_count`, `last_downloaded_at`, and `archived_at`.
- Added append-only `compliance_period_report_access_log` rows for:
  `viewed`, `downloaded_json`, `downloaded_pdf`, `retention_updated`, and `archived`.
- Updated JSON and PDF download paths to increment `download_count`, set `last_downloaded_at`, and
  write custody history without mutating the source artifact hash.
- Added Admin-only `PATCH /compliance/period-reports/{period_report_id}/retention`.
- Added Admin/Auditor `GET /compliance/period-reports/{period_report_id}/access-log`.
- Added Desktop/Tauri/store support for retention updates and access-log loading.
- Updated the Period Compliance Report panel to show retention status, retention date, archive date,
  download count, last download, access log, and Admin-only extend/archive controls.
- Split backend retention/access-log handlers into
  `gitgov-server/src/handlers/compliance_period_reports/retention.rs` to keep the primary Period
  Compliance Report handler from becoming a mixed-responsibility module.

## Validation

Focused real Postgres integration coverage now exercises the full compliance evidence chain:

- Generate reviewed source Framework Review Reports.
- Generate a Period Compliance Report.
- Verify default retention metadata.
- Verify Auditor metadata read creates `viewed`.
- Download JSON and verify `download_count`, `last_downloaded_at`, and `downloaded_json`.
- Generate/download PDF and verify PDF hash plus `downloaded_pdf`.
- Confirm Auditor cannot change retention.
- Mark the report `retention_expired` without physical deletion and still download it.
- Extend retention back to `active`.
- Archive the report logically and confirm the row still exists.
- List access log as Auditor and deny other-tenant access.
- Confirm source hashes, no-claim flags, and Agent Governance evaluation count remain unchanged.

Local checks run during implementation:

- `cargo fmt --manifest-path gitgov\gitgov-server\Cargo.toml`
- `cargo fmt --manifest-path gitgov\src-tauri\Cargo.toml`
- `cargo check --manifest-path gitgov\gitgov-server\Cargo.toml`
- `cargo check --manifest-path gitgov\src-tauri\Cargo.toml`
- `cargo clippy --manifest-path gitgov\gitgov-server\Cargo.toml -- -D warnings`
- `cargo clippy --manifest-path gitgov\src-tauri\Cargo.toml -- -D warnings`
- `cargo test --manifest-path gitgov\src-tauri\Cargo.toml`
- full backend real Postgres suite: `310` passed
- `pnpm --dir gitgov typecheck`
- `pnpm --dir gitgov lint`
- `pnpm --dir gitgov exec vitest run src/test/useControlPlaneStore.test.ts`
- `pnpm --dir gitgov test`
- `pnpm --dir gitgov build`
- local `v57` migration and postcheck through ignored `DATABASE_URL`
- `git diff --check`
- `.\scripts\security\publication_guard.ps1`
- focused real Postgres test:
  `period_compliance_report_aggregates_reviewed_reports_without_claims`

## Future Work

- Automatic retention expiration jobs.
- Customer-configurable retention policies per org/repo/framework.
- Explicit legal hold workflow.
- Physical deletion only after separate legal/security design and approval.
- Formal DOCX/regulatory templates and official wording remain future work.

## Production Validation

- PR `#400` merged to `main` as `1217b35`.
- Render deploy `dep-d8ns2ckm0tmc73bh7550` reached `live`.
- Production `v57` migration and postcheck passed.
- Production smoke created temporary Period Compliance Report
  `cpr_d02adc7f1f3d4389bb612f0be1c9a7d1` with `report_count=1` and artifact hash
  `sha256:6e1157b0ad756026b906923f85d192a56215525bf1b9becaa0c4c37b604b5d5b`.
- JSON download returned schema `gitgov_period_compliance_report.v1`.
- PDF export `cprpdf_c94c078cfb31d5069b529f404dc7082d` downloaded `2571` bytes.
- Retention extension returned `active`; logical archive returned `archived`.
- Access log contained `viewed`, `archived`, `retention_updated`, `downloaded_pdf`, and
  `downloaded_json`.
