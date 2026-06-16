# KAN-123 Change Risk Manual Review & Mitigation Notes MVP

KAN-123 extends the KAN-121/KAN-122 Change Risk Advisory with a human review marker. The product decision is manual-first: GitGov records that a person reviewed an explained risk and captured safe mitigation notes, without turning the review into deployment enforcement.

## Problem

KAN-121 creates deterministic advisory risk evidence. KAN-122 explains the rule trace. A regulated release process still needs a human record showing how the risk was reviewed and what mitigation or accepted-risk rationale was documented.

KAN-123 answers:

- Has a human reviewed this Change Risk evaluation?
- What manual status did they set?
- What safe review, mitigation, or decision notes were recorded?
- Who updated the review and when?
- Did the review preserve the original risk level, ruleset version, trace hash, and advisory-only posture?

## Implemented Scope

- Review metadata on `change_risk_evaluations`:
  - `review_status`.
  - `reviewed_by_user_id`.
  - `reviewed_at`.
  - `review_notes_safe`.
  - `mitigation_notes_safe`.
  - `decision_reason_safe`.
  - `review_updated_at`.
- Review statuses:
  - `needs_review`.
  - `reviewed`.
  - `accepted_risk`.
  - `needs_mitigation`.
  - `rejected`.
- Backend routes:
  - `GET /change-risk/evaluations/{evaluation_id}/review`.
  - `PATCH /change-risk/evaluations/{evaluation_id}/review`.
- Admin audit action: `change_risk_review_updated`.
- Desktop/Tauri/store support for a `Manual Review` panel in the Change Risk detail.

## Access Model

- Admin can update review.
- Admin and Auditor can read review.
- Developer and Agent Governance keys are denied.
- Tenant isolation follows the existing Change Risk org scope rules.
- Platform/global keys must provide explicit tenant scope and cannot accidentally operate a tenant workspace.

## Non-Goals

KAN-123 does not add enforcement, release blocking, deploy execution, provider mutation, repository mutation, AI/LLM, BYOM, MCP, chatbot behavior, Agent Governance dependency, compliance score, certification, legal attestation, regulatory claim, notifications, approval quorum, or multi-reviewer workflow.

## Verification Requirements

- New medium/high evaluations default to `needs_review`.
- Admin can update to `reviewed`, `accepted_risk`, `needs_mitigation`, or `rejected` with safe notes.
- `accepted_risk` and `rejected` require a decision reason.
- `needs_mitigation` requires mitigation notes.
- Secret-like notes are rejected and not stored.
- Review updates do not change `risk_level`, `ruleset_version`, `trace_hash`, `evaluation_trace`, no-claim flags, Deployment Gate records, provider state, repository state, or Agent Governance evaluation counts.
- Auditor can read but cannot update in this MVP.
- Developer and Agent Governance keys cannot update.
- Tenant A cannot read or update tenant B review records.
