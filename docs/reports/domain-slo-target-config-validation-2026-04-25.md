# Domain SLO Target Config Validation

Date: 2026-04-25

## Scope

Static guardrail for `ops/slo/domain-slo-targets.json`.

## Change

- Added `scripts/control-plane/validate_domain_slo_target_config.ps1`.
- CI `Workflow Lint` validates the SLO target lock file without requiring GitGov API secrets.
- `.github/workflows/domain-slo-validation.yml` validates the lock file before deciding whether live validation can run.

## Required Scope

For the current production GitGov repository, every domain entry must define:

- `org_name`
- `repo_full_name`
- `branch`

This prevents accidental unscoped telemetry reads that can overstate Jira traceability gap or other live SLO metrics.

## Local Command

```powershell
.\scripts\control-plane\validate_domain_slo_target_config.ps1 `
  -TargetsPath ops\slo\domain-slo-targets.json `
  -RequireOrgName `
  -RequireRepoFullName `
  -RequireBranch
```
