# KAN-75 - Public Web, Roadmap, And Claims Documentation Audit

Date: 2026-05-02

## Scope

`KAN-75` closes the remaining documentation reality-audit phases that were left after the backend/API, Desktop/dashboard, and workflows/scripts/ops passes:

- public web documentation and public-facing product claims.
- roadmap, context, and product-state handoff material.
- systematic cleanup of stale claims that make the product look less mature, more absolute, or less accurate than the repository state supports.

This is documentation-only work. It does not implement `KAN-69 - Enterprise Action Center guided UX`.

## Sources Checked

- `gitgov-web/package.json`
- `gitgov-web/README.md`
- `gitgov-web/content/docs/*.md`
- `gitgov-web/content/docs/es/*.md`
- `gitgov-web/CONTENT_ARCHITECTURE_GUIDE.md`
- `docs/design/enterprise-self-service-and-ai-copilot-roadmap.md`
- `docs/design/enterprise-action-center-ux-focus.md`
- `docs/IMPLEMENTATION_STATUS.md`
- `docs/CURRENT_CONTEXT.md`
- `docs/QUICKSTART.md`
- current code/config references for `blocked_push`, `/policy/check`, release governance gate templates, and Risk Outcomes metric surfacing

## Corrections Made

- Updated public web runtime documentation from stale Next.js `15.5.10` wording to the current `15.5.15` package baseline.
- Reframed Jira from preview language to operational API and native signed webhook support.
- Clarified governance blocking boundaries:
  - workstation checks can block configured pushes and record `blocked_push` evidence.
  - `/policy/check` remains advisory by default and blocks only for explicitly configured scopes.
  - release governance enforcement is opt-in through the release governance workflow gate, not the default release behavior.
- Separated managed Render HTTPS production wording from self-hosted reverse-proxy guidance.
- Reframed pricing as enterprise evaluation and pilot fit instead of a finished public pricing matrix.
- Corrected risk-outcome documentation so surfaced Time-to-Evidence and MTTR metrics are described as sample-based and not SLO-backed until calibration work exists.
- Removed stale public-doc links to `/docs/privacy` where only the public `/privacy` page exists.
- Updated roadmap/context/status material so `KAN-69` remains pending and `KAN-75` is clearly a documentation-reality follow-up, not a new MVP.

## Remaining Product State

- `KAN-69 - Enterprise Action Center guided UX` remains pending.
- Product direction from `KAN-68` still stands: stop defaulting to standalone hardening/report chains and package existing capabilities into a guided user workflow.
- After `KAN-75`, new documentation passes should be opened only for concrete stale-docs defects or specific public web copy/design targets.

## Non-Goals

- No runtime code changes.
- No provider mutation.
- No GitHub branch-protection changes.
- No secret, variable, or token reads beyond normal secret-safe local validation.
- No SonarCloud proposal.
- No Jenkins trigger-only flow.
- No OpenAPI/SDK blocker.
- No Action Center UI implementation.

## Validation

Local validation passed:

- `git diff --check`
- `.\scripts\security\publication_guard.ps1`
- stale-claim search for corrected public documentation phrases
- `pnpm --dir gitgov-web build`
- `pnpm --dir gitgov-web typecheck` after Next regenerated `.next/types`

PR validation passed:

- PR `#200` merged as `b393a82`.
- Required checks passed before merge: `Security Guard`, `Server Clippy + Check`, `Desktop Rust Clippy`, `Frontend Lint + Typecheck`, `Website Lint + Typecheck + Build`, and `Validate quality_gates warn/block matrix`.
- Supporting checks passed before merge: `Workflow Lint`, `Sonar Scan + Quality Gate`, Vercel, and Vercel Preview Comments.
- Post-merge `main` checks passed for `b393a82`: `CI` run `25265387894`, `Release Readiness Gate` run `25265387888`, `Secret Scan` run `25265387889`, `Public Naming Guard` run `25265387885`, `Quality Gate Policy Matrix` run `25265387902`, `Governance Correlation Smoke` run `25265387895`, `Desktop Updater Readiness` run `25265387900`, and `SonarQube Governance` run `25265387890`.
