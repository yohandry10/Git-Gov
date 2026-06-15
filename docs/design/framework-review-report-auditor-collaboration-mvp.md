# Framework Review Report Auditor Collaboration MVP

Updated: 2026-06-15
Ticket: `KAN-109`

## Product Decision

After `KAN-108`, the next enterprise-safe slice is not signed manifests, PDF/DOCX export, official
regulatory mapping, BYOM, MCP, chatbot behavior, or a broader Agent Governance feature. The next gap
is reviewer collaboration over existing Framework Review Reports.

`KAN-109` adds granular assignment and safe comments for reports that already exist. Admins can
assign a report to active tenant `Auditor` principals. Assigned Auditors can find reports assigned to
them, add comments, and update manual review metadata. Unassigned same-tenant Auditors are blocked
from collaboration surfaces once a report has active assignments.

## Scope

- `PUT /compliance/framework-review-reports/{report_id}/assignments` lets Admins replace active
  Auditor assignments for one report.
- `GET /compliance/framework-review-reports/{report_id}/assignments` lists assignment metadata for
  Admins or assigned Auditors.
- `GET /compliance/framework-review-reports/assigned-to-me` lists metadata-only reports assigned to
  the authenticated principal, with the same framework/mapping/package filters as report history.
- `POST /compliance/framework-review-reports/{report_id}/comments` creates safe reviewer comments
  with an optional `needs_review`, `reviewed`, `needs_changes`, or `rejected` suggestion.
- `GET /compliance/framework-review-reports/{report_id}/comments` lists safe comments.
- Desktop Governance Evidence Review exposes assignment, assigned-to-me, and comments controls.

## Guardrails

- Only active tenant Auditors can be assigned.
- Assignment notes and comments are plain text, length-limited, and reject common HTML/script and
  secret-like token patterns.
- Assignments and comments do not mutate `payload_json_redacted`, `artifact_hash`, source hashes,
  no-claim flags, policy, Deployment Gates, or Agent Governance state.
- `compliance_claim=false`, `regulatory_claim=false`, `certification=false`, and
  `requires_auditor_review=true` remain unchanged.
- When a report has no active assignments, KAN-108 read/review compatibility remains. Once active
  assignments exist, unassigned Auditors cannot comment, list comments/assignments, or update review
  metadata for that report.

## Non-Goals

- No official SOC 2, ISO, NIST, PCI, SBS, LGPD, or other regulatory mapping.
- No certification, compliance score, badge, legal attestation, or auditor-signature claim.
- No PDF/DOCX export.
- No OPA/Rego execution.
- No Agent Governance dependency and no agent authorization behavior.
- No provider mutation, policy mutation, notifications, due dates, or external reviewer workflow.

