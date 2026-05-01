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

PR `#164` merged on `main` as `e5c259d`.

PR checks passed before merge:

- `Security Guard`: passed.
- `Server Clippy + Check`: passed.
- `Desktop Rust Clippy`: passed.
- `Frontend Lint + Typecheck`: passed.
- `Website Lint + Typecheck + Build`: passed.
- `Workflow Lint`: passed.
- `Validate quality_gates warn/block matrix`: passed.
- `Sonar Scan + Quality Gate`: passed.
- `Block internal-assistant markers in branch/commits`: passed.
- `Vercel`: passed.
- `Vercel Preview Comments`: passed.

Post-merge validation for commit `e5c259d` passed:

- `CI` - run `25212383270`.
- `Release Readiness Gate` - run `25212383263`.
- `Quality Gate Policy Matrix (Optional)` - run `25212383274`.
- `Secret Scan` - run `25212383265`.
- `Public Naming Guard` - run `25212383284`.
- `Governance Correlation Smoke (Optional)` - run `25212383273`.
- `Desktop Updater Readiness (Optional)` - run `25212383275`.
- `SonarQube Governance (Non-Blocking)` - run `25212383279`.

First manual trend workflow validation passed:

- Workflow: `Enterprise Onboarding Readiness Trend Report`.
- Run: `25212387234`.
- Artifact: `enterprise-onboarding-readiness-trend-report`.
- Artifact ID: `6748686954`.
- Artifact status: not expired.
- Artifact expires at `2026-07-30T11:15:52Z`.

## Current Status

KAN-55 implementation is complete and merged through PR `#164`.

The trend report is operational on `main`: it parses recent KAN-53 readiness artifacts and reports score/status deltas without reading provider secrets, mutating repositories/providers, or treating onboarding `needs-action` as a release-blocking failure.
