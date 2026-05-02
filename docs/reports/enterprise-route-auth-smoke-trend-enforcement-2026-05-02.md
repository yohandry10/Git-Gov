# KAN-66 Enterprise Route Auth Smoke Trend Enforcement

Updated: 2026-05-02

## Summary

KAN-66 turns the Enterprise Route Auth Smoke trend from informational evidence into an optional enforcement gate.

## Why

KAN-62 generates auth-smoke evidence.
KAN-63 verifies smoke artifact freshness.
KAN-64 builds trend evidence across smoke artifacts.
KAN-65 verifies trend artifact freshness.

KAN-66 adds the next control: fail the enforcement workflow when the latest trend is no longer healthy enough for the configured baseline.

## Enforcement Rules

Default rules:

- Latest parsed trend report must have `0` failed checks.
- Failed-check count must not increase versus the oldest analyzed report.
- The latest successful auth-smoke workflow run must have a parseable source artifact.

## Files

- `.github/workflows/enterprise-route-auth-smoke-trend-enforcement.yml`.
- `scripts/control-plane/generate_enterprise_route_auth_smoke_trend_report.ps1`.
- `docs/design/enterprise-route-auth-smoke-trend-enforcement-mvp.md`.
- `docs/runbooks/enterprise-self-service-adoption.md`.

## Validation

Local validation before PR:

- Command: `.\scripts\control-plane\generate_enterprise_route_auth_smoke_trend_report.ps1 -Repository yohandry10/Git-Gov -WorkflowFile enterprise-route-auth-smoke.yml -ArtifactNamePrefix enterprise-route-auth-smoke- -MaxReports 12 -Enforce -MaxLatestFailedChecks 0 -FailOnFailureIncrease -RequireLatestRunArtifact -OutputMarkdownPath out\KAN-66-enterprise-route-auth-smoke-trend-enforcement.md -OutputJsonPath out\KAN-66-enterprise-route-auth-smoke-trend-enforcement.json`.
- Enforcement result: `pass`.
- Reports analyzed: `1`.
- Latest successful source run: `25246304135`.
- Latest source artifact status: `parsed`.
- Latest counts: `9` passed, `0` failed.
- Failed-check delta vs oldest report: `0`.
- `git diff --check`: passed.
- `.\scripts\security\publication_guard.ps1`: passed.

## Current Status

KAN-66 is in progress.
