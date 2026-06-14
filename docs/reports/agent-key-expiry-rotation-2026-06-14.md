# KAN-97 Agent Key Expiry And Rotation Validation

KAN-97 implements Agent Key Expiry and Rotation UX for optional Agent Governance.

## Decision

GPT/product review selected KAN-97 as credential lifecycle hardening, not MCP. The reason is
enterprise readiness: before opening broader agentic surfaces, GitGov must prove agent credentials
can expire, rotate, revoke, and leave safe audit evidence.

Manual-first remains unchanged:

- Agent Governance stays disabled by default.
- Deployment Gates do not use agent keys.
- Manual-only customers do not need agent keys.
- Expired/revoked agent keys cannot create evaluation rows.

## Implemented

- `POST /agent-governance/agent-keys` now defaults new keys to 90-day expiry.
- `no_expiry=true` is explicit and mutually exclusive with `expires_at`.
- Agent key responses include derived `status`, `expiring_soon`, and `no_expiry`.
- Added `POST /agent-governance/agent-keys/{key_id}/rotate`.
- Rotation creates a replacement key with a one-time plaintext token.
- Rotation links old and new keys with `rotated_from_key_id` and `replaced_by_key_id`.
- Replaced keys keep working during the configured grace period unless revoked.
- Revocation takes precedence over expiry.
- Auth audit now separates `agent_key.denied_expired` and `agent_key.denied_revoked`.
- Supabase migration `v42` adds lifecycle columns and indexes.

## Local Validation

Temporary PostgreSQL 16:

```text
127.0.0.1:55440
```

Migration validation:

```text
supabase_schema_v42.sql
checks/v42_postcheck.sql
```

Result:

```text
agent_governance_agent_keys.lifecycle_columns PASS
agent_governance_agent_keys.lifecycle_indexes PASS
```

Focused Agent Governance tests with `TEST_DATABASE_URL`:

```text
27 passed
```

Full backend tests with `TEST_DATABASE_URL`:

```text
292 passed
```

Rust validation:

```text
cargo check --manifest-path gitgov\gitgov-server\Cargo.toml
cargo clippy --manifest-path gitgov\gitgov-server\Cargo.toml -- -D warnings
cargo fmt --manifest-path gitgov\gitgov-server\Cargo.toml
```

All passed.

## Real Scenarios Covered

- Default key creation returns expiry and `status=active`.
- Explicit `no_expiry=true` returns `status=no_expiry`.
- `no_expiry=true` plus `expires_at` is rejected.
- Expired keys return `agent_key_expired` and create no evaluation rows.
- Revoked keys return `agent_key_revoked` and create no evaluation rows.
- Rotation creates a replacement key and one-time token.
- Old key enters `rotation_pending` and links to replacement.
- Replacement key links back to the old key.
- Old key works during grace.
- Replacement key works immediately.
- Old key fails after grace expiry.
- Revoked old key fails as revoked even if also expired.
- Rotation audit is written.
- Expired/revoked audit events are specific.

## Production Validation

Pending until PR merge, Render deployment, production `v42` application, and smoke validation.
