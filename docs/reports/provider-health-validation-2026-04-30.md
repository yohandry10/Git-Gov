# KAN-32 Provider Health Validation

Updated: 2026-04-30

## Summary

KAN-32 adds a Provider Health section to the Enterprise Adoption dashboard.

The MVP turns the persisted adoption profile into an operator-facing readiness view: for every selected provider, GitGov now shows whether the provider is ready, needs more telemetry evidence, or needs configuration intent before it can be treated as adoption-ready.

## Changes

- Added provider health helper types and builder in `dashboard-helpers.ts`.
- Added secret-safe evidence inputs for:
  - GitHub event totals.
  - Jira ticket coverage.
  - Jenkins pipeline runs.
  - Sonar/quality runs inferred from Jenkins correlations.
  - active repository count.
- Added Provider Health cards to `EnterpriseAdoptionPanel.tsx`.
- Added focused helper tests covering:
  - ready provider checks when evidence exists.
  - needs-evidence state when telemetry has not arrived.
  - needs-config state when Jira project key is missing.

## Safety

No provider credentials are read, stored, printed, or displayed.

This MVP does not validate raw secrets. It validates adoption readiness from profile intent plus existing GitGov evidence.

## Validation

Local validation passed:

- `npm test -- --run src/test/components/dashboard-helpers.test.ts`
  - `11` tests passed.
- `npm run typecheck`
  - passed.
- `npm run lint`
  - passed.
- `npm test -- --run`
  - `25` files passed.
  - `274` tests passed.
- `npm run build`
  - passed.
  - Vite reported the existing large chunk warning.
- `git diff --check`
  - passed.
- `.\scripts\security\publication_guard.ps1`
  - passed.

## PR Validation

- PR: `#115` - `product(KAN-32): add provider health validation`.
- Merge commit: `1a16d88`.
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

- `CI` run `25188414404`.
- `Release Readiness Gate` run `25188414418`.
- `Quality Gate Policy Matrix (Optional)` run `25188414443`.
- `Secret Scan` run `25188414428`.
- `SonarQube Governance (Non-Blocking)` run `25188414417`.
- `Public Naming Guard` run `25188414424`.
- `Governance Correlation Smoke (Optional)` run `25188414421`.
- `Desktop Updater Readiness (Optional)` run `25188414432`.

## Remaining Product Work

- Direct provider connection checks with explicit customer authorization.
- Customer workflow template installation.
- Formal enterprise release approval.
- Vercel AI SDK Copilot over adoption readiness, provider health, evidence packets, and security findings.
