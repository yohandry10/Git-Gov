# KAN-130 Multi-Repo Executive Governance Filters Report

Date: 2026-06-16

Issue: `#456`

PR: pending

Main commit: pending

## Summary

KAN-130 extends `GET /executive/repositories` with read-only filters over existing governance
evidence.

Added filters:

- `repository`
- `environment`
- `posture`
- `gate_decision`
- `risk_level`
- `review_status`

The route remains tenant-scoped and advisory-only. It does not create a new evidence domain or a
parallel endpoint.

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
- Focused real Postgres backend test:
  `multi_repo_executive_governance_view_is_read_only_and_tenant_scoped`
- `cargo fmt --manifest-path gitgov/src-tauri/Cargo.toml --check`
- `cargo check --manifest-path gitgov/src-tauri/Cargo.toml`
- `cargo clippy --manifest-path gitgov/src-tauri/Cargo.toml -- -D warnings`
- `cargo test --manifest-path gitgov/src-tauri/Cargo.toml` (`49` tests)
- `npm --prefix gitgov run typecheck`
- `npm --prefix gitgov run lint`
- `npm --prefix gitgov test -- --run` (`384` tests)
- `npm --prefix gitgov test -- --run src/test/useControlPlaneStore.test.ts src/test/components/MultiRepoExecutiveGovernancePanel.test.tsx` (`50` tests)
- `npm --prefix gitgov run build` with the pre-existing Vite large chunk warning
- `git diff --check`
- `scripts/security/publication_guard.ps1`

The backend test now covers:

- Baseline unfiltered executive view.
- `posture=attention&environment=production`.
- `environment=staging&review_status=needs_review`.
- `gate_decision=blocked`.
- `repository=portal&risk_level=low`.
- Conflicting `gate_decision=blocked&risk_level=low` returning no repositories.
- Invalid `posture=critical` returning HTTP `400`.
- Auditor read access.
- Developer denial.
- Other-tenant isolation.
- No source mutation across Deployment Gates, Change Risk evaluations, CAB packets, CAB manifests,
  or Agent Governance evaluations.

## Production Validation

Pending until PR merge, Render deploy, and production smoke.
