# Native Terminal Branch Gate Status Advisory MVP

Updated: 2026-06-16
Ticket: `KAN-140`

## Product Decision

Add a compact branch gate status badge to the Desktop native terminal header. The badge helps a
developer see whether the currently detected repository and branch have recent Deployment Gate
evidence without opening the larger Governance Context drawer.

This is advisory only. It does not block terminal commands, commits, pushes, pull requests,
deployments, or any other local action.

## User Experience

The badge appears beside the existing `repo:branch` terminal context when GitGov can map the local
repo to a GitHub `owner/repo` and the Desktop Control Plane config exists.

States:

- `Gate ready`: the latest Deployment Gate authorization for the repo/branch is approved and does
  not carry blocking or would-block evidence.
- `Gate review`: the latest evidence is blocked, would block, requires approval, is denied/rejected,
  or reports insufficient/missing evidence.
- `No gate`: GitGov found no Deployment Gate authorization for the repo/branch filter.
- `Gate n/a`: Control Plane is not configured or the read-only lookup is unavailable.

The badge stays hidden while Git context is pending, outside a Git repo, or missing a GitHub remote.
This keeps the terminal header quiet instead of showing a noisy warning for normal non-repo use.

## Implementation

- `TerminalBranchGateStatusBadge` renders the small header badge.
- `terminalBranchGateStatus.ts` contains the pure status mapping logic.
- The badge reuses `buildTerminalGovernanceTarget` from the KAN-135 Governance Context panel so it
  derives the same safe repository/branch target and does not expose local absolute paths.
- The badge calls existing Tauri command `cmd_server_list_deployment_gate_authorizations` with
  `limit=1`, `repository_full_name`, and `branch`.
- No backend route, schema, DB migration, provider integration, repository mutation, or deployment
  execution was added.

## Guardrails

- Advisory-only UI.
- No command interception.
- No command approval or blocking.
- No auto-run.
- No commit/push/deploy enforcement.
- No Control Plane audit write.
- No provider, repository, or deployment mutation.
- No AI, Agent Governance, OPA/Rego, MCP, or chatbot dependency.
- No compliance, certification, legal, or regulatory claim.

## Validation Focus

Focused tests cover:

- pending and unmapped terminal contexts stay visually quiet.
- missing Control Plane config is explicit but does not call the API.
- approved gate evidence becomes `Gate ready`.
- blocked, would-block, requires-approval, and insufficient-evidence cases become `Gate review`.
- the component queries the exact repo/branch/org filter through the existing read-only command.
- the badge does not expose local absolute path details.
