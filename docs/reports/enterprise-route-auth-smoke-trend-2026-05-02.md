# KAN-64 Enterprise Route Auth Smoke Trend Report

Updated: 2026-05-02

## Summary

KAN-64 adds multi-run trend reporting for Enterprise route auth smoke artifacts.

The trend shows whether auth-smoke route behavior is stable, improving, declining, or actively failing across recent workflow artifacts. It does not make release blocking the default.

## Changes

| Area | Change |
| --- | --- |
| Script | Added `scripts/control-plane/generate_enterprise_route_auth_smoke_trend_report.ps1`. |
| GitHub Actions | Added `.github/workflows/enterprise-route-auth-smoke-trend-report.yml`. |
| Documentation | Added design/report docs and updated the Enterprise Self-Service runbook. |

## Trend Policy

| Setting | Value |
| --- | --- |
| Source workflow | `enterprise-route-auth-smoke.yml` |
| Source artifact prefix | `enterprise-route-auth-smoke-` |
| Trend artifact | `enterprise-route-auth-smoke-trend-report` |
| Default max reports | `12` |
| Scheduled run | Wednesday `15:17 UTC` |
| Manual input | `max_reports` |

## Safety

- No `.env` files are read by the trend workflow.
- No provider tokens are read.
- No Authorization headers are printed.
- No GitHub Actions variables or secrets are created.
- No customer repositories are mutated.
- No provider settings are mutated.
- No branch protection is changed.
- Release blocking remains opt-in only and is not enabled by this report.

## Validation

Local validation before PR:

| Command | Result |
| --- | --- |
| `.\scripts\control-plane\generate_enterprise_route_auth_smoke_trend_report.ps1 -Repository yohandry10/Git-Gov -WorkflowFile enterprise-route-auth-smoke.yml -ArtifactNamePrefix enterprise-route-auth-smoke- -MaxReports 12 -GitHubToken <redacted> -OutputMarkdownPath out\KAN-64-enterprise-route-auth-smoke-trend.md -OutputJsonPath out\KAN-64-enterprise-route-auth-smoke-trend.json` | PASS. Parsed run `25246304135`, artifact `enterprise-route-auth-smoke-25246304135`, artifact ID `6761394808`, latest status `passed`, `9` passed checks, `0` failed checks, trend `stable`. |
| `git diff --check` | PASS. |
| `.\scripts\security\publication_guard.ps1` | PASS. |

PR and post-merge validation will be added after implementation merge.

## Current Status

KAN-64 implementation is in progress.
