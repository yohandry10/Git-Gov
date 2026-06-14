# KAN-92 Agent Governance Control Boundary Report

Date: 2026-06-14

## Scope

KAN-92 hardens the Agent Governance primitive delivered in KAN-90.

Implemented:

- tenant-level `agent_governance_settings`.
- disabled-by-default behavior.
- Admin-only settings API.
- Admin-only evaluation history API.
- `403 agent_governance_disabled` when a tenant has not opted in.
- denied-attempt audit logging.
- opt-in and opt-out audit logging.
- minimized and secret-redacted persisted request payload.
- focused integration coverage against real PostgreSQL.

## Product Behavior

Manual-first remains the default.

Tenants that do not want agents do nothing. `POST /agent-governance/evaluate` rejects requests and
does not create `agent_governance_evaluations` evidence.

Tenants that explicitly opt in can allow developer-scoped or future agent-scoped keys to ask GitGov
for deterministic governance decisions before acting.

## API Surface

New routes:

```text
GET /agent-governance/settings
PUT /agent-governance/settings
GET /agent-governance/evaluations
```

Updated behavior:

```text
POST /agent-governance/evaluate
```

This route now checks tenant settings before evaluating or persisting a decision.

## Migration

New migration:

```text
gitgov/gitgov-server/supabase/supabase_schema_v39.sql
gitgov/gitgov-server/supabase/checks/v39_postcheck.sql
```

Base schema was also updated for fresh environments.

## Validation Plan

Local validation must prove:

- default disabled returns `403 agent_governance_disabled`.
- disabled requests create no evaluation rows.
- disabled requests create a denial audit event.
- Developer keys cannot read/update settings.
- Developer keys cannot read evaluation history.
- Admin keys can read default settings.
- Admin keys can enable Agent Governance with a reason.
- enabled tenants can evaluate with scoped non-Admin keys.
- persisted response and history redact secret-like metadata.
- migration and postcheck pass against real PostgreSQL.
- full backend tests pass against real PostgreSQL.

Production validation must leave the customer tenant manual-only after smoke:

- `/health` is ok.
- authenticated `/stats` returns 200.
- anonymous settings read returns 401.
- authenticated default settings show manual-only.
- disabled evaluation returns 403 without persisted evaluation.
- temporary opt-in allows one real evaluation and redacts secret-like metadata.
- settings are restored to disabled/manual-only.

## Status

Local implementation validation passed on branch
`feature/KAN-92-agent-governance-control-boundary`.

Validated locally:

- `cargo fmt --manifest-path gitgov\gitgov-server\Cargo.toml --check`
- `cargo check --manifest-path gitgov\gitgov-server\Cargo.toml`
- `cargo clippy --manifest-path gitgov\gitgov-server\Cargo.toml -- -D warnings`
- fresh PostgreSQL 16 container on `127.0.0.1:55435`
- base schema plus `supabase_schema_v39.sql`
- `v39_postcheck.sql` with `PASS` for table, constraints, and index
- focused `agent_governance` tests: `10` passed
- full backend tests: `275` passed
