# KAN-94 Agent-Scoped API Keys Report

Date: 2026-06-14

## Scope

KAN-94 adds optional agent-scoped credentials for the Agent Governance API.

Implemented:

- Admin-only `GET /agent-governance/agent-keys`.
- Admin-only `POST /agent-governance/agent-keys`.
- Admin-only `DELETE /agent-governance/agent-keys/{key_id}`.
- One-time plaintext token return with `ggag_` prefix.
- Hash-only token storage with prefix/last-four metadata for administration.
- Agent-key auth path limited to `POST /agent-governance/evaluate`.
- Scope enforcement for `agent_governance:evaluate`.
- Per-key `allowed_actions` enforcement.
- Revoked, expired, invalid-scope, disabled-tenant, action-denied, and tenant-scope failures do not
  create evaluation rows.
- Agent identity persisted in `agent_governance_evaluations`.
- Agent key use/deny/revoke audit events.
- Supabase migration `supabase_schema_v40.sql` and postcheck `v40_postcheck.sql`.
- Handler split so evaluation, settings/history, and key administration are separate modules.

## Product Behavior

GitGov remains manual-first.

Deployment Gates, formal approvals, Policy-as-Code, PR review, audit exports, and evidence packets do
not require Agent Governance or agent-scoped credentials. A tenant with Agent Governance disabled
continues to reject agent evaluations with `403 agent_governance_disabled` and no persisted
evaluation evidence.

Agent keys exist only for customers that explicitly opt in to Agent Governance.

## Security Boundary

Agent keys cannot use general GitGov API routes. The middleware accepts a `ggag_` bearer token only
when all of these are true:

- the key hash exists.
- the key is not revoked.
- the key is not expired.
- the request is exactly `POST /agent-governance/evaluate`.
- the key has `agent_governance:evaluate`.
- the requested action is present in the key's allowed action set.
- the tenant has Agent Governance enabled.
- the request org scope matches the key tenant.

The default action set excludes `change_policy`.

## Validation

Local validation performed against a real temporary PostgreSQL 16 database on
`127.0.0.1:55437`.

Passed so far:

- `cargo fmt --manifest-path gitgov\gitgov-server\Cargo.toml --check`
- `cargo check --manifest-path gitgov\gitgov-server\Cargo.toml`
- `cargo clippy --manifest-path gitgov\gitgov-server\Cargo.toml -- -D warnings`
- local `supabase_schema_v40.sql` migration and `v40_postcheck.sql`
- focused Agent Governance tests with `TEST_DATABASE_URL`: `15` passed
- full backend tests with `TEST_DATABASE_URL`: `280` passed

Focused coverage:

- Agent key management is Admin-only.
- Created token is returned once, starts with `ggag_`, and is not stored plaintext.
- Listing keys does not expose the token.
- Enabled tenant agent key can evaluate and records `principal_type=agent`.
- Evaluation response, history, and DB rows include the agent key id and display name.
- Disabled tenant returns `agent_governance_disabled` and creates no evaluation row.
- `change_policy` is denied by the default allowed-action boundary.
- Cross-tenant use is rejected and creates no evaluation row.
- Revoked keys return `revoked_key`, create no evaluation row, and write denial audit.
- Expired keys return `expired_key`, create no evaluation row, and write denial audit.
- Agent keys cannot call admin routes such as `GET /agent-governance/agent-keys`.
- Manual-only behavior remains covered by the disabled-by-default Agent Governance test.

## Production Validation

PR `#333` merged to `main` as `aa0a9c9`.

Post-merge `main` checks passed:

- `CI`
- `Release Readiness Gate`
- `Secret Scan`
- `Public Naming Guard`
- `Quality Gate Policy Matrix`
- `Governance Correlation Smoke`
- `Desktop Updater Readiness`
- `SonarQube Governance`

Production migration `supabase_schema_v40.sql` was applied manually through ignored
`DATABASE_URL`. `v40_postcheck.sql` returned `PASS` for:

- `agent_governance_agent_keys.table`
- `agent_governance_agent_keys.constraints`
- `agent_governance_agent_keys.indexes`
- `agent_governance_evaluations.agent_identity_columns`

Render deploy `dep-d8n6pv77f7vs73fgfta0` for `aa0a9c9` reached `live`.

Production smoke passed:

- `/health` returned `ok`.
- authenticated `/stats` returned `200`.
- Agent Governance started as `enabled=false`, `mode=manual_only`.
- Disabled evaluation returned `403 agent_governance_disabled` before opt-in.
- Admin-created temporary agent key `agk_fcbda5f73c324fba99d33b195400bbcc` returned a one-time
  token and list response did not expose plaintext token material.
- Temporary opt-in changed settings to `enabled=true`, `mode=opt_in_enabled`.
- Agent-key evaluation created `agv_356ca31100864046b104e2184eaec0ba` with
  `decision=allowed`, `principal_type=agent`, matching agent key id, and `llm_decision=false`.
- `change_policy` with the same key returned `403 action_not_allowed`.
- Revoking the key set `revoked_at`.
- Reusing the revoked token returned `401 revoked_key`.
- Agent Governance was restored to `enabled=false`, `mode=manual_only`.

## Status

Implemented and production-validated.
