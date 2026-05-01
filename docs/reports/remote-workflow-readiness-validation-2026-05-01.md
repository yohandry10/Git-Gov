# KAN-51 Remote Workflow Readiness Validation Report

Updated: 2026-05-01

## Scope

KAN-51 adds read-only validation for customer repositories after GitGov workflow template installation.

The validator checks:

- expected workflow files.
- workflow content match by SHA-256.
- required GitHub Actions variable names.
- required GitHub Actions secret names.

It does not read secret values and does not mutate remote repositories.

## Changes

| Area | Change |
| --- | --- |
| Script | Added `scripts/control-plane/validate_enterprise_workflow_installation_readiness.ps1`. |
| Design | Added `docs/design/remote-workflow-readiness-validation-mvp.md`. |
| Runbook | Updated `docs/runbooks/enterprise-self-service-adoption.md`. |
| Roadmap | Updated `docs/design/enterprise-self-service-and-ai-copilot-roadmap.md`. |

## Safety

- No `.env` files are read.
- GitHub token values are not printed.
- Actions secrets are validated by name only.
- No repository files, variables, secrets, branch protection, workflows, PRs, or provider settings are changed.
- Output reports contain config names and content hashes, not secret values or workflow contents.

## Validation

Local validation completed on 2026-05-01:

| Command | Result |
| --- | --- |
| PowerShell parse check for `validate_enterprise_workflow_installation_readiness.ps1` | PASS. |
| `.\scripts\control-plane\generate_enterprise_workflow_templates.ps1 -ProfilePath docs\examples\enterprise-adoption-profile.example.json -OutputDir out\KAN-51-enterprise-workflow-templates -Force` | PASS. Generated the ExampleCo workflow template pack. |
| `.\scripts\control-plane\validate_enterprise_workflow_installation_readiness.ps1 -PackDir out\KAN-51-enterprise-workflow-templates -Repository yohandry10/Git-Gov -Ref main -ReportOnly -OutputPath out\KAN-51-workflow-readiness.json` | PASS as report-only. Result was expected `needs-action`: `workflows_missing=0`, `workflows_different=13`, `variables_missing=0`, `secrets_missing=1`. |
| Minimal dashboard-style `-PackPath` readiness smoke | PASS as report-only. Result was expected `needs-action`: `workflows_missing=1`, `workflows_different=0`, `variables_missing=0`, `secrets_missing=0`. |
| Output review | PASS. Reports contained config names and hashes only; no token values or Authorization headers were stored. |
| `git diff --check` | PASS. |
| `.\scripts\security\publication_guard.ps1` | PASS. |

Reference commands:

```powershell
.\scripts\control-plane\generate_enterprise_workflow_templates.ps1 `
  -ProfilePath docs\examples\enterprise-adoption-profile.example.json `
  -OutputDir out\KAN-51-enterprise-workflow-templates `
  -Force

.\scripts\control-plane\validate_enterprise_workflow_installation_readiness.ps1 `
  -PackDir out\KAN-51-enterprise-workflow-templates `
  -Repository yohandry10/Git-Gov `
  -Ref main `
  -ReportOnly `
  -OutputPath out\KAN-51-workflow-readiness.json

git diff --check
.\scripts\security\publication_guard.ps1
```

Expected result against GitGov itself is `needs-action`, because ExampleCo templates are generic customer templates and are not expected to exactly match GitGov's hand-maintained production workflows.

## Residual Risk

The validator proves installation/readiness metadata. It does not prove that workflows will succeed in a customer's runtime, because provider reachability, runner access, branch protection, and secret values still need separate customer-side validation.
