# Enterprise Self-Service And AI Copilot Roadmap

Updated: 2026-06-08

Ticket: `KAN-68`

## Decision

GitGov already has the core product promise: it connects pipeline, ticket, review, policy, and release-readiness evidence into a governed record.

The next product work is not to invent a new category. It is to package the proven GitGov operating model so another company can adopt it with less manual setup.

## Current Product Focus After KAN-67

The post-hardening decision is to stop adding standalone technical features by default.

GitGov now needs a usability layer:

```text
Enterprise Action Center UX
```

The Action Center should make the existing capabilities obvious to a customer by showing:

- current onboarding state.
- next recommended action.
- why that action matters.
- one clear button to continue.
- evidence behind the recommendation.
- which actions are read-only and which require explicit operator approval.

This was implemented as:

```text
KAN-69 - Enterprise Action Center guided UX
```

KAN-69 is a dedicated `/action-center` product surface that reuses existing capabilities from `KAN-29` through `KAN-67`. It did not introduce another independent monitor, trend, or enforcement chain. The follow-up Desktop runtime QA then clarified the surrounding IA: Action Center owns the global recommendation, Governance owns operational governance tools, and Settings owns Control Plane connection/configuration.

## Next Product Features

### 1. Enterprise Self-Service Adoption

Status: started in `KAN-29`; the first Desktop adoption profile builder was added in `KAN-30`; persisted profiles were added in `KAN-31`; provider health evidence MVP was added in `KAN-32`; workflow template generation was added in `KAN-33`; Desktop workflow template pack download was added in `KAN-34`; reviewed workflow installation was added in `KAN-35`; direct provider connection validation was added in `KAN-36`; formal release approval backend MVP was added in `KAN-37`; Desktop release approval wizard was added in `KAN-43`; release governance profile policy was added in `KAN-45`; release governance evaluator was added in `KAN-46`; optional release governance enforcement gate was added in `KAN-47`; environment-scoped release governance policy overrides start in `KAN-48`; release governance gate artifact monitoring starts in `KAN-49`; remote workflow installation PRs start in `KAN-50`; remote workflow readiness validation starts in `KAN-51`; consolidated onboarding readiness reporting starts in `KAN-52`; onboarding readiness automation starts in `KAN-53`; onboarding readiness artifact monitoring starts in `KAN-54`; onboarding readiness trend reporting starts in `KAN-55`; onboarding readiness trend monitoring starts in `KAN-56`; onboarding remediation planning starts in `KAN-57`; Desktop remediation export starts in `KAN-58`; guided Desktop onboarding checklist starts in `KAN-59`; persisted guided checklist tracking starts in `KAN-60`; route auth hardening/smoke/trend/enforcement safety chain completed through `KAN-67`; Enterprise Action Center UX was documented in `KAN-68` and implemented in `KAN-69`; the current Desktop runtime QA organizes those capabilities into Action Center, Governance, Settings, and Workspace instead of one oversized Control Plane dashboard.

Current state:

- GitGov already ingests GitHub, Jira, Jenkins, SonarQube, Render/deployment, audit, policy, evidence packet, and vulnerability review signals.
- GitGov already has release readiness, ticket coverage, evidence exports, artifact monitors, trend reports, and security review automation.
- This is proven against the GitGov repository and current provider setup.

Missing product packaging:

- Onboarding flow:
  - connect GitHub.
  - connect Jira.
  - connect Jenkins or SonarQube when used.
  - select repositories and branches.
  - generate recommended workflows.
  - install recommended workflows after review.
    - local checkout installation now exists through `KAN-35`.
    - PR-based remote installation starts in `KAN-50`.
    - remote readiness validation starts in `KAN-51`.
    - consolidated onboarding readiness reporting starts in `KAN-52`.
    - recurring onboarding readiness evidence starts in `KAN-53`.
    - onboarding readiness artifact freshness monitoring starts in `KAN-54`.
    - onboarding readiness trend reporting starts in `KAN-55`.
    - onboarding readiness trend deterioration monitoring starts in `KAN-56`.
    - onboarding remediation planning starts in `KAN-57`.
    - dashboard remediation export starts in `KAN-58`.
    - guided dashboard onboarding checklist starts in `KAN-59`.
    - persisted guided checklist tracking starts in `KAN-60`.
    - direct GitHub App installation remains future optional packaging.
- Configurable workflow templates:
  - audit-only.
  - moderate enforcement.
  - strict enforcement.
- UI configuration:
  - enable or disable modules.
  - set policy thresholds.
  - choose required evidence.
  - review integration health.
- Company policy rules:
  - block on critical or high reachable vulnerabilities.
  - allow medium risk only with documented acceptance.
  - require Jira ticket IDs.
  - require PR review evidence.
  - require fresh evidence artifacts.
- Formal release approval:
  - who approved.
  - when they approved.
  - what risk they accepted.
  - when the approval expires.
  - which evidence packet supported the decision.
  - backend persistence and validation now exist through `KAN-37`.
  - dashboard create/list workflow now exists through `KAN-43`.
  - KAN-44 clarifies that quorum and release-blocking enforcement are opt-in customer policy choices, not defaults.
  - KAN-45 adds the first `release_governance` policy field to the adoption profile, dashboard, backend validation, adoption pack, and workflow template manifest.
  - KAN-46 adds the first evaluator that compares release policy with approval evidence and reports `recorded`, `advisory-warning`, `approved`, `would-block`, or `blocked`.
  - KAN-47 adds the first optional workflow gate that can consume `blocking=true` only when enforcement is explicitly requested.
  - KAN-49 adds an opt-in monitor for the evidence artifact emitted by that gate.

Customer-facing value:

GitGov turns existing delivery tooling into governed release evidence without asking teams to manually assemble screenshots, spreadsheets, or one-off audit notes.

First MVP:

- `scripts/control-plane/generate_enterprise_adoption_pack.ps1`.
- `scripts/control-plane/generate_enterprise_workflow_templates.ps1`.
- `scripts/control-plane/install_enterprise_workflow_templates.ps1`.
- `scripts/control-plane/open_enterprise_workflow_template_pr.ps1`.
- `scripts/control-plane/validate_enterprise_workflow_installation_readiness.ps1`.
- `scripts/control-plane/generate_enterprise_onboarding_readiness_report.ps1`.
- `scripts/control-plane/generate_enterprise_onboarding_remediation_plan.ps1`.
- `scripts/control-plane/validate_enterprise_provider_connections.ps1`.
- `GET /enterprise/release-approvals`.
- `POST /enterprise/release-approvals`.
- `gitgov/src/components/control_plane/ReleaseApprovalPanel.tsx`.
- `GET /enterprise/release-governance/evaluate`.
- `scripts/control-plane/validate_release_governance_gate.ps1`.
- `.github/workflows/release-governance-gate.yml`.
- `.github/workflows/enterprise-onboarding-readiness.yml`.
- `.github/workflows/enterprise-onboarding-readiness-artifact-monitor.yml`.
- `.github/workflows/enterprise-onboarding-readiness-trend-report.yml`.
- `.github/workflows/enterprise-onboarding-readiness-trend-monitor.yml`.
- `docs/design/enterprise-self-service-adoption-mvp.md`.
- `docs/design/workflow-template-generation-mvp.md`.
- `docs/design/dashboard-workflow-template-pack-mvp.md`.
- `docs/design/reviewed-workflow-installation-mvp.md`.
- `docs/design/remote-workflow-installation-pr-mvp.md`.
- `docs/design/remote-workflow-readiness-validation-mvp.md`.
- `docs/design/enterprise-onboarding-readiness-report-mvp.md`.
- `docs/design/enterprise-onboarding-readiness-automation-mvp.md`.
- `docs/design/enterprise-onboarding-readiness-artifact-monitor-mvp.md`.
- `docs/design/enterprise-onboarding-readiness-trend-mvp.md`.
- `docs/design/enterprise-onboarding-readiness-trend-monitor-mvp.md`.
- `docs/design/enterprise-onboarding-remediation-plan-mvp.md`.
- `docs/design/dashboard-onboarding-remediation-export-mvp.md`.
- `docs/design/dashboard-guided-onboarding-checklist-mvp.md`.
- `docs/design/guided-onboarding-checklist-tracking-mvp.md`.
- `docs/design/provider-connection-validation-mvp.md`.
- `docs/design/formal-release-approval-mvp.md`.
- `docs/design/release-approval-dashboard-mvp.md`.
- `docs/design/release-governance-evaluator-mvp.md`.
- `docs/design/release-governance-enforcement-gate-mvp.md`.
- `docs/examples/enterprise-adoption-profile.example.json`.
- `gitgov/src/components/control_plane/EnterpriseAdoptionPanel.tsx`.
- `docs/design/adoption-profile-dashboard-mvp.md`.
- `docs/design/adoption-profile-persistence-mvp.md`.
- `docs/design/provider-health-validation-mvp.md`.
- `docs/design/release-governance-profile-policy-mvp.md`.

This MVP creates a reusable adoption pack from a customer profile, exposes the first dashboard UI for shaping that profile, persists it per organization, shows evidence-based provider health, generates reviewed workflow template packs from both CLI and dashboard, installs those packs into a local customer repository checkout only after dry-run review and explicit `-Apply`, can open a remote draft PR for those workflow templates only after explicit `-Apply`, validates remote workflow/configuration readiness read-only, consolidates onboarding readiness into one customer-facing Markdown/JSON report, turns that readiness report into a prioritized remediation plan through both CLI and dashboard export, shows the same remediation state as a guided dashboard checklist, persists admin tracking notes for that checklist without changing readiness scoring, automates readiness as a recurring/manual GitHub Actions evidence artifact, monitors that readiness artifact for freshness, trends readiness artifacts over time, monitors trend deterioration in report-only mode by default, validates explicitly provided provider credentials without printing secret values, stores formal release approvals with evidence packet hashes and risk expiration, provides a dashboard wizard for create/list approval workflows, carries explicit release governance policy through the adoption profile and generated packs, evaluates a release against that policy when an admin asks, provides an optional manual gate for customers who explicitly select enforcement, starts per-environment overrides so production can be stricter than staging without changing the safe default, and can monitor the gate artifact when the customer also selects artifact monitoring. It does not yet create GitHub Actions variables/secrets, mutate branch protection, require cryptographic signatures, or block releases from approval state by default.

Release governance default:

- GitGov defaults to `record-only` release approval behavior.
- `record-only` means approvals and evidence can be saved and reported, but customer pipelines are not blocked by default.
- Multi-approver quorum must be explicitly enabled by customer policy.
- Blocking release enforcement must be explicitly enabled by customer policy.
- KAN-46 can report a blocking result for an explicitly blocking policy, but a workflow must still opt in to treating that result as a deployment gate.
- KAN-47 supplies that opt-in workflow gate, still manual/report-only unless enforcement is selected.
- KAN-49 monitors the gate artifact only when release governance and artifact monitoring are both selected; it is not generated for default `record-only`.
- Generated workflows should remain non-blocking unless the adoption profile clearly selects advisory or blocking enforcement.
- Adoption profile validation now rejects accidental blocking `record-only` configurations and requires `formal-approval` before non-`record-only` governance modes.

### 2. Vercel AI SDK Copilot

Status: first MVP implemented in `KAN-38` with a server-side Vercel AI SDK evidence brief route; Desktop copilot UI added in `KAN-39`; AI/fallback mode validation started in `KAN-40`; direct Google Gemini provider activation started in `KAN-41`.

Current state:

- GitGov already has enough structured evidence for an assistant to explain risk, readiness, tickets, pipelines, findings, and policy decisions.
- The product already exposes Governance, exports, and Evidence Packets MVP.
- KAN-38 adds the first AI SDK route, `POST /api/copilot/governance`, which gathers bounded GitGov evidence and returns a cited governance brief.
- KAN-39 starts the first Desktop UI for the copilot route through a secret-safe Tauri desktop proxy. In the current IA this belongs under Governance Copilot.
- KAN-40 adds a secret-safe validator and GitHub workflow for checking whether the route is healthy, evidence-grounded, and running in `mode=ai` or deterministic `fallback`.
- KAN-41 selects direct Google Gemini through `@ai-sdk/google` as the practical production AI path because Vercel AI Gateway generation required billing-card activation. AI Gateway remains optional future infrastructure.
- Production validation should use the canonical `https://www.gitgov.cloud/api/copilot/governance` route. Older Vercel deployment aliases were historical validation paths, not the public URL to document as current.

Missing product packaging:

- Copilot chat that can answer:
  - why a release is or is not ready.
  - what changed since the last review.
  - which ticket, PR, pipeline, or finding is blocking.
  - what evidence supports an approval.
  - what should be fixed first.
- Tool-backed answers:
  - read release readiness.
  - read ticket coverage.
  - read evidence packets.
  - read vulnerability trend status.
  - summarize accepted risks.
- First MVP:
  - server-side Next.js route.
  - Governance Copilot panel.
  - AI/fallback mode validator.
  - Vercel AI SDK `generateText()`.
  - direct Google Gemini provider through `@ai-sdk/google`.
  - Evidence Packet, ticket coverage, release approval, and adoption profile evidence.
  - deterministic fallback when Google Gemini or AI Gateway is not configured.
- Guardrails:
  - no secret exposure.
  - cite evidence sources.
  - distinguish confirmed issues from expected findings.
  - do not invent approvals or provider state.

Customer-facing value:

GitGov can explain governance evidence in plain language for CTOs, engineering managers, auditors, and delivery teams instead of forcing them to interpret raw pipeline output.

## Operating Sequence

The original agreed order through hardening and enterprise packaging was:

1. Keep automatic vulnerability review, artifact monitor, and trend workflows on their weekly cadence.
2. Implement KAN-28 trend enforcement so the vulnerability trend fails when security posture worsens.
3. Keep the known `rsa` / inactive `sqlx-mysql` dependency finding documented as expected and not reachable unless upstream or dependency cleanup makes a clean removal safe.
4. Start the next product feature design/implementation for Enterprise Self-Service Adoption. This starts in `KAN-29`.
5. Finish the Enterprise Self-Service Onboarding gaps before Vercel AI SDK Copilot. Reviewed local workflow installation is covered by `KAN-35`; direct provider checks are covered by `KAN-36`; formal release approval persistence is covered by `KAN-37`; remote PR-based workflow installation starts in `KAN-50`; read-only remote workflow readiness validation starts in `KAN-51`; consolidated onboarding readiness reporting starts in `KAN-52`; recurring onboarding readiness evidence starts in `KAN-53`; onboarding readiness artifact monitoring starts in `KAN-54`; onboarding readiness trend reporting starts in `KAN-55`; onboarding readiness trend monitoring starts in `KAN-56`; onboarding remediation planning starts in `KAN-57`; dashboard remediation export starts in `KAN-58`; guided dashboard onboarding checklist starts in `KAN-59`; persisted checklist tracking starts in `KAN-60`; direct GitHub App installation remains optional future packaging.
6. Start the Vercel AI SDK Copilot feature when the onboarding/evidence surfaces are ready enough for the copilot to explain a complete adoption state. The first route is implemented in `KAN-38`, dashboard UI is implemented in `KAN-39`, AI-mode validation starts in `KAN-40`, and direct Google Gemini activation starts in `KAN-41`.

The agreed order after `KAN-67` was:

1. Stop adding incremental hardening/features by default.
2. Keep existing scheduled evidence workflows running.
3. Only open new hardening tickets for real bugs, confirmed vulnerabilities, production risks, or customer-selected enforcement requirements.
4. Make the next product feature UX-focused: `KAN-69 - Enterprise Action Center guided UX`.
5. After KAN-69, use Desktop runtime QA to simplify information architecture before adding another feature chain.
6. Use a dedicated `/action-center` route with existing evidence/readiness/onboarding/release-governance/checklist/copilot capabilities to show a simple guided path.
7. Treat AI work as useful only when it explains or simplifies the guided product experience.

## Non-Goals

- Do not claim GitGov removes all vulnerabilities.
- Do not claim multi-approver enterprise release governance is complete until quorum rules, signatures, approval UI, and release-gate enforcement are productionized for customer-selected policies.
- Do not make multi-approver quorum or release-blocking enforcement default behavior; both must be explicit customer choices.
- Do not continue adding standalone monitor/trend/enforcement tickets by default after `KAN-67`.
- Do not make the customer learn internal workflow names or artifact chains when a guided Action Center can explain the next global step.
- Do not turn Control Plane into an AWS-style dashboard. Control Plane is now connection/configuration in Settings; Governance owns evidence, policy, adoption, releases, and copilot work.
- Do not start new capability work unless it improves usability, closes a real risk, or supports a customer-selected policy.
- Do not require SonarCloud for this personal repository.
- Do not make OpenAPI/SDK work a blocker unless generated SDKs or contract tests become explicit scope.
- Do not claim the AI copilot is a full autonomous agent until streaming, tool-loop behavior, and governed tool approval are complete.
