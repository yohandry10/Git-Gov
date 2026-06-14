# KAN-103 Customer-Owned Framework Pack Import

Date: 2026-06-14

## Summary

KAN-103 implements customer-owned framework pack import for the Governance Evidence Review flow.

Customers can paste JSON or YAML framework packs, have GitGov validate them, persist them as tenant-owned control frameworks, map KAN-99 evidence exports against them, and generate KAN-101 review packages that preserve provenance and no-claim flags.

This is not an official regulatory framework import, certification engine, OPA/Rego execution path, compliance score, provider mutation, repository mutation, or Agent Governance requirement.

## Implemented

- Backend route `POST /compliance/framework-packs/import`.
- Backend route `GET /compliance/framework-packs`.
- Backend route `GET /compliance/framework-packs/{framework_pack_id}`.
- Tenant-aware `GET /compliance/control-frameworks` and `GET /compliance/control-frameworks/{framework_id}`.
- KAN-100 mapping support for tenant-owned customer frameworks.
- Generic deterministic evidence-type mapper over KAN-99 artifacts.
- KAN-101 review package framework provenance:
  - `owner_type`
  - `owner`
  - `source`
  - `framework_pack_id`
  - `pack_hash`
  - `official_regulatory_mapping=false`
- Supabase migration `supabase_schema_v46.sql`.
- Supabase postcheck `supabase_schema_v46_postcheck.sql`.
- Tauri DTOs, client methods, and commands for framework listing/import.
- Governance Evidence Review UI controls for framework import, reload, selection, and provenance display.
- Focused backend and frontend tests.

## Safety Rules

Imported framework packs are forced to:

- `owner_type=customer`
- `source=customer_provided`
- `compliance_claim=false`
- `regulatory_claim=false`
- `gitgov_certifies=false`
- `official_regulatory_mapping=false`
- `requires_auditor_review=true`

The import rejects:

- Reserved official/GitGov framework prefixes.
- Unsupported evidence types.
- Duplicate controls.
- Empty controls.
- Oversized fields.
- HTML/script-like text.
- Secret-like metadata keys or values.
- Non-admin callers.

## Local Validation

- `cargo check` in `gitgov/gitgov-server`.
- Focused `compliance_framework_packs` tests with `TEST_DATABASE_URL` on local PostgreSQL: `2` passed.
  - Covers valid JSON import, valid YAML import, forced customer ownership/no-claim flags, reserved framework IDs, unsupported evidence types, duplicate controls, oversized control packs, secret-like metadata rejection, non-admin denial, agent-key denial, tenant isolation, KAN-99 export mapping, KAN-101 review package provenance, and no new Agent Governance evaluations.
- Focused existing `compliance_evidence_mappings` tests with `TEST_DATABASE_URL`: `2` passed.
- Focused existing `compliance_review_packages` tests with `TEST_DATABASE_URL`: `2` passed.
- Full backend tests with `TEST_DATABASE_URL` on local PostgreSQL: `304` passed.
- `npm --prefix gitgov run typecheck`.
- `npm --prefix gitgov test -- ComplianceEvidenceFlowPanel useControlPlaneStore --run`: `37` passed.
- `npm --prefix gitgov test -- --run`: `366` passed.
- `npm --prefix gitgov run lint`.
- `npm --prefix gitgov run build` passed with the existing Vite large chunk warning.
- `cargo check` in `gitgov/src-tauri`.
- `cargo test` in `gitgov/src-tauri`: `49` passed.
- `cargo fmt --check` in `gitgov/gitgov-server`.
- `cargo fmt --check` in `gitgov/src-tauri`.
- Migration `v46` plus postcheck validated in an isolated local PostgreSQL schema seeded with minimal v44 state.
- `cargo clippy -- -D warnings` in `gitgov/gitgov-server`.
- `cargo clippy -- -D warnings` in `gitgov/src-tauri`.
- `git diff --check`.
- Publication guard.
- Vite smoke on `http://127.0.0.1:5174/governance/releases` loaded the bundle with no console/page errors and showed the expected `GitGov Desktop required` state for a Desktop-only surface.

Post-merge validation:

- PR checks.
- Production migration `v46`.
- Render deploy.
- Production smoke for import -> export -> mapping -> review package using a temporary customer framework pack.

## Production Validation

- PR `#363` merged to `main` as `2e9d243`.
- Production `v46` migration and postcheck passed.
- Render deploy `dep-d8nga5km0tmc73basn0g` for `2e9d243` reached `live`.
- `/health` returned `ok`; authenticated `/stats` returned `200`.
- Production smoke used real Deployment Gate authorization `dga_6bbb0ce5200a4d36ae6dc9fac1146c7a`.
- Imported JSON customer framework `customer_kan103_runtime_smoke_17814669449_9655a36246a2`.
- Imported YAML customer framework `customer_kan103_runtime_yaml_178146694497_87ce14558b38`.
- Created KAN-99 evidence export `cee_e04f107ee6e04d0891a40129373ce6ef`.
- Created customer-framework mapping `cem_0277f6372a7a4792a1575130f4b80236`.
- Created KAN-101 review package `crp_003ae13483485afd55a5cd696c605496`.
- Downloaded review package hash matched stored `artifact_hash`.
- Downloaded artifact preserved `owner_type=customer`, `pack_hash`, `compliance_claim=false`, `regulatory_claim=false`, and `requires_auditor_review=true`.
- Agent Governance evaluations stayed unchanged at `7 -> 7`.
