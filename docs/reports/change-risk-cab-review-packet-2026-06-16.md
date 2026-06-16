# KAN-125 Change Risk CAB Review Packet Validation

KAN-125 packages existing deterministic Change Risk evaluations into a manual CAB review artifact.

The selected slice came from GPT/product review after KAN-124:

- keep GitGov manual-first for banks and regulated customers.
- package selected/filtered Change Risk evaluations as hashable JSON.
- preserve no-claim flags.
- do not add enforcement, release blocking, deploy execution, provider/repo mutation, AI, MCP,
  BYOM, chatbot behavior, Agent Governance dependency, approval quorum, scheduler, public links,
  PDF/DOCX, or compliance/certification/legal/regulatory claims.

## Implemented

- Supabase migration/postcheck `v65`.
- `change_risk_cab_packets` with artifact hash, filters, evaluation IDs, lifecycle, download count,
  and no-claim JSON constraints.
- Backend routes:
  - `POST /change-risk/cab-packets`
  - `GET /change-risk/cab-packets`
  - `GET /change-risk/cab-packets/{packet_id}`
  - `GET /change-risk/cab-packets/{packet_id}/download`
  - `PATCH /change-risk/cab-packets/{packet_id}/archive`
- Artifact schema `gitgov_change_risk_cab_packet.v1`.
- Admin audit actions:
  - `change_risk_cab_packet_created`
  - `change_risk_cab_packet_downloaded`
  - `change_risk_cab_packet_archived`
- Tauri DTOs, client methods, commands, and invoke registration.
- Control Plane store state/actions.
- Governance > Releases `ChangeRiskCabPacketsPanel`.
- Design, roadmap, architecture, status, public context, and handoff docs.

## Local Validation

Completed:

- Backend `cargo fmt --check`.
- Backend `cargo check`.
- Backend `cargo clippy -- -D warnings`.
- Backend `cargo test --no-run`.
- Tauri `cargo fmt --check`.
- Tauri `cargo check`.
- Tauri `cargo clippy -- -D warnings`.
- Tauri `cargo test` passed with `49` tests.
- Frontend `pnpm --dir gitgov typecheck`.
- Frontend `pnpm --dir gitgov lint`.
- Frontend `pnpm --dir gitgov test` passed with `378` tests.
- Frontend `pnpm --dir gitgov build` passed with the pre-existing Vite large chunk warning.
- Focused real Postgres backend test:

```powershell
cargo test change_risk_cab_packets_are_hashable_manual_artifacts_without_mutation -- --nocapture
```

Result: passed with `1` real DB test.

Focused real Postgres Change Risk suite:

```powershell
cargo test change_risk -- --nocapture
```

Result: passed with `3` real DB tests.

The focused backend test covers:

- real low, medium, and high Change Risk evaluations.
- manual review states `reviewed`, `needs_mitigation`, and `accepted_risk`.
- packet creation by filters.
- packet creation by explicit evaluation IDs.
- artifact schema and hash presence.
- no-claim and manual-only flags.
- Auditor list/get/download.
- Developer denial.
- Auditor archive denial.
- Agent Governance key denial.
- tenant isolation for foreign evaluation IDs.
- download counter update.
- archived packet download conflict.
- audit events for create/download/archive.
- unchanged source evaluation review status and trace hash.
- unchanged Deployment Gate authorization count.
- unchanged Agent Governance evaluation count.

Focused frontend store test:

```powershell
pnpm --dir gitgov test src/test/useControlPlaneStore.test.ts
```

Result: passed with `44` tests.

Migration validation:

- `supabase_schema_v65.sql` plus `supabase_schema_v65_postcheck.sql` passed in a real Postgres
  transaction with rollback. The local `DATABASE_URL` was sanitized in memory for `psql` by removing
  the SQLx-only `pgbouncer` URI parameter; no secret values were printed.

Other guards:

- `git diff --check` passed.
- `scripts/security/publication_guard.ps1` passed.

Known local validation limit:

- A full backend `cargo test -- --test-threads=2` run exceeded the local `7` minute command timeout
  without returning useful failure output. Backend test compilation passed, and the affected Change
  Risk real Postgres suite passed.

## Pending Before Merge

- PR checks, merge, Render deploy, production migration, and production smoke.
