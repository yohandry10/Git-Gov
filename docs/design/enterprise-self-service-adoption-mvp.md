# Enterprise Self-Service Adoption MVP

Updated: 2026-04-30

Ticket: `KAN-29`

## Goal

Make the proven GitGov operating model easier to adopt for another company without hand-writing the first setup plan.

The MVP is intentionally not a full UI wizard yet. It is a reproducible adoption pack generator that turns a customer profile into:

- recommended GitHub workflows.
- required variables and secrets by name.
- selected GitGov modules.
- policy preset and evidence rules.
- manual setup checklist.
- known product gaps, especially formal enterprise release approval.

## Why This Comes Before The AI Copilot

The Vercel AI SDK Copilot will be more useful after GitGov has a clearer self-service adoption shape. The copilot needs stable evidence surfaces, module names, policy presets, and customer setup context to answer questions reliably.

KAN-29 creates that packaging vocabulary first.

## MVP Scope

### Inputs

The generator accepts either direct PowerShell parameters or a JSON profile like:

```text
docs/examples/enterprise-adoption-profile.example.json
```

Core profile fields:

- `customer_name`.
- `repository_full_name`.
- `default_branch`.
- `jira_project_key`.
- `policy_preset`: `audit-only`, `moderate`, or `strict`.
- `providers`: `github`, `jira`, `jenkins`, `sonarqube`, `render`, `vercel`.
- `modules`: `traceability`, `github-evidence`, `release-readiness`, `quality-gates`, `evidence-packets`, `vulnerability-review`, `artifact-monitoring`, `trend-enforcement`, `formal-approval`.

### Outputs

The generator writes:

- `enterprise-adoption-pack.md`.
- `enterprise-adoption-pack.json`.

The pack contains no secret values.

### Policy Presets

`audit-only`:

- gather evidence.
- avoid release blocking.
- useful for first adoption, demos, and discovery.

`moderate`:

- require ticket traceability.
- require fresh evidence artifacts.
- block reachable critical/high vulnerabilities.
- target release readiness score `75`.

`strict`:

- require ticket traceability.
- require PR review evidence.
- require fresh evidence artifacts.
- block reachable critical/high vulnerabilities.
- require medium-risk acceptance.
- target release readiness score `85`.
- enable vulnerability trend enforcement.

## What This Enables

For GitGov sales/product positioning:

- "Choose a policy preset."
- "Choose your providers."
- "Generate an adoption pack."
- "Install the recommended workflows."
- "Validate that the evidence exists."
- "Use GitGov readiness and evidence packets for release decisions."

For future product work:

- The generated pack can become the backend contract for a UI onboarding wizard.
- The same JSON can feed a future Vercel AI SDK Copilot so the copilot understands which modules are expected for a tenant.

## Non-Goals

- This does not connect provider accounts automatically.
- This does not store customer secrets.
- This does not implement full formal release approval.
- This does not require SonarCloud.
- This does not generate a complete OpenAPI/SDK.

## Next Steps After MVP

1. Add a dashboard onboarding view that writes the same profile shape.
   - Status: implemented as the KAN-30 Adoption Profile Dashboard MVP.
2. Add backend persistence for tenant adoption profiles.
3. Add validation endpoints that compare expected modules against real provider evidence.
4. Add formal release approval records with approver, expiration, risk acceptance, and linked evidence packet.
5. Add Vercel AI SDK Copilot on top of the adoption profile and evidence APIs.
