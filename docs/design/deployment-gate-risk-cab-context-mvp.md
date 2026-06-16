# KAN-128 Deployment Gate Risk & CAB Evidence Context MVP

## Product Decision

KAN-128 returns the completed Change Risk/CAB chain to the Deployment Gates workflow as read-only
manual context.

Operators can open a Deployment Gate authorization and see the related Change Risk evaluations, CAB
packets, and CAB decision manifests without changing the gate decision, executing a deploy, or making
any compliance claim.

## Scope

- Backend route: `GET /deployment-gates/{deployment_gate_id}/risk-context`.
- Response includes the Deployment Gate authorization, related Change Risk evaluations, related CAB
  packets, related CAB decision manifests, latest risk/review status, triggered rule count, and
  explicit no-claim flags.
- Tauri client/command and Control Plane store action.
- Desktop `Deployment Gate History` row-level `Risk & CAB Context` section.

## Relationship Model

KAN-128 does not need a new table.

The context is derived from existing tenant-scoped records:

- `deployment_gate_authorizations.authorization_id`
- `change_risk_evaluations.deployment_gate_id`
- `change_risk_cab_packets.evaluation_ids_json`
- `change_risk_cab_packets.filters_json.deployment_gate_ids`
- `change_risk_cab_decision_manifests.cab_packet_id`

## Non-Goals

- No enforcement.
- No release blocking changes.
- No deployment execution.
- No provider or repository mutation.
- No risk recalculation.
- No automatic CAB packet or manifest creation.
- No AI, LLM, BYOM, MCP, or chatbot dependency.
- No Agent Governance dependency.
- No compliance score, certification, legal attestation, or official regulatory claim.
- No Action Center write, email, public link, scheduler, PDF, or DOCX.

## Access

- Admin and Auditor can read the context through Compliance Reviewer access.
- Developer and Agent keys are denied.
- Tenant isolation is enforced by resolving `org_name`/auth scope before reading the gate and related
  evidence.
- The endpoint returns `404` when the requested Deployment Gate authorization is not in the resolved
  tenant.

## Validation Requirements

The focused real Postgres test must build the full chain:

1. Create a Deployment Gate authorization.
2. Create Change Risk evaluations linked to that gate.
3. Apply manual review.
4. Create a CAB packet from the evaluation.
5. Apply CAB disposition.
6. Create a CAB decision manifest.
7. Read `GET /deployment-gates/{deployment_gate_id}/risk-context`.
8. Verify IDs, hashes, risk/review state, no-claim flags, RBAC, tenant isolation, Agent key denial,
   and no mutation of the gate/evaluation/packet/manifest source records.

