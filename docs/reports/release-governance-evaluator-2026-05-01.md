# KAN-46 Release Governance Evaluator

Updated: 2026-05-01

## Summary

KAN-46 adds the first release governance evaluator for GitGov Enterprise Self-Service.

The evaluator consumes the persisted KAN-45 `release_governance` profile policy and the KAN-37 formal release approval records, then returns a clear release status for operators and future gates.

Default behavior remains non-blocking. A customer must explicitly configure blocking release governance before the evaluator can report a blocking policy result.

## Traceability

- Jira issue: `KAN-46 - Add release governance evaluator`.
- Branch: `product/KAN-46-release-governance-evaluator`.
- PR: `#146 - product(KAN-46): add release governance evaluator`.
- Merge commit: `025243214639757e901830d958e60e2ba3eb55cd`.
- Design: `docs/design/release-governance-evaluator-mvp.md`.

## Changes

- Added admin backend endpoint `GET /enterprise/release-governance/evaluate`.
- Added backend response models for policy summary, approval summary, quorum rule summary, and evaluation result.
- Added release governance evaluation logic for `record-only`, `advisory`, `approval-required`, and `quorum-required`.
- Added stale-auth-cache sensitive-path coverage for `/enterprise/release-governance/evaluate`.
- Added Tauri client structs, method, command, and command registration.
- Added dashboard store state/action for `evaluateEnterpriseReleaseGovernance`.
- Added `Approver role` to the release approval dashboard form.
- Added dashboard governance evaluation controls and result display.
- Added focused backend tests for record-only, advisory, approval-required, and quorum-required behavior.
- Added frontend store test coverage for the new Tauri command path.

## Product Behavior

The evaluator does not silently enforce release blocking.

- `record-only` returns `recorded` and does not block even when no approval exists.
- `advisory` returns `advisory-warning` when approval evidence is missing, but still does not block.
- `approval-required` can return `blocked` only when the customer configured blocking approval enforcement.
- `quorum-required` can return `blocked` only when the customer configured role quorum enforcement.

For the quorum MVP, approver roles are stored in `evidence_summary.approver_role` on the existing approval record. This avoids a database migration while still making multi-role policy evaluation possible.

## Security And Secret Safety

- No provider token, `.env` value, Authorization header, webhook secret, or raw customer credential is read or printed.
- The route is admin-only.
- Global admin access still requires explicit org scope.
- Evidence packet binding uses SHA-256 hashes.
- The dashboard forwards only the configured GitGov API key through the existing Tauri control-plane client.

## Local Validation

Completed locally:

- `cargo fmt` from `gitgov/gitgov-server`: passed.
- `cargo fmt` from `gitgov/src-tauri`: passed.
- `cargo test release_approval_tests` from `gitgov/gitgov-server`: passed, `9` tests.
- `cargo test sensitive_admin_path_detection_matches_expected_routes` from `gitgov/gitgov-server`: passed, `1` test.
- `cargo check` from `gitgov/gitgov-server`: passed.
- `cargo clippy -- -D warnings` from `gitgov/gitgov-server`: passed.
- `cargo check` from `gitgov/src-tauri`: passed.
- `cargo clippy -- -D warnings` from `gitgov/src-tauri`: passed.
- `cargo test` from `gitgov/src-tauri`: passed, `23` tests.
- `npm test -- --run src/test/useControlPlaneStore.test.ts` from `gitgov`: passed, `22` tests.
- `npm run lint` from `gitgov`: passed.
- `npm run typecheck` from `gitgov`: passed.
- `npm test -- --run` from `gitgov`: passed, `25` test files and `283` tests.
- `npm run build` from `gitgov`: passed with the existing Vite large chunk warning.
- `git diff --check`: passed.
- `.\scripts\security\publication_guard.ps1`: passed.

## Deployment Notes

- No database migration is required.
- No provider setting is changed.
- No customer workflow installation is triggered.
- Render deploy `dep-d7q5qmkvikkc73cmfg0g` reached `live` on commit `025243214639757e901830d958e60e2ba3eb55cd`.

## GitHub Validation

PR `#146` checks passed before merge:

- `Security Guard`.
- `Server Clippy + Check`.
- `Desktop Rust Clippy`.
- `Frontend Lint + Typecheck`.
- `Website Lint + Typecheck + Build`.
- `Workflow Lint`.
- `Validate quality_gates warn/block matrix`.
- `Sonar Scan + Quality Gate`.
- `Block internal-assistant markers in branch/commits`.
- `Vercel`.
- `Vercel Preview Comments`.

Post-merge checks passed on `main` commit `025243214639757e901830d958e60e2ba3eb55cd`:

- `CI` run `25207328590`.
- `Release Readiness Gate` run `25207328587`.
- `Quality Gate Policy Matrix (Optional)` run `25207328585`.
- `Secret Scan` run `25207328605`.
- `SonarQube Governance (Non-Blocking)` run `25207328608`.
- `Public Naming Guard` run `25207328592`.
- `Governance Correlation Smoke (Optional)` run `25207328584`.
- `Desktop Updater Readiness (Optional)` run `25207328581`.

## Production Smoke

Production validation passed after Render deploy:

- `GET https://gitgov-api.onrender.com/health` returned `status=ok`.
- Anonymous `GET /enterprise/release-governance/evaluate?...` returned `401`.
- Authenticated `GET /enterprise/release-governance/evaluate?...` returned `200`.
- The authenticated evaluator response returned `status=recorded`, `policy_mode=record-only`, `blocking=false`, `would_block=false`, `valid=0`, and `required=0`.

## Residual Work

- KAN-47 adds the optional workflow gate that consumes `blocking=true` only when the customer has opted into release enforcement.
- Add richer approval role management if customers need roles as first-class database fields.
- Add per-environment policy expansion after the current profile shape has enough usage evidence.
