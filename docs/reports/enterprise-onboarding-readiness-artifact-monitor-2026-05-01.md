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

Remaining validation before closure:

- PR checks.
- first manual monitor workflow run on `main` after merge.

## Current Status

Implementation in progress on branch `ops/KAN-54-enterprise-onboarding-readiness-artifact-monitor`.
