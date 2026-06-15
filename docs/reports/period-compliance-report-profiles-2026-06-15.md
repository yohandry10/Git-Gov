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

- PR `#410` merged to `main` as `1ecb61b`.
- Post-merge `main` checks passed, including `CI`, `Release Readiness Gate`,
  `Quality Gate Policy Matrix`, `Secret Scan`, `Public Naming Guard`, `Governance Correlation
  Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance`.
- Render deploy `dep-d8o29ccvikkc73evb8cg` reached `live` for `1ecb61b`.
- Production `v60` migration ran and `supabase_schema_v60_postcheck.sql` passed.
- `/health` returned `ok`.
- Production smoke used reviewed Framework Review Report
  `frr_ac4ee214bc051caee783485d5755d34a` and framework
  `gitgov_release_governance_baseline_v1`.
- Saved profile `cprprof_0f4c3ece4eb04856b4928b3eaeeed469` ran manually and created:
  - Period Compliance Report `cpr_9389010c74a34484a8e080942b56956e`.
  - PDF export `cprpdf_0d2e6aad239125a198e64c1a307b158d`.
  - Provenance manifest `cprm_fdf8d9344b81fcd2111300511e139c00`.
- JSON download returned schema `gitgov_period_compliance_report.v1`.
- PDF download returned `%PDF-1.4`, `2571` bytes, and recomputed SHA-256 matched the stored
  `pdf_artifact_hash`.
- Manifest download returned schema `gitgov_period_compliance_report_provenance_manifest.v1`.
- After profile update disabled PDF and manifest creation, second run
  `cpr_66dd549b6c2a4ad9ade49a20721e979a` created no PDF export and no manifest.
- Archiving the profile changed status to `archived`; a subsequent run returned HTTP `409`.
- A temporary Auditor API key was created only to validate mutation denial; profile creation returned
  HTTP `403`, and the temporary key was revoked.
- Agent Governance evaluation count stayed unchanged at `7`, confirming KAN-118 did not create
  Agent Governance evidence.
