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

Passed after PR `#339` merged to `main` as `3f24c0b`.

- Post-merge GitHub workflows passed for `3f24c0b`:
  - `CI`
  - `Release Readiness Gate`
  - `Secret Scan`
  - `Public Naming Guard`
  - `Quality Gate Policy Matrix (Optional)`
  - `Governance Correlation Smoke (Optional)`
  - `Desktop Updater Readiness (Optional)`
  - `SonarQube Governance (Non-Blocking)`
- Production migration `supabase_schema_v41.sql` was applied manually through ignored
  `DATABASE_URL`.
- Production `v41_postcheck.sql` returned `PASS` for:
  - `agent_governance_evaluations.attribution_columns`
  - `agent_governance_evaluations.attribution_indexes`
- Render deploy `dep-d8n7nb4m0tmc73b5hveg` reached `live` for commit `3f24c0b`.
- `GET https://gitgov-api.onrender.com/health` returned `ok`.
- Authenticated `GET https://gitgov-api.onrender.com/stats` returned HTTP `200`.
- Before smoke, Agent Governance settings for `yohandry10` were
  `enabled=false`, `mode=manual_only`, and `payload_mode=minimized`.
- Manual-only attributed evaluate returned HTTP `403` with
  `code=agent_governance_disabled`; history for the smoke agent returned `total=0`.
- Temporary agent key `agk_ec4eec47df1f42ec9a78b921309ee44c` was created with
  `allowed_actions=["commit"]`; the one-time plaintext token was not printed in logs or committed.
- Agent Governance was temporarily enabled for smoke and returned `mode=opt_in_enabled`.
- Agent-key dry-run with attribution returned HTTP `200`,
  `consumer_type=agent_dry_run`, matching correlation
  `corr-kan96-prod-dry-1781431652106`, and `would_persist_evaluation=false`.
- Dry-run history for the smoke dry-run agent returned `total=0`, proving no formal evaluation row
  was created.
- Agent-key formal evaluate with attribution returned HTTP `201`, created
  `agv_833b8b31c41947ccaf5c69d153890035`, returned correlation
  `corr-kan96-prod-eval-1781431652106`, parent correlation
  `corr-kan96-prod-dry-1781431652106`, matching agent key id, and deterministic
  `llm_decision=false`.
- History lookup by `correlation_id=corr-kan96-prod-eval-1781431652106` returned `total=1` with
  `tool_name=codex-cli`.
- Unsafe attribution with credential-looking `tool_name` returned HTTP `400`.
- Temporary agent key was revoked.
- Agent Governance settings were restored to `enabled=false` and `mode=manual_only`.
