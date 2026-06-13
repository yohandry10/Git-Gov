# KAN-82 Platform Principals Superadmin Hardening

Date: 2026-06-13

## Implemented

- Added `platform_principals` through `gitgov/gitgov-server/supabase/supabase_schema_v34.sql`.
- Updated the consolidated `gitgov/gitgov-server/supabase/supabase_schema.sql` so fresh databases include the same platform-principal foundation.
- Seeded the production founder principal as `client_id=bootstrap-admin`, `principal_type=platform_founder`, `status=active`, and `auth_method=api_key`.
- Changed backend auth so Platform Founder is resolved from `platform_principals`; a global Admin API key alone is not sufficient.
- Extended `/me` with `platform_principal_id`.
- Extended Desktop/Tauri `/me` DTOs and frontend types with `platform_principal_id`.
- Removed the old `VITE_FOUNDER_GITHUB_LOGIN`/`VITE_FOUNDER_LOGIN` requirement from the active Desktop identity check.
- Added regression coverage showing founder access works without GitHub identity matching and that tenant provisioning requires an active platform principal.

## Production DB Validation

Applied `supabase_schema_v34.sql` through ignored local `DATABASE_URL`.

Postcheck results:

- `platform_principals.table`: `PASS`
- `platform_principals.constraints`: `PASS`
- `platform_principals.bootstrap_founder`: `PASS`

No secret values were printed or committed.

## Product Decision

The superadmin/founder is a GitGov platform principal, not a GitHub-authenticated tenant user. GitHub remains useful for Desktop operator identity and repository workflows, but platform administration is authorized by GitGov API key plus an active `platform_principals` row.

## Validation

- `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`
- `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`
- `npm run typecheck`
- `npm run lint`
- `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml auth::tests::founder_global_admin_detection_matches_expected_scope`
- `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml create_org_requires_founder_global_admin_key`
- `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml platform_tenant_administration_requires_founder_and_audits_lifecycle`
- `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml org_discovery_and_me_return_human_scope`
- `npm run test -- src/test/useControlPlaneStore.test.ts`
