# Agent Read-Only Governance Context MVP

Updated: 2026-06-14

Ticket: `KAN-98`

## Decision

KAN-98 adds a read-only context contract for optional Agent Governance before any MCP surface.

The product decision is deliberately conservative:

- GitGov remains manual-first.
- Manual-only tenants keep the same experience; agent principals are denied while Agent Governance is disabled.
- This is not a chatbot, not BYOM, not an autonomous agent feature, and not a deployment authorization path.
- The endpoint reads existing GitGov evidence and returns context; it does not create formal evaluations and does not authorize execution.

This gives customers that already use tools such as Codex, Claude Code, Cursor, Copilot, internal bots, or CI agents a safe way to let those tools inspect governance context without giving them the `agent_governance:evaluate` capability.

## API Shape

Admin users and read-scoped agent keys can call:

```text
GET /agent-governance/context
```

Supported query fields:

- `org_name`: required for global admin keys; optional for tenant-scoped keys.
- `repository_full_name`: required, in `owner/repo` form.
- `branch`: optional branch filter.
- `target_sha`: optional full 40 or 64 character hex commit SHA.
- `environment`: optional environment filter.

The response includes:

- `read_only=true`
- `will_authorize_execution=false`
- `mcp_surface=false`
- principal metadata, including agent key identity when used.
- `branch_status`
- `policy_compliance`
- `pipeline_state`
- `risk_score`
- `recent_activity`

The endpoint reads current GitGov evidence from existing tables: repositories, policies, client events, pipeline events, deployment gate authorizations, Agent Governance evaluations, and agent-key audit events.

## Auth And Scopes

`KAN-98` introduces `agent_governance:read` as a separate agent-key scope.

- `agent_governance:read` can call `GET /agent-governance/context`.
- `agent_governance:read` cannot call `POST /agent-governance/evaluate` or `POST /agent-governance/dry-run`.
- `agent_governance:evaluate` keeps its existing evaluate/dry-run permissions and cannot call the read context endpoint unless the key is explicitly created with both scopes.
- Unknown scopes are rejected when an Admin creates an agent key.
- Agent-key usage is audited with the requested scope.

## Manual-First Boundary

For human Admin principals, the endpoint is an administrative read of existing context.

For agent principals, the endpoint requires tenant Agent Governance to be enabled. If the tenant is disabled/manual-only, GitGov returns `403 agent_governance_disabled` with:

- `manual_governance_available=true`
- `read_only=true`
- `will_authorize_execution=false`

That makes the regulated/manual customer path explicit: a bank can leave Agent Governance disabled and still use all manual GitGov controls, including Policy-as-Code, Deployment Gates, approvals, release readiness, and evidence packets.

## Non-Scope

KAN-98 does not add:

- MCP server.
- Agent execution.
- Provider mutation.
- Policy changes by agents.
- Deployment Gate coupling to Agent Governance.
- Prompt, model, session transcript, or source-code storage.
- New database tables or migrations.
- A requirement that customers use agents.

## Validation

The backend integration coverage uses real PostgreSQL-backed Axum tests and seeds real governance evidence. The tests verify:

- A read-scoped agent key can load context from branch, policy, pipeline, deployment-gate, risk, and activity evidence.
- Read-only context does not persist an `agent_governance_evaluations` row.
- A key with only `agent_governance:evaluate` cannot call the context endpoint.
- A key with only `agent_governance:read` cannot call evaluate.
- Disabled/manual-only tenants deny agent-principal context reads while preserving manual governance.
- Agent-key use and invalid-scope attempts are audited.
