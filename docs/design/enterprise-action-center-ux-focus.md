# KAN-68 Enterprise Action Center UX Focus

Updated: 2026-05-02

## Decision

GitGov has enough core governance capability for the current stage.

The next product stage is not to keep adding isolated hardening workflows, monitors, reports, and technical features by default. The next stage is to make GitGov easier to understand and easier to operate.

The agreed product direction is:

```text
Turn GitGov from a powerful evidence platform into a guided product experience.
```

That means the next major work should package existing capabilities into a clear Enterprise Action Center instead of adding another standalone capability.

## Why This Matters

GitGov now has many strong building blocks:

- GitHub, Jira, Jenkins, SonarQube, Render, Vercel, and local evidence integrations.
- Evidence Packets.
- Product vulnerability review automation.
- Artifact freshness monitors.
- Trend reports.
- Enforcement gates.
- Enterprise adoption profiles.
- Provider health validation.
- Workflow template generation and installation.
- Remote workflow installation PRs.
- Release approvals.
- Release governance evaluation.
- Onboarding readiness reports.
- Remediation plans.
- Guided onboarding checklist with persisted tracking.
- Enterprise route auth regression hardening and recurring smoke evidence.
- Governance copilot with deterministic fallback and AI mode support.

These pieces are valuable, but a normal customer should not have to understand every internal workflow or artifact chain to use the product.

The product goal is now:

```text
Show the user what to do next, why it matters, and which button moves them forward.
```

## Product Rule Going Forward

Do not start another hardening or evidence-chain ticket just because it is possible.

New work should pass at least one of these tests:

- It makes the product easier for a customer to use.
- It turns several existing technical pieces into one clear workflow.
- It reduces the number of manual steps needed for onboarding.
- It makes the next action obvious.
- It prevents a real confirmed security or production risk.
- It fixes a real bug.

If a proposed feature does not pass one of those tests, defer it.

## Recommended Next Feature

Candidate next implementation:

```text
KAN-69 - Enterprise Action Center guided UX
```

This should be a product/UX feature, not another backend hardening chain.

The Action Center should gather existing GitGov capabilities into one operator workflow:

1. Connect tools.
2. Validate provider health.
3. Generate adoption pack.
4. Generate or download workflow templates.
5. Open reviewed remote workflow PR when explicitly requested.
6. Validate readiness.
7. Show the next recommended action.
8. Track completion.
9. Keep optional governance controls clearly optional.

## User Experience Target

Instead of showing a user many separate concepts:

- readiness reports.
- provider checks.
- workflow packs.
- artifact monitors.
- trend reports.
- enforcement gates.
- remediation exports.
- checklist tracking.

GitGov should show one guided state:

```text
Your onboarding is 75% ready.
Next step: validate Jenkins evidence and install the readiness workflow.
```

With clear actions:

- `Validate providers`
- `Generate workflows`
- `Open PR`
- `Check readiness`
- `View remediation`
- `Mark step done`

## What Already Exists For This UX

The Action Center should reuse these existing pieces:

- Adoption profile persistence: `KAN-31`.
- Provider health UI: `KAN-32`.
- Workflow templates: `KAN-33`, `KAN-34`.
- Reviewed installation: `KAN-35`.
- Provider connection validation: `KAN-36`.
- Release approvals: `KAN-37`, `KAN-43`.
- Release governance policy/evaluator/gate: `KAN-45`, `KAN-46`, `KAN-47`.
- Remote workflow PR path: `KAN-50`.
- Remote workflow readiness validation: `KAN-51`.
- Onboarding readiness report: `KAN-52`.
- Readiness automation, monitor, trend, and deterioration monitor: `KAN-53` through `KAN-56`.
- Remediation plan and dashboard export: `KAN-57`, `KAN-58`.
- Guided checklist and persisted tracking: `KAN-59`, `KAN-60`.
- Enterprise route auth safety evidence: `KAN-61` through `KAN-67`.

## Non-Goals

For the next stage, do not make these default goals:

- More standalone monitor workflows.
- More trend artifacts unless they directly feed the Action Center.
- More enforcement gates unless a customer-selected policy needs them.
- More backend endpoints unless the Action Center cannot be built from existing data.
- More AI work unless it explains or simplifies the Action Center.
- Release blocking by default.
- Quorum or multi-approver rules by default.
- Provider mutation without explicit operator action.
- Customer repository mutation without explicit reviewed action.

## Implementation Shape For KAN-69

If the next session starts KAN-69, start from the dashboard.

Preferred product shape:

- Add an `Action Center` section to the Enterprise Adoption dashboard.
- Compute a small set of recommended next actions from existing profile, provider health, workflow pack, readiness, remediation, and checklist state.
- Render status in plain language.
- Provide buttons that call existing local/dashboard actions where available.
- Keep all external mutations opt-in and reviewed.
- Do not introduce new enforcement defaults.

Suggested first version action states:

- `not-started`
- `needs-config`
- `needs-validation`
- `ready-to-install`
- `needs-readiness-check`
- `needs-remediation`
- `ready`

Suggested action model:

```text
id:
title:
status:
why_it_matters:
primary_action:
secondary_action:
source_evidence:
safe_by_default:
```

## Acceptance Criteria For KAN-69

- A customer can understand the onboarding state without reading workflow docs.
- The next recommended action is visible.
- Actions reuse existing capabilities where possible.
- Optional enforcement remains clearly opt-in.
- No secret values are displayed, exported, or logged.
- No provider or repository mutation happens without explicit operator action.
- The dashboard remains usable even when some providers are not configured.
- Documentation explains that this is UX packaging of existing capabilities, not a new enforcement default.

## Resume Instruction

When resuming from a new session, do not continue the KAN-61 through KAN-67 hardening chain unless there is a real security bug or production risk.

Start product discussion from this question:

```text
How do we make the existing GitGov capabilities obvious and useful to a first customer?
```

The default answer should be:

```text
Build the Enterprise Action Center guided UX.
```
