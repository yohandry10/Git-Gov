# KAN-50 Remote Workflow Installation PR Report

Updated: 2026-05-01

## Scope

KAN-50 adds a remote PR creation path for installing GitGov enterprise workflow templates in customer GitHub repositories.

The implementation is intentionally conservative:

- dry-run by default.
- remote mutation requires `-Apply`.
- existing differing files require `-Overwrite`.
- PRs are draft by default.
- only `.github/workflows/*.yml` and `.github/workflows/*.yaml` are eligible.
- no secret values are read or printed.

## Changes

| Area | Change |
| --- | --- |
| Script | Added `scripts/control-plane/open_enterprise_workflow_template_pr.ps1`. |
| Runbook | Updated `docs/runbooks/enterprise-self-service-adoption.md` with remote PR flow. |
| Design | Added `docs/design/remote-workflow-installation-pr-mvp.md`. |
| Roadmap | Updated Enterprise Self-Service status to include remote workflow installation PRs. |

## Safety

The script creates a remote branch, commit, and PR only with explicit `-Apply`.

Dry-run computes a remote install plan by comparing the pack with the target repository base branch through the GitHub API.

The plan contains metadata and content hashes, not workflow file contents or secret values.

## Validation

Local validation completed on 2026-05-01:

| Command | Result |
| --- | --- |
| `.\scripts\control-plane\generate_enterprise_workflow_templates.ps1 -ProfilePath docs\examples\enterprise-adoption-profile.example.json -OutputDir out\KAN-50-enterprise-workflow-templates -Force` | PASS. Generated the ExampleCo workflow template pack. |
| `.\scripts\control-plane\open_enterprise_workflow_template_pr.ps1 -PackDir out\KAN-50-enterprise-workflow-templates -Repository yohandry10/Git-Gov -BaseBranch main -TicketId KAN-50 -OutputPlanPath out\KAN-50-remote-workflow-pr-plan.json` | PASS. Dry-run only; remote mutation was not performed. Plan result: `create=0`, `update=0`, `skip=0`, `blocked=13`. |
| `.\scripts\control-plane\open_enterprise_workflow_template_pr.ps1 -PackDir out\KAN-50-enterprise-workflow-templates -Repository yohandry10/Git-Gov -BaseBranch main -TicketId KAN-50 -Overwrite -OutputPlanPath out\KAN-50-remote-workflow-pr-overwrite-plan.json` | PASS. Dry-run only; remote mutation was not performed. Plan result: `create=0`, `update=13`, `skip=0`, `blocked=0`. |
| Minimal dashboard-style `-PackPath` smoke with one new workflow file | PASS. Dry-run only; remote mutation was not performed. Plan result: `create=1`, `update=0`, `skip=0`, `blocked=0`. |
| PowerShell parse check for `open_enterprise_workflow_template_pr.ps1` | PASS. |
| Secret/string scan of dry-run plan outputs | PASS. No token, Authorization header, or secret-value assignment patterns found. |
| `git diff --check` | PASS. |
| `.\scripts\security\publication_guard.ps1` | PASS. |

GitHub validation:

| Check | Result |
| --- | --- |
| PR `#154` | Merged into `main` as `eb7482b`. |
| PR checks | PASS: Security Guard, Server Clippy + Check, Desktop Rust Clippy, Frontend Lint + Typecheck, Website Lint + Typecheck + Build, Workflow Lint, quality gate matrix, Sonar Scan + Quality Gate, internal marker guard, Vercel, and Vercel Preview Comments. |
| Post-merge `CI` | PASS, run `25210329452`. |
| Post-merge `Release Readiness Gate` | PASS, run `25210329443`. |

Reference commands:

```powershell
.\scripts\control-plane\generate_enterprise_workflow_templates.ps1 `
  -ProfilePath docs\examples\enterprise-adoption-profile.example.json `
  -OutputDir out\KAN-50-enterprise-workflow-templates `
  -Force

.\scripts\control-plane\open_enterprise_workflow_template_pr.ps1 -PackDir out\KAN-50-enterprise-workflow-templates -Repository yohandry10/Git-Gov -BaseBranch main -TicketId KAN-50 -OutputPlanPath out\KAN-50-remote-workflow-pr-plan.json
```

No `-Apply` validation is expected against the GitGov repository unless an operator deliberately wants to create a real workflow-install PR. The MVP can be validated safely with dry-run because the apply path uses the same validated pack parsing, remote comparison, path safety, and plan calculation before making GitHub API mutations.

## Residual Risk

The script prepares a PR with generated workflow YAML, but it does not guarantee that every workflow command is appropriate for a specific customer repository. Customer owners still need to review the PR and configure GitHub Actions variables/secrets before relying on those workflows.
