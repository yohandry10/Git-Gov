# KAN-54 Enterprise Onboarding Readiness Artifact Monitor Report

Updated: 2026-05-01

## Summary

KAN-54 adds a freshness monitor for the KAN-53 Enterprise Onboarding Readiness evidence artifact.

The monitor proves that recurring onboarding readiness evidence exists and has not expired. It does not enforce that a customer is fully onboarded and does not make release governance blocking by default.

## Changes

| Area | Change |
| --- | --- |
| GitHub Actions | Added `.github/workflows/enterprise-onboarding-readiness-artifact-monitor.yml`. |
| Shared validation | Reused `scripts/control-plane/validate_github_evidence_report_artifact.ps1` with `WorkflowFile=enterprise-onboarding-readiness.yml` and `ArtifactNamePrefix=enterprise-onboarding-readiness-`. |
| Documentation | Added design/report docs and updated the Enterprise Self-Service runbook and roadmap. |

## Monitor Policy

| Setting | Value |
| --- | --- |
| Monitored workflow | `enterprise-onboarding-readiness.yml` |
| Expected artifact prefix | `enterprise-onboarding-readiness-` |
| Monitor artifact | `enterprise-onboarding-readiness-artifact-monitor` |
| Default maximum age | `192` hours |
| Scheduled run | Thursday `14:07 UTC` |
| Manual input | `max_age_hours` |

## Safety

- No `.env` files are read.
- No provider tokens are read.
- No Authorization headers are printed.
- No GitHub Actions variables or secrets are created.
- No customer repositories are mutated.
- No provider settings are mutated.
- No branch protection is changed.
- No workflow dispatch occurs.
- Release blocking remains opt-in only.

## Validation

Local validation before PR:

| Command | Result |
| --- | --- |
| `.\scripts\control-plane\validate_github_evidence_report_artifact.ps1 -Repository yohandry10/Git-Gov -WorkflowFile enterprise-onboarding-readiness.yml -ArtifactNamePrefix enterprise-onboarding-readiness- -MaxAgeHours 192 -OutputPath out\enterprise-onboarding-readiness-artifact-monitor.json` | PASS. Latest successful KAN-53 run `25211644692` had fresh artifact `enterprise-onboarding-readiness-25211644692`, ID `6748421926`, age `0.19h`. |
| `git diff --check` | PASS. |
| `.\scripts\security\publication_guard.ps1` | PASS. |

PR `#162` merged on `main` as `ec99b7c`.

PR checks passed before merge:

- `Security Guard`: passed.
- `Server Clippy + Check`: passed.
- `Desktop Rust Clippy`: passed.
- `Frontend Lint + Typecheck`: passed.
- `Website Lint + Typecheck + Build`: passed.
- `Workflow Lint`: passed.
- `Validate quality_gates warn/block matrix`: passed.
- `Sonar Scan + Quality Gate`: passed.
- `Block internal-assistant markers in branch/commits`: passed.
- `Vercel`: passed.
- `Vercel Preview Comments`: passed.

Post-merge validation for commit `ec99b7c` passed:

- `CI` - run `25212018101`.
- `Release Readiness Gate` - run `25212018102`.
- `Quality Gate Policy Matrix (Optional)` - run `25212018092`.
- `Secret Scan` - run `25212018105`.
- `Public Naming Guard` - run `25212018093`.
- `Governance Correlation Smoke (Optional)` - run `25212018110`.
- `Desktop Updater Readiness (Optional)` - run `25212018091`.
- `SonarQube Governance (Non-Blocking)` - run `25212018095`.

First manual monitor workflow validation passed:

- Workflow: `Enterprise Onboarding Readiness Artifact Monitor`.
- Run: `25212021793`.
- Artifact: `enterprise-onboarding-readiness-artifact-monitor`.
- Artifact ID: `6748551922`.
- Artifact status: not expired.
- Artifact expires at `2026-07-30T11:01:29Z`.

## Current Status

KAN-54 implementation is complete and merged through PR `#162`.

The monitor is operational on `main`: it validates KAN-53 artifact freshness without reading provider secrets, mutating repositories/providers, or treating onboarding `needs-action` as a release-blocking failure.
