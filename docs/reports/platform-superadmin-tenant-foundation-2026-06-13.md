# KAN-81 Platform Superadmin Tenant Foundation

Date: 2026-06-13

## Implemented

- Added tenant catalog metadata to `orgs` through `supabase_schema_v33.sql`.
- Added `/platform/tenants` and `/platform/tenants/{login}/lifecycle`.
- Preserved `/orgs` as a compatibility create/list surface while routing create through audited platform provisioning semantics.
- Added `/me.principal_type` and `/me.requires_workspace_for_tenant_surfaces`.
- Added platform tenant audit events for create, update, lifecycle change, and denied non-founder attempts.
- Added integration tests covering founder-only tenant administration, scoped-admin denial, lifecycle suspension timestamping, and audit log evidence.

## Validation

- `cargo fmt --manifest-path .\gitgov\gitgov-server\Cargo.toml --check`
- `cargo check --manifest-path .\gitgov\gitgov-server\Cargo.toml`
- `cargo clippy --manifest-path .\gitgov\gitgov-server\Cargo.toml -- -D warnings`
- `TEST_DATABASE_URL=postgresql://gitgov:gitgov_dev_password@127.0.0.1:5433/gitgov cargo test --manifest-path .\gitgov\gitgov-server\Cargo.toml` (`254` tests)
- Focused Postgres integration tests:
  - `platform_tenant_administration_requires_founder_and_audits_lifecycle`
  - `create_org_requires_founder_global_admin_key`
  - `org_discovery_and_me_return_human_scope`

## Production Validation

- PR `#299` merged to `main` as `0d2e5e2`.
- Supabase migration `v33` was applied manually through ignored local `DATABASE_URL` before merge.
- DB postcheck found `8` tenant catalog columns and `3` tenant constraints.
- Post-merge GitHub `CI`, `Release Readiness Gate`, `Secret Scan`, `Public Naming Guard`, `Quality Gate Policy Matrix`, `Governance Correlation Smoke`, `Desktop Updater Readiness`, and `SonarQube Governance` passed.
- Render deploy `dep-d8mc9stckfvc73e5umn0` reached `live`.
- Production endpoint smoke:
  - `GET https://gitgov-api.onrender.com/health` returned `200`.
  - Authenticated `GET /stats` returned `200`.
  - Authenticated `GET /me` returned `principal_type=platform_founder` and `requires_workspace_for_tenant_surfaces=true`.
  - Authenticated `GET /platform/tenants` returned `200`, `21` tenants, and lifecycle fields.

## Follow-Ups

- Desktop UI should present Platform Founder as a mode outside tenant workspaces.
- Do not auto-switch into a tenant immediately after provisioning without explicit operator choice.
- Future enterprise work can add `platform_principals`, granular capabilities, lifecycle billing states, support access controls, SSO/MFA, and physical platform metadata isolation.
