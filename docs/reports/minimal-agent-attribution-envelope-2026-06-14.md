# KAN-96 Minimal Agent Attribution Envelope Report

Date: 2026-06-14

## Scope

KAN-96 implements the next roadmap slice after KAN-95: a minimal attribution envelope for optional
Agent Governance requests.

Implemented:

- optional `attribution` request object for `POST /agent-governance/dry-run`.
- optional `attribution` request object for `POST /agent-governance/evaluate`.
- strict field and size validation for attribution data.
- server-generated `agcorr_` correlation id when omitted.
- response attribution envelope with `attr_` attribution id.
- formal evaluation persistence of attribution columns.
- `GET /agent-governance/evaluations?correlation_id=...` history filtering.
- dry-run attribution response and admin audit metadata without formal evaluation persistence.
- Supabase migration `v41` and postcheck.

Not implemented:

- MCP server.
- chatbot or BYOM.
- autonomous execution.
- provider writes.
- Deployment Gate changes.
- prompt, diff, source code, or raw tool trace storage.
- full agent session graph.

## Local Validation

Passed so far:

- `cargo check --manifest-path gitgov\gitgov-server\Cargo.toml`
- `cargo clippy --manifest-path gitgov\gitgov-server\Cargo.toml -- -D warnings`
- `cargo fmt --manifest-path gitgov\gitgov-server\Cargo.toml --check`
- local `supabase_schema_v41.sql` postcheck against PostgreSQL 16 on `127.0.0.1:55439`
- focused Agent Governance tests with `TEST_DATABASE_URL`: `24` passed
- full backend tests with `TEST_DATABASE_URL`: `289` passed
- `git diff --check`
- `.\scripts\security\publication_guard.ps1`

Focused coverage:

- formal evaluate persists attribution and `GET /agent-governance/evaluations` can filter by
  `correlation_id`.
- dry-run returns attribution but creates no `agent_governance_evaluations` row.
- dry-run audit metadata includes safe correlation/tool/session attribution.
- GitGov generates an `agcorr_` correlation id when omitted.
- manual-only tenants reject attributed evaluate requests without persistence.
- unsafe attribution containing credential material is rejected without persistence.

## Production Validation

Pending PR merge, Render deploy, production migration `v41`, and production smoke validation.
