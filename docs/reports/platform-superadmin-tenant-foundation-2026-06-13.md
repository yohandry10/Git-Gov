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

## Follow-Ups

- Desktop UI should present Platform Founder as a mode outside tenant workspaces.
- Do not auto-switch into a tenant immediately after provisioning without explicit operator choice.
- Future enterprise work can add `platform_principals`, granular capabilities, lifecycle billing states, support access controls, SSO/MFA, and physical platform metadata isolation.
