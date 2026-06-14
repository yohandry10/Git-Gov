# KAN-92 Agent Governance Control Boundary

KAN-92 makes the KAN-90 Agent Governance API safe for manual-first enterprise customers.

The product rule is simple: Agent Governance exists as an optional control primitive, but it is not
available to a tenant until an Admin explicitly enables it.

## Product Decision

GitGov remains manual-first.

Banks and regulated companies that do not permit autonomous agents can leave Agent Governance off.
Their normal GitGov flows still work:

- Policy-as-Code.
- PR review.
- formal release approval.
- Deployment Gates.
- audit export and evidence packets.

For customers that do use agents, GitGov now has a clear control boundary:

- tenant-level default is disabled.
- Admin opt-in is required.
- opt-in and opt-out are audited.
- denied evaluation attempts are audited without creating approval evidence.
- persisted request payload is minimized and redacted.
- evaluation history is visible only to Admin users.

## API Contract

Settings:

```text
GET /agent-governance/settings
PUT /agent-governance/settings
```

Both routes are Admin-only and org-scoped.

Evaluation:

```text
POST /agent-governance/evaluate
```

This route remains available to scoped non-Admin keys only after the tenant is enabled, because an
agent or developer key must be able to ask before acting. When disabled, it returns
`403 agent_governance_disabled` and creates no evaluation record.

History:

```text
GET /agent-governance/evaluations
```

This route is Admin-only and supports filters by evaluation id, repository, action, decision, and
agent id.

## Database

New table:

```text
agent_governance_settings
```

Important fields:

- `enabled`: false by default.
- `mode`: `manual_only` or `opt_in_enabled`.
- `payload_mode`: fixed to `minimized`.
- `reason`: Admin-provided business reason.
- `updated_by`: authenticated client id.

The existing append-only table remains:

```text
agent_governance_evaluations
```

KAN-92 changes the write path so an evaluation is persisted only when the tenant has opted in.

## Audit Events

KAN-92 writes these audit actions:

- `agent_governance.enabled`
- `agent_governance.disabled`
- `agent_governance.evaluation_denied`
- `agent_governance.evaluation_requested`

The denied event is important: it proves GitGov saw and rejected an agent-governance attempt while
the tenant was manual-only.

## Payload Minimization

`request_payload` is no longer treated as a raw request archive.

GitGov persists the governance context needed for audit and redacts secret-like metadata keys:

- `token`
- `secret`
- `password`
- `credential`
- `authorization`
- `api_key`
- `apikey`
- `key`

Long strings and arrays are bounded so agent metadata cannot become an unreviewed data dump.

## Out Of Scope

KAN-92 does not add:

- Desktop settings UI.
- MCP tools.
- OPA/Rego execution.
- provider mutation.
- agent-scoped token type beyond the existing scoped API-key model.
- autonomous approval.

Those are future slices. The mandatory boundary is now in the backend API and database.
