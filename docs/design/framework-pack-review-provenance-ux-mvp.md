# KAN-104 Framework Pack Review And Provenance UX

Ticket: `KAN-104`
Issue: GitHub `#365`

## Product Decision

KAN-104 promotes customer-owned framework packs from "imported artifact" to "reviewed tenant-ready artifact" through an explicit admin review gate.

Importing a JSON/YAML customer framework pack is not enough to use it for evidence mapping. A pack starts as `needs_review`; the backend blocks customer framework evidence mappings and new review packages until the pack is marked `reviewed`.

This remains a manual-first enterprise flow. It is not SOC 2, ISO, NIST, PCI, SBS, LGPD, an official regulatory mapping, a compliance score, a certification claim, OPA/Rego execution, Policy-as-Code enforcement, provider mutation, Action Center automation, MCP, chatbot, BYOM, or Agent Governance.

## Scope

- Supabase migration `v47` adds review provenance fields to `compliance_framework_packs`.
- Legacy KAN-103 statuses are migrated:
  - `customer_review_required` -> `needs_review`
  - `customer_reviewed` -> `reviewed`
- Supported statuses are `needs_review`, `reviewed`, `needs_changes`, `rejected`, and `archived`.
- New route: `PATCH /compliance/framework-packs/{framework_pack_id}/review`.
- Customer frameworks are hidden from `GET /compliance/control-frameworks` until their pack is `reviewed`.
- `POST /compliance/evidence-mappings` returns `409` for customer packs that are not reviewed.
- `POST /compliance/review-packages` re-checks current pack status so old mappings cannot produce new packages after a pack is rejected or archived.
- Review package artifacts include `review_status`, `reviewed_by_user_id`, `reviewed_at`, and safe review notes in the `framework` section.
- Governance Evidence Review UI adds a Framework Pack Review panel for review/provenance actions.

## Backend Blocking Codes

- `framework_pack_not_reviewed`
- `framework_pack_needs_changes`
- `framework_pack_rejected`
- `framework_pack_archived`
- `framework_pack_review_invariant_failed`

## Guardrails

- Review is Admin-only in this slice because the current product roles are `Admin`, `Architect`, `Developer`, and `PM`; there is no concrete `Auditor` role yet.
- Developer and agent-key principals cannot review packs.
- `reviewed` is refused if no-claim/provenance invariants are unsafe.
- Review notes are plain text only, length bounded, and reject script/secret-like content.
- Archived packs stay inspectable by id but are hidden from normal pack listing and blocked from mapping/package creation.

## Validation Focus

The KAN-104 test path intentionally exercises the real business boundary:

- Import customer pack and verify it starts `needs_review`.
- Confirm it is not available from the mapping framework list before review.
- Attempt mapping before review and require `409 framework_pack_not_reviewed`.
- Mark reviewed and confirm mapping succeeds.
- Create a review package and verify the artifact carries review provenance and no-claim flags.
- Flip the same pack through `needs_changes`, `rejected`, `archived`, and `needs_review`; each state blocks mapping with the expected code.
- Create a mapping while reviewed, then reject the pack and verify review package creation is blocked.
- Verify Developer and agent-key principals cannot review.
