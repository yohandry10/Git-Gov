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

## Local, CI, and Production Validation

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
- Production migration attempt after PR `#425` initially applied columns/indexes but postcheck failed
  because the migration checked `pg_constraint.conname` without scoping to
  `public.change_risk_evaluations`. The follow-up fix scopes every idempotency check to
  `conrelid = 'public.change_risk_evaluations'::regclass` before reapplying the same `v63`
  migration/postcheck.
- After the fix, production `v63` was re-applied idempotently and
  `supabase_schema_v63_postcheck.sql` passed.
- Backend `cargo clippy -- -D warnings`.
- Backend `cargo test --no-run`.
- Tauri `cargo clippy -- -D warnings`.
- Tauri `cargo test` (`49` passed).
- Frontend `pnpm --dir gitgov lint`.
- Frontend `pnpm --dir gitgov test` (`376` passed).
- Frontend `pnpm --dir gitgov build`; existing Vite chunk-size warning remains.
- `git diff --check`.
- `scripts/security/publication_guard.ps1`.
- PR `#425` merged the KAN-122 feature.
- PR `#426` fixed production `v63` migration constraint idempotency checks so they are scoped to
  `public.change_risk_evaluations`.
- PR `#427` fixed CI evidence detection so real GitHub Actions run URLs count as CI evidence.
- PR checks passed for `#425`, `#426`, and `#427`.
- Post-merge `main` checks passed for final commit `243b8998`, including `CI`, `Release Readiness
  Gate`, `Quality Gate Policy Matrix`, `Secret Scan`, `Public Naming Guard`, `Governance
  Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance`.
- Render deploy `dep-d8oc3r8jo6nc73b2s07g` for `243b8998` reached `live`.
- Production smoke passed:
  - `/health=ok`.
  - Authenticated `/stats=200`.
  - `GET /change-risk/rules` returned `change_risk_rules.v1` and `12` catalog rules.
  - `POST /change-risk/evaluations` created
    `cra_e70a4dfbee3546cd8ae976ff3bcd4ee3` for
    `KAN-122-ci-ref-production-smoke-final`.
  - Created evaluation returned `risk_level=medium` and triggered rules
    `insufficient_evidence`, `production_environment`, and `stale_evidence`.
  - Created evaluation returned trace hash
    `sha256:ee2bb0714ce4e83117581f9ab8ea3c98979693d2ce8a7d7f46711ae274790410`.
  - `GET /change-risk/evaluations/{evaluation_id}` with `org_name=yohandry10` returned the same
    evaluation.
  - `GET /change-risk/evaluations/{evaluation_id}/trace` returned the same trace hash and `12`
    rule trace entries.
  - Real GitHub Actions and PR evidence did not trigger `missing_ci_evidence`,
    `missing_code_review`, or `missing_change_link`.
  - No-claim flags stayed `advisory_only=true`, `llm_used=false`,
    `agent_governance_used=false`, `compliance_claim=false`, and `certification=false`.
  - Agent Governance and Deployment Gate authorization counts stayed unchanged.

## Known Local Validation Limit

- Full backend `cargo test -- --test-threads=2` was retried with `TEST_DATABASE_URL` mapped from
  the ignored backend DB URL, but timed out twice (`10` minutes, then `15` minutes) without useful
  failure output. Focused real Change Risk coverage and backend test compilation passed locally.
- Required GitHub CI passed before merge.
