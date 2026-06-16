# KAN-121 Change Risk Assessment Advisory MVP

Updated: 2026-06-16

## Decision

Implement Change Risk as a deterministic advisory surface, not as an automatic deployment decision,
AI result, compliance score, certification, or legal/regulatory attestation.

This slice exists because change advisory boards and release managers already review this kind of
risk manually. GitGov should make the review auditable and repeatable using evidence it already
stores: Deployment Gates, release governance, release approvals, evidence packets, and first
governed repo setup state.

## Scope

KAN-121 adds:

- `change_risk_evaluations`, an append-only advisory evidence table.
- Backend routes:
  - `POST /change-risk/evaluations`.
  - `GET /change-risk/evaluations`.
  - `GET /change-risk/evaluations/{evaluation_id}`.
- Deterministic risk levels: `low`, `medium`, `high`, and `unknown`.
- Risk reasons, missing evidence, blocking gaps, and recommended manual actions.
- Admin-only creation/list/read for the MVP, tenant scoped through the same org resolution model as
  Deployment Gates and release governance.
- Desktop/Tauri client methods, commands, store state/actions, and a `ChangeRiskPanel` under
  Governance > Releases.

## Inputs

The evaluator accepts release/change context:

- `repository_full_name`.
- `branch`.
- `environment`.
- optional `change_id`.
- optional `deployment_gate_id`.
- optional `release_id`.
- optional `commit_sha`.
- optional `evidence_packet_hash`.
- optional `evidence_refs`.

When `deployment_gate_id` is present, the backend loads the persisted Deployment Gate authorization
inside the same tenant and derives risk from its decision, blocking state, warnings, break-glass
state, approval counts, and evidence gaps.

## Output Contract

Every persisted record keeps:

- `risk_level`.
- `risk_reasons`.
- `missing_evidence`.
- `blocking_gaps`.
- `recommended_manual_actions`.
- `advisory_only=true`.
- `llm_used=false`.
- `agent_governance_used=false`.
- `compliance_claim=false`.
- `certification=false`.

Database constraints enforce the advisory/no-AI/no-agent/no-claim flags, so they are not only UI
copy.

## Explicit Non-Goals

KAN-121 does not:

- approve a deployment.
- block a deployment.
- execute or trigger deployment.
- mutate provider state.
- mutate a customer repository.
- call Agent Governance.
- create Agent Governance evaluation rows.
- use an LLM.
- compute or display a compliance score.
- certify a release or create a regulatory/legal claim.
- replace manual CAB/release-manager judgment.

## Product Rationale

This keeps GitGov useful to conservative enterprise buyers:

- Teams that want manual release control get a repeatable advisory record.
- Teams with Deployment Gates get risk context next to the gate history.
- Agent-enabled customers can still opt into separate Agent Governance later, but KAN-121 does not
  require it.
- The same evidence can later feed richer dashboards or customer-selected blocking policies without
  changing the MVP into hidden enforcement.

## Validation Strategy

The implementation is validated with:

- real Postgres integration tests covering complete gate context, blocked gates, break-glass,
  missing evidence, tenant isolation, Developer denial, Agent Governance key denial, and no Agent
  Governance row mutation.
- migration/postcheck validation in a real transaction with rollback.
- Tauri compile, clippy, and tests.
- frontend typecheck, lint, focused store tests, full Vitest suite, and production build.

