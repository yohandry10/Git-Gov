# KAN-123 Change Risk Manual Review & Mitigation Notes Validation

KAN-123 adds human review metadata to the deterministic Change Risk Advisory.

## Product Decision

The next roadmap slice after KAN-122 is not enforcement, AI, BYOM, MCP, chatbot behavior, or executive dashboards. It is manual review evidence for an already explained Change Risk evaluation.

Change Risk remains:

- advisory-only;
- deterministic;
- manual-first;
- qualitative, not a compliance score;
- independent from Agent Governance;
- non-mutating for providers, repositories, deployment gates, and release evidence.

## Implemented Locally

- Supabase migration/postcheck `v64`.
- Consolidated Supabase schema update.
- Backend review metadata on `change_risk_evaluations`.
- Backend route `GET /change-risk/evaluations/{evaluation_id}/review`.
- Backend route `PATCH /change-risk/evaluations/{evaluation_id}/review`.
- Admin-only update; Admin/Auditor read.
- Safe note normalization and secret-like text rejection.
- Dedicated backend review handler module `gitgov-server/src/handlers/change_risk_review.rs`.
- Admin audit action `change_risk_review_updated`.
- Tauri DTOs/client methods/commands/invoke registration.
- Control Plane store state/actions/tests.
- Governance > Releases `ChangeRiskPanel` `Manual Review` panel.

## Local Validation

- Backend `cargo fmt --check`.
- Backend `cargo check`.
- Backend `cargo clippy -- -D warnings`.
- Backend `cargo test --no-run`.
- Tauri `cargo fmt --check`.
- Tauri `cargo check`.
- Tauri `cargo clippy -- -D warnings`.
- Tauri `cargo test` (`49` passed).
- Frontend `pnpm --dir gitgov typecheck`.
- Frontend `pnpm --dir gitgov lint`.
- Frontend `pnpm --dir gitgov test` (`377` passed).
- Frontend `pnpm --dir gitgov build` passed with the pre-existing Vite large chunk warning.
- Focused backend Change Risk test command with `TEST_DATABASE_URL` mapped from the ignored backend
  `DATABASE_URL`:
  - `cargo test change_risk -- --nocapture`.
  - Result: `2` passed, `0` failed.
- Focused store test:
  - `pnpm --dir gitgov test src/test/useControlPlaneStore.test.ts`.
  - Result: `43` passed.
- `v64` migration and postcheck passed in a real Postgres rollback transaction:
  - `BEGIN`.
  - `supabase/supabase_schema_v64.sql`.
  - `supabase/supabase_schema_v64_postcheck.sql`.
  - `ROLLBACK`.
- `git diff --check` passed.
- `scripts/security/publication_guard.ps1` passed.

Validation note:

- `pnpm --dir gitgov test -- --run` failed because this repository's test script does not accept
  the extra `--run` option through `pnpm`; the correct full-suite command is `pnpm --dir gitgov
  test`, which passed.

## Remaining

- PR checks.
- Production `v64` migration/postcheck.
- Render deploy.
- Production smoke covering GET/PATCH review, audit evidence, immutable trace hash, no Deployment Gate mutation, no Agent Governance mutation, and no-claim flags.
