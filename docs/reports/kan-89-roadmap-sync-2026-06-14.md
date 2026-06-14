# KAN-89 Roadmap Sync After Break-glass Approval Routing

KAN-89 updates the roadmap and handoff context after KAN-88 so Deployment Gates 0.1 no longer
describes pre-approved break-glass approval routing as future work.

## Scope

- Move KAN-88 into the implemented Deployment Gates primitives.
- Remove stale roadmap wording that listed break-glass approval routing as future scope.
- Clarify the remaining Deployment Gates backlog after KAN-88.
- Mark `0.2 Agentic Governance Layer` as the next major roadmap block.

## Product State

Deployment Gates 0.1 now has the core customer-facing control path:

- first governed repo setup;
- deployment authorization API;
- persisted authorization history;
- Desktop history UI;
- GitHub Actions, Jenkins Pipeline, and GitLab CI examples;
- environment policy matrix;
- audited break-glass exception records;
- pre-approved break-glass approval routing bound to release evidence.

The remaining Deployment Gates work is no longer the basic authorization/routing contract. The
remaining work is product packaging around provider installation, richer provider coverage,
environment-specific routing workflows, notifications/escalations, and multi-approver exception
chains.

The next major roadmap block is `0.2 Agentic Governance Layer`: deterministic policy/API primitives
for agents to ask GitGov whether they may commit, push, merge, change policy, or deploy.

## Validation

- Search must no longer find the stale phrase that describes break-glass approval routing as future
  scope in the roadmap.
- `docs/CURRENT_CONTEXT.md` must point at KAN-89 as the current documentation sync ticket while
  preserving the KAN-88 production validation evidence.
- Publication guard must pass.
