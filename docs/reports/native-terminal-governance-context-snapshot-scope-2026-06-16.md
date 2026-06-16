# Native Terminal Governance Context Snapshot Scope - 2026-06-16

Ticket: `KAN-142`
Issue: `#493`
Branch: `fix/KAN-142-terminal-context-snapshot-scope`

## Review Finding

The KAN-141 drawer could reuse loaded governance evidence by repository and branch only. That was
too weak for a multi-tenant Control Plane UI because the same repository and branch can be viewed
under a different org or server configuration.

Risk:

- changing org/server while the drawer was open could briefly show the previous context.
- a slower old async response could overwrite newer drawer state.
- the connection status text could be stale because it came from the loaded snapshot.

## Fix

- Scope loaded drawer snapshots by Control Plane URL, org, target status, repository, and branch.
- Ignore stale async responses with a request id.
- Clear loading state when a pending request is invalidated by a non-loadable target.
- Render current connection status from the live prop instead of the loaded snapshot.
- Add regression tests for org changes and out-of-order responses.

## Guardrails

- No backend/API route change.
- No database migration.
- No Control Plane audit write.
- No command interception, approval, blocking, or auto-run.
- No provider, repository, or deployment mutation.
- No AI/Agent Governance/OPA/Rego/MCP dependency.
- No compliance/certification/legal/regulatory claim.

## Validation

Passed locally:

```powershell
npm --prefix gitgov run test -- --run src/test/components/terminal-branch-gate-status.test.tsx
npm --prefix gitgov run test -- --run src/test/components/terminal-branch-gate-status.test.tsx src/test/components/terminal-governance-context.test.ts src/test/components/terminal-git-context.test.ts src/test/components/terminal-quick-commands.test.ts
npm --prefix gitgov run typecheck
npm --prefix gitgov run lint
```

Results:

- focused branch gate/context tests: `10` passed.
- focused terminal suite: `22` passed.
- frontend typecheck passed.
- frontend lint passed.
