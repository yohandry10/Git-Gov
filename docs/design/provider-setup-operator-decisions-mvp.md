# Provider Setup Operator Decisions MVP

Date: 2026-06-18

Ticket: `KAN-149`

## Product Decision

KAN-149 persists human setup decisions for the KAN-147/KAN-148 provider setup guide without turning
GitGov into a provider installer.

The decision record lives inside the existing Enterprise Adoption profile JSON:

- `retry-later` for selected providers that still need configuration or evidence.
- `reviewed` for selected providers that are ready and have been reviewed by the operator.
- `intentionally-skipped` for providers not selected in the onboarding profile.

## Persistence

The field is `provider_setup_decisions`.

It reuses the existing `/enterprise/adoption-profile` save/load path and existing
`enterprise_adoption_profiles.profile` JSONB column. No table, migration, or backend route is
required.

## Guardrails

- Manual decisions only.
- No OAuth flow.
- No provider API call.
- No `.env`, provider token, API key, or secret value read.
- No new backend route, database migration, or Render deploy requirement.
- No customer repository mutation.
- No provider state mutation.
- No Agent Governance, MCP, OPA/Rego, or AI dependency.
- No release blocking or compliance/certification/legal/regulatory claim.

## UX Intent

The provider setup guide stays compact. Decision controls are small draft updates saved through the
normal profile save action. They help operators remember human state across sessions without
creating executable setup buttons or implying that GitGov connected the provider.
