# Enterprise Self-Service And AI Copilot Roadmap

Updated: 2026-06-14

Ticket: `KAN-68`; KAN-77 roadmap intake update; KAN-89 roadmap sync after KAN-88; KAN-93 shared governance decision model; KAN-94 agent-scoped API keys; KAN-95 agent governance dry-run; KAN-96 minimal agent attribution envelope; KAN-97 agent key expiry and rotation; KAN-98 read-only agent governance context; KAN-99 compliance evidence export

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

## First-Priority Future Feature Queue

Source: external planning note `GITGOV_ROADMAP.md` reviewed on 2026-06-12.

These are future product bets, not claims of complete implementation. They should sit ahead of the
older roadmap queue because they define the next enterprise story after KAN-69/KAN-77: GitGov as the
governance gate for human delivery first, and optionally for agentic delivery when a customer
chooses to allow agents. Several items already have partial GitGov primitives; the future work is to
turn those primitives into complete customer-facing product flows.

### 0.1 Deployment Gates: GitGov Authorizes Deploy

Future goal: customer CI/CD systems call GitGov before production deployment. GitGov returns a
structured `approved` or `blocked` result with reasons, evidence, and policy identifiers.

Why first:

- This is the clearest enterprise value proposition: no production deploy without GitGov evidence.
- It connects release readiness, ticket coverage, CI evidence, policy, risk, and approvals into one
  enforceable decision.
- It turns GitGov from an evidence dashboard into an operational control point.

Current primitives:

- Release readiness, release governance evaluation, optional release-governance gate, policy checks,
  Jenkins/GitHub Actions evidence, evidence packets, and formal release approvals already exist.
- Default behavior remains safe: record-only/advisory unless a customer explicitly selects blocking
  enforcement.
- `KAN-80` adds the first governed repo setup slice: Admin-selected repo/branch, provider/module
  selection, policy preset, policy/workflow preview acknowledgement, persisted baseline readiness,
  Action Center gaps, and a CTA into advisory gate simulation. It is deliberately not a broad
  integration wizard.
- `KAN-83` adds the first CI/CD-facing deployment authorization API with persisted history:
  `POST /deployment-gates/authorize` and `GET /deployment-gates/authorizations`.
- `KAN-84` adds the Desktop history surface under `Governance > Releases` and migrates generated
  release governance workflow templates plus `validate_release_governance_gate.ps1` to call
  `POST /deployment-gates/authorize` instead of the lower-level evaluator.
- `KAN-85` adds provider-specific Deployment Gate examples for GitHub Actions, Jenkins Pipeline, and
  GitLab CI, plus a validator that keeps those examples on the Deployment Gates authorization
  contract.
- `KAN-86` adds the Desktop/admin Environment Policy Matrix for release governance so stricter
  production policy can be reviewed without making the base policy blocking.
- `KAN-87` adds audited break-glass deployment authorization for genuinely blocking policy results.
- `KAN-88` adds pre-approved break-glass approval routing: Deployment Gate callers can use
  break-glass only when a valid unexpired approval matches the same release, repository, branch,
  target SHA, environment, optional ticket, and evidence packet hash.
- `KAN-93` adds the shared governance decision model to Deployment Gates. Deployment authorizations
  now expose `shared-governance-decision.v1` with `consumer_type=deployment_gate` and
  `agent_governance_used=false`, so the CI/CD manual-first deploy path can feed future audit,
  Action Center, and approval-routing surfaces without depending on Agent Governance.

Future scope:

- Additional deployer examples beyond GitHub Actions, Jenkins Pipeline, and GitLab CI.
- Advanced environment policy workflows, such as environment-specific approval routing,
  notification/escalation rules, and multi-approver break-glass chains.
- Customer-facing installation flows that wire Deployment Gates into provider repositories after
  explicit operator review, instead of examples/manual copy-paste only.
- Broader deployment target coverage and richer provider-specific evidence artifacts.

Next major roadmap block after the KAN-80 through KAN-88 Deployment Gates slice:
`0.2 Agentic Governance Layer`.

### 0.2 Agentic Governance Layer

Future goal: when a customer explicitly allows agents such as Codex, Claude Code, Cursor, Windsurf,
Copilot, JetBrains AI, internal bots, or CI agents to operate near production code, those agents
consult GitGov before acting.

This block is opt-in. It is not the default operating model for GitGov, and it must not weaken the
manual governance path for banks or other regulated customers that prohibit autonomous agents.

Why first:

- Agents already operate in some customer repositories; enterprise governance has not caught up.
- GitGov can own that optional governance gap: what an allowed agent may do, what needs approval,
  and what evidence is left behind.
- This differentiates GitGov beyond generic DevOps dashboards.

Current primitives:

- `KAN-90` starts the REST Policy API with `POST /agent-governance/evaluate`. Agents can ask
  GitGov whether `commit`, `push`, `open_pr`, `merge_pr`, `change_policy`, or `deploy` is
  `allowed`, `requires_approval`, or `blocked`. The decision is deterministic and persisted as
  audit evidence; `llm_decision=false` remains explicit in the response.
- KAN-90 is optional. It is not a chatbot, not a bring-your-own-model requirement, and not a
  replacement for manual GitGov flows. Human pull request review, policy review, formal release
  approval, Deployment Gates, and Policy-as-Code remain valid without any agent integration.
- `KAN-92` adds the control boundary required before agent governance can be considered enterprise
  safe: Agent Governance is disabled by default per tenant, Admin opt-in is required, opt-in/out is
  audited, disabled evaluation attempts return `403 agent_governance_disabled` without creating
  evaluation evidence, history is Admin-only, and persisted request payload is minimized/redacted.
- `KAN-93` adds the same `shared-governance-decision.v1` shape to Agent Governance evaluations, but
  keeps Agent Governance optional. Deployment Gates do not call `/agent-governance/evaluate` and do
  not create agent evaluation rows.
- `KAN-94` adds optional agent-scoped API keys for tenants that explicitly enable Agent Governance.
  The key type is limited to `POST /agent-governance/evaluate`, stores only token hashes plus
  prefix/last-four metadata, records agent identity on evaluation rows, audits create/use/deny/revoke
  events, and keeps disabled/manual-only tenants unaffected.
- `KAN-95` adds `POST /agent-governance/dry-run`, a safe preview that returns the deterministic
  decision, missing evidence, principal identity, and shared governance decision without persisting
  an `agent_governance_evaluations` row and without authorizing execution.
- `KAN-96` adds a minimal attribution envelope for optional Agent Governance dry-run/evaluate
  requests. It records safe correlation, session, tool, agent, external run, principal, and
  consumer metadata for formal evaluations, returns the same envelope in dry-run responses, and keeps
  dry-run out of `agent_governance_evaluations`.
- `KAN-97` hardens the lifecycle of optional agent-scoped keys before any MCP surface. New keys
  default to 90-day expiry unless an Admin explicitly chooses `no_expiry=true`; key records expose
  derived lifecycle status; `POST /agent-governance/agent-keys/{key_id}/rotate` creates a
  replacement token shown once, links old and new keys, gives the old key a bounded grace period,
  and audits rotate/expired/revoked outcomes without storing plaintext tokens.
- `KAN-98` adds the first read-only agent context contract before any MCP surface. Admins can create
  keys with `agent_governance:read`, and those keys can call `GET /agent-governance/context` to read
  existing branch, policy, pipeline, deployment-gate, risk, and activity evidence. The endpoint is
  read-only, returns `will_authorize_execution=false` and `mcp_surface=false`, creates no formal
  evaluation rows, and is denied for agent principals while the tenant remains disabled/manual-only.
- Post-KAN-98 product decision: pause Agentic Governance expansion before MCP. The next slice is
  `KAN-99 Compliance Evidence Export v1`, because it strengthens the manual-first enterprise buyer
  story and turns Deployment Gate decisions into audit-ready packages without requiring agents.

Future scope:

- MCP server exposing scoped governance tools:
  - `get_branch_status`
  - `check_policy_compliance`
  - `list_audit_logs`
  - `get_pipeline_state`
  - `get_risk_score`
- Broader agent attribution chain beyond the KAN-96 minimal envelope, if customers need full
  session/operation linking later.
- Broader read-only agent scopes beyond KAN-98's single `agent_governance:read` scope, if customers
  need more granular separation such as audit-only, policy-only, or branch-status-only keys.
- Broader REST Policy API coverage beyond the KAN-90 MVP rules.
- Human-in-the-loop approval for sensitive operations from agents.
- Ephemeral agent session logs linked to existing audit trail.
- Agent attribution chain showing agent, token, human approver, operation, commit/deploy outcome,
  timestamp, and scope at the time of action.

- GitGov already has deterministic policy checks, audit logs, release governance evidence, Desktop
  approval surfaces, Deployment Gates, and a governance copilot. Future agentic work must reuse
  those primitives rather than create a parallel decision system.

Guardrail:

- LLMs/agents should not decide critical controls. They can request, explain, simulate, and propose;
  GitGov policy and human approval decide.
- Manual-first remains non-negotiable. Agentic features must degrade to "unused" for customers that
  do not permit agents, not to a broken or reduced GitGov experience.

### 0.3 Regulatory Framework Mapper

Future goal: map GitGov audit evidence to concrete controls for frameworks such as PCI-DSS,
ISO 27001, SBS Peru, and LGPD Brazil.

Why first:

- This creates direct enterprise ROI: audit evidence packages in minutes instead of consultant-led
  manual assembly.
- It turns existing evidence into compliance language executives, auditors, and regulators recognize.

Current primitives:

- Evidence packets, audit export, GitHub/Jira/Jenkins/Sonar evidence, release approvals, policy
  history, and compliance/reporting helpers already exist.
- `KAN-99` adds the prerequisite compliance package layer: Admins can create, inspect metadata for,
  and download a JSON-only, hashable, read-only evidence export from an existing Deployment Gate
  authorization. The artifact includes gate decision, policy checksum/source, readiness, approvals,
  evidence counts/references, explicit gaps, audit timestamps, `agent_governance_used=false`, and
  `compliance_claim=false`. It does not map controls yet and does not claim framework compliance.

Future scope:

- Static, versioned control mapping from GitGov event/evidence types to framework control IDs.
- Framework-specific report output with evidence links and hashes.
- Configurable framework packs so new regulatory mappings can be added without changing core product
  logic.
- PDF/JSON export suitable for quarterly or annual audits.

### 0.4 Bring Your Own Model And AI Routing

Future goal: enterprise customers choose which AI provider/model GitGov uses for governance
explanations.

Why first:

- Banks and regulated companies often cannot use a vendor-selected LLM.
- BYOM reduces privacy objections and shifts AI usage cost/control to the customer.

Current primitives:

- Governance copilot route and Desktop UI exist.
- Direct Google Gemini production mode exists; deterministic fallback exists.
- Vercel AI SDK is already the right abstraction layer for future providers.

Future scope:

- Organization settings for provider, model, encrypted API key reference, fallback behavior, and test
  connection.
- Providers: OpenAI, Anthropic, OpenRouter, Groq, Google Gemini, and optional Vercel AI Gateway.
- Rate limiting and fallback by organization.
- Strict evidence-citation guardrails remain mandatory.

### 0.5 Emergency Break Glass Protocol

Future goal: emergency production changes can bypass normal gates only through a more audited,
time-bounded process.

Why first:

- Every enterprise needs a safe emergency path, especially for production outages.
- Today many teams use informal approvals; GitGov can make the emergency exception itself auditable.

Current primitives:

- Release approvals, policy source metadata, audit logs, Jira integration, Slack/provider connection
  direction, and release governance evaluation exist.

Future scope:

- Break-glass request with mandatory justification, ticket, target repo/branch/environment, and
  expiration.
- Configurable approver quorum.
- Time-boxed active bypass with visible countdown and automatic expiration.
- Automatic post-mortem ticket creation when configured.
- Permanent `BREAK_GLASS` audit marker and Evidence Packet inclusion.

### 0.6 Integration Wizard And Enterprise Integration Hub

Future goal: the first-run path connects GitHub, Jira, Jenkins, SonarQube, Slack, and deployment
providers before the customer reaches a blank dashboard.

Why first:

- Faster onboarding is still the largest packaging gap.
- Customers should see real evidence on first login, not configure pieces manually.

Current primitives:

- Adoption profiles, provider health, direct provider connection validation, workflow template packs,
  onboarding readiness reports, remediation plans, and Action Center guidance exist.

Future scope:

- Wizard-style onboarding with skip/retry per provider.
- Secret-safe OAuth/API-key connection status per provider.
- Recommended workflow/policy preset generated from selected tools.
- Deep links into Action Center, Governance Evidence, and Settings/System.

### 0.7 Change Risk Score

Future goal: each commit/release gets an operational risk score based on deterministic rules, not ML.

Why first:

- Change advisory boards already do this manually in banks and regulated teams.
- A deterministic score can become input to Deployment Gates and Release Governance.

Current primitives:

- Release readiness, ticket coverage, policy checks, branch/repo evidence, pipeline/Sonar evidence,
  risk outcomes helpers, and governance reporting exist.

Future scope:

- Configurable scoring signals:
  - sensitive module touched.
  - new team member.
  - outside working hours.
  - large file/change volume.
  - missing tests.
  - missing Jira ticket.
  - unusual deletion ratio.
- Score `0-100` stored with audit evidence.
- Optional policy threshold for advisory or blocking deployment gates.

### 0.8 Multi-Repo Executive Governance View

Future goal: CISO/CTO view across all repositories in an organization.

Why first:

- Enterprise buyers need fleet-level governance posture, not only single-repo evidence.

Current primitives:

- Organization scoping, governance evidence, provider health, readiness, release approvals, audit
  export, and adoption profile surfaces already exist.

Future scope:

- Repo-by-repo compliance score, active violations, red gates, latest activity, agent tokens,
  release state, and trend direction.
- Filters by team, repo criticality, environment, and violation type.
- Admin-only access.

### 0.9 Compliance Report Generator

Future goal: generate formal monthly/quarterly/annual compliance reports from existing GitGov
evidence.

Why first:

- This is a packaging step with high customer value because much of the underlying evidence already
  exists.

Current primitives:

- Audit export, evidence packets, release approvals, GitHub/Jira/Jenkins/Sonar evidence, policy
  history, and compliance helpers exist.

Future scope:

- Organization/repository report templates.
- PDF and JSON outputs with hashes.
- Digitally signable evidence packet references.
- Regulator/auditor wording that maps to the Regulatory Framework Mapper when enabled.

### 0.10 Developer Distribution Surfaces

Future goal: meet developers and agents where they work, while keeping GitGov as the policy source.

Future scope:

- VS Code extension with read-only branch gate status, policy preview, pipeline state, and audit
  trail snippets.
- Embedded terminal improvements:
  - command history by session.
  - repo/branch context in prompt.
  - quick commands for selected deploy providers.
  - no command interception by default.

Guardrail:

- These are convenience and distribution surfaces. They must not bypass Desktop/Control Plane policy
  or create a second enforcement model.

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
- `gitgov/src/components/control_plane/DeploymentGateHistoryPanel.tsx`.
- `GET /enterprise/release-governance/evaluate`.
- `POST /deployment-gates/authorize`.
- `GET /deployment-gates/authorizations`.
- `POST /deployment-gates/break-glass-approvals`.
- `GET /deployment-gates/break-glass-approvals`.
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
- `docs/design/deployment-authorization-api-mvp.md`.
- `docs/design/environment-policy-ux-mvp.md`.
- `docs/design/break-glass-deployment-authorization-mvp.md`.
- `docs/examples/enterprise-adoption-profile.example.json`.
- `gitgov/src/components/control_plane/EnterpriseAdoptionPanel.tsx`.
- `docs/design/adoption-profile-dashboard-mvp.md`.
- `docs/design/adoption-profile-persistence-mvp.md`.
- `docs/design/provider-health-validation-mvp.md`.
- `docs/design/release-governance-profile-policy-mvp.md`.

This MVP creates a reusable adoption pack from a customer profile, exposes the first dashboard UI for shaping that profile, persists it per organization, shows evidence-based provider health, generates reviewed workflow template packs from both CLI and dashboard, installs those packs into a local customer repository checkout only after dry-run review and explicit `-Apply`, can open a remote draft PR for those workflow templates only after explicit `-Apply`, validates remote workflow/configuration readiness read-only, consolidates onboarding readiness into one customer-facing Markdown/JSON report, turns that readiness report into a prioritized remediation plan through both CLI and dashboard export, shows the same remediation state as a guided dashboard checklist, persists admin tracking notes for that checklist without changing readiness scoring, automates readiness as a recurring/manual GitHub Actions evidence artifact, monitors that readiness artifact for freshness, trends readiness artifacts over time, monitors trend deterioration in report-only mode by default, validates explicitly provided provider credentials without printing secret values, stores formal release approvals with evidence packet hashes and risk expiration, provides a dashboard wizard for create/list approval workflows, carries explicit release governance policy through the adoption profile and generated packs, evaluates a release against that policy when an admin asks, provides an optional manual gate for customers who explicitly select enforcement, supports per-environment overrides so production can be stricter than staging without changing the safe default, persists first governed repo setup readiness, exposes a stable deployment authorization API plus history for CI/CD callers, shows deployment authorization history in Desktop, records audited break-glass exceptions for genuinely blocking deployment decisions, and now requires matching pre-approved break-glass approval routing before a deployment can use that exception. It does not yet create GitHub Actions variables/secrets, mutate branch protection, require cryptographic signatures, or automate multi-person notification/escalation routing.

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
