# KAN-131 Multi-Repo Executive Governance Snapshot Export MVP

## Product Decision

KAN-131 turns a filtered Executive Governance View into a hashable JSON artifact for executive,
CAB, or internal audit review.

KAN-129 created the read-only multi-repo view. KAN-130 added filters. KAN-131 freezes that filtered
view without adding scoring, enforcement, deploy execution, provider/repo mutation, AI, Agent
Governance dependency, or compliance/certification/legal claims.

GPT/product-leader consultation was performed after KAN-130 in the existing ChatGPT thread. The
returned ordinance selected `KAN-131: Multi-Repo Executive Governance Snapshot Export`.

## Scope

- Add append-only `executive_governance_snapshots`.
- Add create/list/get/download/archive routes under `/executive/snapshots`.
- Reuse `GET /executive/repositories` and KAN-130 filters as the only source.
- Store normalized filters, JSON artifact, artifact hash, repository count, download count, and
  archive metadata.
- Add Tauri commands, Control Plane store actions, and Desktop snapshot controls.

## Artifact

Schema: `gitgov_executive_governance_snapshot.v1`.

The artifact contains metadata, source endpoint, filters, summary totals, repository rows,
artifact hash, and explicit no-claim flags.

## Non-Goals

- No scoring or compliance score.
- No enforcement or release blocking.
- No deploy execution.
- No provider or repository mutation.
- No risk recalculation.
- No automatic Deployment Gate, Change Risk, CAB, or compliance report creation.
- No AI, LLM, BYOM, MCP, chatbot, or Agent Governance dependency.
- No public links, email, scheduler, PDF, or DOCX.
- No certification, legal attestation, official regulatory mapping, or compliance claim.

## Validation Requirements

Real Postgres coverage must verify create/list/get/download/archive, hash recalculation, filtered
repository membership, Admin/Auditor access, Developer/Agent denial, tenant isolation, archived
download conflict, and no mutation to Deployment Gates, Change Risk, CAB, or Agent Governance
source evidence.
