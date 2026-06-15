# Period Compliance Report Review/Sign-off

Ticket: `KAN-117`
Date: 2026-06-15

## Product Decision

After KAN-113 through KAN-116 made Period Compliance Reports generateable, downloadable as PDF,
retention/custody-aware, and provenance-manifested, the next slice is manual review/sign-off
metadata.

The goal is to let a human Admin or Auditor record a review state directly on an existing Period
Compliance Report before the report is shared or treated as ready for a compliance package.

## Included

- `GET/PATCH /compliance/period-reports/{period_report_id}/review`.
- Review statuses: `needs_review`, `reviewed`, `needs_changes`, and `rejected`.
- Reviewer id, review timestamp, and safe review notes.
- Admin and source-authorized Auditor access.
- Developer denial and tenant/source-assignment isolation.
- Archived-report conflict on review updates.
- Audit log entry `compliance_period_report.reviewed`.
- Period report custody log action `review_updated`.
- Provenance manifest payload includes review status/reviewer/timestamp/note-presence metadata.
- Source Period Compliance Report JSON payload and artifact hash remain unchanged.

## Not Included

- No certification or official regulatory approval.
- No legal attestation.
- No compliance score.
- No DOCX/formal regulatory template.
- No scheduler.
- No KMS or cryptographic signature.
- No AI summary and no Agent Governance dependency.
- No mutation of source Framework Review Reports, PDF exports, provenance manifests, Deployment
  Gate evidence, or policy.

## Implementation

- Supabase migration/postcheck:
  - `gitgov/gitgov-server/supabase/supabase_schema_v59.sql`
  - `gitgov/gitgov-server/supabase/supabase_schema_v59_postcheck.sql`
- Backend:
  - period report review request/record fields.
  - DB update method scoped by org and blocked for archived reports.
  - safe note validation and status validation.
  - review routes wired into the Axum route table.
  - custody/audit evidence on successful updates.
- Desktop/Tauri:
  - DTOs, client method, Tauri command, and invoke registration.
  - Control Plane store action and state flag.
  - `CompliancePeriodReportReviewPanel` rendered inside the existing Period Compliance Report panel.
- Provenance:
  - new manifests include review metadata and explicit non-claim flags.

## Validation

Local validation covers the real chain rather than a shallow unit-only pass:

- Real backend integration test creates the compliance evidence chain, generates a Period
  Compliance Report, rejects Developer review, rejects unassigned Auditor review, rejects unsafe
  notes, rejects `needs_changes` without notes, accepts assigned Auditor review, verifies source
  artifact hash immutability, verifies custody log `review_updated`, verifies manifest review
  metadata, and verifies archived-report conflict.
- Frontend store test invokes the real Tauri command shape and checks the selected report plus
  historical list are updated.

Commands:

```powershell
cargo fmt --manifest-path gitgov\gitgov-server\Cargo.toml
cargo fmt --manifest-path gitgov\src-tauri\Cargo.toml
cargo check --manifest-path gitgov\gitgov-server\Cargo.toml
cargo clippy --manifest-path gitgov\gitgov-server\Cargo.toml -- -D warnings
cargo test --manifest-path gitgov\gitgov-server\Cargo.toml compliance_period_reports -- --nocapture
cargo test --manifest-path gitgov\gitgov-server\Cargo.toml -- --test-threads=2
cargo check --manifest-path gitgov\src-tauri\Cargo.toml
cargo clippy --manifest-path gitgov\src-tauri\Cargo.toml -- -D warnings
cargo test --manifest-path gitgov\src-tauri\Cargo.toml
npm run typecheck
npm run lint
npm test -- --run src/test/useControlPlaneStore.test.ts
npm test -- --run
npm run build
git diff --check
.\scripts\security\publication_guard.ps1
```

Local result:

- Full backend suite: `310` passed against an isolated test schema.
- Tauri suite: `49` passed.
- Full frontend suite: `367` passed.
- Focused store suite: `35` passed.
- Build passed with the existing Vite large-chunk warning.

## Production Validation

- Implementation PR `#406` merged to `main` as `ade6302`.
- Postcheck schema-scope fix PR `#407` merged to `main` as `05e0706`.
- PR checks passed for both PRs.
- Post-merge `main` checks passed for `05e0706`, including `CI`, `Release Readiness Gate`,
  `Quality Gate Policy Matrix`, `Secret Scan`, `Public Naming Guard`, `Governance Correlation
  Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance`.
- Render deploy `dep-d8ntnem47okc73f85pig` reached `live` for `ade6302`.
- Final Render deploy `dep-d8nts6f7f7vs73ftqgdg` reached `live` for `05e0706`.
- Production `v59` migration ran and `supabase_schema_v59_postcheck.sql` passed.
- `/health` returned `ok`.
- Active Period Compliance Report `cpr_132e9f0fdef841278be3e167ff22cf32` was reviewed in
  production:
  - before: `needs_review`
  - after: `reviewed`
  - reviewer: `bootstrap-admin`
  - artifact hash unchanged: `true`
  - custody log contains `review_updated`
- Archived Period Compliance Report `cpr_d02adc7f1f3d4389bb612f0be1c9a7d1` rejected review update
  with HTTP `409` and code `period_report_archived`.
