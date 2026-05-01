# KAN-48 Environment-Scoped Release Governance Policy

Updated: 2026-05-01

## Summary

KAN-48 adds environment-scoped release governance overrides to the Enterprise Self-Service Adoption profile.

This lets a customer keep the base profile `record-only` while explicitly making `production` advisory, approval-required, or quorum-required. The default remains non-blocking.

## Traceability

- Jira issue: `KAN-48 - Add environment-scoped release governance policy overrides`.
- Branch: `product/KAN-48-environment-release-governance-policy`.
- PR: `#150 - product(KAN-48): add environment release governance overrides`.
- Merge commit: `cba3f9d`.
- Design: `docs/design/environment-scoped-release-governance-policy-mvp.md`.

## Changes

- Added `release_governance.environment_overrides` support to backend adoption profile validation.
- Updated the release governance evaluator to select a matching environment override before falling back to the base policy.
- Updated dashboard adoption profile helpers and UI to model, export, and edit environment overrides.
- Updated CLI adoption pack generation to include release governance gate planning when an override opts into enforcement.
- Updated CLI workflow template generation so generated gate defaults follow the first customer-selected blocking override.
- Updated the example adoption profile with an explicit empty override list.

## Safety

- No default blocking behavior was added.
- `record-only` remains the default.
- Non-`record-only` overrides require the `formal-approval` module.
- Generated packs and templates include variable/secret names only.
- No `.env` values, provider tokens, webhook secrets, Authorization headers, or raw customer credentials are read, printed, or stored.
- No database migration is needed because adoption profiles are already stored as JSON.

## Validation So Far

- `cargo fmt` from `gitgov/gitgov-server`: passed.
- `cargo test release_governance` from `gitgov/gitgov-server`: passed, `6` tests.
- `cargo test enterprise_adoption_profile_validation` from `gitgov/gitgov-server`: passed, `8` tests.
- `cargo check` from `gitgov/gitgov-server`: passed.
- `cargo clippy -- -D warnings` from `gitgov/gitgov-server`: passed.
- `cargo test` from `gitgov/gitgov-server`: passed, `189` tests.
- `npm test -- --run src/test/components/dashboard-helpers.test.ts` from `gitgov`: passed, `17` tests.
- `npm run typecheck` from `gitgov`: passed.
- `npm run lint` from `gitgov`: passed.
- `npm test -- --run` from `gitgov`: passed, `25` test files and `285` tests.
- `npm run build` from `gitgov`: passed with the existing Vite large chunk warning.
- `.\scripts\control-plane\generate_enterprise_adoption_pack.ps1` with an override profile: passed, generated `14` workflows including `.github/workflows/release-governance-gate.yml`.
- `.\scripts\control-plane\generate_enterprise_workflow_templates.ps1` with an override profile: passed, generated `14` templates including `.github/workflows/release-governance-gate.yml`.
- Generated gate template defaulted to `production` and `enforce_gate=true` for the explicit production `approval-required` override.
- `git diff --check`: passed.
- `.\scripts\security\publication_guard.ps1`: passed.

## Remaining Validation

- Docs refresh PR checks.

## GitHub Validation

PR `#150` passed required checks before merge:

- `Security Guard`
- `Server Clippy + Check`
- `Desktop Rust Clippy`
- `Frontend Lint + Typecheck`
- `Website Lint + Typecheck + Build`
- `Workflow Lint`
- `Validate quality_gates warn/block matrix`
- `Sonar Scan + Quality Gate`
- `Block internal-assistant markers in branch/commits`
- `Vercel`
- `Vercel Preview Comments`

Post-merge validation for commit `cba3f9d` passed:

- `CI` - run `25209198316`
- `Release Readiness Gate` - run `25209198277`
