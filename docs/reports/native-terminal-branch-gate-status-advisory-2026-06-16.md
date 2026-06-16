# Native Terminal Branch Gate Status Advisory Report

Updated: 2026-06-16
Ticket: `KAN-140`

## Summary

KAN-140 adds a minimal branch gate status badge to the Desktop native terminal header. It gives
developers a quick read on the latest Deployment Gate evidence for the detected repository and
branch while keeping manual terminal workflows intact.

The implementation is read-only and advisory-only.

## Implemented

- Added `gitgov/src/components/cli/terminalBranchGateStatus.ts` for pure status derivation.
- Added `gitgov/src/components/cli/TerminalBranchGateStatusBadge.tsx` for the compact UI badge.
- Integrated the badge into `TerminalPanel` next to the existing repo/branch context.
- Reused existing `buildTerminalGovernanceTarget` behavior from KAN-135.
- Reused existing `cmd_server_list_deployment_gate_authorizations` read command.
- Added focused component/helper coverage in
  `gitgov/src/test/components/terminal-branch-gate-status.test.tsx`.

## Product Behavior

- `Gate ready` means the latest branch gate evidence is approved and does not indicate blocking.
- `Gate review` means a human should review the latest gate evidence.
- `No gate` means no authorization was found for the repo/branch filter.
- `Gate n/a` means the badge cannot load evidence in this Desktop context.

Every visible state remains advisory. It does not block commands, commits, pushes, PRs, deployments,
or any terminal action.

## Out Of Scope

- Backend/API changes.
- DB migration.
- Deployment execution.
- Release blocking.
- Command interception or command policy enforcement.
- Provider or repository mutation.
- AI/agent decisioning.
- Compliance/certification/legal/regulatory claims.

## Validation

Local validation:

- `npm --prefix gitgov run test -- --run src/test/components/terminal-branch-gate-status.test.tsx`
  passed with 6 focused tests.
- Focused terminal suite passed with 18 tests.
- `npm --prefix gitgov run typecheck` passed.
- `npm --prefix gitgov run lint` passed.
- `npm --prefix gitgov run test -- --run` passed with 410 frontend tests.
- `npm --prefix gitgov run build` passed with the pre-existing Vite large chunk warning.
- `git diff --check` passed.
- `.\scripts\security\publication_guard.ps1` passed.
- Static grep confirmed the new badge files do not call terminal write/resize/stop commands or
  mutating Control Plane commands; matches were limited to the existing read-only Deployment Gate
  list command and documentation/test text.

Additional validation is tracked in the PR before merge.
