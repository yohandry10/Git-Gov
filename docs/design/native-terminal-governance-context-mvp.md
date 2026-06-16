# Native Terminal Governance Context MVP

Updated: 2026-06-16
Ticket: KAN-135

## Decision

KAN-135 adds a read-only Governance Context drawer to the Desktop native terminal.

KAN-132 introduced local terminal session history. KAN-133 added local Git repo/branch context.
KAN-134 added safe read-only quick command insertion. The next useful developer distribution slice is
to show existing governance context beside the terminal, without turning the terminal into an
enforcement or automation layer.

## Scope

The panel resolves a safe repository target from:

- KAN-133 Git context.
- Existing repo validation remote metadata.

It shows:

- repository and branch;
- latest Deployment Gate decision, if available;
- latest Change Risk level and review status, if available;
- Executive Governance posture, if available;
- Control Plane connection/provider-health status;
- internal links to existing Governance views.

The panel reuses existing read-only Tauri commands and backend routes. It does not create new
backend endpoints, database tables, policy decisions, or evidence records.

## Safe States

- Git context pending.
- Terminal is not inside a Git repository.
- Git repository has no parseable GitHub remote.
- Control Plane not configured.
- Permission denied.
- Repo detected but no governance data is available for the current filters.
- Loading and success.

## Guardrails

- No command execution.
- No command interception, blocking, approval, or auto-run.
- No deployment execution.
- No provider or repository mutation.
- No new backend API, migration, or audit write.
- No Agent Governance, AI, OPA/Rego, MCP, or chatbot dependency.
- No compliance, certification, legal, or regulatory claim.
- No absolute local path exposure in the governance target label.

## Validation

Focused helper tests validate remote parsing, no cwd leak, empty states, and evidence detection from
gate/risk/executive rows. Existing terminal regression tests remain in the focused set to prove
KAN-132 history, KAN-133 context, and KAN-134 quick commands still behave.
