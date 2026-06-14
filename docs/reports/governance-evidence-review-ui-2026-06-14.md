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

Pending merge/deploy. The production smoke should reuse a real Deployment Gate authorization and verify the same chain:

`Deployment Gate authorization -> KAN-99 export -> KAN-100 mapping -> KAN-101 review package -> server JSON download`.
