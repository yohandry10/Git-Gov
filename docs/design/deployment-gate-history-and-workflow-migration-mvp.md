# KAN-84 Deployment Gate History and Workflow Migration MVP

Updated: 2026-06-13

## Summary

KAN-84 turns the KAN-83 Deployment Gates API into an operator-visible and customer-template-ready flow.

Scope:

- Desktop reads `GET /deployment-gates/authorizations` and shows recent deploy authorization attempts in `Governance > Releases`.
- Tauri exposes `cmd_server_list_deployment_gate_authorizations`.
- The Control Plane store keeps deployment authorization history separate from human release approvals.
- Generated release-governance workflow templates call `POST /deployment-gates/authorize`.
- `scripts/control-plane/validate_release_governance_gate.ps1` calls `POST /deployment-gates/authorize` and writes the authorization id, decision, policy checksum, warnings, blockers, and nested evaluation.

Out of scope for this slice:

- mutating customer deploy providers;
- branch protection or environment protection updates;
- break-glass execution;
- OPA/Rego execution;
- making blocking the default for record-only customers.

## Product Behavior

`Governance > Releases` now has two distinct surfaces:

- `Release Approvals`: human approval and risk acceptance records.
- `Deployment Gate History`: CI/CD authorization attempts created by the Deployment Gates API.

This separation matters because a release approval is input evidence, while a deployment authorization is the final deploy decision record.

## Workflow Template Behavior

Generated `release-governance-gate.yml` still appears only when the adoption profile explicitly enables formal release governance. The template:

- writes a skipped artifact when GitGov configuration is missing;
- writes a skipped artifact when release-bound evidence hash is missing;
- calls `POST /deployment-gates/authorize` when it has URL, API key, repo, branch, target SHA, environment, deployer, and evidence hash;
- fails only when the customer-selected enforcement flags require failure.

The evidence hash requirement is intentional. The API validates that the evidence packet binding matches repository, branch, target SHA, release id, and environment.

## Follow-Ups

- Provider-specific deployment examples for Jenkins, GitHub Actions, GitLab CI, and other deployers.
- Break-glass workflow design and authorization evidence.
- Environment policy UX that makes production stricter than staging while preserving safe defaults.
