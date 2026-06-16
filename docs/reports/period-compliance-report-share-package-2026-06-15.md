# Period Compliance Report Share Packages

Ticket: `KAN-119`
Date: 2026-06-15

## Product Decision

After KAN-113 through KAN-118 made Period Compliance Reports generateable, downloadable as PDF,
retention/custody-aware, provenance-manifested, manually reviewed, and runnable from saved manual
profiles, the next slice is a manual share package.

The package is an offline JSON bundle for auditor/customer review. It organizes existing GitGov
evidence, binds hashes, and records custody. It is not a certification, legal attestation,
compliance score, or official regulatory claim.

## Included

- `GET/POST /compliance/period-reports/{period_report_id}/share-packages`.
- `GET /compliance/period-report-share-packages/{share_package_id}`.
- `GET /compliance/period-report-share-packages/{share_package_id}/download`.
- `PATCH /compliance/period-report-share-packages/{share_package_id}/revoke`.
- Append-only `compliance_period_report_share_packages` records.
- Share package artifact schema `gitgov_period_compliance_report_share_package.v1`.
- Package hash over the package payload with `verification.package_hash = null` before hashing.
- Required source state:
  - Period Compliance Report exists and is not archived.
  - Period Compliance Report `review_status` is `reviewed`.
  - Latest PDF export exists.
  - Latest provenance manifest exists.
- Package payload includes:
  - period report id, artifact id, artifact hash, schema, framework, and period bounds.
  - redacted period report summary.
  - PDF export id, hash, content type, and page count.
  - provenance manifest id, hash, schema, and hash-chain pointer.
  - review snapshot.
  - retention snapshot.
  - no-claim flags.
  - manual verification instructions.
- Custody/access-log actions:
  - `share_package_created`.
  - `share_package_downloaded`.
  - `share_package_revoked`.
- Admin audit record on package creation.
- Admin-only create and revoke.
- Admin/source-authorized Auditor list, read, and download.
- Developer denial, tenant isolation, and Agent Governance key denial.
- Desktop/Tauri DTOs, client methods, commands, and invoke registration.
- Control Plane store actions and `CompliancePeriodReportSharePackagePanel`.

## Not Included

- No public link generation.
- No email delivery.
- No scheduler.
- No DOCX/formal regulatory template.
- No compliance score.
- No certification, official regulatory claim, or legal attestation.
- No official regulatory mapping.
- No AI summary.
- No BYOM, MCP, chatbot, or Agent Governance dependency.
- No mutation of source Period Compliance Report JSON, PDF export, provenance manifest, review
  metadata, retention metadata, policy, or deployment gates.

## Implementation

- Supabase:
  - `gitgov/gitgov-server/supabase/supabase_schema_v61.sql`.
  - `gitgov/gitgov-server/supabase/supabase_schema_v61_postcheck.sql`.
- Backend:
  - share package DTOs in `compliance_framework_review_reports.rs`.
  - DB input structs in `db.rs`.
  - DB accessors in `db/compliance_period_reports.rs`.
  - route handlers in `handlers/compliance_period_reports/share_packages.rs`.
  - route registrations in `server/routes.rs`.
- Desktop/Tauri:
  - DTOs in `src-tauri/src/control_plane/server/models/compliance.rs`.
  - API client methods in `src-tauri/src/control_plane/server/client/compliance.rs`.
  - Tauri commands in `src-tauri/src/commands/server_commands.rs`.
  - invoke registration in `src-tauri/src/lib.rs`.
- Frontend:
  - store types/state/actions under `src/store/useControlPlaneStore/`.
  - UI component `CompliancePeriodReportSharePackagePanel.tsx`.
  - rendering inside `CompliancePeriodReportPanel.tsx`.

## Local Validation

The validation is intentionally end-to-end for the actual business chain, not a shallow mock-only
pass.

Backend focused integration coverage:

- rejects create before review with `period_report_not_reviewed`.
- rejects create after review but before PDF with `period_report_pdf_required`.
- rejects create after PDF but before provenance manifest with `period_report_manifest_required`.
- creates a real share package after JSON/PDF/manifest/review are present.
- verifies package schema, report artifact hash, PDF hash, manifest hash, review status, no-claim
  flags, and manual-only instructions.
- recomputes the package hash from the returned artifact and verifies it equals the stored hash.
- denies Agent Governance key access.
- denies Auditor create.
- denies Developer list.
- denies other-tenant Auditor download.
- allows source-authorized Auditor list/download.
- increments download count and timestamps on download.
- records `share_package_created` and `share_package_downloaded`.
- denies Auditor revoke.
- allows Admin revoke.
- blocks revoked package download with `share_package_revoked`.
- verifies Agent Governance evaluation count does not change.

Commands run successfully:

```powershell
cargo fmt --manifest-path gitgov\gitgov-server\Cargo.toml --check
cargo check --manifest-path gitgov\gitgov-server\Cargo.toml
cargo clippy --manifest-path gitgov\gitgov-server\Cargo.toml -- -D warnings
cargo test --manifest-path gitgov\gitgov-server\Cargo.toml period_compliance_report_aggregates_reviewed_reports_without_claims -- --nocapture
cargo test --manifest-path gitgov\gitgov-server\Cargo.toml compliance_period_reports -- --test-threads=1
cargo test --manifest-path gitgov\gitgov-server\Cargo.toml compliance_framework_review_reports -- --test-threads=1
cargo test --manifest-path gitgov\gitgov-server\Cargo.toml compliance_review_packages -- --test-threads=1
cargo test --manifest-path gitgov\gitgov-server\Cargo.toml compliance_evidence_exports -- --test-threads=1
cargo fmt --manifest-path gitgov\src-tauri\Cargo.toml --check
cargo check --manifest-path gitgov\src-tauri\Cargo.toml
cargo clippy --manifest-path gitgov\src-tauri\Cargo.toml -- -D warnings
cargo test --manifest-path gitgov\src-tauri\Cargo.toml
npm --prefix gitgov run typecheck
npm --prefix gitgov run lint
npm --prefix gitgov test -- --run src/test/useControlPlaneStore.test.ts src/test/components/CompliancePeriodReportSharePackagePanel.test.tsx
npm --prefix gitgov test -- --run
npm --prefix gitgov run build
```

Results:

- Focused backend period-report integration test passed.
- Affected backend module suites passed in serial.
- Tauri suite passed with `49` tests.
- Focused frontend store/component tests passed.
- Full frontend suite passed with `371` tests.
- Frontend production build passed with the existing Vite large-chunk warning.
- Migration `v61` and postcheck passed in a real rollback transaction against the configured
  Postgres connection.

Known local validation limit:

- Full parallel backend `cargo test` hit local Supabase/Postgres session exhaustion
  (`EMAXCONNSESSION max clients reached`).
- A full serial backend retry timed out after 15 minutes.
- The focused KAN-119 chain and affected backend modules passed; CI remains the final full-suite
  authority after PR creation.

## Production Validation

- Implementation PR `#416` merged to `main` as `1d1df77`.
- PR checks passed:
  - `Security Guard`.
  - `Server Clippy + Check`.
  - `Desktop Rust Clippy`.
  - `Frontend Lint + Typecheck`.
  - `Website Lint + Typecheck + Build`.
  - `Validate Policy-as-Code`.
  - `Validate quality_gates warn/block matrix`.
  - `Workflow Lint`.
  - `Sonar Scan + Quality Gate`.
  - Vercel preview checks.
- Post-merge `main` checks passed:
  - `CI`.
  - `Release Readiness Gate`.
  - `Quality Gate Policy Matrix`.
  - `Secret Scan`.
  - `Public Naming Guard`.
  - `Governance Correlation Smoke`.
  - `Desktop Updater Readiness`.
  - `SonarQube Governance`.
- Render deploy `dep-d8o8t9u47okc738l7g7g` reached `live` for commit `1d1df77`.
- Production `supabase_schema_v61.sql` and `supabase_schema_v61_postcheck.sql` passed.
- `/health` returned `ok`.
- Production smoke used Period Compliance Report
  `cpr_9389010c74a34484a8e080942b56956e`.
- The report was changed from `needs_review` to `reviewed` with reviewer `bootstrap-admin`.
- Existing PDF export `cprpdf_0d2e6aad239125a198e64c1a307b158d` was found with hash
  `sha256:0d2e6aad239125a198e64c1a307b158d54268612fcf8295550dda6c38ccf318e`.
- A new provenance manifest was created:
  - manifest id: `cprm_c3473263ece408fc12ca0bd5c7adc206`.
  - manifest hash: `sha256:48bce2f7c8b36c0c72e7324f3d0f7535cb9a315a34a082aeefe525b7e0526ef9`.
  - previous manifest hash:
    `sha256:d380b0e66bb0bec0684c8a7adaa3ea10473f75e7d5fba49f99114647a072386f`.
- Share package `cprsp_afaaed71cf684e63860915923722ce65` was created with:
  - status: `active`.
  - schema: `gitgov_period_compliance_report_share_package.v1`.
  - artifact hash:
    `sha256:49aa46f29c12a8a48286d099f171826c40584afc0094ad592344abd57b822e38`.
- Download returned schema `gitgov_period_compliance_report_share_package.v1` and the same
  package hash.
- No-claim/manual flags stayed false for compliance claim, regulatory claim, certification,
  compliance score, public link, email delivery, and Agent Governance usage.
- Revoke changed package status to `revoked`.
- A second download after revoke returned HTTP `409` with code `share_package_revoked`.
