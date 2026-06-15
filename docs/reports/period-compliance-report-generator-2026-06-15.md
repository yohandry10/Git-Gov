# KAN-113 Period Compliance Report Generator

Date: 2026-06-15

## Decision

Implement the first Compliance Report Generator slice as a manual, on-demand JSON period report. This is the next step after Framework Review Reports, manual review, Auditor collaboration, provenance manifests, PDF export, and framework pack diff.

## Included

- Backend storage: `compliance_period_reports` via `supabase_schema_v55.sql`.
- Backend API:
  - `POST /compliance/period-reports`
  - `GET /compliance/period-reports`
  - `GET /compliance/period-reports/{period_report_id}`
  - `GET /compliance/period-reports/{period_report_id}/download`
- Tauri client, models, and commands for create/list/get/download.
- Desktop Evidence Review panel for date-range generation, history, and JSON download.
- Period artifact schema `gitgov_period_compliance_report.v1`.

## Guardrails

- Creation is Admin-only.
- Read/download is Admin or Auditor, but Auditors only see/download a period report when they can access every source Framework Review Report.
- Source reports must be `reviewed`, in `[date_range_start, date_range_end)`, and preserve no-claim invariants.
- Empty period reports are rejected with `period_report_no_reviewed_reports`.
- The artifact is not a certification, compliance score, official regulatory report, legal attestation, or Agent Governance evaluation.
- Flags remain:
  - `compliance_claim=false`
  - `regulatory_claim=false`
  - `certification=false`
  - `requires_auditor_review=true`

## Deferred

- Scheduler.
- PDF/DOCX formal templates.
- Official regulatory wording.
- Compliance scores.
- Regulatory/certification claims.
- AI summaries.
- Agent Governance dependency.

## Validation

- Backend compile: `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`.
- Tauri compile: `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`.
- Frontend typecheck: `pnpm --dir gitgov typecheck`.
- Store test: `pnpm --dir gitgov test -- useControlPlaneStore`.
- Real Postgres integration test:
  - Loaded ignored local `.env`.
  - Set `TEST_DATABASE_URL` from `DATABASE_URL`.
  - Ran `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml period_compliance_report_aggregates_reviewed_reports_without_claims -- --nocapture`.

The real integration test builds the full chain: deployment gate evidence, evidence export, evidence mapping, review package, two in-range reviewed Framework Review Reports, one out-of-range reviewed report, provenance manifests, Auditor source assignment, period report generation, Auditor list/download, unassigned/other-tenant denial, hash recomputation, no-claim assertions, source report non-mutation, and no Agent Governance evaluation creation.

## Production

- PR: `#394`
- Merge commit: `3b6d760`
- Render deploy: `dep-d8nplmh9rddc739v7ivg`
- Production migration: `supabase_schema_v55.sql`
- Production postcheck: `supabase_schema_v55_postcheck.sql`

Production smoke passed:

- `/health=ok`
- Created period report `cpr_132e9f0fdef841278be3e167ff22cf32`.
- `report_count=1`.
- `artifact_hash=sha256:24ae1cb58186e2185dbccf931531225b4b861c6805bc6b9249a3f723a3df0d32`.
- Downloaded artifact schema was `gitgov_period_compliance_report.v1`.
- Source report count was `1`.
- Manifest hash count was `1`.
- No-claim flags remained false and `requires_auditor_review=true`.
- `agent_governance_required=false`.
