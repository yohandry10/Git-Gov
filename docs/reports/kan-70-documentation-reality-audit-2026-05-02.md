# KAN-70 Documentation Reality Audit

Updated: 2026-05-02

## Summary

KAN-70 starts the phased cleanup of GitGov living documentation. The goal is to compare docs against the actual repository state and update stale claims without adding product features.

## Product Context

- `KAN-68` closed the decision to stop adding standalone hardening/features by default.
- `KAN-69` is pending as `Enterprise Action Center guided UX`.
- `KAN-70` is documentation-only: it records the real state and prepares docs for the next product/UX phase.

## First Pass Verified

| Area | Verified state |
| --- | --- |
| Branch | `docs/KAN-70-documentation-reality-audit` |
| Workflows | `.github/workflows` contains `32` workflow files |
| Backend migrations | Latest schema migration is `supabase_schema_v25.sql` |
| Backend tests | `cargo test -- --list` reports `193` tests |
| Frontend tests | `gitgov/src` contains `25` test files |
| Enterprise backend routes | `main.rs` includes adoption profile, onboarding checklist tracking, release approvals, and release governance evaluation routes |
| Product direction | `KAN-69` remains pending guided UX work, not another standalone evidence chain |

## Included Local Edits

Existing local documentation edits are being kept only where they match repository reality:

- Render is the current production backend route.
- PostgreSQL/Supabase wording is generalized where docs apply to both production and local installs.
- Migration references are updated from `v22` to `v25`.
- CI/CD workflow count is updated to `32`.
- Desktop/frontend and backend test counts are updated from verified local commands.
- Enterprise adoption, release approval, checklist tracking, copilot, evidence packets, policy drift, GDPR, Prometheus metrics, and SSE are documented as implemented capabilities where matching source files/routes exist.

## Phase Boundaries

This first PR should not attempt to cover the entire repository. Follow-up phases should audit:

1. Backend route/API/schema docs against `gitgov/gitgov-server/src` and SQL migrations.
2. Desktop/dashboard docs against `gitgov/src` and `gitgov/src-tauri`.
3. Web public docs and marketing claims against `gitgov-web`.
4. Workflow/runbook docs against `.github/workflows`, `scripts`, `ops`, `Jenkinsfile`, and Docker Compose.
5. Product roadmap and handoff docs against the post-`KAN-68` Action Center direction.

## Non-Goals

- No new feature implementation.
- No provider mutation.
- No GitHub Actions secret/variable mutation.
- No SonarCloud proposal.
- No Jenkins trigger-only flow unless explicitly requested.
- No OpenAPI completeness work as a blocker.
- No secret values in documentation.
