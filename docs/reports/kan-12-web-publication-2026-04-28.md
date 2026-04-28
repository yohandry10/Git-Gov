# KAN-12 Website Publication - 2026-04-28

## Outcome

The GitGov marketing/download website updates were published to `main` with Jira traceability restored.

The prior local-only commit `f2bdb24` with message `dle` was not pushed to GitHub. Instead, the web diff was recreated on a traced branch and merged through PR `#77` as commit `a0a4174`.

## What Changed

- Created Jira ticket `KAN-12` for the publication flow.
- Reapplied the local `gitgov-web` changes on branch `web/KAN-12-web-push`.
- Recommitted the changes as `web(KAN-12): publish marketing updates`.
- Opened PR `#77` and merged it to `main`.

## Validation

- Local publication guard passed on branch `web/KAN-12-web-push`.
- Local website checks passed:
  - `pnpm run lint`
  - `pnpm run typecheck`
  - `pnpm run build`
- PR checks passed after rerunning a transient `Workflow Lint` failure caused by `actionlint` download/extract failure.
- Post-merge checks on `main` passed:
  - `CI` run `24974947818`
  - `Release Readiness Gate` run `24974947816`

## Operator Note

If a local commit violates branch/commit/title traceability policy, do not push it from `main`. Recreate the diff on a Jira-tagged branch and merge through the normal PR path.
