# KAN-35 Reviewed Workflow Installation

Updated: 2026-04-30

## Summary

KAN-35 adds a reviewed, dry-run-first workflow installer for Enterprise Self-Service onboarding.

This closes the gap between "GitGov can generate workflow templates" and "an operator can install those templates into a customer repository checkout after review." It deliberately avoids direct GitHub remote mutation.

## Changes

- Added `scripts/control-plane/install_enterprise_workflow_templates.ps1`.
- Supported the CLI workflow-template output directory from KAN-33 through `-PackDir`.
- Supported the dashboard workflow-template JSON pack from KAN-34 through `-PackPath`.
- Added dry-run install plans with per-file `create`, `update`, `skip`, and `blocked` statuses.
- Added explicit `-Apply` for writes.
- Added explicit `-Overwrite` for replacing existing workflow files.
- Added path and pack-safety validation before any write.

## PR

- PR: `#121` - `product(KAN-35): add reviewed workflow installation`.
- Merge commit: `c60c486`.

## Safety

The installer:

- writes only under `.github/workflows`.
- refuses rooted paths, drive-qualified paths, parent directory traversal, nested workflow paths, duplicate paths, null bytes, and non-YAML workflow files.
- refuses packs that declare secret values.
- refuses packs that declare repository mutation behavior.
- does not read local `.env` files.
- does not read provider tokens.
- does not print secret values.
- does not call GitHub APIs or mutate remote repositories.

## Local Validation

Validation passed from the repository root:

```powershell
.\scripts\control-plane\generate_enterprise_workflow_templates.ps1 -ProfilePath docs\examples\enterprise-adoption-profile.example.json -OutputDir out\KAN-35-enterprise-workflow-templates -Force
```

Result:

- generated the ExampleCo workflow template pack.
- generated `13` workflow files plus manifest and README.

```powershell
.\scripts\control-plane\install_enterprise_workflow_templates.ps1 -PackDir out\KAN-35-enterprise-workflow-templates -TargetRepoPath out\KAN-35-install-target-packdir -OutputPlanPath out\KAN-35-install-plan-packdir-dry-run.json
```

Result:

- dry-run passed.
- `create=13`.
- `update=0`.
- `skip=0`.
- `blocked=0`.

```powershell
.\scripts\control-plane\install_enterprise_workflow_templates.ps1 -PackDir out\KAN-35-enterprise-workflow-templates -TargetRepoPath out\KAN-35-install-target-packdir -OutputPlanPath out\KAN-35-install-plan-packdir-apply.json -Apply
```

Result:

- apply passed.
- wrote `13` workflow files into the local simulated checkout.

Dashboard JSON pack validation:

- minimal dashboard-style pack dry-run passed with `create=1`.
- minimal dashboard-style pack apply passed with `create=1`.

Negative validation:

- unsafe path `.github/workflows/../escape.yml` was rejected.
- differing existing workflow file produced dry-run `blocked=1` without writing.

Repository guardrails:

- `git diff --check` passed.
- `.\scripts\security\publication_guard.ps1` passed.
- targeted secret-pattern scan over KAN-35 files returned no committed secret-like assignments.

## Post-Merge Validation

Post-merge `main` checks passed for commit `c60c486`:

- `CI` run `25191857023`.
- `Release Readiness Gate` run `25191857006`.
- `Quality Gate Policy Matrix (Optional)` run `25191857008`.
- `Secret Scan` run `25191856999`.
- `Public Naming Guard` run `25191857012`.
- `SonarQube Governance (Non-Blocking)` run `25191857029`.
- `Governance Correlation Smoke (Optional)` run `25191857024`.
- `Desktop Updater Readiness (Optional)` run `25191857020`.

## Remaining Product Work Before AI SDK

- Formal enterprise release approval.
- Optional future GitHub App or PR-based remote workflow installation if a customer wants GitGov to propose workflow changes directly.

Vercel AI SDK Copilot remains pending until these onboarding surfaces are complete enough to explain a full adoption state.

Follow-up `KAN-36` adds direct provider credential/reachability checks.
