# Enterprise Onboarding Readiness Report

Updated: 2026-05-01

Ticket: `KAN-52`

## Summary

KAN-52 adds a consolidated readiness snapshot for Enterprise Self-Service Adoption.

The new report turns the existing onboarding artifacts into a single customer-facing view:

- adoption profile validity.
- provider connection/evidence status.
- workflow template pack status.
- remote workflow readiness status.
- GitHub Actions variable/secret name readiness.
- release governance policy posture.

## Implementation

Files:

- `gitgov/src/components/control_plane/dashboard-helpers.ts`
- `gitgov/src/components/control_plane/EnterpriseAdoptionPanel.tsx`
- `gitgov/src/test/components/dashboard-helpers.test.ts`
- `scripts/control-plane/generate_enterprise_onboarding_readiness_report.ps1`
- `docs/design/enterprise-onboarding-readiness-report-mvp.md`
- `docs/runbooks/enterprise-self-service-adoption.md`

Dashboard behavior:

- adds an `Onboarding` readiness card in the Enterprise Adoption panel.
- adds a `Readiness` JSON download action.
- keeps the dashboard export local and secret-safe.

CLI behavior:

- generates `enterprise-onboarding-readiness.md`.
- generates `enterprise-onboarding-readiness.json`.
- can generate an adoption pack from profile input if no pack path is supplied.
- can consume existing KAN-36 provider connection reports.
- can consume existing KAN-51 workflow readiness reports.
- exits non-zero on not-ready status unless `-ReportOnly` is passed.

## Safety

The KAN-52 report:

- does not read secret values.
- does not print secret values.
- does not create GitHub Actions variables or secrets.
- does not mutate customer repositories.
- does not mutate provider state.
- does not open PRs.
- does not dispatch workflows.
- does not change branch protection.
- does not make release blocking the default.

## Validation

Local validation:

- PowerShell parser check for `generate_enterprise_onboarding_readiness_report.ps1`: passed.
- `npm test -- --run src/test/components/dashboard-helpers.test.ts`: passed, `22` tests.
- `npm run typecheck`: passed.
- `npm run lint`: passed.
- `npm test -- --run`: passed, `25` test files and `290` tests.
- `npm run build`: passed with the existing Vite large chunk warning.
- profile-only onboarding readiness generation:
  - command used `-ProfilePath docs/examples/enterprise-adoption-profile.example.json`.
  - output directory `out/KAN-52-onboarding-readiness-profile`.
  - result `needs-action`, readiness score `75`, `3` ready stages, `3` needs-action stages, `0` blocked stages.
- provider connection validation input:
  - output `out/KAN-52-provider-connections.json`.
  - selected providers `github,jira`.
  - result `ready`, `2` ready provider checks, `0` missing config, `0` failed.
- remote workflow readiness input:
  - output `out/KAN-52-workflow-readiness.json`.
  - result `needs-action`, `workflows_missing=0`, `workflows_different=13`, `variables_missing=0`, `secrets_missing=1`.
  - this is expected against the GitGov repo because the current repo workflows are not identical to the freshly generated customer template pack.
- consolidated onboarding readiness generation with provider/workflow inputs:
  - output directory `out/KAN-52-onboarding-readiness-full`.
  - result `needs-action`, readiness score `83`, `4` ready stages, `2` needs-action stages, `0` blocked stages.
- generated output scan for `Authorization`, `Bearer`, `GITGOV_API_KEY=`, `SONAR_TOKEN=`, `ATATT`, and `vck_`: passed with no matches.

- `git diff --check`: passed.
- `scripts/security/publication_guard.ps1`: passed.

## Current Status

PR `#158` merged on `main` as `268770a`.

Post-merge validation passed:

- `CI` - run `25211254174`.
- `Release Readiness Gate` - run `25211254160`.
- `Quality Gate Policy Matrix (Optional)` - run `25211254185`.
- `Secret Scan` - run `25211254159`.
- `Public Naming Guard` - run `25211254165`.
- `Governance Correlation Smoke (Optional)` - run `25211254168`.
- `Desktop Updater Readiness (Optional)` - run `25211254172`.
- `SonarQube Governance (Non-Blocking)` - run `25211254202`.

No database migration, Render deploy, Vercel production environment change, GitHub Actions secret/variable creation, branch protection mutation, provider mutation, remote apply run, or workflow dispatch was needed.
