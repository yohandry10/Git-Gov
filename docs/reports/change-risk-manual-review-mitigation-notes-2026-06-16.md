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

## Implemented

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

## PR And Production Validation

- PR `#430` merged to `main` as `3aa5f894`.
- PR checks passed.
- Production `v64` migration/postcheck passed.
- Render deploy `dep-d8od0bt8nd3s73adtalg` for `3aa5f894` reached `live`.
- Production smoke passed:
  - `/health=ok`.
  - Authenticated `/stats=200`.
  - `POST /change-risk/evaluations` created
    `cra_4d59c84859a747789e577ca24945ec50` for `KAN-123-production-smoke`.
  - `GET /change-risk/evaluations/{id}/review` returned default `needs_review` before updates
    and final `accepted_risk` after updates.
  - `PATCH /change-risk/evaluations/{id}/review` moved the same evaluation through
    `reviewed`, `needs_mitigation`, and final `accepted_risk`.
  - A secret-like note containing `Authorization: Bearer` was rejected with HTTP `400`.
  - `GET /change-risk/evaluations/{id}/trace` preserved the same trace hash after review updates.
  - `3` `change_risk_review_updated` audit events were recorded with `trace_changed=false`.
  - Deployment Gate authorization and Agent Governance evaluation counts did not change.
  - No-claim flags stayed false for LLM use, Agent Governance use, compliance claim, and
    certification.
