# KAN-127 Change Risk CAB Decision Manifest Report

## Implemented Locally

- Supabase migration/postcheck `v67`.
- Backend append-only `change_risk_cab_decision_manifests`.
- JSON manifest schema `gitgov_change_risk_cab_decision_manifest.v1`.
- Create/list/get/download/revoke routes.
- Admin create/revoke, Admin/Auditor read/download, Developer/Agent denial, tenant isolation.
- Tauri DTOs, client methods, commands, and invoke registration.
- Control Plane store state/actions and focused store coverage.
- Governance > Releases `Decision Manifest` panel under selected CAB Packet detail.

## Local Validation

- Backend `cargo check`.
- Backend `cargo fmt --check`.
- Backend `cargo clippy -- -D warnings`.
- Backend `cargo test --no-run`.
- Tauri `cargo check`.
- Tauri `cargo fmt --check`.
- Tauri `cargo clippy -- -D warnings`.
- Tauri `cargo test` (`49` passed).
- Frontend `pnpm --dir gitgov typecheck`.
- Frontend `pnpm --dir gitgov lint`.
- Frontend full Vitest `pnpm --dir gitgov test` (`380` passed).
- Frontend build `pnpm --dir gitgov build` passed with the pre-existing Vite large chunk warning.
- Focused frontend store test:
  `pnpm --dir gitgov exec vitest run src/test/useControlPlaneStore.test.ts` (`46` passed).
- Focused backend real Postgres test:
  `cargo test change_risk_cab_packets_are_hashable_manual_artifacts_without_mutation -- --nocapture`.
- Real Postgres migration/postcheck `v67` in rollback transaction.
- `git diff --check`.
- `.\scripts\security\publication_guard.ps1`.

## Product Guardrails Verified

- Source CAB packet hash remains unchanged.
- Source Change Risk evaluation trace hash remains unchanged.
- Deployment Gate authorization count remains unchanged.
- Agent Governance evaluation count remains unchanged.
- Manifest download count increments.
- Revoked manifest download returns conflict.
- Manifest audit actions are recorded:
  - `cab_decision_manifest_created`
  - `cab_decision_manifest_downloaded`
  - `cab_decision_manifest_revoked`
- No-claim flags remain false for compliance/certification/legal/regulatory claims.

## Pending

- PR checks.
- Merge to `main`.
- Production `v67` migration/postcheck.
- Render deploy and production smoke.
