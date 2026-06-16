# KAN-133 Native Terminal Repo/Branch Context MVP

Updated: 2026-06-16

## Decision

After KAN-132 added local native-terminal session history, the next `0.10 Developer Distribution
Surfaces` gap is safe local context: the operator should see which Git repository and branch the
Desktop native terminal is associated with before typing commands.

This is a convenience feature only. It does not intercept commands, block commands, approve
releases, enforce policy, execute deploys, mutate providers, mutate repositories, or create
Control Plane evidence.

## Scope

- Add a local Tauri command that resolves native terminal Git context from a cwd:
  - Git repo or non-git state.
  - repo name.
  - branch or detached HEAD.
  - short commit when available.
- Display the context in the Desktop terminal header as a compact `repo:branch` label.
- Refresh context on terminal start and after simple directory-change commands (`cd`, `chdir`,
  `sl`, `Set-Location`) that can be safely inferred without executing Git or shell commands.
- Keep KAN-132 session history behavior intact.

## Non-Goals

- No backend/API/server DB changes.
- No backend persistence.
- No command interception, blocking, approval, or enforcement.
- No Git `push`, `pull`, `fetch`, checkout, or repository mutation.
- No provider, deployment, branch-protection, or workflow mutation.
- No quick commands, VS Code extension, branch gate status, or policy preview.
- No AI, Agent Governance, MCP, BYOM, chatbot, compliance, certification, legal, or regulatory claim.

## Validation

- Rust unit tests cover non-git directories, real temporary Git repositories, branch/commit
  reporting, and safe `cd` inference.
- Frontend tests cover refresh trigger detection and safe label formatting without cwd leakage.
- Full Tauri check/clippy/tests and frontend typecheck/lint/build/full tests are required before
  merge.

