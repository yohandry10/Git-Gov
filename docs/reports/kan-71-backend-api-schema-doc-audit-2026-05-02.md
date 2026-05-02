# KAN-71 Backend/API/Schema Documentation Reality Audit

Updated: 2026-05-02

## Summary

KAN-71 is phase 2 of the GitGov documentation reality audit. It checks backend route, API, schema, and server configuration documentation against the repository instead of extending product functionality.

## Product Context

- `KAN-69 - Enterprise Action Center guided UX` remains pending product/UX work.
- `KAN-70` completed the first broad documentation cleanup pass.
- `KAN-71` narrows the audit to backend/API/schema docs and tracked server configuration examples.

## Verified Sources

| Area | Source checked | Verified state |
| --- | --- | --- |
| Router | `gitgov/gitgov-server/src/main.rs` | `72` production Axum `.route(...)` registrations plus Swagger UI at `/api-docs` |
| Handlers | `gitgov/gitgov-server/src/handlers` | `23` handler files |
| Schema files | `gitgov/gitgov-server/supabase/supabase_schema*.sql` | `21` schema/migration files; latest migration is `supabase_schema_v25.sql` |
| Postchecks | `gitgov/gitgov-server/supabase/checks/*postcheck.sql` | `7` postcheck files, latest `v25_postcheck.sql` |
| Backend tests | `cargo test -- --list` from `gitgov/gitgov-server` | `193` tests, `0` benchmarks |
| Server env template | `gitgov/gitgov-server/.env.example` and `main.rs` | Chat concurrency and queue-timeout defaults aligned to code; missing webhook/invitation rate-limit knobs documented |

## Corrections Made

- `gitgov/gitgov-server/README.md` now uses Axum brace path parameters and removes the stale `PUT /policy/:repo` contract in favor of `PUT /policy/{repo_name}/override`.
- Backend endpoint docs now include policy request approval/rejection, policy drift events, violation decisions, audit stream ingest, PR merge evidence, `/api-docs`, and `/api-docs/openapi.json`.
- `docs/ARCHITECTURE.md` explicitly records that `/api-docs` is a partial schema explorer and that `main.rs` remains the operational route source of truth.
- `docs/QUICKSTART.md` updates the backend handler summary from `65+ endpoints` to `72` production Axum route registrations and points operators to the tracked `.env.example` for the full server config surface.
- `docs/DEPLOYMENT.md` fixes the Docker bootstrap schema path to `gitgov/gitgov-server/supabase/supabase_schema.sql`.
- `gitgov/gitgov-server/.env.example` now matches `main.rs` defaults for `GITGOV_CHAT_LLM_MAX_CONCURRENCY=16` and `GITGOV_CHAT_LLM_QUEUE_TIMEOUT_MS=3000`.

## Non-Goals

- No runtime code changes.
- No database migration.
- No provider mutation.
- No GitHub Actions secret or variable mutation.
- No full OpenAPI annotation work as a blocker.
- No implementation of `KAN-69`; it remains pending guided UX work.
