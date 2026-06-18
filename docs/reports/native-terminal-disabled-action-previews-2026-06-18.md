# Native Terminal Quiet Disabled Action Previews Report

Date: 2026-06-18
Ticket: `KAN-145`
Issue: `#500`

## Summary

`KAN-145` adds quiet disabled action previews to the Desktop Workspace native terminal
quick-command menu.

The feature is deliberately narrow: when local provider/tool context is detected, the menu shows a
small advisory section explaining which categories are intentionally absent from shortcuts. The
section does not contain runnable unsafe command strings, is not clickable, and cannot insert text
into the terminal.

## Implemented

- Added `TerminalDisabledActionPreview` metadata in `terminalQuickCommands.ts`.
- Added four excluded action categories:
  - state-changing tool actions.
  - cloud/provider API actions.
  - secret or value inspection.
  - repository write actions.
- Added `buildTerminalDisabledActionPreviews`, which returns previews only when at least one local
  provider/tool is detected.
- Rendered previews as passive text under `Not offered as shortcuts` in
  `TerminalQuickCommandsMenu`.
- Added focused helper and UI tests that verify previews are advisory-only and do not expose common
  unsafe command strings.

## Guardrails Verified

- No unsafe command preview has a `command` property.
- Disabled previews are not buttons.
- Disabled previews do not call `onInsert`.
- Existing quick-command insertion remains limited to exact allowlisted safe commands.
- No backend/API route, database migration, provider integration, or Render deploy is involved.

## Validation

- `npm --prefix gitgov run test -- terminal-quick-commands.test.ts terminal-quick-commands-menu.test.tsx`
  - Result: passed.
  - Files: `2` passed.
  - Tests: `15` passed.
- `npm --prefix gitgov run test -- terminal-quick-commands.test.ts terminal-quick-commands-menu.test.tsx terminal-git-context.test.ts terminal-governance-context.test.ts terminal-branch-gate-status.test.tsx terminal-session-history.test.ts terminal-status.test.ts`
  - Result: passed.
  - Files: `7` passed.
  - Tests: `41` passed.
- `npm --prefix gitgov run typecheck`
  - Result: passed.
- `npm --prefix gitgov run lint`
  - Result: passed.
- `npm --prefix gitgov run test`
  - Result: passed.
  - Files: `45` passed.
  - Tests: `424` passed.
- `npm --prefix gitgov run build`
  - Result: passed with the pre-existing Vite large chunk warning.
- `git diff --check`
  - Result: passed.
- `.\scripts\security\publication_guard.ps1`
  - Result: passed after renaming the branch to `feature/KAN-145-terminal-quiet-action-previews`
    to satisfy the repository neutral naming policy.
- Static grep for common unsafe command strings in the new product code and KAN-145 docs:
  - Result: no matches.

Full PR validation remains required before merge.
