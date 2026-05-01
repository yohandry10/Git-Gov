# KAN-47 Release Governance Enforcement Gate

Updated: 2026-05-01

## Summary

KAN-47 adds an optional enforcement gate for the KAN-46 release governance evaluator.

The gate is intentionally not default release blocking. It can fail only when enforcement is explicitly requested and the evaluator reports an unsatisfied blocking policy.

## Traceability

- Jira issue: `KAN-47 - Add optional release governance enforcement gate`.
- Branch: `ops/KAN-47-release-governance-enforcement-gate`.
- Design: `docs/design/release-governance-enforcement-gate-mvp.md`.
- Runbook: `docs/runbooks/release-governance-gate.md`.

## Changes

- Added `scripts/control-plane/validate_release_governance_gate.ps1`.
- Added manual workflow `.github/workflows/release-governance-gate.yml`.
- Updated CLI workflow template generation to include `release-governance-gate.yml` only when the profile has explicit non-`record-only` release governance and `formal-approval`.
- Updated dashboard workflow template pack generation with the same inclusion rule.
- Added dashboard helper tests for release governance gate template generation.
- Documented gate behavior and operator runbook.

## Product Behavior

- `record-only` remains non-blocking.
- Report-only mode exits successfully even when it observes a potential governance warning.
- `-Enforce` fails only on `blocking=true`.
- `-FailOnWouldBlock` is available when an operator wants advisory/would-block states to fail.
- `-RequirePolicySatisfied` is available for stricter policy checks.

## Local Validation

Completed locally so far:

- `.\scripts\control-plane\validate_release_governance_gate.ps1 -RepositoryFullName yohandry10/Git-Gov -ReleaseId KAN-47 -Environment production -OrgName yohandry10 -OutputPath out\KAN-47-release-governance-gate-report.json`: passed with `status=recorded`, `policy_mode=record-only`, `blocking=false`, `would_block=false`.
- `.\scripts\control-plane\validate_release_governance_gate.ps1 -RepositoryFullName yohandry10/Git-Gov -ReleaseId KAN-47 -Environment production -OrgName yohandry10 -Enforce -OutputPath out\KAN-47-release-governance-gate-enforce-report.json`: passed because current policy is `record-only` and not blocking.
- `.\scripts\control-plane\generate_enterprise_workflow_templates.ps1 -ProfilePath docs\examples\enterprise-adoption-profile.example.json -OutputDir out\KAN-47-enterprise-workflow-templates -Force`: passed; record-only profile generated `13` templates and no release governance gate.
- Quorum opt-in validation generated `14` templates and included `.github/workflows/release-governance-gate.yml`.
- YAML parse validation passed for `.github/workflows/release-governance-gate.yml` and the generated release governance gate template.
- `npm test -- --run src/test/components/dashboard-helpers.test.ts`: passed, `16` tests.
- `npm run typecheck`: passed.
- `npm run lint`: passed.
- `npm run build`: passed with the existing Vite large chunk warning.
- `npm test -- --run`: passed, `25` test files and `284` tests.
- `git diff --check`: passed.
- `.\scripts\security\publication_guard.ps1`: passed.

## Secret Safety

- No token values, Authorization headers, provider credentials, or `.env` values are printed.
- Generated template packs include secret names only.
- Script output records whether an evidence hash was present, not the hash value itself.
- The gate report omits approval identity details by default.

## Residual Work

- Validate the workflow manually on GitHub after PR merge.
- If customers need stricter policy semantics, add per-environment/per-release profile settings instead of changing default behavior.
