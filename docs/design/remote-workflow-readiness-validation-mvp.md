# KAN-51 Remote Workflow Readiness Validation MVP

Updated: 2026-05-01

## Summary

KAN-51 adds a read-only validator for checking whether a customer GitHub repository is ready after GitGov workflow templates have been installed.

It answers:

- Are the expected workflow files present?
- Do installed workflow files exactly match the generated template pack?
- Are the required GitHub Actions variable names present?
- Are the required GitHub Actions secret names present?

The validator does not read secret values and does not mutate the repository.

## Script

```text
scripts/control-plane/validate_enterprise_workflow_installation_readiness.ps1
```

Supported sources:

- `-PackDir`: output from `generate_enterprise_workflow_templates.ps1`.
- `-PackPath`: dashboard workflow template pack JSON.

Supported target:

- GitHub repository in `owner/repo` format.
- If `-Repository` is omitted, the script can infer `repository_full_name` from the pack manifest.
- If `-Ref` is omitted, the script can infer `default_branch` from the pack manifest, then falls back to `main`.

## Behavior

Report-only example:

```powershell
.\scripts\control-plane\validate_enterprise_workflow_installation_readiness.ps1 `
  -PackDir out\enterprise-workflow-templates `
  -Repository owner/repo `
  -Ref main `
  -ReportOnly `
  -OutputPath out\workflow-readiness.json
```

Blocking example:

```powershell
.\scripts\control-plane\validate_enterprise_workflow_installation_readiness.ps1 `
  -PackDir out\enterprise-workflow-templates `
  -Repository owner/repo `
  -Ref main `
  -OutputPath out\workflow-readiness.json
```

Without `-ReportOnly`, the script exits non-zero when readiness is not complete.

## Status Model

Overall status:

- `ready`: workflows match and all required variable/secret names are present.
- `needs-action`: at least one workflow is missing/different, or a required variable/secret name is missing.

Workflow statuses:

- `matched`: remote workflow exists and its content matches the pack.
- `different`: remote workflow exists but differs from the pack.
- `present`: remote workflow exists, but no content comparison was available.
- `missing`: remote workflow is not present at the selected ref.

Variable/secret statuses:

- `present`.
- `missing`.

## Safety

- No `.env` files are read.
- No provider secret values are read.
- GitHub Actions secrets are checked by name only through the GitHub metadata API.
- No repository files, branches, PRs, variables, secrets, branch protection, or provider settings are mutated.
- The JSON report stores workflow hashes and config names only, not workflow contents or secret values.

## Non-Goals

- No GitHub Actions variable creation.
- No GitHub Actions secret creation.
- No workflow dispatch.
- No PR merge or branch protection mutation.
- No claim that customer workflow commands are safe beyond matching the reviewed pack.
