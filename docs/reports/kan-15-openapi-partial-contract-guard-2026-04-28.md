# KAN-15 OpenAPI Partial Contract Guard

Date: 2026-04-28

## Purpose

Prevent `/api-docs` from drifting back into an overstated contract claim. The current OpenAPI output is useful as a schema explorer, but it is not the complete operational route contract.

## Decision

Do not implement full `#[utoipa::path]` route coverage yet.

Rationale:

- The documentation now states that `docs/ARCHITECTURE.md` plus `gitgov/gitgov-server/src/server/routes.rs` are the operational source of truth.
- Full OpenAPI path annotation is only worth the cost when generated SDKs or Swagger-based contract tests become a product requirement.
- The immediate risk is not missing annotations; it is accidentally presenting the partial schema explorer as complete.

## Change

Added unit test `openapi_description_declares_partial_schema_explorer_scope` in `gitgov/gitgov-server/src/openapi.rs`.

The test requires the generated OpenAPI description to include:

- `intentionally partial`
- `docs/ARCHITECTURE.md`
- `gitgov-server/src/server/routes.rs`

## Validation

Run from `gitgov/gitgov-server`:

```powershell
cargo test openapi_description_declares_partial_schema_explorer_scope
```

The normal repository publication guard should also run before merge:

```powershell
.\scripts\security\publication_guard.ps1
```

## Remaining Work

OpenAPI path completeness remains optional. Implement it only if:

- Swagger becomes a contract-testing source.
- Generated SDKs are required.
- External consumers need a machine-readable endpoint contract beyond the current architecture documentation.
