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

## PR And Production Validation

- PR `#433` merged to `main` as `d145d6fe`.
- PR checks passed, including Security Guard, Server Clippy + Check, Desktop Rust Clippy, Frontend
  Lint + Typecheck, Website Lint + Typecheck + Build, Validate Policy-as-Code, and the quality gate
  matrix.
- Post-merge `main` checks passed, including CI, Release Readiness Gate, Secret Scan, Public Naming
  Guard, Governance Correlation Smoke, Desktop Updater Readiness, Quality Gate Policy Matrix, and
  SonarQube Governance.
- Render deploy `dep-d8odh5c2m8qs73amf7p0` for commit `d145d6fe` reached `live`.
- Production smoke passed:
  - `/health=ok`.
  - Authenticated `/stats=200`.
  - `GET /change-risk/evaluations?review_status=accepted_risk` returned KAN-123 smoke evaluation
    `cra_4d59c84859a747789e577ca24945ec50`.
  - `GET /change-risk/evaluations?review_status=needs_review` excluded that accepted-risk
    evaluation.
  - Invalid `review_status=approved` returned HTTP `400`.
  - The accepted-risk record retained `advisory_only=true`, `llm_used=false`,
    `agent_governance_used=false`, `compliance_claim=false`, and `certification=false`.
  - Deployment Gate and Agent Governance counts stayed unchanged before and after read-only smoke:
    `deployment_gate_authorizations=2;agent_governance_evaluations=7`.

## Result

KAN-124 is complete and production-smoked. The feature gives CAB/Admin/Auditor users a real manual
review queue without turning Change Risk into an enforcement gate or agentic workflow.
