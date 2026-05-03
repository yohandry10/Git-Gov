# KAN-76 - Public Agent Documentation Visibility

Date: 2026-05-03

## Scope

`KAN-76` creates a public, tracked context bridge for external agents and research models after the `KAN-70` through `KAN-75` documentation reality audit.

The goal is to make current product state readable without publishing restricted local forensic or strategy documents.

## Decision

Do not force-add ignored local files:

- `docs/ENTERPRISE_READINESS_DECISION.md`
- `docs/AUDIT_*.md`
- `docs/INTEGRATIONS_AUDIT_*.md`

Those files remain restricted by `docs/PUBLICATION_POLICY.md` and blocked by `scripts/security/publication_guard.ps1`.

Instead, publish sanitized current context in:

- `docs/AGENT_PUBLIC_CONTEXT.md`
- `docs/CURRENT_CONTEXT.md`
- `docs/IMPLEMENTATION_STATUS.md`
- ticket-scoped reports under `docs/reports/`

## External Report Takeaways

The external deep-research report is useful as strategic input, but not as current implementation truth.

Durable conclusions kept:

- GitGov should package existing evidence and traceability capabilities into a clearer product workflow.
- The product should avoid becoming a maze of disconnected surfaces.
- Deterministic systems should own scoring, policy, permissions, and enforcement; AI should explain and compose evidence.
- MCP and broader agent interoperability can follow after the core UX path is coherent.
- `KAN-69 - Enterprise Action Center guided UX` is the right next product direction.

Mitigated or stale implementation findings from older reports should not be repeated as active backlog unless they also appear in current tracked context or a current Jira ticket.

## Public Context Added

`docs/AGENT_PUBLIC_CONTEXT.md` now gives external agents:

- the current product phase.
- implemented capability inventory at a high level.
- completed documentation audit phases.
- safe reading order.
- ignored documentation policy.
- conclusions from the external report that remain useful.
- current recommended next work and non-goals.

## Non-Goals

- No runtime code changes.
- No provider mutation.
- No branch-protection changes.
- No secret or token reads beyond normal safe validation.
- No force-add of restricted forensic/strategy docs.
- No SonarCloud proposal.
- No Jenkins trigger-only setup.
- No OpenAPI/SDK blocker.
- No `KAN-69` implementation.

## Validation

Local validation passed:

- `git diff --check`
- `.\scripts\security\publication_guard.ps1`

PR validation passed:

- PR `#202` merged as `8f311f2`.
- Required checks passed before merge: `Security Guard`, `Server Clippy + Check`, `Desktop Rust Clippy`, `Frontend Lint + Typecheck`, `Website Lint + Typecheck + Build`, and `Validate quality_gates warn/block matrix`.
- Supporting checks passed before merge: `Workflow Lint`, `Sonar Scan + Quality Gate`, Vercel, and Vercel Preview Comments.
- Post-merge `main` checks passed for `8f311f2`: `CI` run `25266101104`, `Release Readiness Gate` run `25266101089`, `Secret Scan` run `25266101093`, `Public Naming Guard` run `25266101097`, `Quality Gate Policy Matrix` run `25266101102`, `Governance Correlation Smoke` run `25266101092`, `Desktop Updater Readiness` run `25266101090`, and `SonarQube Governance` run `25266101101`.
