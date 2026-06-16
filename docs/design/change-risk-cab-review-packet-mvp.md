# KAN-125 Change Risk CAB Review Packet

Updated: 2026-06-16

KAN-125 extends the KAN-121 through KAN-124 Change Risk Advisory chain with a manual CAB packet.
The product decision from GPT/product review was to package existing deterministic Change Risk
evaluations into a hashable JSON artifact that a CAB, release manager, or internal auditor can
review offline.

The feature is manual-first. It does not approve, block, deploy, mutate providers, mutate
repositories, call Agent Governance, use AI, create compliance scores, or create certification,
legal, regulatory, or compliance claims.

## Product Scope

The packet answers:

- Which Change Risk evaluations are in this CAB review set?
- Which filters or explicit evaluation IDs selected them?
- What risk level, review status, triggered rules, missing evidence, trace hash, and safe notes did
  each evaluation have at packet creation time?
- Which hash verifies the packet artifact?
- Which no-claim/manual flags prove this is review evidence, not an enforcement decision?

## Implemented Surface

Backend:

- `POST /change-risk/cab-packets`
- `GET /change-risk/cab-packets`
- `GET /change-risk/cab-packets/{packet_id}`
- `GET /change-risk/cab-packets/{packet_id}/download`
- `PATCH /change-risk/cab-packets/{packet_id}/archive`

Database:

- `change_risk_cab_packets`
- Supabase migration/postcheck `v65`
- Append-only packet artifact JSON with `artifact_hash`
- Active/archived lifecycle
- Download count and timestamp. Existing installations are repaired to keep `download_count` as
  `BIGINT`, and the `v65` postcheck enforces that type.

Desktop/Tauri:

- Tauri DTOs, client methods, commands, and invoke registration
- Control Plane store state/actions
- Governance > Releases `ChangeRiskCabPacketsPanel`
- Create packet from current queue filters or from visible evaluations
- Download local JSON artifact
- Archive active packet

## Access Model

- Admin:
  - create packets
  - list/read/download packets
  - archive packets
- Auditor:
  - list/read/download packets
  - cannot create or archive packets
- Developer:
  - denied
- Agent Governance key:
  - denied
- Tenant scope:
  - packet selection cannot include another tenant's evaluations.

## Artifact Contract

Schema version:

```text
gitgov_change_risk_cab_packet.v1
```

Required artifact content:

- packet metadata
- filters
- summary counts by risk level and review status
- evaluation snapshots
- trace hashes
- safe review and mitigation notes
- recommended manual actions
- no-claim flags
- verification instructions
- `verification.packet_hash`

No-claim/manual flags:

- `claims.advisory_only=true`
- `claims.manual_review_packet=true`
- `claims.compliance_claim=false`
- `claims.regulatory_claim=false`
- `claims.certification=false`
- `claims.legal_attestation=false`
- `audit_metadata.llm_used=false`
- `audit_metadata.agent_governance_used=false`
- `audit_metadata.enforcement=false`
- `audit_metadata.deployment_execution=false`
- `audit_metadata.provider_mutation=false`
- `audit_metadata.repository_mutation=false`
- `audit_metadata.source_evaluations_mutated=false`

## Non-Goals

- No release blocking.
- No deployment authorization.
- No policy enforcement.
- No provider or repository mutation.
- No approval quorum.
- No notification or scheduler.
- No public links.
- No email or Slack delivery.
- No PDF/DOCX export.
- No AI/LLM/BYOM/MCP/chatbot behavior.
- No Agent Governance dependency.
- No compliance score, badge, certification, official regulatory mapping, or legal attestation.

## Real Validation

The focused backend integration test creates real low, medium, and high Change Risk evaluations in a
real Postgres test schema, updates manual review states, creates CAB packets by filter and by
explicit evaluation IDs, downloads and archives packets, validates RBAC and tenant isolation, checks
audit events, and verifies that source evaluations, Deployment Gate authorizations, and Agent
Governance evaluations are not mutated.

The focused store test verifies command payload normalization and state transitions for list,
create, download, and archive.

Production validation on 2026-06-16 created, listed, read, downloaded, archived, and blocked
download of an archived CAB packet against `https://gitgov-api.onrender.com` after hotfix PR `#437`
and Render deploy `dep-d8oemvuq1p3s73fecrug`. The smoke also verified `download_count` is `bigint`,
verified no-claim flags, and confirmed Deployment Gate authorization plus Agent Governance
evaluation counts did not change.
