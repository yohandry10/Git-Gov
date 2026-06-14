# Governance Evidence Review UI MVP

Updated: 2026-06-14

Ticket: `KAN-102`

## Decision

KAN-102 makes the completed KAN-99/KAN-100/KAN-101 compliance evidence primitives usable from the Desktop Governance UI.

GPT/product review selected this before customer-provided framework import because the backend already created real value, but it still required API knowledge. The product gap was visibility and an explicit manual flow for Admin/Auditor-style review.

## Scope

- Add a Governance > Releases panel named `Governance Evidence Review`.
- Let an Admin select a recent Deployment Gate authorization.
- Generate a KAN-99 Compliance Evidence Export from the selected authorization.
- Generate a KAN-100 Evidence-to-Control Mapping from that export.
- Generate a KAN-101 Control Mapping Review Package from that mapping.
- Download the server-generated review package JSON.
- Show artifact ids, hashes, control counts, missing evidence, and no-claim flags.
- Keep the flow explicit and manual-first; no agent is required.

## Product Guardrails

The UI must show that the flow organizes governance evidence for review only.

Required visible flags:

- `compliance_claim=false`
- `regulatory_claim=false`
- `requires_auditor_review=true`
- `certification=false`

Forbidden scope:

- SOC 2/ISO/NIST/PCI/SBS/LGPD official pack
- Customer-provided framework import
- Compliance score or badge
- Certification language
- Auditor signature workflow
- OPA/Rego execution
- Policy mutation
- Provider mutation
- MCP, chatbot, BYOM, or required agent behavior

## Implementation Shape

- Tauri models mirror existing backend JSON contracts for:
  - Compliance Evidence Export
  - Evidence-to-Control Mapping
  - Control Mapping Review Package
- Tauri client methods call the existing `/compliance/...` backend routes.
- Tauri commands expose those methods to React.
- Zustand action slice `compliance` keeps flow state separate from enterprise release approvals.
- React component `ComplianceEvidenceFlowPanel` mounts under Governance > Releases before Deployment Gate History.

## Validation Requirements

Local validation must cover:

- TypeScript typecheck.
- Focused store test proving exact Tauri payloads:
  - `scope=deployment_gate`
  - `framework_id=gitgov_release_governance_baseline_v1`
  - JSON review package generation
  - server package download
- Focused UI test proving:
  - no-certification warning is visible
  - buttons unlock in the correct order
  - missing evidence is visible
  - no-claim flags are visible
  - JSON download uses the server artifact
- Tauri `cargo check`.

Production validation should use a real recent Deployment Gate authorization, then generate export, mapping, package, and download through the UI-backed command path or equivalent authenticated API smoke.
