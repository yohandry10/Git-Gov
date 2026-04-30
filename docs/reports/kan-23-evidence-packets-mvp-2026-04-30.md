# KAN-23 Evidence Packets MVP

Date: 2026-04-30

## Summary

`KAN-23` implements the first GitGov Evidence Packets MVP. The feature adds a ticket-scoped evidence packet endpoint, exposes it through the desktop Control Plane client, and adds a compact admin dashboard panel for generating and downloading packet JSON.

## Implemented

- Backend route:
  - `GET /evidence/packets/tickets/{ticket_id}`
- Evidence packet model:
  - ticket metadata
  - commit-ticket-pipeline correlations
  - related merged PR evidence
  - quality-gate-like pipeline runs
  - completeness counters
  - SHA-256 content hash
- Desktop/Tauri bridge:
  - `cmd_server_get_ticket_evidence_packet`
- Dashboard UI:
  - ticket input
  - generate action
  - JSON download
  - completeness and hash summary
- Design note:
  - `docs/design/evidence-packets-mvp.md`

## Scope Boundaries

- This ticket does not implement Vercel AI SDK, MCP, SonarCloud, Jenkins trigger-only flow, or full OpenAPI/SDK work.
- The first packet type is ticket-based. PR and release packets remain follow-up product work.
- `branch` is preserved in request/packet metadata; strict branch filtering remains a follow-up because the underlying ticket-flow correlation query is not branch-scoped yet.

## Validation

Completed locally:

```text
cargo check                    # gitgov/gitgov-server
cargo check                    # gitgov/src-tauri
cargo test                     # gitgov/gitgov-server, 170 tests
npm run typecheck              # gitgov
npm run lint                   # gitgov
npm test -- --run              # gitgov, 267 tests
npm run build                  # gitgov
git diff --check               # repo root
.\scripts\security\publication_guard.ps1
```

Completed on GitHub after PR `#95` merged as `6d3fb85`:

```text
CI                                      run 25153717623
Release Readiness Gate                  run 25153717624
Quality Gate Policy Matrix (Optional)   run 25153717652
Secret Scan                             run 25153717622
SonarQube Governance (Non-Blocking)     run 25153717650
Public Naming Guard                     run 25153717646
Governance Correlation Smoke (Optional) run 25153717617
Desktop Updater Readiness (Optional)    run 25153717635
```

Production validation:

```text
Render deploy: dep-d7pgh97aqgkc738i2dv0
Render status: live
Backend health: ok
Evidence packet: KAN-23 found=true
Completeness: commits=1, pull_requests=1, pipelines=1, quality_gates=1
Hash prefix: 7fa12531dc10
```
