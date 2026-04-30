# KAN-33 Workflow Template Generation

Updated: 2026-04-30

## Summary

KAN-33 adds the next Enterprise Self-Service Adoption step: a secret-safe workflow template generator.

This answers the onboarding question directly: onboarding is included in the product plan, and KAN-33 implements the workflow-template part of that onboarding. The remaining onboarding work is automatic or guided installation, direct provider credential checks, and formal release approval.

## Changes

- Added `scripts/control-plane/generate_enterprise_workflow_templates.ps1`.
- The generator reads the existing adoption profile shape used by KAN-29 through KAN-31.
- The generator writes:
  - `workflow-template-manifest.json`.
  - `README.md`.
  - selected `.github/workflows/*.yml` templates.
- The ExampleCo profile generates `13` workflow templates.
- The generated manifest records:
  - customer and repository identity from the profile.
  - selected providers.
  - selected modules.
  - workflow template list.
  - required variable names.
  - required secret names.
  - manual setup checklist.
  - safety flags.

## Generated Example

Validation command:

```powershell
.\scripts\control-plane\generate_enterprise_workflow_templates.ps1 -ProfilePath docs\examples\enterprise-adoption-profile.example.json -OutputDir out\enterprise-workflow-templates -Force
```

Expected output files:

```text
out/enterprise-workflow-templates/README.md
out/enterprise-workflow-templates/workflow-template-manifest.json
out/enterprise-workflow-templates/.github/workflows/ci.yml
out/enterprise-workflow-templates/.github/workflows/secret-scan.yml
out/enterprise-workflow-templates/.github/workflows/public-naming-guard.yml
out/enterprise-workflow-templates/.github/workflows/github-evidence-report.yml
out/enterprise-workflow-templates/.github/workflows/github-evidence-artifact-monitor.yml
out/enterprise-workflow-templates/.github/workflows/github-evidence-trend-report.yml
out/enterprise-workflow-templates/.github/workflows/release-readiness-gate.yml
out/enterprise-workflow-templates/.github/workflows/quality-gate-policy-matrix.yml
out/enterprise-workflow-templates/.github/workflows/sonar-governance.yml
out/enterprise-workflow-templates/.github/workflows/product-vulnerability-review.yml
out/enterprise-workflow-templates/.github/workflows/product-vulnerability-review-artifact-monitor.yml
out/enterprise-workflow-templates/.github/workflows/product-vulnerability-review-trend-report.yml
out/enterprise-workflow-templates/.github/workflows/product-vulnerability-review-trend-enforcement.yml
```

## Safety

No secrets are read, printed, generated, or written.

Generated templates reference secret names only:

- `GITGOV_API_KEY`.
- `SONAR_TOKEN`.

Generated templates reference variable names only:

- `GITGOV_URL`.
- `SONAR_HOST_URL`.
- `SONAR_PROJECT_KEY`.

The generator does not mutate external repositories. Installation remains a manual, reviewed step.

## Validation

Local validation performed:

- Generated the ExampleCo workflow template pack.
- Confirmed the output directory is ignored by git under `out/`.
- Parsed all generated YAML files successfully with PyYAML.
- Confirmed no unresolved `__TOKEN__` placeholders remained in generated YAML.
- `git diff --check` passed.
- `.\scripts\security\publication_guard.ps1` passed.
- Targeted secret-pattern scan over new KAN-33 files returned no matches for committed secret assignments.

## PR Validation

- PR: `#117` - `product(KAN-33): generate enterprise workflow templates`.
- Merge commit: `62b67e5`.
- PR checks passed:
  - `Security Guard`.
  - `Server Clippy + Check`.
  - `Desktop Rust Clippy`.
  - `Frontend Lint + Typecheck`.
  - `Website Lint + Typecheck + Build`.
  - `Workflow Lint`.
  - `Validate quality_gates warn/block matrix`.
  - `Sonar Scan + Quality Gate`.
  - `Vercel`.

Post-merge `main` checks passed:

- `CI` run `25189490341`.
- `Release Readiness Gate` run `25189490316`.
- `Quality Gate Policy Matrix (Optional)` run `25189490347`.
- `Secret Scan` run `25189490317`.
- `SonarQube Governance (Non-Blocking)` run `25189490329`.
- `Public Naming Guard` run `25189490343`.
- `Governance Correlation Smoke (Optional)` run `25189490321`.
- `Desktop Updater Readiness (Optional)` run `25189490319`.

## Remaining Product Work

- Dashboard-triggered workflow pack generation from the persisted profile.
- Explicitly authorized workflow installation into customer repositories.
- Direct provider credential/reachability checks.
- Formal enterprise release approval.
- Vercel AI SDK Copilot over onboarding, evidence, readiness, and findings.
