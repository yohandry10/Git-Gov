# Control Mapping Review Package MVP

Updated: 2026-06-14

Ticket: `KAN-101`

## Decision

KAN-101 adds a Control Mapping Review Package over the completed KAN-99 and KAN-100 primitives.

The package is a JSON-only, hashable artifact that a customer can store, download, attach to an audit workspace, and verify. It is built from a persisted KAN-100 Evidence-to-Control mapping, which itself references a KAN-99 Compliance Evidence Export.

This is not a compliance certification, not a regulatory report, and not official SOC 2, ISO, NIST, PCI, SBS, or LGPD mapping.

## Scope

- `POST /compliance/review-packages`
- `GET /compliance/review-packages/{review_package_id}`
- `GET /compliance/review-packages/{review_package_id}/download`
- New table `compliance_review_packages`
- JSON artifact schema `gitgov_control_review_package.v1`
- Deterministic `mapping_hash`
- Idempotent `crp_...` package id for the same org, mapping, schema, and mapping hash
- Required flags:
  - `compliance_claim=false`
  - `regulatory_claim=false`
  - `requires_auditor_review=true`
  - `certification=false`

## Non-Scope

- Official regulatory framework packs
- Customer-provided YAML/JSON framework import
- PDF or DOCX
- Auditor comments, signatures, or workflow
- Compliance score, badge, or certification claim
- OPA/Rego execution
- Policy-as-Code mutation
- Provider mutation
- MCP, chatbot, BYOM, or agent dependency
- New Agent Governance evaluations

## Artifact Shape

The downloaded JSON includes:

- `schema_version`
- `review_package_id`
- `source.evidence_export_id`
- `source.evidence_export_hash`
- `source.mapping_id`
- `source.mapping_hash`
- `framework.id`
- `framework.version`
- no-claim flags
- control status summary
- control matrix
- aggregated missing evidence
- audit metadata stating the artifact is redacted, contains no raw payload, requires no Agent Governance, uses no LLM decision, and performs no provider mutation

## Product Guardrails

Allowed wording:

> Control evidence review package for customer/auditor review.

Forbidden wording:

- SOC 2 report
- ISO report
- compliance certificate
- certified compliant
- regulatory score

## Validation Requirements

Real tests must run the full chain:

`Deployment Gate -> KAN-99 export -> KAN-100 mapping -> KAN-101 review package -> download`

They must verify:

- Download hash matches stored `artifact_hash`
- Source export hash and mapping hash are preserved
- API, DB, and artifact keep no-claim flags
- Developer keys are forbidden
- Cross-tenant mapping/package access returns not found
- Invalid format and unsupported sections are rejected
- Secret-like raw fixture payloads do not appear in package responses or downloads
- Agent Governance evaluation count does not change
