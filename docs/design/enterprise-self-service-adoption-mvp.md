# Enterprise Self-Service Adoption MVP

Updated: 2026-05-01

Ticket: `KAN-45`

## Goal

Make the proven GitGov operating model easier to adopt for another company without hand-writing the first setup plan.

The MVP is intentionally not a full UI wizard yet. It is a reproducible adoption pack generator that turns a customer profile into:

- recommended GitHub workflows.
- required variables and secrets by name.
- selected GitGov modules.
- policy preset and evidence rules.
- release governance policy.
- manual setup checklist.
- known product gaps and manual setup steps.

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
- `release_governance`: default `record-only` policy, or explicit customer-selected `advisory`, `approval-required`, or `quorum-required`.

### Outputs

The generator writes:

- `enterprise-adoption-pack.md`.
- `enterprise-adoption-pack.json`.

The pack contains no secret values.

### Release Governance

KAN-45 adds the first release governance policy field to the adoption profile.

Default:

```text
mode: record-only
enforcement: disabled
quorum: disabled
```

Meaning:

- GitGov can record release approval evidence.
- GitGov can include release approval state in generated packs and dashboard exports.
- GitGov does not block releases by default.
- GitGov does not require multiple approvers by default.

Stricter modes require explicit customer configuration and the `formal-approval` module.

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
   - Status: implemented by `KAN-31`.
3. Add validation endpoints that compare expected modules against real provider evidence.
   - Status: provider health evidence implemented by `KAN-32`; direct provider connection validation implemented by `KAN-36`.
4. Generate workflow templates for selected modules and providers.
   - Status: implemented by `KAN-33`.
5. Add a reviewed workflow installation flow for customer repositories.
   - Status: implemented by `KAN-35` for local checkout installation after dry-run review.
6. Add formal release approval records with approver, expiration, risk acceptance, and linked evidence packet.
   - Status: backend implemented by `KAN-37`; dashboard wizard implemented by `KAN-43`.
7. Add Vercel AI SDK Copilot on top of the adoption profile and evidence APIs.
   - Status: first MVP implemented by `KAN-38`; dashboard UI and AI mode validation/activation completed through `KAN-39` to `KAN-42`.
8. Carry explicit release governance policy through the profile, generated packs, and backend validation.
   - Status: implemented by `KAN-45`.
