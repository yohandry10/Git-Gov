# Provider Setup Operator Decisions

Date: 2026-06-18

Ticket: `KAN-149`

## Summary

KAN-149 lets operators persist manual provider setup decisions inside the existing Enterprise
Adoption profile JSON. The setup guide can now show remembered decisions for provider rows while
keeping KAN-147/KAN-148 behavior manual-first and navigation-only.

## Files

- `gitgov/src/components/control_plane/dashboard-helpers/adoption-profile.ts`
- `gitgov/src/components/control_plane/dashboard-helpers/provider-setup-guidance.ts`
- `gitgov/src/components/control_plane/EnterpriseAdoptionPanel.tsx`
- `gitgov/src/components/control_plane/EnterpriseProviderSetupPanel.tsx`
- `gitgov/src/components/control_plane/dashboard-helpers.ts`
- `gitgov/src/test/components/dashboard-helpers.test.ts`
- `gitgov/src/test/components/EnterpriseAdoptionPanel.test.tsx`
- `gitgov/gitgov-server/src/handlers/adoption_profiles.rs`
- `docs/design/provider-setup-operator-decisions-mvp.md`
- `docs/IMPLEMENTATION_STATUS.md`
- `docs/CURRENT_CONTEXT.md`
- `docs/AGENT_PUBLIC_CONTEXT.md`
- `docs/design/enterprise-self-service-and-ai-copilot-roadmap.md`

## Safety

The feature stores only provider ID, decision kind, and timestamp. It does not store notes or secret
values. It does not start provider setup, call provider APIs, mutate providers, mutate repositories,
create workflows, authorize deployments, or depend on agents/AI.

## Validation

- `npm --prefix gitgov test -- --run src/test/components/dashboard-helpers.test.ts src/test/components/EnterpriseAdoptionPanel.test.tsx`
  - Passed: `44` tests.
- `cargo test enterprise_adoption_profile_validation_accepts_provider_setup_decisions`
  - Passed: `1` backend validation test.
- `npm --prefix gitgov run typecheck`
  - Passed.
- `npm --prefix gitgov run lint`
  - Passed.
- `npm --prefix gitgov test -- --run`
  - Passed: `47` files, `436` tests.
- `npm --prefix gitgov run build`
  - Passed with the pre-existing Vite large chunk warning.
- `cargo fmt --check`
  - Passed for `gitgov-server`.
- `cargo check`
  - Passed for `gitgov-server`.
- `git diff --check`
  - Passed.
- `.\scripts\security\publication_guard.ps1`
  - Passed.
- Static guardrail grep found only existing secret-validation code, documentation guardrails, and
  negative test assertions; diff-only grep found no executable OAuth/provider API/backend mutation
  code.
