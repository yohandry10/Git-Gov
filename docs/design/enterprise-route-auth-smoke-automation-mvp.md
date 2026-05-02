# KAN-62 Enterprise Route Auth Smoke Automation MVP

Updated: 2026-05-02

## Summary

KAN-62 turns the KAN-61 manual production smoke into repeatable evidence.

The workflow validates that Enterprise admin routes keep the expected basic auth behavior:

- public health stays available.
- anonymous Enterprise requests are rejected.
- authenticated admin Enterprise requests still work.

This is operational hardening, not a new release-blocking product default.

## Components

| Component | Purpose |
| --- | --- |
| `scripts/control-plane/validate_enterprise_route_auth_smoke.ps1` | Runs route probes and writes sanitized JSON/Markdown evidence. |
| `.github/workflows/enterprise-route-auth-smoke.yml` | Runs the smoke manually or weekly and uploads an artifact. |
| `docs/reports/enterprise-route-auth-smoke-automation-2026-05-02.md` | Implementation and validation record. |
| `docs/runbooks/enterprise-self-service-adoption.md` | Operator usage notes. |

## Route Matrix

| Check | Expected |
| --- | --- |
| `GET /health` | `200` |
| Anonymous `GET /enterprise/adoption-profile?org_name=...` | `401` |
| Anonymous `GET /enterprise/onboarding-checklist-tracking?org_name=...` | `401` |
| Anonymous `GET /enterprise/release-approvals?org_name=...` | `401` |
| Anonymous `GET /enterprise/release-governance/evaluate?...` | `401` |
| Authenticated `GET /enterprise/adoption-profile?org_name=...` | `200` |
| Authenticated `GET /enterprise/onboarding-checklist-tracking?org_name=...` | `200` |
| Authenticated `GET /enterprise/release-approvals?org_name=...` | `200` |
| Authenticated `GET /enterprise/release-governance/evaluate?...` | `200` |

## Secret Safety

- The script accepts `GITGOV_API_KEY` only through environment or explicit parameter.
- The API key is used only as a Bearer header.
- The output never writes tokens, headers, `.env` values, or response bodies.
- Evidence contains only route ids, sanitized paths, expected/actual HTTP codes, timings, and pass/fail status.
- Missing `GITGOV_API_KEY` can be recorded as `skipped` when `-AllowMissingApiKey` is used, so forks or unconfigured environments do not leak or invent credentials.

## Workflow Behavior

- Manual runs can override URL, org, repository, release id, environment, and report-only mode.
- Weekly scheduled runs use repository variable `GITGOV_URL` when present, otherwise production.
- The workflow uses repository secret `GITGOV_API_KEY`.
- Failures are blocking for this workflow unless `report_only=true`.
- The workflow uploads `enterprise-route-auth-smoke-{run_id}`.

## Non-Goals

- No provider mutation.
- No customer repository mutation.
- No GitHub Actions variable or secret creation.
- No workflow dispatch against customer repositories.
- No branch protection change.
- No release governance enforcement change.
- No database migration.
