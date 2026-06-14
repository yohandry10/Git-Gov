# KAN-102 Governance Evidence Review UI

Date: 2026-06-14

## Summary

KAN-102 adds the Desktop UI layer for the KAN-99/KAN-100/KAN-101 evidence flow.

The feature lets an Admin start from an existing Deployment Gate authorization, generate a Compliance Evidence Export, map it to the GitGov release governance baseline, create a Control Mapping Review Package, and download the server-generated JSON artifact.

This remains review evidence only. It is not a compliance certification, regulatory report, score, badge, OPA/Rego execution, policy mutation, provider mutation, or Agent Governance requirement.

## Implemented

- New Tauri compliance DTOs.
- New Tauri Control Plane client methods for:
  - `POST /compliance/evidence-exports`
  - `GET /compliance/evidence-exports/{export_id}`
  - `GET /compliance/evidence-exports/{export_id}/download`
  - `POST /compliance/evidence-mappings`
  - `GET /compliance/evidence-mappings/{mapping_id}`
  - `POST /compliance/review-packages`
  - `GET /compliance/review-packages/{review_package_id}`
  - `GET /compliance/review-packages/{review_package_id}/download`
- New Tauri commands wrapping those client methods.
- New Zustand `compliance` action slice.
- New `ComplianceEvidenceFlowPanel` in Governance > Releases.
- UI surfacing for:
  - selected Deployment Gate authorization
  - export id/hash
  - mapping id/control count
  - review package id/hash
  - no-claim flags
  - missing evidence
  - server JSON artifact download
- Focused component and store tests.

## Explicitly Not Implemented

- Customer-provided framework pack import.
- Official SOC 2, ISO, NIST, PCI, SBS, or LGPD framework packs.
- PDF/DOCX output.
- Auditor comments, signatures, approval workflow, or certification.
- Compliance score.
- Agent Governance dependency.
- MCP/chatbot/BYOM behavior.

## Local Validation

- `npm --prefix gitgov run typecheck`
- `npm --prefix gitgov run test -- ComplianceEvidenceFlowPanel useControlPlaneStore --run` (`35` tests passed)
- `npm --prefix gitgov run test -- --run` (`364` tests passed)
- `npm --prefix gitgov run lint`
- `npm --prefix gitgov run build`
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`
- `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`49` tests passed)
- `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`
- `git diff --check`
- `.\scripts\security\publication_guard.ps1`

Browser/Vite smoke opened `http://127.0.0.1:5174/governance/releases` and verified the bundle loaded with no page errors. The browser route shows the expected `GitGov Desktop required` screen because Governance is a Desktop runtime surface, not the public web app.

## Production Validation

Production validation found and fixed one UI payload-contract mismatch before the final docs/patch merge:

- The KAN-99 export route accepts `gate_decision`, `policy`, `readiness`, `approvals`, `evidence`, `gaps`, and `audit`.
- The KAN-101 review-package route accepts `summary`, `source_hashes`, `framework`, `control_matrix`, `missing_evidence`, `no_claims`, and `audit_metadata`.

Final production smoke against `https://gitgov-api.onrender.com` reused a real Deployment Gate authorization and verified:

`Deployment Gate authorization -> KAN-99 export -> KAN-100 mapping -> KAN-101 review package -> server JSON download`.

- Render deploy for merge commit `88cda2a` reached `live` as `dep-d8nbjmp9rddc739n0jj0`.
- Render deploy for final hotfix commit `ba655c2` reached `live` as `dep-d8nbnatckfvc73em0vrg`.
- `/health` returned `ok`.
- Authenticated `/stats` returned HTTP `200`.
- Source authorization: `dga_6bbb0ce5200a4d36ae6dc9fac1146c7a`.
- Created export: `cee_7610ff2db7a44f56875ee2709b486295`.
- Created mapping: `cem_1e731f2983e4451ea89722c48a27adae`.
- Mapping returned `10` controls.
- Created review package: `crp_6f36f65322b3da03f404ee24edd38855`.
- Downloaded artifact schema: `gitgov_control_review_package.v1`.
- Downloaded artifact contained `10` controls and `summary.total_controls=10`.
- `compliance_claim=false`.
- `regulatory_claim=false`.
- `requires_auditor_review=true`.
- `certification=false`.
- `agent_governance_required=false`.
- `policy_mutation=false`.
- `provider_mutation=false`.
- `raw_payload_included=false`.
