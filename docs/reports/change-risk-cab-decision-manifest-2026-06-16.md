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

- None for KAN-127.

## Production Validation

- PR `#444` merged as `12aff10d` with all required checks passing.
- Production `v67` migration/postcheck passed against the Render/Supabase database.
- Render deploy `dep-d8og45t7vvec73fsgk4g` for `12aff10d` reached `live`.
- Production smoke found that the terminal detail route
  `GET /change-risk/cab-decision-manifests/{manifest_id}?org_name=...` was not reliable for global
  admin keys behind the deployed route/proxy path. Hotfix PRs `#445`, `#446`, and `#447` hardened ID
  parsing; PR `#448` added stable read route
  `GET /change-risk/cab-decision-manifests/{manifest_id}/detail` and moved the Desktop client to it.
- Final Render deploy `dep-d8oh7gu7r5hc73c2tt40` for `9f1c5c9c` reached `live`.
- Final production smoke passed:
  - `/health=ok`.
  - authenticated `/stats=200`.
  - source CAB packet `crcab_23d138be426a4967ae0895810e679a19`.
  - source packet hash stayed
    `sha256:d314caf9c2e41886cdfcbd5e56c841ea84f6329b0c00c7f6f7398a3dbe3b1d9a`.
  - source packet review stayed `needs_mitigation`.
  - created decision manifest `crcabdm_841ddc3eda30a3b0fceffe27fa7e856a`.
  - manifest hash
    `sha256:45badc8d054d0ad3b8c58a0b2d64eb3e998bac2e27d4940123ce06d122c12733`.
  - `/detail`, list, download, revoke, and revoked-download conflict all passed.
  - final manifest status `revoked`, download count `1`, revoked download HTTP `409`.
  - Deployment Gate authorization count stayed `2`.
  - Agent Governance evaluation count stayed `7`.
  - audit rows existed in `admin_audit_log` for `cab_decision_manifest_created`,
    `cab_decision_manifest_downloaded`, and `cab_decision_manifest_revoked`.
