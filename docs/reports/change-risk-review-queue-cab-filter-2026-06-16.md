# KAN-124 Change Risk Review Queue And CAB Evidence Filter Validation

KAN-124 adds a manual review queue filter to the deterministic Change Risk Advisory.

## Decision

ChatGPT was consulted twice in the existing product-lead conversation after KAN-123. The visible
answers did not produce a usable KAN-124 ordinance: one repeated KAN-123 context and the follow-up
rendered an empty answer. The implementation decision was therefore made from the repo roadmap and
current product state.

The selected slice is `KAN-124 - Change Risk Review Queue and CAB Evidence Filter`.

## Product Scope

- Add `review_status` filtering to `GET /change-risk/evaluations`.
- Add `review_status` to backend, Tauri, and frontend query contracts.
- Add a Desktop `Review queue` selector to `ChangeRiskPanel`.
- Keep the KAN-123 manual review state as the source of truth.
- Keep the feature advisory-only and manual-first.

## Non-Goals

- No scoring.
- No enforcement or release blocking.
- No deployment execution.
- No provider or repository mutation.
- No AI/LLM, BYOM, MCP, chatbot behavior, or Agent Governance dependency.
- No compliance, certification, legal, or regulatory claim.
- No notifications, approval quorum, or multi-reviewer workflow.

## Local Validation

- Backend `cargo fmt --check`.
- Backend `cargo check`.
- Backend `cargo clippy -- -D warnings`.
- Backend `cargo test --no-run`.
- Focused backend Change Risk tests with real Postgres:
  - `cargo test change_risk -- --nocapture`.
  - Result: `2` passed, `0` failed.
- Tauri `cargo check`.
- Tauri `cargo fmt --check`.
- Tauri `cargo clippy -- -D warnings`.
- Tauri `cargo test` (`49` passed).
- Frontend `pnpm --dir gitgov typecheck`.
- Frontend `pnpm --dir gitgov lint`.
- Frontend `pnpm --dir gitgov test` (`377` passed).
- Frontend `pnpm --dir gitgov build` passed with the pre-existing Vite large chunk warning.
- Focused store test:
  - `pnpm --dir gitgov test src/test/useControlPlaneStore.test.ts`.
  - Result: `43` passed.
- `git diff --check` passed.
- `scripts/security/publication_guard.ps1` passed.

## Remaining

- PR checks.
- Production deploy and smoke.
