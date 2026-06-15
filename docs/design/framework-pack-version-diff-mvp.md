# KAN-112 Framework Pack Versioning And Diff MVP

Updated: 2026-06-15

## Decision

After KAN-111, the next 0.3 Regulatory Framework Mapper slice is framework pack versioning and diff, not official regulatory mapping, DOCX export, compliance scoring, BYOM, MCP, or broader Agent Governance.

The product gap is practical: before a customer uses a new internal control pack version in audit evidence, an Admin must understand what changed from the previously reviewed pack. This is especially important for banks and regulated customers that need manual review evidence and may not allow agents.

## Scope

- Add a read-only Admin endpoint:

```text
GET /compliance/framework-packs/diff?org_name=<tenant>&base_pack_id=<cfp_...>&target_pack_id=<cfp_...>
```

- Compare only same-tenant, customer-owned packs with `source=customer_provided`.
- Require both packs to be no-claim packs:
  - `compliance_claim=false`
  - `regulatory_claim=false`
  - `gitgov_certifies=false`
  - `official_regulatory_mapping=false`
  - `requires_auditor_review=true`
- Require both raw packs to share the same original customer framework id from `framework.id`.
- Return:
  - base and target pack metadata.
  - original framework id.
  - summary counts for `added`, `removed`, `changed`, and `unchanged`.
  - per-control diff for `title`, `description`, and `required_evidence_types`.
  - explicit no-claim flags.
- Add Tauri command/client/models.
- Add Desktop Governance Evidence Review diff UI for customer framework packs.
- Add real integration coverage across tenant boundaries, role boundaries, changed controls, mismatch rejection, same-pack rejection, secret redaction, and no Agent Governance side effects.

## Non-Goals

- No official regulatory mapping.
- No compliance score, badge, certification, attestation, or legal claim.
- No OPA/Rego execution.
- No Agent Governance dependency.
- No mutation of mappings, reports, review packages, or framework packs.
- No new persistence table for diff history in this MVP.
- No automatic customer repository mutation.

## Product Behavior

The diff is an inspection tool. It helps an Admin or auditor decide whether a customer-provided pack version is acceptable before generating evidence mappings or reports. It does not approve the pack by itself and does not make GitGov the owner of the customer's framework.

## Validation Plan

- Backend real Postgres integration test imports v1/v2 customer packs, reviews both, compares them, verifies changed/added/removed/unchanged controls, blocks Developer, blocks other tenant, rejects same-pack diff, rejects framework id mismatch, checks no secret-like fixture data leaks, and verifies no Agent Governance evaluation rows are created.
- Frontend store test verifies the Desktop command contract, org scope, ID trimming, stored response, and no-claim flags.
- Rust and TypeScript compile/lint/test/build gates run before PR.
- Production smoke imports two temporary customer framework pack versions into the GitGov tenant, reviews them, calls the diff endpoint, verifies summary/no-claim behavior, and records evidence in the issue.
