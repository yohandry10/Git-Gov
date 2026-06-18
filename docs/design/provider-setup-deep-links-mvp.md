# Provider Setup Deep Links MVP

Date: 2026-06-18

Ticket: `KAN-148`

## Product Decision

KAN-147 made provider setup status understandable. KAN-148 makes the next navigation step obvious
without turning the guidance into provider automation.

Provider setup actions now expose navigation-only targets:

- `Connect` -> `Settings / System` through `/settings#control-plane`.
- `Retry` -> `Governance / Evidence` through `/governance/evidence`.
- `Review` -> `Action Center` through `/action-center`.
- `Skipped` -> Enterprise Adoption profile through `/governance/adoption#enterprise-adoption`.

## Guardrails

- Navigation only.
- No OAuth flow.
- No provider API call.
- No `.env`, provider token, API key, or secret value read.
- No backend route, database migration, or Render deploy requirement.
- No customer repository mutation.
- No provider state mutation.
- No Agent Governance, MCP, OPA/Rego, or AI dependency.
- No release blocking or compliance/certification/legal/regulatory claim.

## UX Intent

The visible controls are links, not provider execution buttons. They help the operator move to the
existing GitGov surface that owns the next manual step while preserving the current manual-first
enterprise path.
