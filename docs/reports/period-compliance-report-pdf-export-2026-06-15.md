# KAN-114 Period Compliance Report PDF Export

Date: 2026-06-15

## Decision

Implement the next Compliance Report Generator slice as a manual PDF export for already generated
Period Compliance Reports.

The PDF is a customer/auditor review artifact. It is not a DOCX/formal regulatory template,
certification, compliance score, legal attestation, official regulatory claim, AI summary, scheduler,
or Agent Governance dependency.

## Included

- Backend storage: `compliance_period_report_pdf_exports` via `supabase_schema_v56.sql`.
- Backend API:
  - `POST /compliance/period-reports/{period_report_id}/pdf-export`
  - `GET /compliance/period-reports/{period_report_id}/pdf-export`
  - `GET /compliance/period-reports/{period_report_id}/pdf-export/download`
- Tauri client, models, and commands for create/get/download.
- Desktop Evidence Review period report panel support for PDF generation and download.
- Store state/actions and focused store coverage.
- Maintained source boundaries: period PDF backend handlers live in a focused module, and Desktop
  store actions split Period Reports plus Framework Review Report artifacts out of the large
  compliance action aggregator.

## Guardrails

- Creation/read/download require Admin or Auditor access to every source Framework Review Report in
  the source Period Compliance Report.
- Source Period Compliance Report must already be `generated` JSON.
- The PDF is bound to `source_period_report_hash` and has its own `pdf_artifact_hash`.
- The source JSON artifact is not mutated.
- No-claim flags remain:
  - `compliance_claim=false`
  - `regulatory_claim=false`
  - `certification=false`
  - `requires_auditor_review=true`

## Deferred

- DOCX export.
- Formal report templates.
- Official regulatory wording.
- Compliance scores.
- Regulatory/certification claims.
- Scheduler.
- AI summaries.
- Agent Governance dependency.

## Validation

- Backend compile: `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`.
- Backend clippy: `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`.
- Full backend real Postgres suite: `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml -- --test-threads=2` (`310` passed).
- Tauri compile: `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`.
- Tauri clippy/tests: `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings` and `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`49` passed).
- Frontend typecheck: `pnpm --dir gitgov typecheck`.
- Frontend lint: `pnpm --dir gitgov lint`.
- Focused store test: `pnpm --dir gitgov test -- useControlPlaneStore`.
- Full frontend tests/build: `pnpm --dir gitgov test` (`367` passed) and `pnpm --dir gitgov build`.
- Maintained frontend file size after refactor: `compliance.ts` `782` lines, `period-reports.ts`
  `214` lines, `framework-review-report-artifacts.ts` `132` lines.
- Local DB migration/postcheck: `supabase_schema_v56.sql` and `supabase_schema_v56_postcheck.sql`.
- Publication guard: `.\scripts\security\publication_guard.ps1`.
- Real Postgres integration test:
  - Loaded ignored local `.env`.
  - Set `TEST_DATABASE_URL` from `DATABASE_URL`.
  - Ran `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml period_compliance_report_aggregates_reviewed_reports_without_claims -- --nocapture`.

The real integration test builds the full chain from Deployment Gate evidence through Period
Compliance Report generation, then verifies PDF creation/download with assigned Auditor access,
unassigned/other-tenant denial, `%PDF-1.4` bytes, recomputed PDF SHA-256 hash, no secret-like fixture
leakage, unchanged source Period Compliance Report hash, no-claim flags, audit log creation, and no
Agent Governance evaluation creation.

## Production Validation

- PR `#397` merged to `main` as `04bc6f5`.
- Render deploy `dep-d8nr1n7aqgkc73cbmi50` for `04bc6f5` reached `live`.
- Production `v56` migration and postcheck passed.
- Production smoke:
  - `/health=ok`.
  - Source Period Compliance Report: `cpr_132e9f0fdef841278be3e167ff22cf32`.
  - Source artifact hash stayed bound to KAN-113 hash
    `sha256:24ae1cb58186e2185dbccf931531225b4b861c6805bc6b9249a3f723a3df0d32`.
  - PDF export: `cprpdf_609b267d32c178f420a72a9c0f9256b5`.
  - `content_type=application/pdf`, `page_count=1`, downloaded byte prefix `%PDF-1.4`,
    byte length `2571`.
  - Stored hash, `x-gitgov-artifact-hash`, and recomputed downloaded-file hash all matched
    `sha256:609b267d32c178f420a72a9c0f9256b57519df6e72e8a6248a7380e60f0cbf34`.
