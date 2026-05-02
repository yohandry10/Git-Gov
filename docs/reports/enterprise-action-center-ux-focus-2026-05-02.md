# KAN-68 Enterprise Action Center UX Focus

Updated: 2026-05-02

## Summary

KAN-68 documents the product decision to stop expanding GitGov through endless standalone features by default.

The next stage should prioritize usability:

```text
Make GitGov easier to use by turning existing evidence, onboarding, readiness, workflow, and governance capabilities into one guided Enterprise Action Center.
```

## Decision Recorded

- GitGov has completed a large security and enterprise-readiness hardening sequence through `KAN-67`.
- There is no current need to keep extending that chain with more monitor/trend/enforcement tickets unless a real bug, confirmed vulnerability, or production risk appears.
- The next recommended implementation is the product/UX milestone `KAN-69 - Enterprise Action Center guided UX`.
- New work should be evaluated by whether it makes GitGov easier for customers to understand and operate.

## Why

The current product has many powerful pieces:

- Evidence Packets.
- Enterprise adoption profile.
- Provider health.
- Workflow template generation.
- Reviewed local install.
- Remote workflow PR generation.
- Remote workflow readiness validation.
- Onboarding readiness report.
- Onboarding remediation plan.
- Dashboard remediation export.
- Guided checklist.
- Checklist tracking.
- Formal release approvals.
- Release governance evaluator and optional gate.
- Product vulnerability review automation.
- Enterprise route auth smoke automation and enforcement.
- Governance copilot.

The risk is not lack of capability. The risk is that a customer sees too many technical parts and does not know what to do first.

## Product Direction

The next UX should show:

- current onboarding readiness.
- next recommended action.
- why that action matters.
- one primary button to move forward.
- evidence source behind the recommendation.
- clear separation between safe read-only actions and explicit mutating actions.

## Recommended Next Ticket

Candidate:

```text
KAN-69 - Enterprise Action Center guided UX
```

Scope should be dashboard-first:

- Add an Action Center section to the Enterprise Adoption dashboard.
- Reuse existing profile/provider/readiness/remediation/checklist data.
- Show a plain-language next step.
- Provide buttons for existing actions.
- Keep provider/customer repository mutations opt-in and reviewed.
- Keep release enforcement and quorum optional, never default.

## Validation

Documentation validation:

| Check | Result |
| --- | --- |
| Product decision documented in design doc | Passed |
| Product decision documented in report | Passed |
| `docs/design/enterprise-self-service-and-ai-copilot-roadmap.md` updated | Passed |
| `AGENTS.md` updated | Passed |
| `docs/CURRENT_CONTEXT.md` updated | Passed |
| Scoped `git diff --check` | Passed |
| `.\scripts\security\publication_guard.ps1` | Passed |

## Worktree Note

At the time of this documentation update, local uncommitted edits existed in docs/README files outside KAN-68 scope. They are intentionally excluded from the KAN-68 commit and PR.

## Current Status

KAN-68 was completed as documentation-only product direction work. It is the handoff source for keeping `KAN-69` focused on the Enterprise Action Center guided UX instead of another standalone hardening or evidence-chain ticket.
