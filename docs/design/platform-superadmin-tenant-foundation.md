# KAN-81 Platform Superadmin Tenant Foundation

## Decision

GitGov separates platform administration from tenant administration.

- `Platform Founder` is a platform principal outside every tenant (`org_id=null`).
- `GitGov Internal` is a normal tenant used for dogfooding GitGov against GitGov's own repos.
- Tenant admins, including admins of the GitGov internal tenant, cannot create sibling tenants.
- Tenant provisioning and lifecycle management are platform actions, not workspace actions.

## Backend Shape

- `/me` returns `principal_type` and `requires_workspace_for_tenant_surfaces`.
- `/platform/tenants` lists or provisions tenants and is restricted to Platform Founder.
- `/platform/tenants/{login}/lifecycle` changes lifecycle state and is restricted to Platform Founder.
- `/orgs` remains as a compatibility route for historical clients, but create/upsert follows the same audited Platform Founder semantics.

## Database Shape

`orgs` remains the tenant catalog and now includes:

- `tenant_type`: `customer`, `internal`, or `sandbox`.
- `lifecycle_status`: `trial`, `active`, `suspended`, `archived`, or `deleted`.
- `provisioning_source`: `legacy`, `github_webhook`, `platform_founder`, or `migration`.
- `provisioned_by`: platform actor client id.
- `platform_metadata`: non-secret JSON metadata.
- `suspended_at`, `archived_at`, `deleted_at`: lifecycle timestamps.

Migration: `gitgov/gitgov-server/supabase/supabase_schema_v33.sql`.

## Audit Contract

Platform tenant administration writes `admin_audit_log` entries:

- `platform.tenant.created`
- `platform.tenant.updated`
- `platform.tenant.lifecycle_changed`
- `platform.tenant.provision_denied`

Audit metadata identifies `actor_scope=platform`, target tenant login, lifecycle status, and provisioning source where applicable.

## Non-Goals

This slice does not build billing, SSO/MFA, support impersonation, physical platform service separation, plan enforcement, tenant deletion jobs, or a new Desktop UI.
