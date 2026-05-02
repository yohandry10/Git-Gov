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

PR `#182` merged on `main` as `45eec39`.

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

Post-merge validation for commit `45eec39` passed:

- `CI` - run `25247275328`.
- `Release Readiness Gate` - run `25247275146`.
- `Quality Gate Policy Matrix (Optional)` - run `25247275134`.
- `Secret Scan` - run `25247275150`.
- `Public Naming Guard` - run `25247275137`.
- `Governance Correlation Smoke (Optional)` - run `25247275132`.
- `Desktop Updater Readiness (Optional)` - run `25247275133`.
- `SonarQube Governance (Non-Blocking)` - run `25247275143`.

First manual trend workflow validation passed:

- Workflow: `Enterprise Route Auth Smoke Trend Report`.
- Run: `25247310737`.
- Artifact: `enterprise-route-auth-smoke-trend-report`.
- Artifact ID: `6761702022`.
- Artifact status: not expired.
- Artifact expires at `2026-07-31T07:53:51Z`.
- Parsed latest successful smoke run: `25246304135`.
- Parsed latest check counts: `9` passed, `0` failed.

## Current Status

KAN-64 implementation is complete and merged through PR `#182`.

The trend report is operational on `main`: it parses recent KAN-62 auth-smoke artifacts and reports whether route auth behavior is stable, improving, declining, or failing without reading provider secrets, probing routes directly, or making release blocking the default.
