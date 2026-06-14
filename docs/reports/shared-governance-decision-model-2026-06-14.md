# KAN-93 Shared Governance Decision Model Report

Date: 2026-06-14

## Scope

KAN-93 adds a shared deterministic governance decision model for Deployment Gates and Agent
Governance.

Implemented:

- `shared-governance-decision.v1` contract builder.
- Deployment Gate authorization records now expose `governance_decision`.
- Deployment Gate details persist `details.shared_governance_decision`.
- Deployment Gate admin audit metadata includes the shared decision.
- Agent Governance evaluations include `evaluation.shared_governance_decision`.
- Desktop/Tauri/frontend models understand the new Deployment Gate `governance_decision` field.
- Deployment Gate history displays the shared decision and makes explicit that agents were not used.
- Integration tests prove Deployment Gates do not create Agent Governance evaluation rows.

## Product Behavior

Deployment Gates remain CI/CD-facing and manual-first.

When a deployment authorization is evaluated, GitGov emits the shared model with:

```json
{
  "consumer_type": "deployment_gate",
  "actor_type": "system",
  "action": "deploy",
  "agent_governance_used": false
}
```

Agent Governance remains optional and emits the same contract with:

```json
{
  "consumer_type": "agent_governance",
  "actor_type": "agent",
  "agent_governance_used": true
}
```

This keeps future audit, Action Center, approval, and reporting work on one decision shape without
making Deployment Gates depend on `/agent-governance/evaluate`.

## Database

No migration is required.

Deployment Gate shared decisions are persisted inside the existing append-only
`deployment_gate_authorizations.details` JSON payload and exposed as a top-level API response field.
Agent Governance shared decisions are embedded inside the existing persisted evaluation JSON.

## Validation

Local validation performed against a real temporary PostgreSQL 16 database on
`127.0.0.1:55436`.

Passed:

- `cargo fmt --manifest-path gitgov\gitgov-server\Cargo.toml --check`
- `cargo check --manifest-path gitgov\gitgov-server\Cargo.toml`
- `cargo clippy --manifest-path gitgov\gitgov-server\Cargo.toml -- -D warnings`
- focused Deployment Gate tests with `TEST_DATABASE_URL`: `10` passed
- focused Agent Governance tests with `TEST_DATABASE_URL`: `10` passed
- full backend tests with `TEST_DATABASE_URL`: `275` passed
- `cargo fmt --manifest-path gitgov\src-tauri\Cargo.toml --check`
- `cargo check --manifest-path gitgov\src-tauri\Cargo.toml`
- `cargo clippy --manifest-path gitgov\src-tauri\Cargo.toml -- -D warnings`
- full Tauri tests: `49` passed
- `npm --prefix gitgov run typecheck`
- focused Deployment Gate history frontend test: `1` passed
- full frontend tests: `361` passed
- `npm --prefix gitgov run lint`
- `npm --prefix gitgov run build` passed with the existing Vite large chunk warning.
- `git diff --check`
- `.\scripts\security\publication_guard.ps1`

Coverage added:

- Deployment Gate advisory history without first governed repo setup returns
  `governance_decision.consumer_type=deployment_gate`.
- Deployment Gate advisory history returns `agent_governance_used=false`.
- Deployment Gate advisory history records missing `first_governed_repo_setup` evidence.
- Deployment Gate authorization creates zero `agent_governance_evaluations` rows in the clean test
  org.
- Deployment Gate blocking approval-required path returns `decision=requires_approval`.
- Shared Deployment Gate policy checksum matches the persisted legacy policy checksum.
- Agent Governance allowed and blocked paths embed `consumer_type=agent_governance`.
- Agent Governance missing deploy context is reflected in shared missing evidence.
- Desktop history renders the shared decision and `agent not used` state.

## Production Validation

PR `#330` merged to `main` as `8a462bd`.

Post-merge `main` checks passed:

- `CI`
- `Release Readiness Gate`
- `Secret Scan`
- `Public Naming Guard`
- `Quality Gate Policy Matrix`
- `Governance Correlation Smoke`
- `Desktop Updater Readiness`
- `SonarQube Governance`

Render deploy `dep-d8n66cn7f7vs73fg79sg` for `8a462bd` reached `live`.

Production smoke passed:

- `/health` returned `ok`.
- authenticated `/stats` returned `200`.
- tenant Agent Governance settings remained `enabled=false`, `mode=manual_only`.
- `KAN-93` release-bound evidence packet generation returned `found=true`.
- `POST /deployment-gates/authorize` created authorization
  `dga_6bbb0ce5200a4d36ae6dc9fac1146c7a`.
- authorization returned legacy `decision=advisory`, `approved=true`.
- authorization returned `governance_decision.consumer_type=deployment_gate`.
- authorization returned `governance_decision.decision=insufficient_evidence`.
- authorization returned `governance_decision.agent_governance_used=false`.
- authorization history returned the same `agent_governance_used=false` shared decision.
- Agent Governance evaluation history for smoke agent `kan93-deployment-gate-smoke` returned
  `total=0`, proving the Deployment Gate smoke did not create agent evaluation rows.

## Status

Implemented and production-validated.
