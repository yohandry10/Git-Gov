# KAN-69 Enterprise Action Center Guided UX

Updated: 2026-06-07

## Decision

KAN-69 implements the Enterprise Action Center as a dedicated desktop route:

```text
/action-center
```

It is not another panel inside the existing Workspace dashboard and it is not another section squeezed into the Control Plane dashboard.

The Workspace dashboard already has the file list, CLI, pipeline visualizer, audit trail, and commit/push controls. The Control Plane dashboard already has metrics, provider health, adoption profile, evidence packets, release approvals, copilot, exports, and chat. Adding the Action Center inside either surface would make the product harder to scan.

The Action Center is a guided operations surface that reads existing GitGov evidence and sends the operator to the right existing workflow.

## Product Model

The Action Center uses:

```text
Goal + Evidence + Permission
```

Goals in the first implementation:

- `Onboarding`: get the customer setup path into a known-good state.
- `Release`: prepare release evidence and approval context.
- `Evidence`: export traceable packets, readiness, and remediation evidence.

Evidence sources:

- Enterprise adoption profile.
- Provider health.
- Onboarding readiness.
- Onboarding remediation.
- Guided onboarding checklist state.
- Jenkins/pipeline stats already loaded by GitGov.
- Jira ticket coverage.
- Evidence Packet state.
- Release approval state.

Permission model:

- Recommendations are advisory and non-blocking.
- Admin-only workflows are marked as Admin actions.
- Non-admin users can still see guidance, but the UI does not pretend they can execute Admin-only workflows.
- Persona/lens selection changes presentation emphasis only. It is not authorization.

## Deterministic Rules

The first implementation ranks one primary action and several alternatives using deterministic rules.

For onboarding:

1. Fix an invalid adoption profile first.
2. Complete provider configuration names before provider evidence checks.
3. Collect provider health evidence when configuration exists but evidence is missing.
4. Continue the guided onboarding checklist.
5. Export adoption/workflow assets when readiness is complete.

For release prep:

1. Fix an invalid adoption profile first.
2. Review pipeline health when recent CI evidence is missing or weak.
3. Repair Jira traceability when coverage is below the release confidence threshold.
4. Generate or review an Evidence Packet.
5. Record the release decision with the packet hash.

For evidence export:

1. Fix an invalid adoption profile first.
2. Review the current Evidence Packet when it exists.
3. Export readiness/remediation evidence when onboarding is not complete.
4. Generate a ticket Evidence Packet when no packet is loaded.

## AI Boundary

The AI copilot may explain a recommendation with citations.

The Action Center itself decides recommendations through deterministic product rules, not through an LLM.

## Implementation Scope

KAN-69 adds:

- `/action-center` route.
- Sidebar navigation entry.
- `ActionCenterWorkspace` React surface.
- Pure recommendation helper with unit tests.
- Deep links to existing Control Plane panels:
  - Enterprise Adoption.
  - Evidence Packet.
  - Release Approvals.
  - Governance Copilot.
- Documentation and report updates.

KAN-69 does not add:

- new backend endpoints.
- new provider integrations.
- new monitor/trend/enforcement chain.
- release blocking by default.
- provider or repository mutation.
- SonarCloud.
- Jenkins trigger-only setup.
- OpenAPI/SDK work.

## Acceptance Criteria

- A user can open a first-class Action Center space from the sidebar.
- The first screen shows the current goal, role/lens context, primary recommendation, reason, evidence, permission, and alternatives.
- The primary recommendation is deterministic and explainable from loaded evidence.
- Actions navigate to existing GitGov workflows instead of duplicating the full Control Plane.
- The Workspace dashboard remains focused on repo/CLI/commit/push work.
- The Control Plane dashboard remains the detailed evidence/admin surface.
- No secrets are displayed, exported, or logged.
- The Action Center remains advisory and non-blocking.
