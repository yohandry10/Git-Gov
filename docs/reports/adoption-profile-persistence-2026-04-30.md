# KAN-31 Adoption Profile Persistence

Updated: 2026-04-30

## Summary

KAN-31 persists Enterprise Adoption profiles per organization.

This closes the first KAN-30 follow-up: the adoption dashboard is no longer only a local builder/export screen. An admin can save the org profile, reload it later, and still export the same secret-safe JSON adoption pack.

## Changes

- Added `enterprise_adoption_profiles` schema in migration `v23`.
- Added backend models and database methods for get/upsert.
- Added authenticated admin endpoint:
  - `GET /enterprise/adoption-profile`
  - `PUT /enterprise/adoption-profile`
- Added backend validation for profile shape, known provider/module IDs, Jira key, repository shape, and payload size.
- Added admin audit logging for profile saves with metadata only.
- Added Tauri Control Plane client methods and commands.
- Wired the React Enterprise Adoption panel to load/save persisted org profile state.
- Kept JSON export secret-safe and independent from provider tokens.

## Validation

Local validation passed:

- `cargo test enterprise_adoption_profile_validation`
  - `3` tests passed.
- `cargo check` in `gitgov/gitgov-server`
  - passed.
- `cargo clippy -- -D warnings` in `gitgov/gitgov-server`
  - passed.
- `cargo check` in `gitgov/src-tauri`
  - passed.
- `cargo clippy -- -D warnings` in `gitgov/src-tauri`
  - passed.
- `npm run typecheck`
  - passed.
- `npm test -- --run src/test/components/dashboard-helpers.test.ts`
  - `8` tests passed.
- `npm test -- --run`
  - `25` files passed.
  - `271` tests passed.
- `npm run lint`
  - passed.
- `npm run build`
  - passed.
  - Vite reported the existing large chunk warning.
- `git diff --check`
  - passed.
- `.\scripts\security\publication_guard.ps1`
  - passed.

## Production Note

The backend route depends on migration `supabase_schema_v23.sql`.

Before using persisted adoption profiles in production, apply `v23` and run:

```powershell
psql "<DATABASE_URL>" -f gitgov/gitgov-server/supabase/supabase_schema_v23.sql
psql "<DATABASE_URL>" -f gitgov/gitgov-server/supabase/checks/v23_postcheck.sql
```

Do not print the database URL or credentials.

## Remaining Product Work

- Provider health validation.
- Customer workflow template installation.
- Formal release approval.
- Vercel AI SDK Copilot over evidence, adoption profile, readiness, and security findings.
