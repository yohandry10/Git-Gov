# KAN-34 Dashboard Workflow Template Pack

Updated: 2026-04-30

## Summary

KAN-34 moves workflow-template onboarding from CLI-only generation into the Enterprise Adoption dashboard.

The dashboard now has a `Workflows` download action that builds a secret-safe workflow template pack from the current adoption profile. This keeps Vercel AI SDK Copilot pending and continues closing Enterprise Self-Service Onboarding first.

## Changes

- Added workflow template pack types and builder helpers in `dashboard-helpers.ts`.
- Added generated workflow file contents for the same workflow families introduced in KAN-33.
- Added `buildEnterpriseWorkflowTemplatePackFilename`.
- Added a `Workflows` download button in `EnterpriseAdoptionPanel.tsx`.
- Added focused tests for:
  - generated workflow template count.
  - manifest safety flags.
  - no unresolved template tokens.
  - no secret assignment strings.
  - stable workflow pack filename.

## Safety

The dashboard workflow pack:

- does not read local `.env` files.
- does not read provider credentials.
- does not print or store provider secret values.
- does not mutate GitHub repositories.
- contains secret names and GitHub Actions secret references only.

Automatic workflow installation remains future work and must require explicit authorization.

## Validation

Local validation passed:

- `npm test -- --run src/test/components/dashboard-helpers.test.ts`
  - `13` tests passed.
- `npm run typecheck`
  - passed.
- `npm run lint`.
  - passed.
- `npm test -- --run`.
  - `25` files passed.
  - `276` tests passed.
- `npm run build`.
  - passed.
  - Vite reported the existing large chunk warning.
- `git diff --check`.
  - passed.
- `.\scripts\security\publication_guard.ps1`.
  - passed.
- Targeted secret-pattern scan over KAN-34 files returned no matches for committed secret assignments.

## PR Validation

- PR: `#119` - `product(KAN-34): add dashboard workflow template pack`.
- Merge commit: `31b109d`.
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

- `CI` run `25190963652`.
- `Release Readiness Gate` run `25190963636`.
- `Quality Gate Policy Matrix (Optional)` run `25190963623`.
- `Secret Scan` run `25190963646`.
- `SonarQube Governance (Non-Blocking)` run `25190963649`.
- `Public Naming Guard` run `25190963657`.
- `Governance Correlation Smoke (Optional)` run `25190963633`.
- `Desktop Updater Readiness (Optional)` run `25190963664`.

## Remaining Product Work Before AI SDK

- Explicitly authorized workflow installation into customer repositories.
- Direct provider credential/reachability checks.
- Formal enterprise release approval.

Vercel AI SDK Copilot remains pending until those onboarding surfaces are complete enough to explain a full adoption state.
