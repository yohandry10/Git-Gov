# KAN-139 OpenAPI Route Source Technical Debt

Date: 2026-06-16

## Summary

KAN-139 corrects a technical-debt mismatch in the partial OpenAPI contract language. GitGov still
keeps `/api-docs` as an intentionally partial schema explorer, but its generated description and
some living docs still pointed operators to the old `main.rs` route table after route composition
moved into `gitgov/gitgov-server/src/server/routes.rs`.

## Decision

- Keep OpenAPI partial.
- Do not implement full `#[utoipa::path]` coverage, generated SDKs, or Swagger contract tests in
  this slice.
- Correct the operational route source of truth to `gitgov/gitgov-server/src/server/routes.rs`.
- Record the current verified route registration count as `158` Axum `.route(...)` registrations.
- Allow production redeploy only to refresh the served `/api-docs` description; no API route
  behavior changes.

## Scope

- Updated `gitgov/gitgov-server/src/openapi.rs` description and unit guard.
- Updated `gitgov/gitgov-server/README.md`.
- Updated `docs/ARCHITECTURE.md`, `docs/CURRENT_CONTEXT.md`, and `docs/IMPLEMENTATION_STATUS.md`.
- Updated prior OpenAPI decision reports that describe the operational route source.

## Validation

- `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml openapi_description_declares_partial_schema_explorer_scope` passed.
- `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check` passed.
- `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml` passed.
- `git diff --check` passed.
- `scripts/security/publication_guard.ps1` passed.
- Stale route-source grep returned no matches for the old `main.rs`/`72` route-count contract.
- Route-count verification returned `158`.
- Required PR checks passed on PR `#485`.
- Render deploy `dep-d8oo81k2m8qs73augv70` reached `live`.
- Production `/api-docs/openapi.json` returned HTTP `200`, contained
  `gitgov-server/src/server/routes.rs`, and no longer contained `main.rs`.
