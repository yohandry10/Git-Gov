# KAN-60 Guided Onboarding Checklist Tracking MVP

Updated: 2026-05-02

## Summary

KAN-60 persists operator tracking metadata for the guided Enterprise Adoption checklist.

KAN-59 made the checklist visible in the dashboard. KAN-60 lets an admin save per-stage tracking state, owner, target date, external reference, and notes per organization.

This tracking does not change readiness evidence, readiness score, release governance evaluation, or release gate behavior.

## Scope

- Add admin `GET/PUT /enterprise/onboarding-checklist-tracking`.
- Store one JSONB tracking document per organization.
- Add Supabase migration `v25` and postcheck.
- Add Tauri commands and dashboard store actions.
- Add dashboard controls inside the guided checklist.
- Treat the endpoint as a sensitive admin route for stale-auth-cache fail-closed behavior.
- Add focused backend/frontend tests.

## Tracking Fields

Each item is keyed by onboarding stage:

- `profile`
- `providers`
- `workflow-pack`
- `remote-workflows`
- `actions-config`
- `release-governance`

Tracking statuses:

- `open`
- `in-progress`
- `waiting`
- `done`

These statuses are human workflow metadata only. A `done` tracking item does not make the underlying readiness stage ready.

## Safety

- Admin-only API.
- Org-scoped by the same rules as adoption profile persistence.
- Sensitive admin endpoint stale-auth-cache protection.
- No `.env` files are read.
- No provider tokens are read.
- No provider APIs are called.
- No secret values are printed or stored intentionally.
- Backend validation rejects common secret-looking markers in notes/owner/reference fields.
- No GitHub Actions variables or secrets are created.
- No customer repositories are mutated.
- No provider settings are mutated.
- No workflow dispatch occurs.
- No branch protection is changed.
- Release blocking remains opt-in only.

## Non-Goals

- No automatic task assignment.
- No provider setup wizard.
- No change to readiness scoring.
- No release gate enforcement.
- No customer repository mutation.
