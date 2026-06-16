# Native Terminal Safe Quick Commands MVP

Updated: 2026-06-16
Ticket: KAN-134

## Decision

KAN-134 adds a local quick-command palette to the Desktop native terminal.

KAN-132 made the terminal remember commands typed in the current UI session. KAN-133 made the same
terminal show safe Git repo/branch context. KAN-134 uses those two primitives to help the operator
insert common read-only Git inspection commands without turning GitGov into an automation or command
enforcement surface.

The command is inserted into the native PTY text buffer only. It is not executed automatically. The
operator must still press Enter.

## Scope

The MVP ships five read-only commands:

- `git status --short`
- `git branch --show-current`
- `git log --oneline -5`
- `git diff --stat`
- `git remote -v`

The UI shows each command with a short preview. Git commands are disabled when KAN-133 reports that
the terminal cwd is not inside a Git repository. Recently inserted quick commands are shown only in
local component state for the current terminal session.

## Guardrails

- No auto-run.
- No command interception.
- No command blocking or approval workflow.
- No backend, Control Plane API, database, or audit write.
- No provider, repository, or deployment mutation.
- No `push`, `pull`, `fetch`, `merge`, `rebase`, `checkout`, `commit`, `reset`, `deploy`, `apply`,
  delete, destroy, or shell compound commands.
- No AI, Agent Governance, MCP, OPA/Rego, or chatbot dependency.
- No compliance, certification, legal, or regulatory claim.

## Validation

The focused helper tests verify the allowlist exactly, reject mutating/compound/redirected commands,
disable Git commands outside a repo, avoid cwd exposure in labels, and prove inserted text has no
newline. The same insert-only text is fed through the KAN-132 session-history parser to prove that
history capture happens only after the human presses Enter.
