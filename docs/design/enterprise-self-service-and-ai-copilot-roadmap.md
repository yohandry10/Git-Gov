# Enterprise Self-Service And AI Copilot Roadmap

Updated: 2026-04-30

Ticket: `KAN-28`

## Decision

GitGov already has the core product promise: it connects pipeline, ticket, review, policy, and release-readiness evidence into a governed record.

The next product work is not to invent a new category. It is to package the proven GitGov operating model so another company can adopt it with less manual setup.

## Next Product Features

### 1. Enterprise Self-Service Adoption

Status: started in `KAN-29`; dashboard profile builder added in `KAN-30`; persisted profiles added in `KAN-31`; provider health evidence MVP added in `KAN-32`; workflow template generation added in `KAN-33`; dashboard workflow template pack download added in `KAN-34`; reviewed workflow installation added in `KAN-35`; direct provider connection validation added in `KAN-36`; formal release approval MVP added in `KAN-37`.

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
    - direct GitHub App or PR-based remote installation remains future optional packaging.
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

Customer-facing value:

GitGov turns existing delivery tooling into governed release evidence without asking teams to manually assemble screenshots, spreadsheets, or one-off audit notes.

First MVP:

- `scripts/control-plane/generate_enterprise_adoption_pack.ps1`.
- `scripts/control-plane/generate_enterprise_workflow_templates.ps1`.
- `scripts/control-plane/install_enterprise_workflow_templates.ps1`.
- `scripts/control-plane/validate_enterprise_provider_connections.ps1`.
- `GET /enterprise/release-approvals`.
- `POST /enterprise/release-approvals`.
- `docs/design/enterprise-self-service-adoption-mvp.md`.
- `docs/design/workflow-template-generation-mvp.md`.
- `docs/design/dashboard-workflow-template-pack-mvp.md`.
- `docs/design/reviewed-workflow-installation-mvp.md`.
- `docs/design/provider-connection-validation-mvp.md`.
- `docs/design/formal-release-approval-mvp.md`.
- `docs/examples/enterprise-adoption-profile.example.json`.
- `gitgov/src/components/control_plane/EnterpriseAdoptionPanel.tsx`.
- `docs/design/adoption-profile-dashboard-mvp.md`.
- `docs/design/adoption-profile-persistence-mvp.md`.
- `docs/design/provider-health-validation-mvp.md`.

This MVP creates a reusable adoption pack from a customer profile, exposes the first dashboard UI for shaping that profile, persists it per organization, shows evidence-based provider health, generates reviewed workflow template packs from both CLI and dashboard, installs those packs into a local customer repository checkout only after dry-run review and explicit `-Apply`, validates explicitly provided provider credentials without printing secret values, and stores formal release approvals with evidence packet hashes and risk expiration. It does not yet mutate remote customer repositories through GitHub APIs or provide a dashboard release-approval wizard.

### 2. Vercel AI SDK Copilot

Status: started in `KAN-38` with a server-side Vercel AI SDK evidence brief route.

Current state:

- GitGov already has enough structured evidence for an assistant to explain risk, readiness, tickets, pipelines, findings, and policy decisions.
- The product already exposes the core dashboard, exports, and Evidence Packets MVP.
- KAN-38 adds the first AI SDK route, `POST /api/copilot/governance`, which gathers bounded GitGov evidence and returns a cited governance brief.

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
  - Vercel AI SDK `generateText()`.
  - Evidence Packet, ticket coverage, release approval, and adoption profile evidence.
  - deterministic fallback when AI Gateway/OIDC is not configured.
- Guardrails:
  - no secret exposure.
  - cite evidence sources.
  - distinguish confirmed issues from expected findings.
  - do not invent approvals or provider state.

Customer-facing value:

GitGov can explain governance evidence in plain language for CTOs, engineering managers, auditors, and delivery teams instead of forcing them to interpret raw pipeline output.

## Operating Sequence

The agreed order is:

1. Keep automatic vulnerability review, artifact monitor, and trend workflows on their weekly cadence.
2. Implement KAN-28 trend enforcement so the vulnerability trend fails when security posture worsens.
3. Keep the known `rsa` / inactive `sqlx-mysql` dependency finding documented as expected and not reachable unless upstream or dependency cleanup makes a clean removal safe.
4. Start the next product feature design/implementation for Enterprise Self-Service Adoption. This starts in `KAN-29`.
5. Finish the Enterprise Self-Service Onboarding gaps before Vercel AI SDK Copilot. Reviewed local workflow installation is covered by `KAN-35`; direct provider checks are covered by `KAN-36`; formal release approval persistence is covered by `KAN-37`; remote PR-based installation and dashboard approval workflows remain optional future packaging.
6. Start the Vercel AI SDK Copilot feature when the onboarding/evidence surfaces are ready enough for the copilot to explain a complete adoption state. This starts in `KAN-38`.

## Non-Goals

- Do not claim GitGov removes all vulnerabilities.
- Do not claim multi-approver enterprise release governance is complete until quorum rules, signatures, approval UI, and release-gate enforcement exist.
- Do not require SonarCloud for this personal repository.
- Do not make OpenAPI/SDK work a blocker unless generated SDKs or contract tests become explicit scope.
- Do not claim the AI copilot is a full autonomous agent until dashboard UI, streaming, tool-loop behavior, and production AI Gateway validation are complete.
