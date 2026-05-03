# GitGov Public Agent Context

Updated: 2026-05-03
Ticket: `KAN-76`

This document gives external agents and research models a safe, public, repo-tracked view of the product state after the documentation reality audit completed in `KAN-70` through `KAN-75`.

It exists because some older forensic or strategy notes remain intentionally ignored by Git. Do not force-add those restricted files. Use this document, `docs/CURRENT_CONTEXT.md`, and `docs/IMPLEMENTATION_STATUS.md` as the public agent-readable context.

## Current Product Phase

GitGov is past the phase of adding standalone hardening workflows, monitor chains, and isolated feature fragments by default.

The current direction is product/UX consolidation:

- make GitGov easier to use.
- package existing capabilities into a guided experience.
- tell the operator what to do next instead of showing another disconnected report.
- keep `KAN-69 - Enterprise Action Center guided UX` pending as the next product work.

The product should not become an "AWS 2.0" style maze of separate surfaces. The next valuable work is a guided Action Center that reuses what already exists.

## What Is Already Implemented

The repo already contains substantial product surface:

- Tauri desktop app with React/TypeScript control-plane UI.
- Rust/Axum backend with authenticated routes, webhook ingestion, compliance signals, policy checks, jobs, exports, SSE, metrics, and admin endpoints.
- GitHub webhook evidence, Jira integration, Jenkins integration, local SonarQube governance, release readiness, evidence packets, release approvals, adoption profiles, workflow templates, provider validation, onboarding readiness, remediation exports, guided checklist tracking, and governance copilot surfaces.
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
- `KAN-69 - Enterprise Action Center guided UX` is pending as product/UX work.
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
- use Jira-ticketed docs branches for any public context changes.

## Conclusions From The External Deep Research Report

The external report is useful as directional product strategy, not as the source of current implementation truth. Its durable conclusions are:

- The valuable core is "Connect. Trace. Prove." rather than generic AI DevOps.
- Existing capability inventory is large; the next stage should package it, not keep expanding it sideways.
- Risk scoring, policy decisions, permissions, and enforcement should stay deterministic; LLMs should explain and compose evidence, not decide critical controls.
- MCP can become useful later, but should not define the product before the core UX is coherent.
- A guided Integration Hub or Action Center is the right kind of product work because it turns existing pieces into a user path.

Implementation details in that report may be outdated because `KAN-70` through `KAN-75` already reconciled public documentation and several capabilities were implemented before this context file. For current facts, use the tracked docs listed above.

## Recommended Next Work

Return to `KAN-69 - Enterprise Action Center guided UX`.

Recommended product shape:

- Dashboard-first Action Center inside the existing Enterprise Adoption area.
- Reuse adoption profile, provider health, workflow templates, remote workflow readiness, onboarding readiness, remediation, checklist tracking, release governance, evidence packets, and copilot evidence.
- Show current state, next recommended action, why it matters, and one primary action.
- Avoid creating another standalone report chain unless it directly improves the guided workflow.

Non-goals for the next phase:

- no SonarCloud proposal.
- no Jenkins trigger-only setup unless explicitly requested.
- no OpenAPI/SDK blocker.
- no broad MCP implementation before the UX path is coherent.
- no new hardening/monitor/trend chain unless tied to a confirmed risk or explicit customer-selected policy.
