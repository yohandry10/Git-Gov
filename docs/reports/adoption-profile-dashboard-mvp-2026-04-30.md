# KAN-30 Adoption Profile Dashboard MVP

Updated: 2026-04-30

## Summary

KAN-30 adds the first UI layer for Enterprise Self-Service Adoption.

KAN-29 created a reproducible adoption pack generator. KAN-30 brings the same concept into the admin dashboard so an operator can choose providers, modules, and policy preset, then download a secret-safe JSON adoption pack.

## Changes

- Added `EnterpriseAdoptionPanel` to the admin Control Plane dashboard.
- Added typed adoption profile/pack helpers in `dashboard-helpers.ts`.
- Added validation for customer name, repository shape, default branch, Jira key, provider selection, and module selection.
- Added tests that verify:
  - the default moderate profile generates `13` workflow recommendations.
  - only variable/secret names are emitted, not secret values.
  - strict preset requires PR review and trend enforcement.
  - formal release approval remains visible as an open product gap.
  - invalid profile inputs are rejected.
- Updated product roadmap and operating memory.

## Product Status

Implemented:

- UI profile builder.
- Policy preset selection.
- Provider/module toggles.
- Live generated workflow/policy/config preview.
- JSON pack download.
- Secret-safe output.

Still future work:

- persisted customer adoption profiles.
- provider health validation.
- workflow template installation.
- formal enterprise release approval.
- Vercel AI SDK Copilot.

## Validation

Local validation passed:

- `npm test -- --run src/test/components/dashboard-helpers.test.ts`
  - `8` tests passed.
- `npm test -- --run`
  - `25` files passed.
  - `271` tests passed.
- `npm run typecheck`
  - passed.
- `npm run lint`
  - passed.
- `npm run build`
  - passed.
  - Vite reported the existing large chunk warning.
- `git diff --check`
  - passed.
- `.\scripts\security\publication_guard.ps1`
  - passed.
- Browser smoke at `http://127.0.0.1:5174/`
  - page title `GitGov`.
  - app shell loaded.
  - console error count: `0`.
  - note: the Enterprise Adoption panel is admin-dashboard gated, so the unauthenticated browser smoke only validates app load and console health.

PR validation:

- PR: `#110` - `product(KAN-30): add adoption profile dashboard MVP`.
- Merge commit: `0412574`.
- PR checks passed:
  - `Security Guard`.
  - `Server Clippy + Check`.
  - `Desktop Rust Clippy`.
  - `Frontend Lint + Typecheck`.
  - `Website Lint + Typecheck + Build`.
  - `Workflow Lint`.
  - `Validate quality_gates warn/block matrix`.
  - `Sonar Scan + Quality Gate`.
  - `Vercel`.

Post-merge `main` checks passed:

- `CI` run `25161644820`.
- `Release Readiness Gate` run `25161644879`.
- `Quality Gate Policy Matrix (Optional)` run `25161644854`.
- `Secret Scan` run `25161644841`.
- `SonarQube Governance (Non-Blocking)` run `25161644861`.
- `Public Naming Guard` run `25161644857`.
- `Governance Correlation Smoke (Optional)` run `25161644871`.
- `Desktop Updater Readiness (Optional)` run `25161644824`.
