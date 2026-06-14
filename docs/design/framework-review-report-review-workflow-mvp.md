# KAN-107 Framework Review Report Review Workflow

Ticket: `KAN-107`
Issue: GitHub `#374`

## Product Decision

After KAN-106, GitGov can create, list, inspect, and download Framework Review Reports. The next product gap is manual review state: a customer operator needs to mark whether a report was reviewed, needs changes, or was rejected before any future official regulatory mapping work.

This slice is Admin-only because the tenant RBAC model currently has `Admin`, `Architect`, `Developer`, and `PM`; it does not yet have a tenant-level `Auditor` role. Adding that role is a separate product/RBAC change. KAN-107 keeps the workflow manual-first and does not create certification, compliance scores, official SOC 2/ISO/NIST/PCI/SBS/LGPD mapping claims, signed manifests, PDF/DOCX output, BYOM, MCP, chatbot behavior, OPA/Rego execution, provider mutation, policy mutation, or Agent Governance dependency.

## Scope

- Add Supabase migration/postcheck `v50`.
- Add review metadata to `compliance_framework_review_reports`:
  - `review_status`: `needs_review`, `reviewed`, `needs_changes`, `rejected`
  - `reviewed_by_user_id`
  - `reviewed_at`
  - `review_notes_safe`
- Add Admin-only route:
  - `PATCH /compliance/framework-review-reports/{report_id}/review`
- Keep `artifact_hash`, source hashes, and `payload_json_redacted` unchanged.
- Add admin audit log action `compliance_framework_review_report.reviewed`.
- Add Tauri command/client/model support.
- Add Desktop Governance Evidence Review controls to save and display report review metadata.

## Guardrails

- Review notes are plain text, length-limited, and reject common secret/token/HTML patterns.
- `needs_changes` and `rejected` require a safe note.
- Tenant isolation is enforced by `org_id` scope.
- Developer/agent keys cannot review reports.
- No-claim flags remain enforced:
  - `compliance_claim=false`
  - `regulatory_claim=false`
  - `certification=false`
  - `requires_auditor_review=true`

## Validation Focus

- Generate a real KAN-99 -> KAN-100 -> KAN-101 -> KAN-105 report chain.
- Mark the report `needs_changes` with a safe note.
- Verify reviewer, timestamp, status, notes, and audit log row.
- Verify invalid status, missing note for `rejected`, and secret-like notes return `400`.
- Verify Developer receives `403`.
- Verify another tenant receives `404`.
- Verify list metadata reflects updated review status.
- Verify downloaded artifact hash is unchanged.
- Verify Agent Governance evaluations do not change.

