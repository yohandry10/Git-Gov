# KAN-90 Agent Governance Policy API MVP

KAN-90 starts roadmap block `0.2 Agentic Governance Layer`.

Agents can ask GitGov whether a planned operation is allowed before they act. The control remains
deterministic: GitGov policy returns the decision; an LLM or coding agent can request, simulate, and
explain, but it does not decide the control.

## Route

```text
POST /agent-governance/evaluate
```

The route is authenticated, org-scoped, and treated as a sensitive governance route by the auth
middleware. It is not Admin-only in this MVP because developer-scoped or future agent-scoped keys
must be able to ask before acting. Global admin keys must pass `org_name`.

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
- `request_payload`.

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

## Non-goals

- No LLM-decided permissions.
- No MCP server yet.
- No agent-specific token scopes yet.
- No provider mutation.
- No branch protection or repository configuration changes.
- No UI in this slice.
