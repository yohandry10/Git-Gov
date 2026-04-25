# GitHub Actions Node 24 Compatibility Upgrade

Date: 2026-04-25

## Scope

This report records the GitHub Actions maintenance update applied after hosted CI emitted Node.js 20 action-runtime deprecation warnings.

The warning affected first-party `actions/*` actions, not the application build runtime configured through `node-version: 20`.

## Versions Verified

Latest official versions were checked through the GitHub API before the change:

- `actions/checkout`: `v6.0.2`
- `actions/setup-node`: `v6.4.0`
- `actions/upload-artifact`: `v7.0.1`
- `pnpm/action-setup`: `v5.0.0`

## Changes Applied

All workflow references were upgraded by major version:

- `actions/checkout@v4` -> `actions/checkout@v6`
- `actions/setup-node@v4` -> `actions/setup-node@v6`
- `actions/upload-artifact@v4` -> `actions/upload-artifact@v7`
- `pnpm/action-setup@v4` -> `pnpm/action-setup@v5`

No job logic, permissions, scripts, cache keys, or build commands were changed.

## Rationale

GitHub-hosted runs warned that Node.js 20 action runtimes are deprecated and will be forced to Node.js 24 by default. Moving first-party actions to their current major versions avoids relying on deprecated internal action runtimes.

`node-version: 20` remains in workflows where the repository intentionally builds Node applications on Node 20. That value controls project runtime/tooling, not the implementation runtime of `actions/*`.

## Validation Plan

- Local publication guard must pass.
- `git diff --check` must pass.
- GitHub `Workflow Lint` must pass on the PR.
- Required CI checks must pass before merge.

## Operational Note

If future GitHub warnings mention other third-party actions, update them separately after checking their current maintained major versions and migration notes.
