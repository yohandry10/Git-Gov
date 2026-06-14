# Agent Read-Only Governance Context Report

Date: 2026-06-14

Ticket: `KAN-98`

GitHub issue: `#345`

Branch: `feature/KAN-98-agent-read-context`

## Product Decision

After KAN-97, the next safe Agent Governance slice is read-only context for agents, not MCP.

The in-app GPT product review thread was consulted for the post-KAN-97 decision, but the new prompt returned blank assistant responses twice. The final decision was made from the local roadmap and repo state:

- GitGov already has opt-in Agent Governance, deterministic evaluate/dry-run, agent-scoped keys, minimal attribution, key expiry, and rotation.
- The next enterprise-safe step is separating read context from evaluate permission.
- This prepares a future MCP/server-tool surface without shipping MCP yet.
- The feature must stay optional, manual-first, and safe for regulated customers that prohibit autonomous agents.

## Implemented

- Added `agent_governance:read` as an explicit agent-key scope.
- Extended agent-key creation to accept explicit scopes while preserving the current default of `agent_governance:evaluate`.
- Rejected unknown agent-key scopes at creation time.
- Added route authorization so:
  - `GET /agent-governance/context` requires Admin or an agent key with `agent_governance:read`.
  - `POST /agent-governance/evaluate` and `POST /agent-governance/dry-run` still require `agent_governance:evaluate`.
- Added `GET /agent-governance/context` for read-only branch, policy, pipeline, deployment-gate, risk, and recent-activity context.
- Returned explicit safety markers: `read_only=true`, `will_authorize_execution=false`, and `mcp_surface=false`.
- Denied agent-principal reads when tenant Agent Governance remains disabled/manual-only.
- Kept the implementation migration-free by reading existing governance evidence only.
- Added Postgres-backed integration tests for success, disabled tenant, invalid scope, and read-only key isolation.

## Validation

Local validation used a temporary PostgreSQL 16 container on `127.0.0.1:55441`.

Commands passed:

```powershell
cargo test --manifest-path gitgov\gitgov-server\Cargo.toml agent_governance_context -- --nocapture
cargo check --manifest-path gitgov\gitgov-server\Cargo.toml
cargo fmt --manifest-path gitgov\gitgov-server\Cargo.toml --check
cargo clippy --manifest-path gitgov\gitgov-server\Cargo.toml -- -D warnings
cargo test --manifest-path gitgov\gitgov-server\Cargo.toml
git diff --check
.\scripts\security\publication_guard.ps1
```

Observed result:

- Focused KAN-98 integration tests: `4` passed.
- Full backend suite: `296` passed.

## Remaining Before Production Closure

- Open PR for `KAN-98`.
- Wait for required GitHub checks.
- Merge.
- Verify Render deploy and production smoke with temporary read-scoped agent key.
- Revoke temporary production key and restore Agent Governance settings to disabled/manual-only.
