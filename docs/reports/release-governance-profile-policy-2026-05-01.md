# KAN-45 Release Governance Profile Policy

Updated: 2026-05-01

## Summary

KAN-45 adds explicit release governance policy configuration to the Enterprise Self-Service Adoption path.

The important product rule is preserved: GitGov defaults to `record-only`. Blocking releases, requiring formal approval, or requiring quorum remains an explicit customer choice.

## Traceability

- Jira issue: `KAN-45 - Add configurable release governance profile policy`.
- Branch: `product/KAN-45-release-governance-profile-policy`.
- PR: `#144 - product(KAN-45): add release governance profile policy`.
- Merge commit: `dc37e9286d0be7159d3b6fb4c799e42862b22f3a`.
- Design: `docs/design/release-governance-profile-policy-mvp.md`.

## Changes

- Added `release_governance` to the adoption profile model used by the dashboard.
- Added dashboard controls for release governance mode and environment.
- Added safe normalization helpers for `record-only`, `advisory`, `approval-required`, and `quorum-required`.
- Added backend validation before adoption profiles are persisted.
- Added `vulnerability-review` and `artifact-monitoring` to backend accepted adoption modules while keeping the legacy `security-review` value accepted for compatibility.
- Updated the CLI adoption pack generator to include release governance in Markdown and JSON output.
- Updated the CLI workflow template generator to include release governance in README and manifest output.
- Updated the example enterprise adoption profile with the safe `record-only` policy.
- Added focused frontend and backend tests.

## Security And Adoption Notes

- Default release governance remains non-blocking.
- Non-`record-only` modes require the `formal-approval` module.
- `record-only` cannot use blocking enforcement.
- Quorum rules are accepted only in `quorum-required` mode.
- No provider tokens, `.env` values, Authorization headers, or secret payloads are read or printed.
- Generated packs contain policy intent and secret names only, not secret values.

## Local Validation

Completed locally:

- `cargo fmt` from `gitgov/gitgov-server`: passed.
- `cargo check` from `gitgov/gitgov-server`: passed.
- `cargo clippy -- -D warnings` from `gitgov/gitgov-server`: passed.
- `cargo test adoption_profile_tests` from `gitgov/gitgov-server`: passed, `6` tests.
- `npm test -- --run src/test/components/dashboard-helpers.test.ts` from `gitgov`: passed, `15` tests.
- `npm run lint` from `gitgov`: passed.
- `npm run typecheck` from `gitgov`: passed.
- `npm run build` from `gitgov`: passed with the existing Vite large chunk warning.
- `.\scripts\control-plane\generate_enterprise_adoption_pack.ps1 -ProfilePath docs\examples\enterprise-adoption-profile.example.json -OutputDir out\KAN-45-enterprise-adoption-pack`: passed.
- `.\scripts\control-plane\generate_enterprise_workflow_templates.ps1 -ProfilePath docs\examples\enterprise-adoption-profile.example.json -OutputDir out\KAN-45-enterprise-workflow-templates -Force`: passed.
- `git diff --check`: passed.
- `.\scripts\security\publication_guard.ps1`: passed.

Generated output confirmed:

- Adoption pack Markdown/JSON: `record-only`, `disabled`, quorum `disabled`.
- Workflow template README/manifest: `record-only`, `disabled`, quorum `disabled`.

## GitHub Validation

PR `#144` checks passed before merge:

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

Post-merge checks passed on `main` commit `dc37e9286d0be7159d3b6fb4c799e42862b22f3a`:

- `CI` run `25203785504`.
- `Release Readiness Gate` run `25203785499`.
- `Quality Gate Policy Matrix (Optional)` run `25203785520`.
- `Secret Scan` run `25203785497`.
- `SonarQube Governance (Non-Blocking)` run `25203785527`.
- `Public Naming Guard` run `25203785483`.
- `Governance Correlation Smoke (Optional)` run `25203785490`.
- `Desktop Updater Readiness (Optional)` run `25203785503`.

## Deployment

- No database migration was needed.
- No Render deploy validation was needed.
- No Vercel production environment change was needed.
- No provider setting changed.
- No customer workflow installation was triggered.

## Residual Work

- Implement actual release gate enforcement only when a customer-selected policy enables it.
- Implement full quorum evaluation only when a customer-selected policy enables it.
- Consider per-environment and per-risk-level policy expansion after the MVP profile shape is stable.
