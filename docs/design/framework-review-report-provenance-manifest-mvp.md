# KAN-110 Framework Review Report Provenance Manifest MVP

Updated: 2026-06-15
Ticket: `KAN-110`

## Product Decision

After `KAN-109`, the next enterprise-safe slice is a provenance manifest for reviewed Framework
Review Reports. This comes before PDF/DOCX export, official regulatory mapping seeds, Integration
Wizard, Change Risk Score, Multi-Repo Executive View, or compliance report generation.

The reason is audit integrity: banks and regulated customers need a small artifact that proves what
was reviewed, which source hashes were used, who generated the manifest, and how it links to prior
manifest materializations. This remains manual-first and does not introduce Agent Governance,
official regulatory mappings, or certification claims.

## Scope

- `POST /compliance/framework-review-reports/{report_id}/provenance-manifests` materializes a JSON
  provenance manifest for an existing report.
- `GET /compliance/framework-review-reports/{report_id}/provenance-manifests/{manifest_id}`
  downloads an already materialized manifest.
- The report must already have `review_status=reviewed`; `needs_review`, `needs_changes`, and
  `rejected` return `409 report_not_reviewed`.
- The manifest is stored append-only in `compliance_framework_review_report_manifests`.
- Each manifest records `manifest_hash`, `previous_manifest_hash`, `signature_algorithm`, signer,
  report artifact hash, source hashes, review provenance, assignment/comment summary, and no-claim
  flags.
- Desktop Governance Evidence Review can generate and save the manifest JSON from a reviewed
  report.

## Guardrails

- Manifest generation does not mutate `payload_json_redacted`, `artifact_hash`, source hashes,
  policy, providers, Deployment Gates, or Agent Governance state.
- `compliance_claim=false`, `regulatory_claim=false`, `certification=false`, and
  `requires_auditor_review=true` remain preserved.
- The signature is a deterministic `sha256-provenance-manifest-v1` hash signature over the manifest
  preimage. It is not PKI, a legal attestation, or a third-party auditor signature.
- Assignment-aware authorization from `KAN-109` applies: Admins can generate manifests, and assigned
  Auditors can generate manifests for their assigned reports. Unassigned Auditors are blocked when
  active assignments exist.

## Non-Goals

- No PDF/DOCX export.
- No official SOC 2, ISO, NIST, PCI, SBS, LGPD, or other regulatory mapping.
- No compliance score, certification badge, legal attestation, or official auditor signature.
- No OPA/Rego execution.
- No Agent Governance dependency, LLM decision, BYOM, MCP, or chatbot behavior.
- No provider, policy, framework pack, Deployment Gate, or report artifact mutation.
