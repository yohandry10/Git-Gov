# KAN-55 Enterprise Onboarding Readiness Trend Report

Updated: 2026-05-01

## Summary

KAN-55 adds a trend report for KAN-53 Enterprise Onboarding Readiness artifacts.

The trend shows whether onboarding readiness is improving, declining, or stable across recent readiness artifacts. It does not enforce release blocking and does not treat `needs-action` as a deployment failure by default.

## Changes

| Area | Change |
| --- | --- |
| Script | Added `scripts/control-plane/generate_enterprise_onboarding_readiness_trend_report.ps1`. |
| GitHub Actions | Added `.github/workflows/enterprise-onboarding-readiness-trend-report.yml`. |
| Documentation | Added design/report docs and updated the Enterprise Self-Service runbook and roadmap. |

## Trend Policy

| Setting | Value |
| --- | --- |
| Source workflow | `enterprise-onboarding-readiness.yml` |
| Source artifact prefix | `enterprise-onboarding-readiness-` |
| Trend artifact | `enterprise-onboarding-readiness-trend-report` |
| Default max reports | `12` |
| Scheduled run | Thursday `14:17 UTC` |
| Manual input | `max_reports` |

## Safety

- No `.env` files are read.
- No provider tokens are read.
- No Authorization headers are printed.
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
| `.\scripts\control-plane\generate_enterprise_onboarding_readiness_trend_report.ps1 -Repository yohandry10/Git-Gov -WorkflowFile enterprise-onboarding-readiness.yml -ArtifactNamePrefix enterprise-onboarding-readiness- -MaxReports 12 -OutputMarkdownPath out\KAN-55-onboarding-readiness-trend.md -OutputJsonPath out\KAN-55-onboarding-readiness-trend.json` | PASS. Parsed run `25211644692`, artifact `enterprise-onboarding-readiness-25211644692`, latest status `needs-action`, score `75`, trend `stable`, `3` ready stages, `3` needs-action stages, `0` blocked stages. |
| `git diff --check` | PASS. |
| `.\scripts\security\publication_guard.ps1` | PASS. |

Remaining validation before closure:

- PR checks.
- first manual trend workflow run on `main` after merge.

## Current Status

Implementation in progress on branch `ops/KAN-55-enterprise-onboarding-readiness-trend`.
