# KAN-86 Environment Policy UX MVP

Updated: 2026-06-14

## Summary

KAN-86 makes release governance policy by environment reviewable and editable from Desktop/admin.

The underlying profile shape already supported `release_governance.environment_overrides`. This slice adds a clearer Environment Policy Matrix in the Enterprise Adoption surface and moves the edit logic into tested helpers so a customer can keep staging/base policy non-blocking while making production stricter.

## Product Decision

- `record-only` remains the default.
- Blocking is still an explicit customer policy choice.
- Environment overrides are allowed so production can be `approval-required` or `quorum-required` while staging remains `record-only`.
- Changing the base release governance mode must not delete existing environment overrides.
- Removing an environment override makes that environment fall back to the base policy.

## UX Shape

The Desktop/admin release governance section now shows:

- base environment policy and enforcement;
- override rows in the same matrix;
- editable base mode and base environment;
- add/remove override controls;
- per-override environment and mode controls;
- quorum summary for quorum-required policies.

## Validation Contract

The tests cover real policy behavior:

- production can be stricter than staging without making the base policy blocking;
- base policy mode changes preserve production overrides;
- removing an override falls back to the base policy;
- override mode changes to `quorum-required` produce concrete approver rules;
- the rendered component shows base and override rows and emits concrete edit intents.

## Non-Goals

- No database migration.
- No provider API mutation.
- No automatic branch protection or deployment provider changes.
- No default blocking behavior.
- No secret storage or secret printing.
