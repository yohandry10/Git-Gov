# Customer-Owned Framework Pack Import MVP

Updated: 2026-06-14

Ticket: `KAN-103`

## Decision

KAN-103 adds customer-owned framework pack import after KAN-99, KAN-100, KAN-101, and KAN-102 made the manual evidence review flow usable.

The product rule is strict: GitGov does not certify external frameworks and does not ship official SOC 2, ISO, NIST, PCI, SBS, or LGPD mappings in this slice. A customer can import its own JSON/YAML control pack to organize GitGov evidence for customer or auditor review.

## Scope

- Import a customer-owned framework pack through `POST /compliance/framework-packs/import`.
- Accept JSON object payloads or YAML/JSON text payloads.
- Validate schema, ownership, no-claim flags, evidence types, size limits, duplicate controls, reserved official prefixes, HTML/script-like text, and secret-like metadata.
- Persist framework pack provenance and controls in Postgres through schema `v46`.
- List imported packs and active frameworks scoped to the tenant.
- Allow KAN-100 evidence mappings to use either the GitGov baseline or a tenant-owned customer framework.
- Generate KAN-101 review packages with framework provenance preserved.
- Expose import/list/select controls in Governance > Releases > Governance Evidence Review.

## Guardrails

Imported packs are always:

- `owner_type=customer`
- `source=customer_provided`
- `compliance_claim=false`
- `regulatory_claim=false`
- `gitgov_certifies=false`
- `official_regulatory_mapping=false`
- `requires_auditor_review=true`

Forbidden:

- Official regulatory pack claims.
- Reserved framework prefixes such as `gitgov_`, `official_`, `soc2_`, `iso27001_`, `nist_`, `pci_`, `sbs_`, and `lgpd_`.
- OPA/Rego execution.
- Compliance scores, badges, or certification language.
- Agent Governance dependency.
- Provider mutation or repository mutation.

## Pack Shape

```json
{
  "schema_version": "gitgov_customer_framework_pack.v1",
  "framework": {
    "id": "bank_internal_release_controls",
    "name": "Bank Internal Release Controls",
    "version": "2026.06",
    "description": "Customer-owned internal controls for release evidence review.",
    "owner_name": "Customer Security Office"
  },
  "controls": [
    {
      "control_id": "BRC-DEPLOY-01",
      "title": "Deployment decision captured",
      "description": "Deployment authorization evidence must include the gate decision.",
      "required_evidence_types": ["deployment_gate.decision"]
    }
  ]
}
```

Allowed evidence types:

- `deployment_gate.decision`
- `policy.checksum`
- `policy.source`
- `release_approval`
- `ci_build_evidence`
- `code_change_evidence`
- `pr_review_evidence`
- `quality_gate_result`
- `deployment_target`
- `missing_evidence`
- `audit_trail`
- `deployment_gate.agent_governance_used`

## Validation Requirements

- Migration `supabase_schema_v46.sql` plus `supabase_schema_v46_postcheck.sql`.
- Backend integration test with real PostgreSQL:
  - import customer pack
  - generate real KAN-99 export from Deployment Gate evidence
  - map export to imported framework
  - create KAN-101 review package
  - verify provenance, pack hash, no-claim flags, tenant isolation, and no Agent Governance evaluations
- Negative backend tests for non-admin import, prohibited claims, reserved IDs, unsupported evidence types, and secret-like metadata.
- Existing KAN-100 and KAN-101 tests must keep passing for the GitGov baseline.
- Frontend tests must verify import controls and framework-aware mapping payloads.
