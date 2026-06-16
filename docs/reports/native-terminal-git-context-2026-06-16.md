# KAN-133 Native Terminal Repo/Branch Context

Date: 2026-06-16

GitHub issue: `#467`

## Summary

KAN-133 adds safe local Git context to the Desktop native terminal header. The header now shows a
compact repo/branch label such as `GitGov:main`, `GitGov:detached@abc1234`, `No git repo`, or
`context pending`.

The context is resolved locally through Tauri and `git2::Repository::discover`. It does not execute
Git commands, does not mutate repositories, does not persist to the backend, and does not create
Control Plane audit evidence.

## Implemented

- `cmd_get_native_terminal_git_context` Tauri command.
- Rust helpers for safe Git context detection and simple `cd`/`chdir`/`sl`/`Set-Location` cwd
  inference.
- Desktop `TerminalPanel` header context label.
- Frontend helpers for refresh trigger detection and safe display labels.
- Rust and frontend tests for non-git, real git, detached/branch labels, unsafe compound commands,
  and KAN-132 history compatibility.

## Guardrails

- No backend migration.
- No Render/API deploy requirement.
- No command blocking, approval, interception, or automatic rerun.
- No Git push/pull/fetch/checkout.
- No provider/repo/deploy mutation.
- No AI/Agent Governance/OPA/Rego/MCP dependency.
- No compliance/certification/legal/regulatory claim.

## Validation

- `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`
  - passed.
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml native_terminal_git_context -- --nocapture`
  - `2` focused Rust tests passed.
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml terminal_cd_resolution -- --nocapture`
  - `1` focused Rust test passed.
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`
  - passed.
- `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`
  - passed.
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml`
  - `52` tests passed.
- `pnpm --dir gitgov exec vitest run src/test/components/terminal-git-context.test.ts src/test/components/terminal-session-history.test.ts`
  - focused frontend tests passed.
- `pnpm --dir gitgov typecheck`
  - passed.
- `pnpm --dir gitgov lint`
  - passed.
- `pnpm --dir gitgov build`
  - passed with the pre-existing Vite large chunk warning.
- `pnpm --dir gitgov exec vitest run`
  - `41` test files passed.
  - `395` tests passed.
- `git diff --check`
  - passed.
- `powershell -ExecutionPolicy Bypass -File scripts/security/publication_guard.ps1`
  - passed.
