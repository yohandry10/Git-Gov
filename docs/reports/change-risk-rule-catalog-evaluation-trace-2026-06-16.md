# KAN-122 Change Risk Rule Catalog & Evaluation Trace Validation

KAN-122 implements rule-level explainability for the Change Risk Advisory.

## Product Decision

The next roadmap slice after KAN-121 is not enforcement, AI, BYOM, MCP, chatbot behavior, or executive dashboards. It is rule-level auditability for the existing deterministic Change Risk evaluator.

Change Risk remains:

- advisory-only;
- deterministic;
- manual-first;
- qualitative, not a compliance score;
- independent from Agent Governance;
- non-mutating for providers, repositories, deployment gates, and release evidence.

## Implemented

- Supabase migration/postcheck `v63`.
- Consolidated Supabase schema update.
- Backend persisted fields:
  - `ruleset_version`;
  - `triggered_rules`;
  - `non_triggered_rules`;
  - `evaluation_trace`;
  - `trace_hash`.
- Backend route `GET /change-risk/rules`.
- Backend route `GET /change-risk/evaluations/{evaluation_id}/trace`.
- `POST /change-risk/evaluations` now persists rule trace metadata.
- `GET /change-risk/evaluations` and `GET /change-risk/evaluations/{evaluation_id}` allow Admin and Auditor read access.
- `POST /change-risk/evaluations` remains Admin-only.
- Tauri DTOs/client/commands/invoke registration.
- Control Plane store state/actions/tests for rules and trace.
- Governance > Releases `ChangeRiskPanel` adds `Why this risk?`.
- Rule/trace helper split to `gitgov-server/src/handlers/change_risk_rules.rs` so `change_risk.rs` stays below the normal maintainability threshold.

## Rule Catalog

Ruleset: `change_risk_rules.v1`.

Rules:

- `missing_release_approval`.
- `missing_ci_evidence`.
- `missing_code_review`.
- `missing_change_link`.
- `provider_unhealthy`.
- `policy_source_conflict`.
- `production_environment`.
- `break_glass_involved`.
- `stale_evidence`.
- `gate_requires_approval`.
- `gate_blocked`.
- `insufficient_evidence`.

## Local Validation So Far

- Backend `cargo fmt`.
- Backend `cargo check`.
- Tauri `cargo fmt`.
- Tauri `cargo check`.
- Frontend focused ESLint on changed files.
- Frontend `pnpm --dir gitgov typecheck`.
- Focused store test:
  - `pnpm --dir gitgov test src/test/useControlPlaneStore.test.ts`.
  - Result: `42` passed.
- Focused backend Change Risk test command with `TEST_DATABASE_URL` mapped from the ignored backend
  `DATABASE_URL`:
  - `cargo test change_risk -- --nocapture`.
- Result: `2` passed, `0` failed, `0` skipped.
- `v63` migration and postcheck passed in a real Postgres rollback transaction:
  - `BEGIN`.
  - `supabase/supabase_schema_v63.sql`.
  - `supabase/supabase_schema_v63_postcheck.sql`.
  - `ROLLBACK`.
- Backend `cargo clippy -- -D warnings`.
- Backend `cargo test --no-run`.
- Tauri `cargo clippy -- -D warnings`.
- Tauri `cargo test` (`49` passed).
- Frontend `pnpm --dir gitgov lint`.
- Frontend `pnpm --dir gitgov test` (`376` passed).
- Frontend `pnpm --dir gitgov build`; existing Vite chunk-size warning remains.
- `git diff --check`.
- `scripts/security/publication_guard.ps1`.

## Remaining Before Merge

- Full backend `cargo test -- --test-threads=2` was retried with `TEST_DATABASE_URL` mapped from the ignored backend DB URL, but timed out twice (`10` minutes, then `15` minutes) without useful failure output. Focused real Change Risk coverage and backend test compilation passed; CI still must run the required backend suite before merge.
- Open PR `KAN-122`, wait for required checks, merge, apply production `v63`, deploy Render, and run production smoke.
