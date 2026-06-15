# KAN-111 Framework Review Report PDF Export MVP

Updated: 2026-06-15
Ticket: `KAN-111`

## Decision

Implement PDF export for reviewed Framework Review Reports before DOCX, formal regulatory report
templates, official regulatory mapping packs, BYOM, MCP, chatbot behavior, or additional Agent
Governance work.

The PDF is a customer/auditor review artifact. It is not a certification, compliance score, legal
attestation, or official regulatory claim.

## Product Scope

- Source is an existing KAN-105 Framework Review Report.
- Report must be manually marked `reviewed` through the KAN-107/KAN-108/KAN-109 workflow.
- A KAN-110 provenance manifest must already exist. The caller can pass `manifest_id`; otherwise
  GitGov uses the latest manifest for the report.
- Admins and assigned Auditors can create, inspect, and download the PDF.
- Unassigned Auditors are blocked once active assignments exist.
- Other tenants cannot access the report or PDF.
- The source report artifact and provenance manifest are read-only inputs.

## Backend Contract

- `POST /compliance/framework-review-reports/{report_id}/pdf-export`
  creates an append-only PDF export row.
- `GET /compliance/framework-review-reports/{report_id}/pdf-export`
  returns latest or selected PDF export metadata.
- `GET /compliance/framework-review-reports/{report_id}/pdf-export/download`
  returns `application/pdf` bytes and the `x-gitgov-artifact-hash` header.

The persisted row stores:

- `pdf_export_id`
- `report_id`
- `manifest_id`
- `source_report_hash`
- `manifest_hash`
- `pdf_artifact_hash`
- `content_type=application/pdf`
- `page_count`
- no-claim flags: `compliance_claim=false`, `regulatory_claim=false`,
  `certification=false`, `requires_auditor_review=true`
- `created_by_user_id`, `created_at`, and `downloaded_at`

## Desktop Scope

Governance Evidence Review now shows a PDF panel for reviewed reports. The panel can generate a PDF
from the selected/latest manifest and download the generated PDF through the Tauri command bridge.

The Desktop client receives PDF bytes as base64 from Tauri and saves them as an `application/pdf`
Blob. Desktop does not render or mutate report content.

## Non-Goals

- No DOCX export.
- No KMS/HSM signing.
- No official SOC 2, ISO, NIST, PCI, SBS, LGPD, or other regulatory mapping claim.
- No compliance score or certification badge.
- No Agent Governance evaluation dependency.
- No policy mutation or provider mutation.
- No LLM-authored conclusions.

## Validation Expectations

The real integration test must prove:

- PDF creation is blocked before `reviewed`.
- PDF creation requires a provenance manifest.
- Assigned Auditor can create/download.
- Unassigned Auditor, Developer, and other tenant paths are blocked.
- Downloaded bytes are real PDF bytes.
- Downloaded PDF hash matches `pdf_artifact_hash`.
- PDF content includes source report hash, manifest hash, no-claim text, and reviewer provenance.
- Secret-like fixture text does not leak.
- Source report artifact hash stays unchanged.
- No Agent Governance evaluations are created.
