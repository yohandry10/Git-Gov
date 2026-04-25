# API Contract Drift Reconciliation - 2026-04-25

## Scope

Jira ticket: `KAN-8`

This pass reconciles the internal enterprise-readiness audit with the current backend and architecture documentation.

## Findings

The API endpoint drift originally called out in the backend audit is already corrected in `docs/ARCHITECTURE.md`:

| Area | Current documented contract | Backend evidence |
|---|---|---|
| Job retry | `/jobs/{job_id}/retry` | `gitgov/gitgov-server/src/main.rs` route table |
| Compliance | `/compliance/{org_name}` | `gitgov/gitgov-server/src/main.rs` route table |
| Violations | `/violations/{violation_id}/decisions` only | `gitgov/gitgov-server/src/main.rs` route table |

There is no documented general `/violations` list endpoint in the current architecture contract.

## Additional Reconciliation

The same audit block contained stale pending items that were already closed by prior work:

- admin authorization semantics now return `403 FORBIDDEN` for valid API keys with insufficient role
- Jira ingest resolves org scope before persisting project tickets
- GitHub webhook and public invitation endpoints have explicit rate limiting
- `/api-docs` is scoped as a partial schema explorer, not a complete operational contract

## Change Made

- Reconciled local ignored internal audit memory in `docs/ENTERPRISE_READINESS_DECISION.md`; this file remains ignored by repository policy and must not be force-added.
- Updated `docs/ARCHITECTURE.md` schema migration chain to include `supabase_schema_v22.sql`.
- Updated tracked implementation memory so future agents treat route-table drift as closed.

## Remaining Debt

The remaining backend contract debt is not endpoint drift. It is optional OpenAPI completeness:

- add `#[utoipa::path]` coverage if generated SDKs or Swagger-based contract tests become a product requirement
- otherwise keep `docs/ARCHITECTURE.md` plus `main.rs` route table as the source of truth
