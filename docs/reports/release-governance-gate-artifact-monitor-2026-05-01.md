# KAN-49 Release Governance Gate Artifact Monitor Report

Updated: 2026-05-01

## Scope

KAN-49 adds an opt-in freshness monitor for Release Governance Gate artifacts.

The goal is to prove that a customer-selected release governance gate produced reviewable evidence and that the evidence artifact has not expired. This does not make release governance blocking by default.

## Changes

| Area | Change |
| --- | --- |
| GitHub Actions | Added `.github/workflows/release-governance-gate-artifact-monitor.yml`. |
| Shared validation | Reused `scripts/control-plane/validate_github_evidence_report_artifact.ps1` with `WorkflowFile=release-governance-gate.yml` and `ArtifactNamePrefix=release-governance-gate-`. |
| Enterprise CLI pack | Added `release-governance-gate-artifact-monitor.yml` only when formal release governance and artifact monitoring are both selected. |
| Dashboard pack | Added the same opt-in workflow template to dashboard workflow pack exports. |
| Documentation | Added design doc and updated the release governance gate runbook. |

## Configurable Behavior

The monitor is not generated for the default `record-only` release governance mode.

It is generated only when:

- `formal-approval` is enabled.
- release governance has a non-`record-only` base policy or environment override.
- `artifact-monitoring` is enabled.

This preserves the product rule requested by the owner: quorum, multi-approver, enforcement, and release governance monitors are customer-selected capabilities, not defaults.

## Secret Safety

- No `.env` files are read.
- No provider tokens are read.
- No Authorization headers are printed.
- Generated customer workflow templates contain names of variables/secrets only.
- The monitor artifact contains artifact metadata only.

## Validation

Local validation completed on 2026-05-01:

| Command | Result |
| --- | --- |
| `.\scripts\control-plane\validate_github_evidence_report_artifact.ps1 -Repository yohandry10/Git-Gov -WorkflowFile release-governance-gate.yml -ArtifactNamePrefix release-governance-gate- -MaxAgeHours 720 -OutputPath out\release-governance-gate-artifact-monitor.json` | PASS. Latest successful gate run `25208470238` had fresh artifact `release-governance-gate-25208470238`. |
| `.\scripts\control-plane\generate_enterprise_workflow_templates.ps1` with a production environment override profile | PASS. Generated both `release-governance-gate.yml` and `release-governance-gate-artifact-monitor.yml`; monitor uses prefix `release-governance-gate-` and max age `720`. |
| `cd gitgov; npm test -- --run src/test/components/dashboard-helpers.test.ts` | PASS. `18` tests. |
| `cd gitgov; npm run typecheck` | PASS. |
| `cd gitgov; npm run lint` | PASS. |
| `cd gitgov; npm test -- --run` | PASS. `25` files, `286` tests. |
| `cd gitgov; npm run build` | PASS with existing Vite large chunk warning. |
| `git diff --check` | PASS. |
| `.\scripts\security\publication_guard.ps1` | PASS. |

Reference commands:

```powershell
.\scripts\control-plane\validate_github_evidence_report_artifact.ps1 -Repository yohandry10/Git-Gov -WorkflowFile release-governance-gate.yml -ArtifactNamePrefix release-governance-gate- -MaxAgeHours 720 -OutputPath out\release-governance-gate-artifact-monitor.json
```

## Residual Risk

The monitor can only validate artifacts that still exist in GitHub Actions. If a customer uses shorter artifact retention or deletes artifacts manually, the monitor fails and the operator must rerun the Release Governance Gate to produce fresh evidence.
