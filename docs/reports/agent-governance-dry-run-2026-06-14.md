# KAN-95 Agent Governance Dry-Run Report

Date: 2026-06-14

## Scope

KAN-95 adds `POST /agent-governance/dry-run`.

Implemented:

- dry-run route using the existing Agent Governance request contract.
- deterministic preview using the same policy evaluator as `POST /agent-governance/evaluate`.
- response flags `dry_run=true`, `would_persist_evaluation=false`, and
  `would_authorize_execution=false`.
- no `agent_governance_evaluations` persistence for dry-runs.
- agent-scoped keys can call dry-run through the existing `agent_governance:evaluate` scope.
- agent key `allowed_actions` enforcement is shared by evaluate and dry-run.
- disabled tenants return `403 agent_governance_disabled` with no evaluation row.
- dry-run requested/denied audit events.
- no Deployment Gate behavior change.

## Product Behavior

Dry-run answers "what would GitGov decide?" without turning that answer into approval evidence.

This protects the manual-first product line:

- Deployment Gates remain CI/CD-facing and do not call Agent Governance.
- Manual-only tenants remain valid and unaffected.
- Agent Governance still requires explicit tenant opt-in.
- Agent keys still cannot execute commits, pushes, merges, policy changes, or deployments.

## Validation

Local validation performed against a real temporary PostgreSQL 16 database on
`127.0.0.1:55438`.

Passed:

- `cargo fmt --manifest-path gitgov\gitgov-server\Cargo.toml`
- `cargo check --manifest-path gitgov\gitgov-server\Cargo.toml`
- `cargo clippy --manifest-path gitgov\gitgov-server\Cargo.toml -- -D warnings`
- local `supabase_schema_v40.sql` postcheck, because KAN-95 builds on KAN-94 agent keys
- focused Agent Governance tests with `TEST_DATABASE_URL`: `19` passed
- full backend tests with `TEST_DATABASE_URL`: `284` passed
- `git diff --check`
- `.\scripts\security\publication_guard.ps1`

Focused coverage:

- human/developer dry-run on disabled tenant returns `agent_governance_disabled`.
- disabled dry-run creates no `agent_governance_evaluations` row.
- enabled dry-run returns deterministic decision preview with no `evaluation_id`.
- enabled dry-run returns `dry_run=true`, `would_persist_evaluation=false`, and
  `would_authorize_execution=false`.
- enabled dry-run includes shared governance decision and deterministic `llm_decision=false`.
- enabled dry-run writes `agent_governance.dry_run_requested`.
- agent key dry-run records agent principal identity in the response without persisting an
  evaluation row.
- agent key dry-run respects `allowed_actions`; `change_policy` is denied when not explicitly
  allowed.
- active agent key plus disabled tenant returns `agent_governance_disabled` and creates no
  evaluation row.

## Production Validation

Passed after PR `#336` merged to `main` as `d63fba3`.

- Post-merge GitHub workflows passed for `d63fba3`:
  - `CI`
  - `Release Readiness Gate`
  - `Secret Scan`
  - `Public Naming Guard`
  - `Quality Gate Policy Matrix (Optional)`
  - `Governance Correlation Smoke (Optional)`
  - `Desktop Updater Readiness (Optional)`
  - `SonarQube Governance (Non-Blocking)`
- Render deploy `dep-d8n79leq1p3s7383pu90` reached `live` for commit `d63fba3`.
- `GET https://gitgov-api.onrender.com/health` returned `ok`.
- Authenticated `GET https://gitgov-api.onrender.com/stats` returned HTTP `200`.
- Before smoke, Agent Governance settings for `yohandry10` were
  `enabled=false`, `mode=manual_only`, and `payload_mode=minimized`.
- Authenticated dry-run while disabled returned HTTP `403`,
  `code=agent_governance_disabled`, `dry_run=true`, and
  `would_persist_evaluation=false`.
- A temporary agent key `agk_1e3e5b1281714b1da883c2d9cd1fa5dc` was created with
  `allowed_actions=["commit"]`; the one-time plaintext token was not printed in
  logs or committed.
- Agent Governance was temporarily enabled for the smoke and returned
  `mode=opt_in_enabled`.
- Agent-key dry-run for `commit` returned HTTP `200`, `decision=allowed`,
  `dry_run=true`, `would_persist_evaluation=false`,
  `would_authorize_execution=false`, `principal_type=agent`, matching
  `agent_key_id`, and deterministic `llm_decision=false`.
- Admin history lookup for smoke agent `kan95-smoke-agent` returned `total=0`,
  proving dry-run did not persist an `agent_governance_evaluations` row.
- Agent-key dry-run for disallowed `change_policy` returned HTTP `403` with
  `code=action_not_allowed`.
- Temporary agent key was revoked.
- Agent Governance settings were restored to `enabled=false` and
  `mode=manual_only`.
