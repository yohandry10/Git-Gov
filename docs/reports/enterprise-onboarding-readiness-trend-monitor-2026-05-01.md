# KAN-56 Enterprise Onboarding Readiness Trend Monitor

Updated: 2026-05-01

## Summary

KAN-56 adds a non-blocking-by-default monitor for the KAN-55 Enterprise Onboarding Readiness trend artifact.

The monitor checks whether trend evidence is fresh and parseable, then reports deterioration conditions such as blocked stages, score drops, or score below the configured threshold. It does not mutate customer repositories or providers and does not turn onboarding readiness into release enforcement by default.

## Changes

| Area | Change |
| --- | --- |
| Script | Added `scripts/control-plane/validate_enterprise_onboarding_readiness_trend_monitor.ps1`. |
| GitHub Actions | Added `.github/workflows/enterprise-onboarding-readiness-trend-monitor.yml`. |
| Documentation | Added design/report docs and updated the Enterprise Self-Service runbook and roadmap. |

## Monitor Policy

| Setting | Value |
| --- | --- |
| Source workflow | `enterprise-onboarding-readiness-trend-report.yml` |
| Source artifact | `enterprise-onboarding-readiness-trend-report` |
| Monitor artifact | `enterprise-onboarding-readiness-trend-monitor` |
| Default max artifact age | `192` hours |
| Default minimum latest score | `75` |
| Scheduled run | Thursday `14:27 UTC` |
| Default mode | `report_only=true` |
| Release blocking default | `false` |

## Finding Semantics

| Status | Meaning |
| --- | --- |
| `ready` | Trend artifact is fresh and no deterioration rule fired. |
| `needs-action` | Trend evidence is parseable, but customer onboarding readiness should be reviewed. |
| `blocked` | Monitor cannot trust trend evidence because it is missing, stale, expired, or not parseable. |

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
| `.\scripts\control-plane\validate_enterprise_onboarding_readiness_trend_monitor.ps1 -Repository yohandry10/Git-Gov -WorkflowFile enterprise-onboarding-readiness-trend-report.yml -ArtifactName enterprise-onboarding-readiness-trend-report -MaxAgeHours 192 -MinLatestScore 75 -ReportOnly -OutputMarkdownPath out\KAN-56-onboarding-readiness-trend-monitor.md -OutputJsonPath out\KAN-56-onboarding-readiness-trend-monitor.json` | PASS. Monitor status `ready`; source trend run `25212387234`; trend artifact `enterprise-onboarding-readiness-trend-report`; artifact ID `6748686954`; latest readiness status `needs-action`; latest score `75`; trend `stable`; blocked stages `0`; findings `0`. |
| Same monitor without `-ReportOnly` against the current healthy trend artifact | PASS. Strict mode exited `0` because monitor status was `ready`. |
| PowerShell parser check for `validate_enterprise_onboarding_readiness_trend_monitor.ps1` | PASS. |
| `git diff --check` | PASS. |
| `.\scripts\security\publication_guard.ps1` | PASS. |

PR validation and first workflow run will be appended after merge.

## Current Status

KAN-56 implementation is in progress.

The local monitor validates the current KAN-55 trend artifact and reports `ready` while preserving the customer-safe default: report-only evidence, no provider secrets, no repository mutation, and no release blocking unless explicitly configured by an operator.
