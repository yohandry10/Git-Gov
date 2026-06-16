# Native Terminal Safe Quick Commands Report

Date: 2026-06-16
Ticket: KAN-134
Issue: #470

## Summary

KAN-134 implements the next Developer Distribution Surfaces slice selected after GPT/product-leader
consultation: a local native-terminal quick-command palette for safe, read-only Git inspection.

The feature is intentionally insert-only. It writes command text into the native PTY without sending
Enter, so the operator remains responsible for execution. This preserves the manual-first posture and
keeps GitGov out of command enforcement, repo mutation, provider mutation, deploy execution, and
compliance-claim territory.

## Implemented

- Added a read-only command catalog for:
  - `git status --short`
  - `git branch --show-current`
  - `git log --oneline -5`
  - `git diff --stat`
  - `git remote -v`
- Added structural validation that rejects compound shell expressions, redirects, non-git commands,
  and mutating Git/deploy/provider verbs.
- Added a Desktop terminal menu that previews commands, disables them outside Git repositories, and
  shows recent commands used in the current local session.
- Inserted quick commands through the existing native terminal write path without newline.
- Updated the KAN-132 input draft when inserting, so manual Enter records the command in session
  history exactly once.

## No-Claim Boundary

KAN-134 does not add backend persistence, API routes, database tables, audit events, command
approval, command blocking, command interception, auto-run, provider/repo/deploy mutation, AI/Agent
Governance, OPA/Rego, compliance scoring, certification, legal attestation, or regulatory claims.

## Validation

Initial focused validation:

- `pnpm --dir gitgov exec vitest run src/test/components/terminal-quick-commands.test.ts src/test/components/terminal-session-history.test.ts src/test/components/terminal-git-context.test.ts`
  - Result: `3` files passed, `14` tests passed.
- `pnpm --dir gitgov typecheck`
  - Result: passed.
- `pnpm --dir gitgov lint`
  - Result: passed.
- `pnpm --dir gitgov build`
  - Result: passed with the existing Vite large chunk warning.
- `pnpm --dir gitgov exec vitest run`
  - Result: `42` files passed, `400` tests passed.
- `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`
  - Result: passed.
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`
  - Result: passed.
- `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`
  - Result: passed.
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml`
  - Result: `52` tests passed.
- `git diff --check`
  - Result: passed.
- `powershell -ExecutionPolicy Bypass -File scripts/security/publication_guard.ps1`
  - Result: passed.

Remaining before merge:

- PR checks.

No Render/API deploy is expected because the change is Desktop/frontend local only.
