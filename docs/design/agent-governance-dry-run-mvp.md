# KAN-95 Agent Governance Dry-Run MVP

KAN-95 adds a dry-run preview for Agent Governance decisions.

The product goal is to let a human operator, integration, or explicitly scoped agent ask "what
would GitGov decide?" before an action is attempted. The dry-run is explanatory only. It does not
authorize execution and it does not create an `agent_governance_evaluations` row.

## Product Decision

GitGov remains manual-first.

Manual-only customers do not need Agent Governance, agent keys, or dry-run previews. Their normal
GitGov flows continue to work:

- Policy-as-Code.
- PR review.
- formal release approvals.
- Deployment Gates.
- evidence packets.
- audit export.

For customers that explicitly enable Agent Governance, dry-run gives a safe preflight surface for
agents and humans:

```text
POST /agent-governance/dry-run
```

It uses the same request body as:

```text
POST /agent-governance/evaluate
```

But it always returns:

```json
{
  "dry_run": true,
  "would_persist_evaluation": false,
  "would_authorize_execution": false
}
```

## API Behavior

Dry-run returns the deterministic policy decision, missing evidence, approval requirement, policy
checksum, sanitized request payload, principal identity, and shared governance decision preview.

It does not return an `evaluation_id`, because no evaluation evidence was persisted.

Agent-scoped keys can call dry-run only through the same narrow KAN-94 scope:

```text
agent_governance:evaluate
```

The scope name stays unchanged because the key is still asking GitGov for a policy evaluation, not
asking GitGov to execute anything.

## Disabled Tenant Behavior

When Agent Governance is disabled, dry-run returns:

```text
403 agent_governance_disabled
```

It creates no evaluation row. It writes an audit denial event so Admins can see that a dry-run
attempt occurred while the tenant was manual-only.

## Audit

KAN-95 writes:

- `agent_governance.dry_run_requested`
- `agent_governance.dry_run_denied`

For agent-scoped keys, KAN-94 key-use audit still records the agent key access attempt. Denied
actions such as disallowed `change_policy` continue to use `agent_key.denied`.

## Non-Goals

KAN-95 does not add:

- MCP tools.
- chatbot or BYOM behavior.
- autonomous execution.
- provider mutation.
- Deployment Gate behavior changes.
- new database tables.
- persisted dry-run history table.
- Desktop UI.

Dry-run is a backend/API slice that makes the next agent-facing surface safer without changing the
manual governance path.
