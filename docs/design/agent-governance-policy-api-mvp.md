# Agent Governance Policy API

KAN-90 starts roadmap block `0.2 Agentic Governance Layer`, but it does not make GitGov an
agent-first product.

Agents can ask GitGov whether a planned operation is allowed before they act. The control remains
deterministic: GitGov policy returns the decision; an LLM or coding agent can request, simulate, and
explain, but it does not decide the control.

## Product Posture

GitGov remains manual-first.

KAN-90 is optional infrastructure for customers that already use agents or want to pilot them under
governance. It is not:

- a chatbot feature
- a bring-your-own-model onboarding requirement
- a replacement for human approvals
- a requirement for regulated customers that prohibit autonomous agents
- a default path for banks or enterprises that want manual-only governance

The canonical governance path still works without this API:

- humans create commits and pull requests
- humans review and approve policy changes
- humans approve releases when policy requires it
- Deployment Gates and Policy-as-Code remain usable without agents
- audit evidence stays valid even when no agent ever calls this route

For agent-enabled customers, the API is a control point: an agent asks before acting, GitGov returns
the deterministic decision, and sensitive operations route back to human approval or existing GitGov
controls. For manual-only customers, the endpoint can simply remain unused.

KAN-92 adds the required control boundary around that primitive:

- Agent Governance is disabled by default per tenant.
- Only Admin users can enable or disable it.
- Disabled tenants get `403 agent_governance_disabled` and no evaluation record is created.
- The denied attempt is still audit-logged as `agent_governance.evaluation_denied`.
- Persisted request payload is minimized and redacts secret-like fields.
- Evaluation history is Admin-only.

## Routes

```text
POST /agent-governance/evaluate
GET /agent-governance/settings
PUT /agent-governance/settings
GET /agent-governance/evaluations
```

All routes are authenticated, org-scoped, and treated as sensitive governance routes by the auth
middleware.

`POST /agent-governance/evaluate` is not Admin-only because developer-scoped or future agent-scoped
keys must be able to ask before acting after a tenant has opted in. Global admin keys must pass
`org_name`.

`GET/PUT /agent-governance/settings` and `GET /agent-governance/evaluations` are Admin-only.

## Settings

Default settings are virtual until an Admin writes an explicit row:

```json
{
  "enabled": false,
  "mode": "manual_only",
  "payload_mode": "minimized",
  "reason": null,
  "updated_by": "system"
}
```

Admin opt-in request:

```json
{
  "org_name": "yohandry10",
  "enabled": true,
  "reason": "Approved controlled agent governance pilot"
}
```

When enabled, the stored mode is `opt_in_enabled`. When disabled, the stored mode is
`manual_only`. Payload mode is fixed to `minimized`.

## Request

```json
{
  "org_name": "yohandry10",
  "agent_id": "codex-agent-1",
  "agent_type": "codex",
  "actor": "engineer@example.com",
  "action": "push",
  "repository_full_name": "yohandry10/Git-Gov",
  "branch": "feature/KAN-90-agent-governance-policy-api",
  "target_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "environment": "production",
  "ticket_id": "KAN-90",
  "operation_id": "op-kan-90",
  "metadata": {}
}
```

Supported `action` values:

- `commit`
- `push`
- `open_pr`
- `merge_pr`
- `change_policy`
- `deploy`

## Response

Each evaluation is persisted with an `agv_...` id and returns:

- `decision`: `allowed`, `requires_approval`, or `blocked`.
- `allowed`: true only when the action can proceed immediately.
- `requires_approval`: true when the operation is valid but must be approved or routed through
  another GitGov control.
- `reason` and `reasons`.
- `required_evidence`.
- `policy_id`.
- `policy_checksum`.
- `evaluation`: deterministic policy summary with `llm_decision=false`.
- `request_payload`, minimized and secret-redacted.

If the tenant has not opted in, `POST /agent-governance/evaluate` returns:

```json
{
  "error": "Agent Governance is disabled for this organization",
  "code": "agent_governance_disabled",
  "enabled": false,
  "mode": "manual_only",
  "manual_governance_available": true,
  "next_step": "An Admin must explicitly enable Agent Governance before agent evaluations are accepted."
}
```

No `agent_governance_evaluations` row is created for that denied request.

## MVP Policy

The first policy is deliberately conservative:

- `commit`: allowed with ticket traceability; otherwise requires approval.
- `push`: blocked without branch or ticket; protected branches require approval.
- `open_pr`: allowed with ticket traceability; otherwise requires approval.
- `merge_pr`: blocked without ticket or branch; otherwise requires human approval and PR review.
- `change_policy`: blocked without ticket or operation id; otherwise requires a policy change
  request and human approval.
- `deploy`: blocked without ticket, branch, target SHA, environment, or operation id; otherwise
  requires Deployment Gates evidence and human approval when policy requires it.

Protected branch names in this MVP:

- `main`
- `master`
- `production`
- `prod`
- `release`

## Persistence

The append-only table is:

```text
agent_governance_evaluations
```

It stores the request, deterministic decision, policy checksum, required evidence, and metadata. The
handler also writes an admin audit log entry best-effort for operational traceability.

KAN-92 adds:

```text
agent_governance_settings
```

This table stores the tenant-level opt-in boundary, reason, updater, mode, and payload mode.

Persisted `request_payload` is intentionally not a raw request dump. It keeps governance context and
redacts secret-like keys such as `token`, `secret`, `password`, `credential`, `authorization`,
`api_key`, `apikey`, and `key`. It also truncates long strings and large arrays.

## Non-goals

- No LLM-decided permissions.
- No MCP server yet.
- No agent-specific token scopes yet.
- No provider mutation.
- No branch protection or repository configuration changes.
- No Desktop UI in this slice; the control is exposed by Admin API.
