# KAN-106 Framework Review Report Inventory History

Ticket: `KAN-106`
Issue: GitHub `#371`

## Product Decision

KAN-106 is the next slice after KAN-105 because KAN-105 could create and download a framework review report only when an operator already knew the `report_id`.

The product gap was recoverability: an Admin needed a safe way to list previous framework review reports, verify metadata, and download a historical JSON report from Desktop. This remains manual-first and does not introduce official regulatory mappings, certification claims, compliance scores, PDF/DOCX generation, OPA/Rego execution, BYOM, MCP, chatbot behavior, provider mutation, Action Center writes, or Agent Governance dependency.

GPT consultation was attempted from the existing ChatGPT thread, but the assistant responses were empty after repeated submissions. The decision was made from the repo state and roadmap: finish inventory/history before moving to auditor workflow, signed provenance manifests, regulatory mappings, or model routing.

## Scope

- Extend `GET /compliance/framework-review-reports` as an Admin-only list endpoint.
- Keep `POST /compliance/framework-review-reports` on the same path for KAN-105 report creation.
- Add safe optional list filters:
  - `framework_id`
  - `mapping_id`
  - `review_package_id`
  - `limit`
- Clamp list limit to `1..100`, defaulting to `25`.
- Return metadata records only. The list response does not include `payload_json_redacted`, `artifact`, or raw evidence payloads.
- Add Tauri client/model/command support for listing reports.
- Add Desktop Governance Evidence Review history controls to load recent reports and download a selected historical report.
- Add Supabase migration/postcheck `v49` with indexes for tenant/framework inventory lookups.

## Guardrails

- Admin-only sensitive route.
- Tenant scope is resolved server-side from auth plus optional `org_name`; queries always include `org_id`.
- Agent keys remain denied because compliance report routes are sensitive admin paths.
- Historical list records preserve no-claim flags:
  - `compliance_claim=false`
  - `regulatory_claim=false`
  - `certification=false`
  - `requires_auditor_review=true`
- Downloading the JSON artifact still requires the explicit report id and admin auth.

## Validation Focus

- Real Postgres integration:
  - create KAN-105 report prerequisites.
  - list by `framework_id`, `mapping_id`, and `review_package_id`.
  - verify `limit=500` is clamped to `100`.
  - verify another tenant Admin sees zero records.
  - verify invalid `mapping_id` query returns `400`.
  - verify list metadata omits heavy artifact/payload fields.
- Frontend store:
  - create export, mapping, review package, report.
  - load report history with trimmed filters.
  - verify history records are metadata-only.
  - download a historical report artifact.

