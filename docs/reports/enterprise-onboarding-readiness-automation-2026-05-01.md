# Enterprise Onboarding Readiness Automation

Updated: 2026-05-01

Ticket: `KAN-53`

## Summary

KAN-53 automates the KAN-52 Enterprise onboarding readiness report through GitHub Actions.

The workflow creates recurring or manual evidence artifacts that show whether a customer adoption profile is ready, needs action, or blocked.

## Implementation

Files:

- `.github/workflows/enterprise-onboarding-readiness.yml`
- `docs/design/enterprise-onboarding-readiness-automation-mvp.md`
- `docs/reports/enterprise-onboarding-readiness-automation-2026-05-01.md`
- `docs/runbooks/enterprise-self-service-adoption.md`
- `docs/design/enterprise-self-service-and-ai-copilot-roadmap.md`

Workflow behavior:

- manual `workflow_dispatch` inputs define a temporary adoption profile.
- scheduled run uses safe defaults for the GitGov repository.
- profile and generated pack files are written only in runner temp/output directories.
- optional remote workflow readiness uses KAN-51 read-only validation.
- KAN-52 generates Markdown/JSON readiness output.
- artifact name is `enterprise-onboarding-readiness-{run_id}`.

## Safety

The workflow:

- does not read `.env` files.
- does not print secret values.
- does not create GitHub Actions variables or secrets.
- does not mutate customer repositories.
- does not mutate provider settings.
- does not open PRs.
- does not dispatch workflows.
- does not change branch protection.
- keeps release blocking opt-in only.

## Validation

Local validation:

- adoption pack generation for GitGov profile: passed.
- workflow template generation for GitGov profile: passed.
- default KAN-53 readiness generation without remote readiness:
  - output directory `out/KAN-53-onboarding-readiness-default`.
  - result `needs-action`, score `75`, `3` ready stages, `3` needs-action stages, `0` blocked stages.
- optional KAN-51 remote workflow readiness path:
  - output `out/KAN-53-workflow-readiness.json`.
  - result `needs-action`, `workflows_missing=0`, `workflows_different=13`, `variables_missing=0`, `secrets_missing=1`.
  - expected because the current GitGov repo workflows are not identical to a freshly generated customer template pack.
- KAN-53 readiness generation with remote readiness input:
  - output directory `out/KAN-53-onboarding-readiness-remote`.
  - result `needs-action`, score `75`, `3` ready stages, `3` needs-action stages, `0` blocked stages.
- generated output scan for `Authorization`, `Bearer`, `GITGOV_API_KEY=`, `SONAR_TOKEN=`, `ATATT`, and `vck_`: passed with no matches.
- `git diff --check`: passed.
- `scripts/security/publication_guard.ps1`: passed.

Remaining validation before closure:

- PR checks.
- first manual workflow run on `main` after merge.

## Current Status

Implementation in progress on branch `product/KAN-53-enterprise-onboarding-readiness-automation`.
