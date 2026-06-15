# KAN-108 Tenant Auditor RBAC MVP

Ticket: `KAN-108`
Issue: GitHub `#377`

## Product Decision

After KAN-107, GitGov has a manual review workflow for Framework Review Reports, but it is still
Admin-only. GPT/product review selected the next slice as a tenant-scoped `Auditor` role because
regulated customers need separation of duties: Admins configure the tenant, while Auditors review
and download compliance evidence without configuration power.

This remains manual-first. It does not add signed manifests, PDF/DOCX exports, official SOC 2/ISO/NIST/PCI/SBS/LGPD mappings, compliance scores, certification claims, BYOM, MCP, chatbot behavior, OPA/Rego execution, provider mutation, policy mutation, deployment authorization mutation, or Agent Governance dependency.

## Scope

- Add `Auditor` to tenant `UserRole`.
- Allow Admins to provision users/API keys/invitations with role `Auditor`.
- Update database role constraints through Supabase migration/postcheck `v51`.
- Add a backend `require_compliance_reviewer` guard for `Admin` or `Auditor`.
- Allow `Auditor` to:
  - read/download KAN-99 Compliance Evidence Exports;
  - read KAN-100 Evidence-to-Control Mappings;
  - read/download KAN-101 Control Mapping Review Packages;
  - read KAN-100 control framework metadata;
  - list/read/download KAN-105/KAN-106 Framework Review Reports;
  - submit KAN-107 Framework Review Report review metadata.
- Keep `Auditor` blocked from:
  - creating evidence exports, mappings, review packages, or reports;
  - importing or reviewing framework packs;
  - managing API keys, org users, org invitations, tenant/platform settings, policies, integrations, Deployment Gates, Agent Governance, or provider configuration.

## Guardrails

- `Auditor` is tenant-scoped. Cross-tenant evidence remains invisible.
- `Auditor` cannot be a platform founder/superadmin role.
- Stale auth cache is blocked for Auditor on sensitive paths, matching the safety expectation for privileged evidence access.
- All evidence artifacts keep existing no-claim flags and immutable hashes.

## E2E Criteria

- Create a real Admin-generated evidence chain: KAN-99 export -> KAN-100 mapping -> KAN-101 review package -> KAN-105 report.
- Create a real `Auditor` API key.
- Verify the Auditor can read/download existing evidence artifacts and submit KAN-107 report review metadata.
- Verify the Auditor cannot create artifacts or mutate admin surfaces.
- Verify another tenant's Auditor cannot read or review the report.
- Verify Developer still cannot review reports.
- Verify no Agent Governance evaluation is created and artifact hashes/no-claim flags stay unchanged.
