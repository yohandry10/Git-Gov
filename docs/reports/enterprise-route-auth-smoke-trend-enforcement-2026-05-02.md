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

PR `#186` merged on `main` as `004eeea`.

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

Post-merge validation for commit `004eeea` passed:

- `CI` - run `25247711351`.
- `Release Readiness Gate` - run `25247711349`.
- `Quality Gate Policy Matrix (Optional)` - run `25247711335`.
- `Secret Scan` - run `25247711345`.
- `Public Naming Guard` - run `25247711342`.
- `Governance Correlation Smoke (Optional)` - run `25247711348`.
- `Desktop Updater Readiness (Optional)` - run `25247711338`.
- `SonarQube Governance (Non-Blocking)` - run `25247711340`.

First manual enforcement workflow validation passed:

- Workflow: `Enterprise Route Auth Smoke Trend Enforcement`.
- Run: `25247747284`.
- Artifact: `enterprise-route-auth-smoke-trend-enforcement`.
- Artifact ID: `6761818040`.
- Artifact status: not expired.
- Artifact expires at `2026-07-31T08:19:25Z`.
- Enforcement status: `pass`.
- Parsed latest successful source run: `25246304135`.
- Parsed latest counts: `9` passed, `0` failed.

## Current Status

KAN-66 is implemented, merged, workflow-validated, and documented.
