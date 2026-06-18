# Provider Setup Guidance

Date: 2026-06-18

Ticket: `KAN-147`

## Summary

KAN-147 adds a compact manual-first provider setup guidance layer to Enterprise Adoption. It reuses
the existing adoption profile and provider health checks, then produces explicit provider actions:
`Connect`, `Retry`, `Review`, or `Skipped`.

## Files

- `gitgov/src/components/control_plane/dashboard-helpers/provider-setup-guidance.ts`
- `gitgov/src/components/control_plane/dashboard-helpers/adoption-profile.ts`
- `gitgov/src/components/control_plane/dashboard-helpers.ts`
- `gitgov/src/components/control_plane/enterprise-adoption-panel-helpers.ts`
- `gitgov/src/components/control_plane/EnterpriseAdoptionPanel.tsx`
- `gitgov/src/test/components/dashboard-helpers.test.ts`
- `gitgov/src/test/components/EnterpriseAdoptionPanel.test.tsx`

## Product Behavior

- Missing provider configuration is prioritized as `Connect`.
- Missing provider evidence is shown as `Retry`.
- Ready selected providers are shown as `Review`.
- Unselected providers are shown as `Skipped`.
- The Enterprise Adoption UI exposes this as a small region above Provider Health.

## Safety

The feature is advisory and deterministic. It does not read secret values, start OAuth, call provider
APIs, mutate customer repositories, mutate provider state, change backend routes, add a database
migration, invoke Agent Governance, or create blocking/release/compliance claims.

## Validation

- `npm --prefix gitgov test -- --run src/test/components/dashboard-helpers.test.ts`
  - Passed: `38` tests.
- `npm --prefix gitgov test -- --run src/test/components/dashboard-helpers.test.ts src/test/components/EnterpriseAdoptionPanel.test.tsx`
  - Passed: `39` tests.
- `npm --prefix gitgov run typecheck`
  - Passed.
- `npm --prefix gitgov run lint`
  - Passed.

Full repository validation is recorded in the pull request evidence before merge.
