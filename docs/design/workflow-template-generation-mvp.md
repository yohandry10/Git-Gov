# Workflow Template Generation MVP

Updated: 2026-04-30

Ticket: `KAN-33`

## Goal

Move Enterprise Self-Service Adoption from a planning pack into an installable onboarding step.

KAN-29 generated the adoption plan. KAN-30 exposed the profile in the dashboard. KAN-31 persisted the profile. KAN-32 showed provider health from existing evidence. KAN-33 now converts the same profile into GitHub Actions workflow templates that a customer can review and copy into a target repository.

This is part of onboarding, but it is not the full onboarding wizard yet.

## MVP Scope

Script:

```text
scripts/control-plane/generate_enterprise_workflow_templates.ps1
```

Example command:

```powershell
.\scripts\control-plane\generate_enterprise_workflow_templates.ps1 -ProfilePath docs\examples\enterprise-adoption-profile.example.json -OutputDir out\enterprise-workflow-templates -Force
```

Outputs:

```text
out/enterprise-workflow-templates/README.md
out/enterprise-workflow-templates/workflow-template-manifest.json
out/enterprise-workflow-templates/.github/workflows/*.yml
```

The generated output directory is ignored by git.

## Generated Workflow Families

The generator emits templates according to the selected adoption modules and providers:

- `ci.yml`
- `secret-scan.yml`
- `public-naming-guard.yml`
- `github-evidence-report.yml`
- `github-evidence-artifact-monitor.yml`
- `github-evidence-trend-report.yml`
- `release-readiness-gate.yml`
- `release-governance-gate.yml` when release governance is explicitly non-`record-only`
- `quality-gate-policy-matrix.yml`
- `sonar-governance.yml`
- `product-vulnerability-review.yml`
- `product-vulnerability-review-artifact-monitor.yml`
- `product-vulnerability-review-trend-report.yml`
- `product-vulnerability-review-trend-enforcement.yml`

For the ExampleCo profile, the current generator writes `13` templates because the example profile remains `record-only`. A customer profile with `formal-approval` plus `approval-required` or `quorum-required` release governance writes `14` templates, including the optional release governance gate.

## Safety Model

The generator is intentionally conservative:

- It writes templates to a local output directory only.
- It does not call GitHub to create workflow files.
- It does not mutate customer repositories.
- It does not read `.env` files.
- It does not read, print, write, or generate provider token values.
- It records secret names only, such as `GITGOV_API_KEY` and `SONAR_TOKEN`.
- It records variable names only, such as `GITGOV_URL`, `SONAR_HOST_URL`, and `SONAR_PROJECT_KEY`.
- It marks generated workflows as requiring customer review before install.
- It includes release-governance enforcement only after the customer profile explicitly selects non-`record-only` governance.

## Policy Presets

`audit-only`:

- generates evidence workflows.
- avoids release blocking defaults.
- is best for first discovery and demos.

`moderate`:

- generates ticket traceability and evidence freshness workflows.
- sets release readiness target `75`.
- generates vulnerability review evidence.

`strict`:

- sets release readiness target `85`.
- enables vulnerability trend enforcement by default.
- expects stronger review and risk-acceptance process around findings.

The portable vulnerability review template reports dependency findings. It does not claim to prove reachability by itself. Customers should add product-specific reachability triage before using dependency findings as a release blocker.

Release governance:

- `record-only` does not generate a release governance gate.
- `advisory` can generate a manual gate when `formal-approval` is enabled, but does not default to blocking.
- `approval-required` and `quorum-required` generate a manual gate that defaults to enforcement because the customer explicitly selected a blocking release governance policy.

## Non-Goals

- No automatic workflow installation in customer repositories.
- No direct provider credential validation.
- No provider token storage.
- No formal enterprise release approval engine.
- No Vercel AI SDK Copilot.
- No generated SDK or OpenAPI contract expansion.

## Next Product Steps

1. Add a dashboard action that generates the same workflow template pack from the persisted profile.
2. Add dashboard generation from the persisted/current adoption profile.
   - Status: implemented by `KAN-34`.
3. Add a reviewed install flow for customer repositories, gated by explicit operator authorization.
4. Add direct provider credential checks where the customer grants explicit access.
5. Add formal enterprise release approval records.
6. Add Vercel AI SDK Copilot over adoption profile, workflow status, provider health, evidence packets, and vulnerability findings.
