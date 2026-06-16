# KAN-126 Change Risk CAB Packet Manual Disposition MVP

Date: 2026-06-16

## Decision

KAN-126 records the human CAB disposition over an existing KAN-125 CAB packet.

The packet artifact remains immutable and hashable. The disposition is operational metadata on the packet record, not a release approval.

## Product Scope

Implemented:

- Manual CAB review status on `change_risk_cab_packets`.
- Allowed statuses: `pending_review`, `reviewed`, `accepted_risk`, `needs_mitigation`, `returned_to_owner`, and `rejected`.
- Reviewer identity, review timestamps, safe review notes, safe mitigation notes, safe decision reason, follow-up flag, and safe follow-up owner.
- `GET /change-risk/cab-packets/{packet_id}/review` for Admin/Auditor read.
- `PATCH /change-risk/cab-packets/{packet_id}/review` for Admin update.
- Admin audit actions for review view and update.
- Desktop CAB disposition panel in Governance > Releases.

Explicitly not implemented:

- Release approval.
- Release blocking.
- Deployment execution.
- Provider or repository mutation.
- Mutation of source Change Risk evaluations.
- Mutation of CAB packet artifact JSON or artifact hash.
- AI, LLM, BYOM, MCP, chatbot behavior, or Agent Governance dependency.
- Compliance score, certification, legal attestation, or regulatory claim.
- Email, Slack, public links, scheduler, PDF/DOCX, digital signatures, KMS, or multi-reviewer quorum.

## Validation Contract

KAN-126 must prove:

- A newly created CAB packet starts as `pending_review`.
- Auditor can read review state.
- Admin can update `reviewed`, `accepted_risk`, and `needs_mitigation`.
- `accepted_risk`, `returned_to_owner`, and `rejected` require a decision reason.
- `needs_mitigation` requires mitigation notes and `follow_up_required=true`.
- Secret-looking notes are rejected before persistence.
- Developer, Auditor update, foreign tenant, and Agent key access are denied as expected.
- CAB artifact hash remains unchanged across review updates.
- Source Change Risk evaluation status and trace hash remain unchanged.
- Deployment Gate authorization and Agent Governance evaluation counts remain unchanged.
- Audit rows are written for review viewed and review updated events.
