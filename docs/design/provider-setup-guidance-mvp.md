# Provider Setup Guidance MVP

Date: 2026-06-18

Ticket: `KAN-147`

## Product Decision

Enterprise Adoption already had provider health, onboarding readiness, remediation, and checklist
tracking. The missing packaging layer was a compact first-run view that translates provider health
into the next human setup action per provider.

KAN-147 adds that layer without turning GitGov into a provider installer.

## Implemented Shape

- `buildEnterpriseProviderSetupGuidance` consumes the adoption profile plus existing provider health.
- Each known provider gets one setup step:
  - `Connect` when selected provider configuration is incomplete.
  - `Retry` when configuration exists but GitGov has not observed evidence yet.
  - `Review` when selected provider evidence is ready.
  - `Skipped` when the provider is not selected for this customer profile.
- `EnterpriseAdoptionPanel` renders a compact `Provider setup` region above existing Provider Health.
- The region shows selected-ready count, skipped count, the next provider action, and per-provider
  reason/validation text.

## Guardrails

- Manual-first and advisory only.
- No OAuth flow.
- No provider API call.
- No provider token, API key, `.env`, or secret value read.
- No customer repository mutation.
- No provider state mutation.
- No backend route, database migration, or Render deploy requirement.
- No Agent Governance, MCP, OPA/Rego, or AI dependency.
- No release blocking or compliance/certification/legal/regulatory claim.

## UX Intent

The UI should be visible but not noisy. Provider Health remains the evidence view. Provider Setup
Guidance is the small translation layer that tells an operator whether to connect missing
configuration, retry validation/evidence ingestion, review ready providers, or leave unused
providers skipped.
