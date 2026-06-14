# KAN-93 Shared Governance Decision Model MVP

Date: 2026-06-14

## Product Decision

KAN-93 introduces a neutral governance decision contract shared by Deployment Gates and Agent
Governance.

The decision is deliberately not an Agent Governance dependency. Deployment Gates remain
manual-first and continue to work when Agent Governance is disabled or unused.

## Why This Exists

KAN-90 and KAN-92 added an optional Agent Governance API. Deployment Gates already existed as the
CI/CD-facing manual-first deploy control. Without a shared model, those two surfaces would drift
into parallel decision shapes:

- Deployment Gates would have release-governance decisions.
- Agent Governance would have agent-evaluation decisions.
- Future Action Center, audit, approval, and reporting surfaces would need to understand both
  independently.

KAN-93 avoids that drift by creating one deterministic governance decision shape that each consumer
can emit.

## Contract

Current contract version:

```text
shared-governance-decision.v1
```

Deployment Gate records expose the shared decision at:

```text
governance_decision
details.shared_governance_decision
```

Agent Governance evaluations expose the shared decision at:

```text
evaluation.shared_governance_decision
```

Common fields include:

- `contract_version`
- `consumer_type`
- `actor_type`
- `action`
- `decision`
- `legacy_decision`
- `approved`
- `manual_approval_required`
- `agent_governance_used`
- `policy`
- `operation`
- `evidence`
- `reason_codes`
- `reasons`
- `action_center_items`

## Deployment Gate Behavior

Deployment Gates emit:

```json
{
  "consumer_type": "deployment_gate",
  "actor_type": "system",
  "action": "deploy",
  "agent_governance_used": false
}
```

This is true even if a tenant has Agent Governance configured elsewhere. The Deployment Gate does
not call `POST /agent-governance/evaluate`, does not require agent settings, and does not create
`agent_governance_evaluations` rows.

Decision mapping:

- `allowed`: legacy decision is `approved` or `break_glass`.
- `requires_approval`: release governance needs more human release approvals.
- `blocked`: release governance is actively blocking and no valid break-glass authorization applies.
- `insufficient_evidence`: advisory/record-only result lacks setup, environment applicability, or
  required evidence but is not blocking under the customer's selected policy.

The legacy fields remain stable for CI/CD consumers:

- `decision`
- `approved`
- `blocking`
- `would_block`
- `reason`
- `blocked_by`
- `warnings`

## Agent Governance Behavior

Agent Governance emits:

```json
{
  "consumer_type": "agent_governance",
  "actor_type": "agent",
  "agent_governance_used": true
}
```

This supplements the KAN-90 evaluation response. It does not make LLMs or agents the authority for
critical controls. The existing deterministic policy still decides; `llm_decision=false` remains the
Agent Governance policy behavior.

## Non-Goals

KAN-93 does not add:

- agent-scoped API keys.
- a new database table.
- a new `/governance-decisions` API.
- automatic provider mutation.
- OPA/Rego execution.
- autonomous agent approval.
- a requirement that customers use Agent Governance.

Agent-scoped credentials remain a separate future slice, likely KAN-94.

## Enterprise Guardrails

Manual-first remains canonical:

- banks and regulated customers can keep Agent Governance disabled.
- Deployment Gates continue to authorize deploys from CI/CD without any agent feature.
- shared decision audit makes the manual path stronger, not dependent on agents.

The shared model exists so future Action Center, approval routing, audit packets, and reporting can
reason over one decision contract regardless of whether the caller is CI/CD, a human approval
surface, or an optional agent.
