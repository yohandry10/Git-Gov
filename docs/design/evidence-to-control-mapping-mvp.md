# Evidence-to-Control Mapping MVP

Updated: 2026-06-14

Ticket: `KAN-100`

## Decision

After `KAN-99`, GPT/product review selected an evidence mapping slice, but not a strong regulatory framework mapper. The safe product shape is:

```text
KAN-100: Evidence-to-Control Mapping MVP
```

The MVP maps a KAN-99 Compliance Evidence Export to a GitGov-owned control baseline. It organizes evidence for review; it does not certify compliance.

## Baseline

The only supported framework in the MVP is:

```text
gitgov_release_governance_baseline_v1
```

It is a GitGov-owned release-governance baseline, not SOC 2, ISO 27001, NIST, PCI-DSS, SBS Peru, LGPD Brazil, or any other official regulatory framework.

The first catalog contains 10 controls:

- `GG-RG-01` Deployment gate decision recorded.
- `GG-RG-02` Policy source and checksum recorded.
- `GG-RG-03` Human approval evidence captured when required.
- `GG-RG-04` CI/build evidence captured.
- `GG-RG-05` Code review or PR evidence captured.
- `GG-RG-06` Security or quality evidence captured.
- `GG-RG-07` Deployment target and environment recorded.
- `GG-RG-08` Missing evidence and gaps are explicit.
- `GG-RG-09` Audit trail exists.
- `GG-RG-10` Agent Governance not required for manual-first gate evidence.

## API

Admin-only routes:

```text
GET  /compliance/control-frameworks
GET  /compliance/control-frameworks/{framework_id}
POST /compliance/evidence-mappings
GET  /compliance/evidence-mappings/{mapping_id}
```

Create request:

```json
{
  "evidence_export_id": "cee_...",
  "framework_id": "gitgov_release_governance_baseline_v1"
}
```

The response persists and returns:

- `mapping_id`
- `evidence_export_id`
- `evidence_export_hash`
- `framework_id`
- `framework_version`
- `compliance_claim=false`
- `regulatory_claim=false`
- `requires_auditor_review=true`
- control items with `status`, `evidence_refs`, `missing_evidence`, and `notes_safe`

## Non-Scope

The MVP does not include:

- official SOC 2, ISO, NIST, PCI, SBS, LGPD, or other regulatory mappings;
- compliance scores or badges;
- certification claims;
- PDF/DOCX generation;
- LLM-generated mapping;
- OPA/Rego execution;
- Policy-as-Code mutation;
- provider mutation;
- MCP, chatbot, BYOM, or agent execution;
- new Agent Governance evaluations;
- auditor portal workflows.

## Deterministic Statuses

Allowed item statuses:

- `evidence_present`
- `partial`
- `missing`
- `not_applicable`
- `manual_review_required`

No percentage score is produced. Missing evidence is explicit and reviewable.

## Safety Rules

- Mapping source must be a KAN-99 export in the same tenant.
- Mapping stores the KAN-99 `artifact_hash` and references evidence paths instead of duplicating raw provider payloads.
- Mapping does not copy secret-like raw payloads.
- All `/compliance/*` routes are sensitive Admin routes.
- Mapping creation writes an admin audit event.
- Mapping creation does not create `agent_governance_evaluations`.
