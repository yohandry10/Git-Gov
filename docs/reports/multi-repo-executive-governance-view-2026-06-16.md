# KAN-129 Multi-Repo Executive Governance View Report

Date: 2026-06-16

Issue: `#453`

PR: pending

Main commit: pending

## Summary

KAN-129 adds a tenant-scoped, read-only executive repository view.

New backend route:

- `GET /executive/repositories`

Returned context:

- Repository summaries across the resolved tenant.
- Deployment Gate counts and latest gate pointer.
- Change Risk counts, latest risk level, and latest manual review state.
- CAB packet and CAB decision manifest counts.
- Latest manifest status/hash.
- Tenant-scoped totals for the current page.
- Explicit no-claim flags.

## Guardrails

- `advisory_only=true`
- `enforcement_used=false`
- `deployment_execution=false`
- `provider_mutation=false`
- `repository_mutation=false`
- `llm_used=false`
- `agent_governance_used=false`
- `compliance_claim=false`
- `certification=false`

The feature does not approve deployments, block releases, execute deploys, mutate providers/repos,
mutate source evidence, create CAB packets/manifests automatically, use AI/Agent Governance, or
create legal/compliance/certification claims.

## Local Validation

Passed:

- `cargo fmt --manifest-path gitgov/gitgov-server/Cargo.toml --check`
- `cargo check --manifest-path gitgov/gitgov-server/Cargo.toml`
- `cargo clippy --manifest-path gitgov/gitgov-server/Cargo.toml -- -D warnings`
- `cargo test --manifest-path gitgov/gitgov-server/Cargo.toml --no-run`
- `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`
- `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`49` tests)
- `npm --prefix gitgov run typecheck`
- `npm --prefix gitgov run lint`
- `npm --prefix gitgov test -- --run` (`383` tests)
- `npm --prefix gitgov test -- --run src/test/useControlPlaneStore.test.ts src/test/components/MultiRepoExecutiveGovernancePanel.test.tsx` (`49` tests)
- `npm --prefix gitgov run build` with the pre-existing Vite large chunk warning
- Focused real Postgres backend test:
  `multi_repo_executive_governance_view_is_read_only_and_tenant_scoped`
- `git diff --check`
- `scripts/security/publication_guard.ps1`

The backend test covers:

- Tenant `executive-org` with repositories `executive-org/payments` and `executive-org/portal`.
- Separate tenant `executive-other` with repository `executive-other/repo`.
- Real Deployment Gate authorization rows.
- Real Change Risk API creation for both tenant repositories.
- Manual risk review for the high-risk repository.
- CAB packet, CAB disposition, and CAB decision manifest creation for one repository.
- Auditor read access.
- Developer denial.
- Other-tenant isolation.
- No-claim flags.
- No source mutation: Deployment Gate authorization and Agent Governance evaluation counts remain
  unchanged after reading the executive view.

## Product Notes

The view is intentionally read-only and manual-first.

Posture values are triage labels:

- `attention` when a repository has blocked gates or high-risk evaluations.
- `review` when evidence still needs human review or has advisory/revoked signals.
- `healthy` when governance evidence exists without those signals.
- `unknown` when no evidence is available in the page.

They are not deployment approvals, compliance scores, or official audit conclusions.

## Production Validation

Pending until PR merge, Render deploy, and production smoke.
