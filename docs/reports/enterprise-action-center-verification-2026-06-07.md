# KAN-69 Enterprise Action Center Verification

Date: 2026-06-07
Ticket: `KAN-69`
Scope: product, business logic, security, infrastructure, maintainability
PR: `#206 - fix(KAN-69): harden Action Center verification logic`
Main commit: `8a55a6d fix(KAN-69): harden action center verification logic (#206)`

## Baseline

- Branch baseline before follow-up: `main` at `30fa716 docs(KAN-69): record action center merge (#205)`.
- Implementation PR: `#204 - product(KAN-69): add guided Action Center workspace`.
- Post-merge context PR: `#205 - docs(KAN-69): record Action Center merge`.
- Worktree was clean before the verification follow-up branch.

## Q/A Analysis

1. Q: Does the Action Center answer "what should I do now" without becoming another dashboard?
   A: Yes. It shows one primary recommendation plus alternatives, then deep-links to existing Workspace or Control Plane workflows.

2. Q: Is the primary recommendation deterministic and explainable?
   A: Yes. `buildActionCenterGuidance` uses explicit `Goal + Evidence + Permission` rules and evidence lines; it is not LLM-driven.

3. Q: Does the user keep freedom to ignore the guide?
   A: Yes. Recommendations are advisory; links navigate, and the Workspace/Control Plane remain available.

4. Q: Do persona/lens choices create false authorization?
   A: No. Lenses change presentation text only. Authorization still comes from Control Plane role and backend route policy.

5. Q: Does it reduce cognitive load versus the Dashboard and Control Plane?
   A: Mostly yes. It avoids embedding into crowded surfaces and acts as a routing layer over existing capabilities.

6. Q: Does every action land on an existing capability?
   A: Yes. Targets are current routes or deep links: Workspace, Control Plane, Enterprise Adoption, Evidence Packet, Release Approvals, and Governance Copilot.

7. Q: What happens if data is missing or contradictory?
   A: The follow-up fix keeps release prep conservative when Jira coverage is missing or empty. Existing rules already treat disconnected, invalid profile, missing providers, weak pipeline, and incomplete packet states as non-ready.

8. Q: Are infrastructure boundaries preserved?
   A: Yes. No backend endpoint, provider mutation, repository mutation, release-blocking default, SonarCloud path, Jenkins trigger-only path, or OpenAPI/SDK dependency was added.

9. Q: Are non-admin users handled safely?
   A: Yes after follow-up. Non-admin users can see guidance, but admin-only workflows are marked as Admin actions and the UI no longer makes known-forbidden admin-only adoption-profile/checklist reads.

10. Q: How will product success be measured?
    A: The useful metrics are time to first correct action, fewer confused navigation loops, more evidence packet usage, cleaner release approval context, and fewer operator questions about whether to start from provider health, Jira coverage, evidence, release approval, or copilot.

## Findings And Fixes

### Fixed: Missing Jira Coverage Was Not Conservative Enough

Symptom: release prep could move to Evidence Packet/release decision when pipeline health was good and Jira ticket coverage had not loaded.

Consequence: the Action Center could imply release confidence before traceability evidence existed.

Fix: `prepareReleasePrimary` now routes to `repair-traceability-coverage` when coverage is missing, the commit window is empty, or coverage is below threshold.

Regression coverage:

- `keeps release prep conservative when Jira traceability is not loaded`
- `does not treat an empty traceability window as release-ready`

### Fixed: Non-Admin UI Made Known-Forbidden Admin Reads

Symptom: the Action Center refreshed general evidence by role, but also attempted adoption-profile and onboarding-checklist reads for every connected role.

Consequence: Developer/Architect/PM users could see silent admin-only 403s while building guidance.

Fix: the UI still refreshes role-appropriate evidence for every connected user, but loads admin-only adoption-profile/checklist state only when `userRole === 'Admin'`.

## Security Review

- No `.env` files are read by the Action Center.
- No provider credentials are read or printed.
- No `Authorization` or `Bearer` header handling exists inside the Action Center component/helper.
- No provider state, customer repository, workflow installation, release approval, or release gate is mutated by opening the Action Center.
- Deep links use internal React Router targets; they do not execute privileged actions by themselves.

## Maintainability Review

- Recommendation logic remains in a pure helper with unit tests.
- UI rendering remains in `ActionCenterWorkspace`.
- Current size is acceptable for the first surface, but future growth should split recommendation panels or rule groups before adding more goals/providers.
- The helper test suite now covers profile priority, provider config/evidence ordering, non-admin admin-action labeling, weak pipeline, low Jira coverage, missing Jira coverage, empty coverage window, and complete Evidence Packet review.

## Validation

- `git status --short --branch` - baseline clean on `main...origin/main`.
- `git log -1 --oneline main` - `30fa716 docs(KAN-69): record action center merge (#205)`.
- PR `#204` - merged; status checks successful.
- PR `#205` - merged; status checks successful.
- PR `#206` - merged; status checks successful before and after merge.
- `npm --prefix gitgov run test -- --run src/test/components/action-center-helpers.test.ts` - passed, `8` tests.
- `npm --prefix gitgov run typecheck` - passed.
- `npm --prefix gitgov run test -- --run` - passed, `304` tests in `26` files.
- `npm --prefix gitgov run lint` - passed.
- `npm --prefix gitgov run build` - passed with the existing Vite large chunk warning.
- `Invoke-WebRequest http://127.0.0.1:5173/action-center` - HTTP `200` from the existing local Vite server.
- `git diff --check` - passed.
- `.\scripts\security\publication_guard.ps1` - passed on branch `product/KAN-69-action-center-verification-fixes`.
- Post-merge checks on `main` commit `8a55a6d` - passed: `CI` run `27100640858`, `Release Readiness Gate` run `27100640831`, `Secret Scan` run `27100640840`, `Public Naming Guard` run `27100640856`, `SonarQube Governance (Non-Blocking)` run `27100640837`, `Quality Gate Policy Matrix (Optional)` run `27100640835`, `Governance Correlation Smoke (Optional)` run `27100640862`, and `Desktop Updater Readiness (Optional)` run `27100640864`.

## Residual Risk

- Full interactive UX validation still belongs in GitGov Desktop/Tauri, because the browser-only Vite smoke cannot exercise the authenticated desktop runtime.
- Release approval readiness is still org/list-level in the current Action Center input; a future refinement could bind approval status to the selected release/ticket/evidence packet when that product decision is made.
