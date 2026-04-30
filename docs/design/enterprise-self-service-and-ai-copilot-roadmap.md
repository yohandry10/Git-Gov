# Enterprise Self-Service And AI Copilot Roadmap

Updated: 2026-04-30

Ticket: `KAN-28`

## Decision

GitGov already has the core product promise: it connects pipeline, ticket, review, policy, and release-readiness evidence into a governed record.

The next product work is not to invent a new category. It is to package the proven GitGov operating model so another company can adopt it with less manual setup.

## Next Product Features

### 1. Enterprise Self-Service Adoption

Status: started in `KAN-29`.

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
  - install recommended workflows.
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

Customer-facing value:

GitGov turns existing delivery tooling into governed release evidence without asking teams to manually assemble screenshots, spreadsheets, or one-off audit notes.

First MVP:

- `scripts/control-plane/generate_enterprise_adoption_pack.ps1`.
- `docs/design/enterprise-self-service-adoption-mvp.md`.
- `docs/examples/enterprise-adoption-profile.example.json`.

This MVP creates a reusable adoption pack from a customer profile. It does not yet replace the future UI onboarding wizard.

### 2. Vercel AI SDK Copilot

Status: next major product feature after or alongside the self-service adoption work.

Current state:

- GitGov already has enough structured evidence for an assistant to explain risk, readiness, tickets, pipelines, findings, and policy decisions.
- The product already exposes the core dashboard, exports, and Evidence Packets MVP.

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
5. Start the Vercel AI SDK Copilot feature when the evidence surfaces and customer-facing workflows are ready.

## Non-Goals

- Do not claim GitGov removes all vulnerabilities.
- Do not claim formal enterprise release approval is complete until the approval model, rules, expiration, and evidence binding exist.
- Do not require SonarCloud for this personal repository.
- Do not make OpenAPI/SDK work a blocker unless generated SDKs or contract tests become explicit scope.
