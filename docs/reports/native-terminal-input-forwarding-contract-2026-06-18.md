# Native Terminal Input Forwarding Contract Report

Date: 2026-06-18
Ticket: `KAN-146`
Issue: `#502`

## Summary

`KAN-146` makes the Desktop native terminal input forwarding behavior explicit and testable.

The terminal still observes input locally for session history and context refresh. The manual input
bytes are then forwarded unchanged to the native PTY by default. This keeps the terminal a manual
workspace surface instead of a hidden policy-enforcement layer.

## Implemented

- Added `terminalInputForwarding.ts`.
- Added `buildNativeTerminalInputForwardingContract`.
- The contract returns:
  - `shouldForward=true`.
  - `interception=none`.
  - `policyEvaluation=not-run`.
  - `mutatesInput=false`.
- Updated `TerminalPanel` to pass manual `onData` input through that forwarding contract before
  calling `cmd_write_native_terminal`.
- Added focused tests for ordinary input, compound-looking input, control bytes, and pasted
  multi-line input.

## Guardrails Verified

- Manual input is not blocked.
- Manual input is not rewritten.
- Manual input is not policy-evaluated by the terminal.
- Local session-history observation does not change the bytes forwarded to the PTY.
- Quick-command insertion remains a separate exact-allowlisted flow.

## Validation

- `npm --prefix gitgov run test -- terminal-input-forwarding.test.ts terminal-session-history.test.ts terminal-quick-commands.test.ts terminal-quick-commands-menu.test.tsx terminal-git-context.test.ts terminal-governance-context.test.ts terminal-branch-gate-status.test.tsx terminal-status.test.ts`
  - Result: passed.
  - Files: `8` passed.
  - Tests: `44` passed.
- `npm --prefix gitgov run typecheck`
  - Result: passed.
- `npm --prefix gitgov run lint`
  - Result: passed.
- `npm --prefix gitgov run test`
  - Result: passed.
  - Files: `46` passed.
  - Tests: `427` passed.
- `npm --prefix gitgov run build`
  - Result: passed with the pre-existing Vite large chunk warning.
- `git diff --check`
  - Result: passed.
- `.\scripts\security\publication_guard.ps1`
  - Result: passed.
- Static grep for common unsafe command strings in the new product code and KAN-146 docs:
  - Result: no matches.

Full PR validation remains required before merge.
