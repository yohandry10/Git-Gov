# KAN-97 Agent Key Expiry And Rotation UX MVP

KAN-97 hardens the optional Agent Governance credential lifecycle before any broader agentic
surface such as MCP.

GitGov remains manual-first. Banks and regulated customers can keep Agent Governance disabled and
continue using Policy-as-Code, human PR review, formal release approvals, Deployment Gates, audit
exports, and Evidence Packets without creating agent keys.

## Product Decision

After KAN-96, the next risk is not missing agent capability. It is operating agent credentials
safely.

KAN-97 adds:

- default 90-day expiry for newly created agent keys.
- explicit `no_expiry=true` for admins that deliberately accept a permanent key.
- derived lifecycle status in list/create/rotate responses.
- `POST /agent-governance/agent-keys/{key_id}/rotate`.
- old/new key linkage through `rotated_from_key_id` and `replaced_by_key_id`.
- grace-period rotation without scheduler dependency.
- specific expired/revoked auth audit events.

## API Contract

Admin key management:

```text
GET /agent-governance/agent-keys
POST /agent-governance/agent-keys
POST /agent-governance/agent-keys/{key_id}/rotate
DELETE /agent-governance/agent-keys/{key_id}
```

`POST /agent-governance/agent-keys` accepts:

```json
{
  "display_name": "codex-staging-agent",
  "description": "Optional staging automation",
  "environment": "staging",
  "allowed_actions": ["commit", "push", "open_pr", "deploy"],
  "expires_at": 1797206400000
}
```

If `expires_at` is omitted and `no_expiry` is not true, GitGov assigns a 90-day expiry. If an admin
really wants no expiry, the request must say so:

```json
{
  "display_name": "codex-staging-agent",
  "allowed_actions": ["commit"],
  "no_expiry": true
}
```

`expires_at` and `no_expiry=true` cannot be combined.

`POST /agent-governance/agent-keys/{key_id}/rotate` accepts:

```json
{
  "reason": "quarterly_rotation",
  "grace_period_hours": 24
}
```

The response returns:

- `replacement`: the new key metadata.
- `replaced`: the old key metadata after the grace expiry/linkage update.
- `token`: the replacement plaintext token, returned once.

The replaced key remains usable until its effective `expires_at` when grace is nonzero, unless it is
revoked first. Revocation always wins over expiry.

## Status Model

Agent key records expose derived status:

- `active`
- `expiring_soon`
- `expired`
- `revoked`
- `rotation_pending`
- `no_expiry`

This is derived from stored timestamps and linkage fields instead of being persisted as a second
source of truth.

## Database

`agent_governance_agent_keys` now includes:

- `rotated_at`
- `rotated_from_key_id`
- `replaced_by_key_id`
- `rotation_reason`

Indexes support rotation lookup and active expiry scans:

- `idx_agent_governance_agent_keys_rotation_from`
- `idx_agent_governance_agent_keys_replaced_by`
- `idx_agent_governance_agent_keys_expiry`

## Audit

KAN-97 uses safe metadata only. Plaintext tokens are never persisted, listed, or audited.

Events:

- `agent_key.created`
- `agent_key.used`
- `agent_key.rotated`
- `agent_key.revoked`
- `agent_key.denied_expired`
- `agent_key.denied_revoked`
- `agent_key.invalid_scope`

Expired/revoked keys do not create `agent_governance_evaluations` rows.

## Non-Goals

KAN-97 does not add:

- MCP server.
- chatbot or BYOM behavior.
- autonomous execution.
- GitHub/Jenkins/deploy provider mutation.
- Deployment Gate dependency on Agent Governance.
- OAuth, mTLS, SSO workload identity, or a new IAM system.
- per-repo/per-environment agent scopes.
- agent session graph.
- prompt, diff, source code, or raw tool trace storage.
- Action Center writes.

MCP can be evaluated later only after the credential lifecycle is safe and auditable.
