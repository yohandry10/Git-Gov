# KAN-84 Deployment Gate History and Workflow Migration Report

Updated: 2026-06-13

## Implemented

- Added Desktop deployment authorization history under `Governance > Releases`.
- Added Tauri DTOs/client method/command for `GET /deployment-gates/authorizations`.
- Added Control Plane store state/action for `deploymentGateAuthorizations`.
- Migrated dashboard workflow template generation from `GET /enterprise/release-governance/evaluate` to `POST /deployment-gates/authorize`.
- Migrated CLI workflow template generation the same way.
- Migrated `validate_release_governance_gate.ps1` to create a persisted deployment authorization and report authorization metadata.
- Updated the release governance gate runbook and roadmap state.

## Validation

Completed locally:

- `npm --prefix gitgov run test -- --run src/test/useControlPlaneStore.test.ts src/test/components/dashboard-helpers.test.ts`
  - `63` tests passed.
- `npm --prefix gitgov run typecheck`
  - passed.
- `npm --prefix gitgov run lint`
  - passed.
- `npm --prefix gitgov run build`
  - passed with the existing Vite large chunk warning.
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`
  - passed.
- `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`
  - passed.
- `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`
  - passed.
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml`
  - `49` tests passed.
- PowerShell parse check for:
  - `scripts/control-plane/validate_release_governance_gate.ps1`;
  - `scripts/control-plane/generate_enterprise_workflow_templates.ps1`.
- Invalid-gate JSON shape check:
  - missing branch, target SHA, and evidence hash fail before network;
  - `authorization.warnings` and `authorization.blocked_by` serialize as arrays.
- Generated an approval-required workflow pack in `out/kan-84-workflow-templates` and verified `release-governance-gate.yml` contains:
  - `POST /deployment-gates/authorize`;
  - `authorization_id = $authorization.authorization_id`;
  - `target_sha = $targetSha`;
  - no lower-level `/enterprise/release-governance/evaluate` call.
- `git diff --check`
  - passed.
- `scripts/security/publication_guard.ps1`
  - passed.
- Vite/browser smoke:
  - `http://127.0.0.1:5174/governance/releases` returned HTTP `200`;
  - in-app browser loaded the route with no console errors;
  - unauthenticated web runtime correctly showed the Desktop-required gate, so authenticated Desktop-only visual validation remains covered by typed component/store tests rather than a live Tauri session.

Pending before production completion:

- PR checks after push;
- merge to `main`;
- production smoke against deployed `GET /deployment-gates/authorizations`.

## Remaining Product Work

- Provider-specific deploy examples.
- Break-glass workflow.
- Environment policy UX.
