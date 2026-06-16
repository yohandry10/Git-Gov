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
- Production initially exposed a schema drift from the first `v65` application:
  `change_risk_cab_packets.download_count` existed as `integer` while the backend record model reads
  it as `bigint`. Render logs showed a `ColumnDecode` panic at `src/db.rs:740`, producing HTTP `502`
  on `POST /change-risk/cab-packets`. The migration is now explicitly re-runnable and alters
  existing tables with `ALTER COLUMN download_count TYPE BIGINT`; the postcheck now fails if the
  column is not `bigint`.

Other guards:

- `git diff --check` passed.
- `scripts/security/publication_guard.ps1` passed.

Known local validation limit:

- A full backend `cargo test -- --test-threads=2` run exceeded the local `7` minute command timeout
  without returning useful failure output. Backend test compilation passed, and the affected Change
  Risk real Postgres suite passed.

## Production Validation

- PR `#436` merged to `main` as `92db41ac`.
- Hotfix PR `#437` merged to `main` as `44c0744b`.
- Render deploy `dep-d8oemvuq1p3s73fecrug` for `44c0744b` reached `live`.
- Production `v65` migration/postcheck passed after the `download_count` type repair.
- Production smoke passed:
  - `/health=ok`.
  - authenticated `/stats=200`.
  - `download_count` type is `bigint`.
  - `POST /change-risk/cab-packets` created packet
    `crcab_d48e546c08b844189ec4fe6d7d4ed7b2` from accepted-risk evaluation
    `cra_4d59c84859a747789e577ca24945ec50`.
  - record hash verification returned
    `sha256:4a262e527c263a293e8d1febbb756b448bb4400ec63ee92959414e0309bf7199`.
  - list, get, download, archive, and archived-download conflict paths passed.
  - archived packet download returned HTTP `409`.
  - no-claim flags stayed `advisory_only=true`, `llm_used=false`,
    `agent_governance_used=false`, `enforcement=false`, `compliance_claim=false`, and
    `certification=false`.
  - `deployment_gate_authorizations` stayed `2`.
  - `agent_governance_evaluations` stayed `7`.
