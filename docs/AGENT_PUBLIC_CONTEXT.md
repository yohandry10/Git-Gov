# GitGov Public Agent Context

Updated: 2026-06-16
Ticket: `KAN-128` Deployment Gate Risk & CAB Evidence Context

This document gives external agents and research models a safe, public, repo-tracked view of the product state after the documentation reality audit completed in `KAN-70` through `KAN-75`.

It exists because some older forensic or strategy notes remain intentionally ignored by Git. Do not force-add those restricted files. Use this document, `docs/CURRENT_CONTEXT.md`, and `docs/IMPLEMENTATION_STATUS.md` as the public agent-readable context.

## Current Product Phase

GitGov is past the phase of adding standalone hardening workflows, monitor chains, and isolated feature fragments by default.

The current direction is product/UX consolidation plus auditor-ready compliance evidence packaging:

- make GitGov easier to use.
- package existing capabilities into a guided experience.
- tell the operator what to do next instead of showing another disconnected report.
- use the completed `KAN-69 - Enterprise Action Center guided UX` as the first product consolidation surface.
- turn the manual-first Deployment Gates setup into a coherent first-run path before adding broader
  integration hubs, provider mutation, OAuth automation, or AI-driven decisions.

The product should not become an "AWS 2.0" style maze of separate surfaces. The valuable work is to
turn existing primitives into coherent operator and auditor workflows without creating unreviewed
claims.

## What Is Already Implemented

The repo already contains substantial product surface:

- Tauri desktop app with React/TypeScript control-plane UI.
- Rust/Axum backend with authenticated routes, webhook ingestion, compliance signals, policy checks, jobs, exports, SSE, metrics, and admin endpoints.
- GitHub webhook evidence, Jira integration, Jenkins integration, local SonarQube governance, release readiness, evidence packets, release approvals, adoption profiles, workflow templates, provider validation, onboarding readiness, remediation exports, guided checklist tracking, and governance copilot surfaces.
- Manual-first compliance evidence chain through customer framework packs, framework review reports,
  auditor assignments/comments, report review metadata, provenance manifests, PDF exports, period
  compliance reports, retention/custody history, period report provenance manifests, period report
  review/sign-off metadata, KAN-118 saved manual report profiles, and KAN-119 manual share
  packages/offline verification bundles for reviewed Period Compliance Reports.
- First Governed Repo Setup from KAN-80 and the completed KAN-120 Integration Wizard that
  orchestrates state, validation, baseline planning, and completion without storing provider secrets,
  mutating providers/customer repos, executing deploys, creating claims, or depending on Agent
  Governance.
- Change Risk Assessment Advisory from KAN-121, which persists deterministic qualitative risk
  context for changes/releases and Deployment Gate authorizations. It is manual-first, Admin-only in
  the MVP, tenant-scoped, and constrained to `advisory_only=true`, `llm_used=false`,
  `agent_governance_used=false`, `compliance_claim=false`, and `certification=false`.
- Change Risk Rule Catalog & Evaluation Trace from KAN-122 is completed. It extends KAN-121 with
  deterministic ruleset `change_risk_rules.v1`, persisted triggered and non-triggered rule trace,
  `trace_hash`, catalog/trace APIs, Auditor read access, and a Desktop `Why this risk?` view. It
  does not add enforcement, AI, Agent Governance dependency, provider or repository mutation,
  compliance scores, or certification/legal/regulatory claims.
- Change Risk Manual Review & Mitigation Notes from KAN-123 is completed. It adds human review
  metadata over an already explained Change Risk evaluation: review status, safe review notes,
  mitigation notes, decision reason, Admin-only update, Admin/Auditor read, audit evidence, and
  Desktop `Manual Review` controls. It deliberately remains advisory-only and does not add
  enforcement, release blocking, deployment execution, provider or repository mutation, AI/LLM,
  Agent Governance dependency, compliance scores, approval quorum, notifications, or
  certification/legal/regulatory claims.
- Change Risk Review Queue and CAB Evidence Filter from KAN-124 is completed. It adds
  `review_status` filtering to existing Change Risk evaluation lists and a Desktop `Review queue`
  selector so CAB/Admin/Auditor users can find pending manual review work. It is not a score, not
  enforcement, and not an Agent Governance or AI feature.
- Change Risk CAB Review Packet from KAN-125 is completed. It packages existing
  deterministic Change Risk evaluations into hashable `gitgov_change_risk_cab_packet.v1` JSON
  artifacts for manual CAB/internal-audit review. Admins create/archive; Admins and Auditors
  list/read/download. It is not release blocking, deploy execution, policy enforcement, provider or
  repository mutation, AI/LLM/BYOM/MCP/chatbot behavior, Agent Governance dependency, public links,
  email/scheduler, PDF/DOCX, compliance score, certification, legal attestation, or official
  regulatory claim.
- Change Risk CAB Packet Manual Disposition from KAN-126 is completed. It records a human
  CAB disposition over a KAN-125 packet with status, reviewer, safe notes, mitigation, decision
  reason, and follow-up metadata. It deliberately does not approve deployments, block releases,
  mutate providers or repositories, mutate source evaluations, change the packet artifact hash, use
  AI/agents, or create certification/compliance/legal claims.
- Change Risk CAB Decision Manifest from KAN-127 is completed and production-validated. It creates
  append-only, hashable, downloadable, revocable JSON manifests for reviewed CAB packets. The
  manifest binds the source packet hash, included evaluation trace hashes, final CAB disposition,
  reviewer, safe notes, follow-up fields, and no-claim flags. Desktop/API read-without-download uses
  stable route `/change-risk/cab-decision-manifests/{manifest_id}/detail`. It remains manual
  evidence only and does not enforce, block releases, deploy, mutate providers/repos, use AI/Agent
  Governance, or create legal/compliance/certification claims.
- Deployment Gate Risk & CAB Evidence Context from KAN-128 is in progress. It reconnects the completed
  Change Risk/CAB chain to Deployment Gate History through a read-only context endpoint and Desktop
  section. It does not create a new table, recalculate risk, mutate gates/evaluations/packets/manifests,
  create CAB artifacts automatically, enforce releases, deploy, use AI/Agent Governance, or create
  compliance/certification/legal claims.
- Public web documentation, marketing/download content, and AI copilot route in `gitgov-web`.
- CI guardrails for traceability, publication safety, workflows, server/frontend/desktop checks, website build, quality-gate matrix, release readiness, public naming, and local SonarQube governance.

The repository is not at a "build the foundation from scratch" stage.

## Documentation Reality Audit Status

The phased documentation audit is complete:

| Phase | Ticket | Status |
| --- | --- | --- |
| General stale-doc cleanup | `KAN-70` | Completed |
| Backend/API/schema | `KAN-71` | Completed |
| Desktop/dashboard | `KAN-72` | Completed |
| CI/workflows/release automation | `KAN-73` | Completed |
| CI helper/runtime follow-up | `KAN-74` | Completed |
| Public web, roadmap/context, stale public claims | `KAN-75` | Completed |

Do not treat mitigated audit findings from older local notes as active backlog. If a finding is not repeated in `docs/CURRENT_CONTEXT.md`, `docs/IMPLEMENTATION_STATUS.md`, or a current Jira ticket/report, consider it historical context only.

Current non-negotiable decisions:

- Local SonarQube is the selected Sonar runtime for this personal repository.
- Jenkins authenticated API access is the normal agent path.
- OpenAPI remains intentionally partial and is not a product blocker.
- GitHub Issues is now the operational planning surface. The former Jira Cloud project is deactivated, so agents must not wait on Jira reactivation before continuing work. Keep `KAN-*` IDs in branches, commits, and PR titles for GitGov traceability.
- Historical Jira planning records were migrated to closed GitHub Issues `#217` through `#290` (`KAN-4` through `KAN-77`) and labeled `migrated-from-jira`, `historical-record`, and `gitgov-recovered`. `KAN-77` is labeled `reconstructed-from-github` because it had GitHub/GitGov evidence but no Jira snapshot.
- `KAN-69 - Enterprise Action Center guided UX` is implemented as a dedicated `/action-center` desktop route, not as another panel inside the crowded dashboard surfaces.
- KAN-69 follow-up verification merged through PR `#206` as `8a55a6d`; it keeps release guidance conservative when Jira coverage is missing or empty, and prevents known-forbidden admin-only adoption-profile/checklist reads for non-admin users.
- KAN-69 Desktop runtime QA is completed and merged to `main` through PR `#209` (`fix/KAN-69-desktop-runtime-qa-maintainability`) and PR `#211` (`fix/KAN-69-control-plane-workspace-auth`); latest main commit `e0c769d`. Tracked report: `docs/reports/kan-69-desktop-runtime-qa-2026-06-07.md`.
- Current Desktop QA rules: do not remove useful UI information to fix clipping; fix layout when the issue is visual. Do not restart or relaunch the Tauri app during a user's manual validation session unless explicitly asked.
- Current Desktop QA information architecture:
  - `/action-center` is the only owner of the global `Next Action`.
  - Workspace keeps local execution only: CLI, pipeline visualizer, audit trail, file/commit/push controls, `Next local step`, and gates/blockers without duplicating the global recommendation.
  - `/control-plane` is no longer a primary sidebar module; it redirects to `/settings#control-plane`.
  - Settings owns technical Desktop/system configuration. Current tabs are `Preferences`, `Organization`, `Account`, `Repository`, and `System`; `System` merges Control Plane connection/API key/role/scope/transport with Desktop updates.
  - `/governance` is the operational governance module. It defaults to `Evidence` and has sections `Evidence`, `Policy`, `Adoption`, `Releases`, and `Copilot`; there is no generic Governance Dashboard tab.
  - Help/FAQ is a full-width operational support page and uses the canonical `https://gitgov.cloud` URL, not the old Vercel app URL.
  - Settings, primary sidebar, and Governance shell are language-reactive through `i18n`; deeper nested feature panels still need targeted i18n work before claiming full app localization.
- Restricted forensic/strategy docs stay ignored; public agent context lives here.

## How To Read The Repo

Start with:

1. `AGENTS.md`
2. `docs/CURRENT_CONTEXT.md`
3. `docs/IMPLEMENTATION_STATUS.md`
4. `docs/AGENT_PUBLIC_CONTEXT.md`
5. `docs/PUBLICATION_POLICY.md`

Then use targeted docs:

| Need | Public doc |
| --- | --- |
| Architecture and route reality | `docs/ARCHITECTURE.md` |
| Setup and first run | `docs/QUICKSTART.md` |
| Deployment route | `docs/DEPLOYMENT.md` |
| Troubleshooting | `docs/TROUBLESHOOTING.md` |
| Product direction | `docs/design/enterprise-action-center-ux-focus.md` |
| Roadmap context | `docs/design/enterprise-self-service-and-ai-copilot-roadmap.md` |
| Documentation audit evidence | `docs/reports/kan-70-documentation-reality-audit-2026-05-02.md` through `docs/reports/kan-75-public-web-roadmap-claims-audit-2026-05-02.md` |
| Public site content guidance | `gitgov-web/CONTENT_ARCHITECTURE_GUIDE.md` |
| Publication safety | `docs/PUBLICATION_POLICY.md` |

## Ignored Documentation Policy

Some local docs are intentionally ignored:

- `docs/ENTERPRISE_READINESS_DECISION.md`
- `docs/AUDIT_*.md`
- `docs/INTEGRATIONS_AUDIT_*.md`

These are restricted by `docs/PUBLICATION_POLICY.md` and blocked by `scripts/security/publication_guard.ps1`.

They may contain historical forensic findings, strategy notes, local absolute paths, internal validation context, or stale claims that were later resolved. They are useful as local memory, but they are not safe or clean public context for external agents.

Public replacement strategy:

- keep the restricted originals ignored.
- publish sanitized conclusions in tracked docs and reports.
- update `docs/CURRENT_CONTEXT.md` after major state changes.
- use `KAN-*` traceable docs branches for any public context changes.

## Conclusions From The External Deep Research Report

The external report is useful as directional product strategy, not as the source of current implementation truth. Its durable conclusions are:

- The valuable core is "Connect. Trace. Prove." rather than generic AI DevOps.
- Existing capability inventory is large; the next stage should package it, not keep expanding it sideways.
- Risk scoring, policy decisions, permissions, and enforcement should stay deterministic; LLMs should explain and compose evidence, not decide critical controls.
- MCP can become useful later, but should not define the product before the core UX is coherent.
- A guided Integration Hub or Action Center is the right kind of product work because it turns existing pieces into a user path.

Implementation details in that report may be outdated because `KAN-70` through `KAN-75` already reconciled public documentation and several capabilities were implemented before this context file. For current facts, use the tracked docs listed above.

## Current Product Work

`KAN-125 - Change Risk CAB Review Packet` is completed through PR `#436` plus hotfix PR `#437`;
current main commit is `44c0744b`. It follows KAN-121 through KAN-124 by turning
reviewed/filterable Change Risk evaluations into a manual CAB packet artifact.

`KAN-120 - First Governed Repo Setup Integration Wizard` is completed through PR `#419` and main
commit `e244c1c`. It resumes `0.1 Deployment Gates` by turning KAN-80's persisted setup into a
manual-first first-run wizard: state read, create/resume, validate, plan, and complete. It is not a
provider OAuth/mutation wizard, not deploy execution, not compliance certification, and not an Agent
Governance dependency.

`KAN-69 - Enterprise Action Center guided UX` remains implemented and merged through PR `#204` as
main commit `aa7e352`.

Recommended product shape:

- Dedicated Action Center route at `/action-center`.
- Reuse adoption profile, provider health, workflow templates, remote workflow readiness, onboarding readiness, remediation, checklist tracking, release governance, evidence packets, and copilot evidence through Governance and Workspace routes instead of duplicating panels inside Action Center.
- Show current state, next recommended action, why it matters, and one primary action.
- Keep recommendations deterministic, advisory, and non-blocking.
- Treat missing or empty Jira coverage as a conservative release-prep signal before Evidence Packet/release decision guidance.
- Treat persona/lens selection as presentation context, not authorization.
- Let non-admin users see guidance without issuing admin-only profile/checklist reads.
- Avoid creating another standalone report chain unless it directly improves the guided workflow.
- Preserve Workspace dashboard utility: CLI, pipeline visualizer, audit trail, manual commit/push flow, and `Gates / Blockers` context are product surfaces, not disposable decoration. If `Next Action` or another item clips, fix responsive layout, text wrapping, or scroll behavior.
- Treat Control Plane as configuration, not as an overloaded dashboard. Operational governance belongs in `/governance/*`; Control Plane connection/update settings belong in Settings `System`.
- Desktop auth should reuse a valid local GitHub session by default. GitHub identifies the operator, while the GitGov API key authorizes Control Plane role/org/evidence. Forced GitHub Device Flow on every launch should remain an explicit hardening mode, not the normal product default.

Non-goals for the next phase:

- no SonarCloud proposal.
- no Jenkins trigger-only setup unless explicitly requested.
- no OpenAPI/SDK blocker.
- no broad MCP implementation before the UX path is coherent.
- no new hardening/monitor/trend chain unless tied to a confirmed risk or explicit customer-selected policy.
