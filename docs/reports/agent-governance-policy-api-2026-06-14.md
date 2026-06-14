# KAN-90 Agent Governance Policy API

KAN-90 adds the first deterministic Agentic Governance Layer API.

## Scope

- Added `POST /agent-governance/evaluate`.
- Added append-only persistence table `agent_governance_evaluations`.
- Added route-sensitive auth classification for `/agent-governance/*`.
- Added deterministic policy decisions for `commit`, `push`, `open_pr`, `merge_pr`,
  `change_policy`, and `deploy`.
- Added real integration tests through the Axum router and Postgres harness.

## Product Decision

Agents do not decide governance controls. They can request or simulate a planned operation. GitGov
returns a deterministic decision:

- `allowed`
- `requires_approval`
- `blocked`

High-impact actions such as merge, policy change, and deploy require human approval or an existing
GitGov control. Missing critical context blocks the operation instead of guessing.

## Local Validation

Local validation completed on 2026-06-14:

- `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`
- `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`
- `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`
- `supabase_schema.sql`, `v28`, `v35`, `v36`, `v37`, `v38`, and `v38_postcheck.sql`
  against temporary Postgres
- focused `agent_governance` integration tests with real Postgres: `7` passed
- sensitive route classification test: `1` passed
- full backend suite with real Postgres: `272` passed

## Runtime Impact

This is backend/API work. It requires Supabase migration `supabase_schema_v38.sql` before production
smoke. It does not require Desktop UI changes.

## Production Readiness

Before closing KAN-90, apply `supabase_schema_v38.sql` in production, run
`checks/v38_postcheck.sql`, wait for the Render backend deploy from `main`, and smoke:

- anonymous `POST /agent-governance/evaluate` returns `401`
- authenticated ticketed `commit` returns `decision=allowed`
- authenticated protected-branch `push` returns `decision=requires_approval`
