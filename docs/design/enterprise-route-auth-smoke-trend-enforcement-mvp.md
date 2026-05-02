# KAN-66 Enterprise Route Auth Smoke Trend Enforcement MVP

Updated: 2026-05-02

## Goal

Turn the auth-smoke trend from informational evidence into an optional enforcement gate.

This is the next control after:

- KAN-62: smoke evidence generation.
- KAN-63: smoke artifact freshness monitoring.
- KAN-64: trend reporting.
- KAN-65: trend artifact freshness monitoring.

## Enforcement Rules

Default rules:

- latest parsed trend report must have `0` failed checks.
- failed-check count must not increase versus the oldest analyzed parsed report.
- the latest successful auth-smoke workflow run must still have a parseable source artifact.

## Scope

- Extend `scripts/control-plane/generate_enterprise_route_auth_smoke_trend_report.ps1` with enforcement options.
- Add `.github/workflows/enterprise-route-auth-smoke-trend-enforcement.yml`.
- Upload `enterprise-route-auth-smoke-trend-enforcement` artifact with Markdown and JSON outputs.

## Safety

- GitHub Actions read-only permissions only.
- Reads artifacts only; no direct probing of GitGov routes.
- No `.env` reads.
- No provider secret reads or prints.
- No repo mutation, branch protection changes, or provider mutation.
- Enforcement is opt-in at workflow level; it does not globally change release defaults.

## Scheduling

- Runs on Friday `15:17 UTC`, after the trend report and its freshness monitor.
