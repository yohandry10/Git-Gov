# Saved Period Compliance Report Profiles

Ticket: `KAN-118`
Date: 2026-06-15

## Product Decision

After KAN-113 through KAN-117 made Period Compliance Reports generateable, downloadable as PDF,
retention/custody-aware, provenance-manifested, and manually reviewed, the next slice is reusable
manual report profiles.

The goal is to let an Admin save a common reporting setup and run it on demand without introducing a
scheduler or changing the manual governance model.

## Included

- `GET/POST /compliance/period-report-profiles`.
- `GET/PATCH /compliance/period-report-profiles/{profile_id}`.
- `PATCH /compliance/period-report-profiles/{profile_id}/archive`.
- `POST /compliance/period-report-profiles/{profile_id}/run`.
- Profile fields for name, period type, optional framework, framework owner type, PDF toggle,
  manifest toggle, retention days, safe filters, run count, last run timestamp, and last artifact ids.
- Admin-only create/update/archive/run.
- Auditor read access.
- Developer denial and tenant isolation.
- Archived-profile conflict on mutation/run.
- Profile runs create normal Period Compliance Report JSON artifacts and optionally create PDF export
  and provenance manifest artifacts.
- Newly generated period reports remain `needs_review`.
- Retention defaults from the profile are applied through the existing retention/custody path.

## Not Included

- No scheduler or background job.
- No email delivery.
- No DOCX/formal regulatory template.
- No compliance score.
- No certification, official regulatory claim, or legal attestation.
- No official regulatory mapping.
- No KMS signing.
- No BYOM, MCP, chatbot, or Agent Governance dependency.
- No new report artifact format outside the existing JSON/PDF/manifest chain.

## Implementation

- Supabase migration/postcheck:
  - `gitgov/gitgov-server/supabase/supabase_schema_v60.sql`
  - `gitgov/gitgov-server/supabase/supabase_schema_v60_postcheck.sql`
- Backend:
  - profile DTOs and DB accessors.
  - profile validation for safe names, safe filters, period types, retention bounds, and archived
    status.
  - manual profile run orchestration that creates a period report, optional PDF export, optional
    provenance manifest, and profile run metadata.
  - admin audit records for profile create/update/archive/run.
- Desktop/Tauri:
  - DTOs, API client methods, commands, and invoke registration.
  - Control Plane store state/actions for profile CRUD and run.
  - `CompliancePeriodReportProfilePanel` inside the existing Period Compliance Report panel.

## Local Validation

Commands already run successfully:

```powershell
$env:TEST_DATABASE_URL='<temporary local postgres url>'; cargo test
npm.cmd run test -- src/test/useControlPlaneStore.test.ts
cargo test
cargo fmt --check
cargo check
cargo clippy -- -D warnings
npm.cmd run typecheck
npm.cmd run lint
npm.cmd run test
npm.cmd run build
```

Results:

- Backend real Postgres suite: `311` passed.
- Focused backend test:
  `period_report_profiles_run_real_artifacts_and_enforce_manual_boundaries` passed and verifies real
  JSON/PDF/manifest creation, retention propagation, Auditor/Developer denial, tenant isolation,
  archived run conflict, no Agent Governance evaluation changes, and profile audit records.
- Tauri suite: `49` passed.
- Store suite: `36` passed.
- Full frontend suite: `368` passed.
- Frontend production build passed with the existing Vite large-chunk warning.
- Backend and Tauri `fmt`, `check`, and `clippy -- -D warnings` passed.
- Migration `v60` and postcheck passed against a real temporary Postgres instance.

## Production Validation

Pending until PR merge, Render deploy, production `v60` migration/postcheck, and production smoke.
