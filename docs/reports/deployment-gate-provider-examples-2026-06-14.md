# KAN-85 Deployment Gate Provider Examples Report

Updated: 2026-06-14

## Implemented

- Added GitHub Actions Deployment Gate example:
  - `docs/examples/deployment-gates/github-actions-deployment-gate.yml`.
- Added Jenkins Pipeline Deployment Gate example:
  - `docs/examples/deployment-gates/Jenkinsfile.deployment-gate`.
- Added GitLab CI Deployment Gate example:
  - `docs/examples/deployment-gates/gitlab-ci-deployment-gate.yml`.
- Added provider examples README:
  - `docs/examples/deployment-gates/README.md`.
- Added validator:
  - `scripts/control-plane/validate_deployment_gate_provider_examples.ps1`.
- Updated roadmap/runbook/current context.

## Validation

Completed locally:

- `scripts/control-plane/validate_deployment_gate_provider_examples.ps1 -OutputPath out/kan-85-provider-examples-validation.json`
  - passed;
  - verified all provider examples call `/deployment-gates/authorize`;
  - verified none call `/enterprise/release-governance/evaluate`;
  - verified required request fields are present;
  - verified evidence artifact handling is present;
  - verified `blocking` and `would_block` handling is present;
  - verified no hardcoded bearer token or direct API-key assignment pattern.

- PowerShell parser check for the validator:
  - passed.
- `git diff --check`
  - passed.
- `scripts/security/publication_guard.ps1`
  - passed.
- Grep checks:
  - no hardcoded bearer token pattern in examples;
  - the old lower-level evaluator route appears only in the validator's negative rule, not in provider examples;
  - `GITGOV_API_KEY` is referenced through provider secret/env mechanisms, not as a committed value.

Pending before completion:

- PR checks and merge.

## Follow-Ups

- Break-glass workflow and authorization evidence.
- Environment policy UX.
- Optional provider install automation after explicit customer authorization.
