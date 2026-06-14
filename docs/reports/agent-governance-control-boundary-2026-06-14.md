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

Implemented and production-validated.

Validated locally:

- `cargo fmt --manifest-path gitgov\gitgov-server\Cargo.toml --check`
- `cargo check --manifest-path gitgov\gitgov-server\Cargo.toml`
- `cargo clippy --manifest-path gitgov\gitgov-server\Cargo.toml -- -D warnings`
- fresh PostgreSQL 16 container on `127.0.0.1:55435`
- base schema plus `supabase_schema_v39.sql`
- `v39_postcheck.sql` with `PASS` for table, constraints, and index
- focused `agent_governance` tests: `10` passed
- full backend tests: `275` passed

PR and production validation:

- PR `#327` merged to `main` as `104131e`.
- Post-merge `main` checks passed: `CI`, `Release Readiness Gate`, `Secret Scan`,
  `Public Naming Guard`, `Quality Gate Policy Matrix`, `Governance Correlation Smoke`,
  `Desktop Updater Readiness`, and `SonarQube Governance`.
- Production migration `supabase_schema_v39.sql` was applied manually through ignored
  `DATABASE_URL`.
- Production `v39_postcheck.sql` returned `PASS` for table, constraints, and index.
- Render deploy `dep-d8n5nhu47okc73eqd510` for `104131e` reached `live`.
- Production `/health` returned `200` with `ok`.
- Authenticated `/stats` returned `200`.
- Anonymous `GET /agent-governance/settings?org_name=yohandry10` returned `401`.
- Authenticated default settings returned `enabled=false`, `mode=manual_only`,
  `payload_mode=minimized`.
- Disabled `POST /agent-governance/evaluate` returned `403` with
  `code=agent_governance_disabled`.
- Disabled-attempt history for `agent_id=kan92-smoke-disabled` returned `total=0`, proving no
  evaluation row was created.
- Temporary Admin opt-in returned `enabled=true`, `mode=opt_in_enabled`.
- Enabled evaluation returned `201`, `decision=allowed`, and
  `evaluation_id=agv_a8375adeebe640be8d6074883d5e1b71`.
- The response and history both redacted secret-like metadata to `[REDACTED]`.
- Final Admin opt-out restored the tenant to `enabled=false`, `mode=manual_only`.
