# KAN-74 - CI Helper/Runtime Follow-up

Date: 2026-05-02

## Scope

KAN-74 is the narrow follow-up from the KAN-73 CI/workflow documentation audit. It does not add product functionality and does not mutate provider configuration.

The work covers:

- aligning branch-protection helper defaults with the live required checks on `main`;
- replacing the Secret Scan dependency on `gitleaks/gitleaks-action@v2`, which currently declares a Node.js 20 action runtime;
- keeping KAN-69 pending as product/UX work.

## Repository Reality

KAN-73 verified that `main` branch protection is strict and requires exactly:

- `Security Guard`
- `Server Clippy + Check`
- `Desktop Rust Clippy`
- `Frontend Lint + Typecheck`
- `Website Lint + Typecheck + Build`
- `Validate quality_gates warn/block matrix`

Before KAN-74, the helper defaults still included older contexts such as `Workflow Lint` and `Block internal-assistant markers in branch/commits`. Those jobs still run, but they are not required contexts in the current `main` protection rule.

KAN-73 also observed a non-failing GitHub Actions annotation from Secret Scan: `gitleaks/gitleaks-action@v2` uses Node.js 20. GitHub announced that Node 24 becomes forced by default on June 2, 2026 and Node 20 is removed from runners on September 16, 2026.

## Changes

- `scripts/github/set_required_checks.ps1` now defaults to the six live required checks.
- `scripts/github/check_branch_protection.ps1` now validates the same six live required checks.
- `.github/workflows/secret-scan.yml` now downloads and runs Gitleaks CLI `v8.30.0` directly on Ubuntu instead of invoking `gitleaks/gitleaks-action@v2`. The command scans the current PR/push commit range with `gitleaks detect --log-opts` so KAN-74 does not turn the runtime follow-up into unrelated historical secret remediation.
- Secret Scan push-range guards now fall back to the current commit parent when `github.event.before` is unavailable in the checkout, which can happen after a force-push.
- `docs/CURRENT_CONTEXT.md`, `docs/IMPLEMENTATION_STATUS.md`, and `AGENTS.md` record KAN-74 as the active follow-up while the branch is in flight.

## Safety

- No branch-protection mutation was performed by this change.
- No GitHub Actions secret or variable was created or updated.
- No provider configuration was changed.
- No Render deploy, database migration, SonarCloud proposal, Jenkins trigger-only flow, or OpenAPI/SDK work is involved.
- No secret value is read, printed, or committed.

## Validation

Local validation before PR:

- `git diff --check`: passed.
- `.\scripts\security\publication_guard.ps1`: passed.
- `.\scripts\github\check_branch_protection.ps1 -Owner yohandry10 -Repo Git-Gov -Branch main -GitHubToken <gh-auth-token>`: passed with strict checks enabled, admins enforced, and all six expected checks configured.
- `Invoke-WebRequest -Method Head` against the Gitleaks `v8.30.0` Linux x64 tarball: returned HTTP `200`.
- Local `gitleaks.exe detect --source . --log-opts 'main..HEAD' --redact --no-banner --no-color`: passed, scanned `1` commit, no leaks found.
- Workflow scan for remaining active `gitleaks/gitleaks-action@v2` usage: no matches.
- First PR check run found historical Git findings when the CLI used `gitleaks detect` across the repository history. KAN-74 changed the workflow to pass an explicit PR/push commit range through `--log-opts`, matching the existing policy intent of blocking newly introduced leaks without reopening historical cleanup.
- Second PR check run failed on CLI syntax while testing a working-tree scan alternative. That path was replaced with range-limited `gitleaks detect` instead.
- Third PR check run showed that the push-triggered Secret Scan job can receive a `before` SHA from a force-pushed-away commit. The workflow now verifies that `before` exists locally before using it in neutral naming, Jira traceability, or Gitleaks ranges.

The first branch-protection validation attempt without an explicit token failed with GitHub `403` because the locally resolved token lacked administration read permission. The successful retry used the already authenticated GitHub CLI token without printing it.
