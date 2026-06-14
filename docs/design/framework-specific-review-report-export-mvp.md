# KAN-105 Framework-specific Review Report Export

Ticket: `KAN-105`
Issue: GitHub `#368`

## Product Decision

KAN-105 turns the KAN-99/KAN-100/KAN-101/KAN-103/KAN-104 chain into a framework-specific JSON report.

The report is for customer or auditor review. It is not SOC 2, ISO, NIST, PCI, SBS, LGPD, an official regulatory mapping, a compliance score, a certification claim, OPA/Rego execution, Policy-as-Code enforcement, provider mutation, Action Center automation, MCP, chatbot, BYOM, or Agent Governance.

## Scope

- New routes:
  - `POST /compliance/framework-review-reports`
  - `GET /compliance/framework-review-reports/{report_id}`
  - `GET /compliance/framework-review-reports/{report_id}/download`
- New Supabase migration/postcheck `v48`.
- New persisted table `compliance_framework_review_reports`.
- Report input requires both:
  - `mapping_id`
  - `review_package_id`
- The backend verifies the review package matches the mapping before generating a report.
- Customer-owned framework reports require the current pack status to remain `reviewed`.
- The JSON artifact includes framework owner/source/review provenance, source hashes, summary counts, control-by-control evidence refs, missing evidence, no-claim flags, and audit metadata.
- Desktop Governance Evidence Review adds a `Framework Review Report` panel for generation and JSON download.

## Guardrails

- Admin-only.
- Agent keys cannot create or download reports because the route is treated as a sensitive admin path.
- `compliance_claim=false`, `regulatory_claim=false`, `certification=false`, and `requires_auditor_review=true` are enforced in the table and artifact.
- A report cannot be generated from mismatched mapping/package sources.
- A report cannot be generated if the current customer pack is `needs_review`, `needs_changes`, `rejected`, or `archived`.
- No raw payload, secrets, LLM summaries, provider mutation, or policy mutation are included.

## Validation Focus

- Baseline framework path:
  - KAN-99 export
  - KAN-100 mapping
  - KAN-101 review package
  - KAN-105 report
  - JSON download hash verification
- Customer framework path:
  - Import customer pack
  - Mark reviewed
  - Create mapping/package/report
  - Verify owner type, pack hash, review status, and no-claim flags
  - Reject the pack and verify new report generation is blocked with `framework_pack_rejected`
