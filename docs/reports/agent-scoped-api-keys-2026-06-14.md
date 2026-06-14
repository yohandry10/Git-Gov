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

Pending PR merge, Render deploy, production migration `v40`, and smoke validation.
