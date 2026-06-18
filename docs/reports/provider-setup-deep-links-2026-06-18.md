# Provider Setup Deep Links

Date: 2026-06-18

Ticket: `KAN-148`

## Summary

KAN-148 extends KAN-147 provider setup guidance with navigation-only targets. The feature does not
execute provider setup. It directs operators to existing GitGov surfaces:

- Settings/System for missing setup/configuration.
- Governance/Evidence for missing observed evidence.
- Action Center for review of ready providers.
- Enterprise Adoption profile for skipped providers.

## Files

- `gitgov/src/components/control_plane/dashboard-helpers/adoption-profile.ts`
- `gitgov/src/components/control_plane/dashboard-helpers/provider-setup-guidance.ts`
- `gitgov/src/components/control_plane/EnterpriseAdoptionPanel.tsx`
- `gitgov/src/test/components/dashboard-helpers.test.ts`
- `gitgov/src/test/components/EnterpriseAdoptionPanel.test.tsx`
- `docs/design/provider-setup-deep-links-mvp.md`
- `docs/IMPLEMENTATION_STATUS.md`
- `docs/CURRENT_CONTEXT.md`
- `docs/AGENT_PUBLIC_CONTEXT.md`
- `docs/design/enterprise-self-service-and-ai-copilot-roadmap.md`

## Safety

Each target carries `navigation_only=true`. The UI renders React Router links, not provider mutation
buttons. There are no provider API calls, OAuth flows, backend routes, database migrations, Render
deploy requirements, secret reads, provider mutations, repository mutations, Agent Governance calls,
or release-blocking behavior.

## Validation

- `npm --prefix gitgov test -- --run src/test/components/dashboard-helpers.test.ts src/test/components/EnterpriseAdoptionPanel.test.tsx`
  - Passed: `40` tests.
- `npm --prefix gitgov run typecheck`
  - Passed.
- `npm --prefix gitgov run lint`
  - Passed.
- `npm --prefix gitgov test -- --run`
  - Passed: `47` files, `432` tests.
- `npm --prefix gitgov run build`
  - Passed with the pre-existing Vite large chunk warning.
- `git diff --check`
  - Passed.
- `.\scripts\security\publication_guard.ps1`
  - Passed.
- Static guardrail grep for OAuth/provider API/backend mutation/token strings found only test names,
  documentation guardrails, and existing safety flags set to `false`.
- PR `#508` checks passed before merge.
- Post-merge `main` checks passed for `3f8f1c36`: CI, Release Readiness Gate, Secret Scan, Public
  Naming Guard, Quality Gate Policy Matrix, Governance Correlation Smoke, Desktop Updater
  Readiness, and SonarQube Governance.

## Merge

- PR: `#508`
- Main commit: `3f8f1c36`
- Backend/API/DB/Render change: none required.
