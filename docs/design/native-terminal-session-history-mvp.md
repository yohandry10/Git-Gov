# KAN-132 Native Terminal Session History MVP

Updated: 2026-06-16

## Decision

The next roadmap slice after KAN-131 starts `0.10 Developer Distribution Surfaces` with a
small Workspace improvement: local command history for the native terminal session.

This is deliberately a convenience surface, not a new governance or enforcement model. GitGov
continues to keep policy, evidence, approvals, and audit artifacts in the Control Plane and
Governance surfaces. The terminal remains a normal shell connected through the existing native PTY.

## Scope

- Capture commands typed into the Desktop native terminal during the current UI session.
- Show a compact `Session commands` drawer in the terminal header with command count, shell, repo,
  branch, timestamp, and command text.
- Keep history newest-first and capped at 50 commands.
- Parse real terminal input behavior: Enter submits, Backspace edits the draft, Ctrl+C clears the
  draft, and ANSI navigation sequences do not become command text.

## Non-Goals

- No command blocking or approval.
- No policy enforcement.
- No backend persistence.
- No Control Plane audit event.
- No provider, repository, branch, or deployment mutation.
- No AI, Agent Governance, OPA/Rego, MCP, chatbot, or compliance/certification/legal claim.
- No automatic command re-run from history.

## Validation

- Focused Vitest coverage for terminal history parsing and retention behavior.
- Frontend typecheck.
- Frontend lint.
- Frontend production build.

