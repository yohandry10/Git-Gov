# KAN-127 Change Risk CAB Decision Manifest MVP

## Decision

KAN-127 freezes the final evidence of a reviewed Change Risk CAB Packet into an append-only,
hashable JSON manifest. It closes the KAN-125/KAN-126 loop without changing the packet, source
evaluations, deployment gates, providers, repositories, or Agent Governance state.

## Scope

- New `change_risk_cab_decision_manifests` table.
- Manifest schema `gitgov_change_risk_cab_decision_manifest.v1`.
- Backend routes:
  - `POST /change-risk/cab-packets/{packet_id}/decision-manifests`
  - `GET /change-risk/cab-packets/{packet_id}/decision-manifests`
  - `GET /change-risk/cab-decision-manifests/{manifest_id}`
  - `GET /change-risk/cab-decision-manifests/{manifest_id}/download`
  - `PATCH /change-risk/cab-decision-manifests/{manifest_id}/revoke`
- Tauri DTOs, client methods, commands, and invoke registration.
- Control Plane store actions/state.
- Governance > Releases CAB packet detail `Decision Manifest` panel.

## Invariants

- Admin creates and revokes.
- Admin/Auditor reads and downloads.
- Developer and Agent Governance keys are denied.
- Tenant isolation is enforced through the resolved org.
- A manifest can only be created for an active CAB packet with a non-pending manual disposition.
- Downloads are blocked after revoke.
- The manifest carries `advisory_only=true`, `llm_used=false`, `agent_governance_used=false`,
  `compliance_claim=false`, and `certification=false`.

## Non-Goals

No enforcement, release blocking, deployment execution, provider/repo mutation, AI/LLM/BYOM/MCP,
required Agent Governance, compliance score, certification/legal/regulatory claim, PDF/DOCX,
public links, email, scheduler, or source artifact mutation.
