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
