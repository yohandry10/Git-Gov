# KAN-132 Native Terminal Session History

Date: 2026-06-16

GitHub issue: `#464`

Pull request: `#465`

Main commit: `86d70861`

## Summary

KAN-132 adds a local, session-scoped native terminal command history to the Desktop Workspace.
Operators can open a compact history drawer from the terminal header and see recent commands typed
in the current native terminal session with shell, repo, branch, and timestamp context.

The feature is intentionally read-only and local. It does not persist commands to the backend, does
not create audit evidence, does not block commands, and does not change the native PTY write path.

## Implemented

- `gitgov/src/components/cli/terminalSessionHistory.ts`:
  - pure parser for native terminal input.
  - history append helper with safe metadata defaults and 50-command cap.
- `gitgov/src/components/cli/TerminalPanel.tsx`:
  - command draft tracking before forwarding input to the PTY.
  - header history button with current session count.
  - compact local session history drawer.
- `gitgov/src/test/components/terminal-session-history.test.ts`:
  - Enter and in-progress draft behavior.
  - pasted multi-command input.
  - Backspace and Ctrl+C controls.
  - ANSI escape navigation handling.
  - newest-first capped history metadata.
  - empty command rejection and safe default labels.

## Guardrails

- No backend migration.
- No Render/API deploy requirement.
- No Control Plane audit write.
- No command interception, enforcement, approval, or policy decision.
- No compliance, certification, legal, or regulatory claim.

## Validation

- `pnpm --dir gitgov exec vitest run src/test/components/terminal-session-history.test.ts src/test/components/terminal-status.test.ts`
  - `2` files passed.
  - `9` tests passed.
- `pnpm --dir gitgov typecheck`
  - passed.
- `pnpm --dir gitgov lint`
  - passed.
- `pnpm --dir gitgov build`
  - passed with the pre-existing Vite large chunk warning.
- PR checks
  - passed: Security Guard, Frontend Lint + Typecheck, Desktop Rust Clippy, Server Clippy + Check,
    Website Lint + Typecheck + Build, Validate Policy-as-Code, Validate quality_gates warn/block
    matrix, Workflow Lint, Sonar Scan + Quality Gate, Vercel, and internal marker guard.
