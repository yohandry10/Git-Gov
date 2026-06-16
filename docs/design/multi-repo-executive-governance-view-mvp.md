# KAN-129 Multi-Repo Executive Governance View MVP

## Product Decision

KAN-129 adds a read-only executive view across repositories in one tenant.

The view answers: which repositories need executive attention based on existing Deployment Gate,
Change Risk, CAB packet, and CAB decision manifest evidence?

This is deliberately an overview surface. It does not approve deployments, block releases, execute
deploys, create CAB evidence, change risk, mutate providers, mutate customer repositories, or create
regulatory/compliance/certification claims.

## Scope

- Backend route: `GET /executive/repositories`.
- Query parameters: `org_name`, `limit`, and `offset`.
- Response includes repository summaries, tenant-scoped totals, posture, gate counts, risk counts,
  CAB packet/manifest counts, latest evidence pointers, and explicit no-claim flags.
- Tauri DTO/client/command and Control Plane store action.
- Desktop `Governance > Releases` executive repository panel.

## Relationship Model

KAN-129 does not require a new table.

The view is derived from existing tenant-scoped records:

- `deployment_gate_authorizations.repository_full_name`
- `change_risk_evaluations.repository_full_name`
- `change_risk_cab_packets.evaluation_ids_json` joined to `change_risk_evaluations`
- `change_risk_cab_decision_manifests.cab_packet_id`

## Posture Semantics

- `attention`: at least one blocked Deployment Gate or high-risk Change Risk evaluation.
- `review`: manual review signal exists, such as `needs_review`, revoked manifest, or advisory gate.
- `healthy`: governance evidence exists and no attention/review signal was found.
- `unknown`: no governance evidence exists in the current result set.

These are executive triage states only. They are not deployment authorization decisions.

## Non-Goals

- No enforcement.
- No release blocking changes.
- No deployment execution.
- No provider or repository mutation.
- No risk recalculation.
- No CAB packet or manifest creation.
- No AI, LLM, BYOM, MCP, or chatbot dependency.
- No Agent Governance dependency.
- No compliance score, certification, legal attestation, or official regulatory claim.
- No Action Center write, notification, public link, scheduler, PDF, or DOCX.

## Access

- Admin and Auditor can read the view through Compliance Reviewer access.
- Developer and Agent keys are denied.
- Tenant isolation is enforced by resolving `org_name`/auth scope before aggregation.
- Other-tenant repositories are excluded from the result set.

## Validation Requirements

The focused real Postgres test must create real tenant data and verify:

1. Two repositories inside one tenant and one repository in another tenant.
2. Deployment Gate authorizations across repositories.
3. Change Risk evaluations linked to those gates.
4. CAB packet and CAB decision manifest evidence for one repository.
5. Admin/Auditor read access and Developer denial.
6. Tenant isolation for another org key.
7. Correct posture, totals, latest evidence pointers, and hashes.
8. No source mutation: Deployment Gate and Agent Governance counts remain unchanged.
9. Safe no-claim flags remain explicit in the response.
