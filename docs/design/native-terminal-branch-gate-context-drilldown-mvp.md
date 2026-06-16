# Native Terminal Branch Gate Context Drilldown MVP

Updated: 2026-06-16
Ticket: `KAN-141`

## Product Decision

After KAN-140 added a compact branch gate status badge, make that badge useful without adding a
second workflow. Clicking the badge opens the existing native terminal `Context` drawer from KAN-135.

This keeps the branch gate status minimal in the terminal header while still giving operators a
one-click path to the read-only Deployment Gate, Change Risk, and Executive Governance context that
already exists.

## User Experience

- The badge still renders as a small `Gate ready`, `Gate review`, `No gate`, or `Gate n/a` control.
- When visible, the badge is clickable and opens the existing `Governance context` drawer.
- The drawer loads the same read-only context as before:
  - latest Deployment Gate authorization.
  - latest Change Risk evaluation.
  - Multi-Repo Executive Governance posture.
- The badge does not open a new modal, create another card-heavy panel, or add a second governance
  surface.

## Implementation

- `TerminalBranchGateStatusBadge` accepts `onOpenContext`.
- `TerminalGovernanceContextPanel` now supports optional controlled open state through `isOpen` and
  `onOpenChange`, while preserving its existing standalone toggle behavior.
- `TerminalPanel` owns the shared `showGovernanceContext` state and wires the badge click to the
  existing `Context` drawer.

No backend/API/schema change was required.

## Guardrails

- Advisory-only.
- Read-only.
- No command interception.
- No command approval or blocking.
- No auto-run.
- No commit/push/deploy enforcement.
- No provider, repository, or deployment mutation.
- No Control Plane audit write.
- No AI, Agent Governance, OPA/Rego, MCP, or chatbot dependency.
- No compliance, certification, legal, or regulatory claim.

## Validation Focus

Focused tests cover:

- the badge remains compact while exposing a click path to the context drawer.
- clicking the badge calls the context-open handler without issuing extra mutating commands.
- externally opening the existing context drawer loads the three existing read-only commands.
- repo/branch/org filters stay scoped to the safe GitHub `owner/repo`.
- local absolute paths are not included in Control Plane read calls.
