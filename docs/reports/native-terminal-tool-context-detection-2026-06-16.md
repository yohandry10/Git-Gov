# Native Terminal Local Provider/Tool Context Detection - 2026-06-16

Ticket: `KAN-144`
Issue: `#498`
Branch: `product/KAN-144-terminal-tool-context-detection`

## Summary

KAN-144 continues roadmap block `0.10 Developer Distribution Surfaces` after KAN-143. It adds local
provider/tool context detection for the native terminal so the safe quick-command menu can highlight
commands that match the current workspace.

The feature remains manual-first:

- no command is executed.
- no newline is inserted.
- no command is approved, blocked, intercepted, or rewritten.
- no backend/API/DB/Render path is touched.
- no provider, repository, cluster, deployment, or workflow state is mutated.

## Implemented

- Added Tauri command `cmd_get_native_terminal_tool_context`.
- Added safe metadata result types for detected tools, scan limits, and no-secret/no-network flags.
- Detects Terraform, Docker Compose, Helm, and Kubernetes local context from safe file/directory
  names only.
- Limits scans to the current cwd, depth `2`, and `200` entries.
- Ignores heavy/sensitive directories such as `.git`, `.terraform`, `node_modules`, `target`,
  `dist`, `build`, and `.next`.
- Stores tool context by native terminal session in Desktop state.
- Protects async loads with a request id so stale scans cannot populate a newer session.
- Adds a quiet `Available in this workspace` section to the existing quick-command menu.
- Keeps all KAN-143 quick commands insert-only and exact-allowlisted.

## Safety Review

The implementation does not read file contents. It reads only bounded filesystem metadata and generic
file/directory names needed for detection. The result intentionally omits absolute paths and does not
return the names of detected files.

The implementation rejects or avoids:

- `.env` content.
- `*.tfvars` and `*.tfstate` content.
- kubeconfig and cloud credential files.
- secret manifest contents.
- provider/cloud CLI calls.
- mutating commands.
- backend writes or audit records.

## Validation

Passed locally:

```powershell
cargo fmt --check
cargo test native_terminal_tool_context -- --nocapture
cargo check
cargo clippy -- -D warnings
cargo test
npm --prefix gitgov run test -- --run src/test/components/terminal-quick-commands.test.ts src/test/components/terminal-quick-commands-menu.test.tsx
npm --prefix gitgov run typecheck
npm --prefix gitgov run lint
npm --prefix gitgov run test -- --run
npm --prefix gitgov run build
git diff --check
.\scripts\security\publication_guard.ps1
```

Results:

- Rust tool-context tests: `6` passed.
- Focused quick-command/menu tests: `13` passed.
- Full frontend Vitest: `422` passed.
- Frontend typecheck passed.
- Frontend lint passed.
- Frontend build passed with the pre-existing Vite large chunk warning.
- Tauri `cargo fmt --check`, `cargo check`, `cargo clippy -- -D warnings`, and full
  `cargo test` passed (`58` tests).
- `git diff --check` passed.
- publication guard passed.
- static dangerous-command/read grep found no mutating provider commands, backend command execution,
  or file-content reads in the new tool-context path.

PR checks are still required before merge.

## Guardrails

- No backend/API route change.
- No DB migration.
- No Render deploy requirement.
- No Control Plane audit write.
- No command interception, approval, blocking, or auto-run.
- No provider, repository, cluster, deployment, or workflow mutation.
- No cloud/provider API commands.
- No secrets, `.env`, environment dumps, shell chaining, redirection, or command substitution.
- No AI/Agent Governance/OPA/Rego/MCP dependency.
- No compliance/certification/legal/regulatory claim.
