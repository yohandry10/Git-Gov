# KAN-73 CI/Workflows Documentation Reality Audit

Updated: 2026-05-02

## Summary

KAN-73 is phase 4 of the GitGov documentation reality audit. It checks GitHub Actions workflow inventory, trigger behavior, artifact-producing automation, branch-protection required checks, and release/readiness documentation against repository and GitHub reality without changing workflow behavior.

## Product Context

- `KAN-69 - Enterprise Action Center guided UX` remains pending product/UX work.
- `KAN-70` completed the first broad documentation cleanup pass.
- `KAN-71` completed the backend/API/schema documentation audit.
- `KAN-72` completed the Desktop/dashboard documentation audit.
- `KAN-73` narrows the audit to `.github/workflows`, `.github/scripts`, `scripts/github`, `scripts/control-plane`, release/readiness automation docs, and live branch-protection metadata.

## Verified Sources

| Area | Source checked | Verified state |
| --- | --- | --- |
| Workflow files | `.github/workflows` | `32` workflow files |
| GitHub workflow registry | `gh workflow list --all` | `32` workflows active |
| Pull request triggers | workflow YAML scan | `5` workflows include `pull_request` |
| Push triggers | workflow YAML scan | `9` workflows include `push` |
| Manual dispatch triggers | workflow YAML scan | `29` workflows include `workflow_dispatch` |
| Scheduled triggers | workflow YAML scan | `22` workflows include `schedule` |
| Artifact uploads | workflow YAML scan | `28` workflows use `actions/upload-artifact` |
| Branch protection | GitHub API for `main` required status checks | strict checks enabled; `6` required contexts |
| Required contexts | GitHub API | `Security Guard`, `Server Clippy + Check`, `Desktop Rust Clippy`, `Frontend Lint + Typecheck`, `Website Lint + Typecheck + Build`, `Validate quality_gates warn/block matrix` |
| Runtime env guard | `.github/scripts/assert-gitgov-env.sh` and workflows | `GITGOV_ENV` is explicitly validated in CI/deploy jobs |

## Corrections Made

- `docs/DEPLOYMENT.md` now lists the live required branch-protection checks instead of the older recommended set.
- `docs/DEPLOYMENT.md` now distinguishes checks that run on PR/push from checks that are actually required by branch protection.
- `docs/ARCHITECTURE.md` now records current workflow trigger counts, artifact count, and live branch-protection required contexts.
- `docs/IMPLEMENTATION_STATUS.md`, `docs/CURRENT_CONTEXT.md`, and `AGENTS.md` now track `KAN-73` as the active documentation-only phase and keep `KAN-69` pending.

## Observed Follow-Up

The helper script defaults in `scripts/github/check_branch_protection.ps1` and `scripts/github/set_required_checks.ps1` still include older check-context assumptions. KAN-73 did not change those scripts because this phase is documentation-only. If the helpers should be made authoritative again, handle that as a small traced automation follow-up.

Post-merge `Secret Scan` for merge commit `9952d47` passed, but GitHub emitted a non-failing annotation that `gitleaks/gitleaks-action@v2` still runs on Node.js 20. GitHub's runner message says Node 24 becomes the default on June 2, 2026, and Node 20 is removed on September 16, 2026. This should be handled as a small CI runtime follow-up, not as a KAN-73 failure.

## Non-Goals

- No workflow YAML behavior changes.
- No branch-protection mutation.
- No GitHub Actions variable or secret mutation.
- No provider mutation.
- No release governance default change.
- No SonarCloud proposal.
- No Jenkins trigger-only flow work.
- No implementation of `KAN-69`; it remains pending guided UX work.
