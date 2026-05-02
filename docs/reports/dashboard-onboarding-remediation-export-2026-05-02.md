# KAN-58 Dashboard Onboarding Remediation Export

Updated: 2026-05-02

## Summary

KAN-58 exposes the KAN-57 remediation plan from the Enterprise Adoption dashboard.

The dashboard now builds a remediation plan JSON from the current onboarding readiness report and adoption pack. It gives self-service users the same action list available from the CLI: priority, stage, owner, action, validation evidence, and placeholder-only GitHub Actions configuration commands.

## Changes

| Area | Change |
| --- | --- |
| Dashboard helpers | Added `buildEnterpriseOnboardingRemediationPlan` and `buildEnterpriseOnboardingRemediationPlanFilename`. |
| Dashboard UI | Added a `Plan` JSON download action in `EnterpriseAdoptionPanel`. |
| Tests | Added dashboard helper tests for remediation plan actions, placeholder commands, safety flags, and filename. |
| Documentation | Added design/report docs and updated the Enterprise Self-Service runbook and roadmap. |

## Safety

- No `.env` files are read.
- No provider tokens are read.
- No provider APIs are called.
- No secret values are printed.
- GitHub Actions secret names may be listed, but values are never read or generated.
- Placeholder commands use `<value>` only.
- No GitHub Actions variables or secrets are created.
- No customer repositories are mutated.
- No provider settings are mutated.
- No branch protection is changed.
- No workflow dispatch occurs.
- Release blocking remains opt-in only.

## Validation

Local validation before PR:

| Command | Result |
| --- | --- |
| `npm test -- --run src/test/components/dashboard-helpers.test.ts` | PASS. `24` tests. |
| `npm run typecheck` | PASS. |
| `npm run lint` | PASS. |
| `npm test -- --run` | PASS. `25` test files, `292` tests. |
| `npm run build` | PASS with existing Vite large chunk warning. |
| `git diff --check` | PASS. |
| `.\scripts\security\publication_guard.ps1` | PASS. |

PR validation:

| Check | Result |
| --- | --- |
| PR | PASS. `#170` - `product(KAN-58): export onboarding remediation plan`. |
| Implementation commit | PASS. `43ac78e product(KAN-58): export onboarding remediation plan`. |
| `Security Guard` | PASS. |
| `Server Clippy + Check` | PASS. |
| `Desktop Rust Clippy` | PASS. |
| `Frontend Lint + Typecheck` | PASS. |
| `Website Lint + Typecheck + Build` | PASS. |
| `Workflow Lint` | PASS. |
| `Validate quality_gates warn/block matrix` | PASS. |
| `Sonar Scan + Quality Gate` | PASS. |
| `Block internal-assistant markers in branch/commits` | PASS. |
| `Vercel` | PASS. |
| `Vercel Preview Comments` | PASS. |

Post-merge evidence:

| Check | Result |
| --- | --- |
| Main merge commit | PASS. `4f0eff5 Merge pull request #170 from yohandry10/product/KAN-58-dashboard-onboarding-remediation-export`. |
| `CI` | PASS. Run `25243856927`. |
| `Release Readiness Gate` | PASS. Run `25243856920`. |
| `Quality Gate Policy Matrix (Optional)` | PASS. Run `25243856933`. |
| `Secret Scan` | PASS. Run `25243856930`. |
| `Public Naming Guard` | PASS. Run `25243856934`. |
| `Governance Correlation Smoke (Optional)` | PASS. Run `25243856931`. |
| `Desktop Updater Readiness (Optional)` | PASS. Run `25243856923`. |
| `SonarQube Governance (Non-Blocking)` | PASS. Run `25243856915`. |

## Current Status

KAN-58 implementation is merged on `main`.

The dashboard can build and download a secret-safe onboarding remediation plan JSON without mutating repositories/providers or changing release-blocking defaults.

No database migration, Render deploy, Vercel production environment change, GitHub Actions secret/variable creation, branch protection mutation, provider mutation, customer repository mutation, remote apply run, workflow dispatch against customer repositories, or provider webhook mutation was needed.
