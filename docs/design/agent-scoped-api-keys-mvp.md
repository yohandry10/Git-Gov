# KAN-94 Agent-Scoped API Keys MVP

KAN-94 adds optional agent-scoped API keys for tenants that explicitly enable Agent Governance.

The decision remains manual-first: banks and regulated customers can keep Agent Governance disabled
and continue using Policy-as-Code, PR review, formal approvals, Deployment Gates, audit exports, and
evidence packets without any agent credential.

## Product Decision

Agent-scoped keys are not normal GitGov API keys.

They are purpose-built credentials for one operation:

```text
POST /agent-governance/evaluate
```

They cannot administer tenants, change policy, list audit data, create deployments, approve
break-glass, mutate repositories, or call Deployment Gates. A tenant Admin must still enable Agent
Governance before any agent-scoped key can create an evaluation.

## API Contract

Admin key management:

```text
GET /agent-governance/agent-keys
POST /agent-governance/agent-keys
POST /agent-governance/agent-keys/{key_id}/rotate
DELETE /agent-governance/agent-keys/{key_id}
```

These routes are Admin-only and org-scoped.

Agent evaluation:

```text
POST /agent-governance/evaluate
Authorization: Bearer ggag_...
```

Agent tokens are accepted only for this route and only with scope:

```text
agent_governance:evaluate
```

The created token is returned once at creation or rotation time. List/revoke responses expose key
metadata, prefix, and last four characters, but never plaintext token material.

## Scope Model

The first allowed action set defaults to:

- `commit`
- `push`
- `open_pr`
- `merge_pr`
- `deploy`

`change_policy` is deliberately not included by default. An agent key can only request an action
that is present in its `allowed_actions` list. Unsupported actions, revoked keys, expired keys,
tenant mismatch, disabled tenants, and invalid scopes do not create evaluation rows.

## Lifecycle

KAN-97 extends KAN-94 with operational credential lifecycle.

New keys default to a 90-day expiry. Admins can explicitly request `no_expiry=true`, which keeps
the key active but exposes `status=no_expiry` as an operational risk signal. `expires_at` and
`no_expiry=true` cannot be combined.

Agent key records expose derived lifecycle status:

- `active`
- `expiring_soon`
- `expired`
- `revoked`
- `rotation_pending`
- `no_expiry`

Rotation creates a replacement key, returns the new plaintext token once, links the old key to the
replacement, and sets the old key's effective expiry to the configured grace period. Revocation
wins over expiry if both apply.

## Database

New table:

```text
agent_governance_agent_keys
```

Important fields:

- `key_id`
- `org_id`
- `token_hash`
- `token_prefix`
- `token_last4`
- `display_name`
- `description`
- `environment`
- `scopes`
- `allowed_actions`
- `expires_at`
- `last_used_at`
- `revoked_at`
- `rotated_at`
- `rotated_from_key_id`
- `replaced_by_key_id`
- `rotation_reason`
- `created_by`
- `revoked_by`

`agent_governance_evaluations` now also records:

- `principal_type`
- `agent_key_id`
- `agent_display_name`

This lets Admin history distinguish human/API-key evaluations from agent-scoped evaluations.

## Audit Events

KAN-94 writes:

- `agent_key.created`
- `agent_key.used`
- `agent_key.rotated`
- `agent_key.revoked`
- `agent_key.denied`
- `agent_key.denied_expired`
- `agent_key.denied_revoked`
- `agent_key.invalid_scope`

Denied and invalid-scope events intentionally do not create evaluation evidence.

## Non-Goals

KAN-94 does not add:

- MCP server tools.
- chatbot or BYOM behavior.
- OAuth/IAM replacement.
- provider mutation.
- autonomous execution.
- Deployment Gate dependency on Agent Governance.
- default enablement for any tenant.
- Desktop UI for key management.

The backend API and database are prepared first because external agents and CI adapters need a
stable, narrow, auditable credential boundary before any richer agent surface is safe.
