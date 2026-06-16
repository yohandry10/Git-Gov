# First Governed Repo Setup MVP

Updated: 2026-06-16

Ticket: `KAN-80`; KAN-120 continuity note

## Product Decision

`KAN-80` is the first implementation slice under roadmap item `0.1 Deployment Gates`.

The product goal is not a broad integration wizard. The goal is to make one customer repository ready
for an explainable, advisory deployment-gate simulation:

```text
repo selected -> policy/workflow preview reviewed -> baseline persisted -> gaps visible -> gate simulation CTA
```

This makes Deployment Gates demonstrable without pretending that GitGov has completed OAuth,
marketplace connectors, bulk onboarding, hard blocking deploy enforcement, or a deployment-provider
matrix.

## User Outcome

An Admin can open `Governance > Adoption` and prepare a first governed repo by defining:

- target repository in `owner/repo` form.
- default branch.
- onboarding goal.
- policy preset.
- selected evidence providers.
- selected governance modules.
- acknowledgement that the policy/workflow preview was reviewed.

GitGov persists this as one active setup run per organization. Re-saving updates the same `run_id`
instead of creating a new run, so the setup remains idempotent and auditable.

## Backend Contract

Route:

```text
GET /enterprise/first-governed-repo-setup
PUT /enterprise/first-governed-repo-setup
```

Both routes are Admin-only and use the same org scoping model as the existing enterprise adoption
routes. Scoped org keys can omit `org_name`; global Admin keys must pass `org_name`.

Persisted table:

```text
enterprise_first_governed_repo_setups
```

The table stores:

- `org_id` as primary key.
- stable `run_id`.
- setup `status`.
- setup `goal`.
- `repository_full_name`.
- `default_branch`.
- `selected_providers`.
- `selected_modules`.
- `policy_preset`.
- normalized `baseline` JSON.
- `created_by`, `updated_by`, timestamps, and optional `completed_at`.

The backend validates and normalizes the baseline before save. It rejects:

- malformed repository names.
- missing GitHub provider.
- unsupported providers/modules/goals/presets/statuses.
- baseline JSON larger than 24 KiB.
- baseline values or keys that look like secrets.
- `completed` status unless the baseline is ready.

The backend adds deterministic baseline fields:

- `version`.
- `gate_readiness`.
- `setup_summary`.
- `action_center_gaps`.
- `first_result`.

Admin audit log action:

```text
upsert_first_governed_repo_setup
```

Audit metadata records org, run, status, goal, repo, branch, preset, counts, and gate readiness. It
does not record tokens or raw provider secrets.

## Desktop Contract

Tauri commands:

```text
cmd_server_get_first_governed_repo_setup
cmd_server_upsert_first_governed_repo_setup
```

Zustand state keeps:

- `firstGovernedRepoSetup`.
- `firstGovernedRepoSetupUpdatedAt`.
- loading/saving flags.
- API error string.

The UI panel is mounted above the existing Enterprise Adoption panel under:

```text
Governance > Adoption
```

The panel shows readiness, unsaved state, run id, providers/modules, Action Center gaps, and a CTA
to `Governance > Releases` for advisory gate simulation.

## KAN-120 Continuity

`KAN-120` keeps this KAN-80 table and baseline shape as the canonical source. It adds an
orchestration layer rather than a new data model:

```text
GET /onboarding/first-governed-repo/state
POST /onboarding/first-governed-repo/runs
PATCH /onboarding/first-governed-repo/runs/{run_id}
POST /onboarding/first-governed-repo/runs/{run_id}/validate
POST /onboarding/first-governed-repo/runs/{run_id}/plan
POST /onboarding/first-governed-repo/runs/{run_id}/complete
```

The continuity rule is important: KAN-120 is a manual-first Integration Wizard for state,
evidence validation, baseline planning, and first-result completion. It is not provider OAuth, not
repository mutation, not deploy execution, not a compliance/certification claim, and not an Agent
Governance or AI dependency.

## Current Scope

Implemented now:

- one setup run per org.
- repository selection.
- provider/module/policy preset selection.
- backend baseline normalization.
- secret-safe payload validation.
- Admin-only save/load.
- idempotent upsert preserving `run_id`.
- Desktop commands and store integration.
- focused UI panel.
- backend unit tests, backend integration test, and frontend helper tests.

Not implemented in `KAN-80`:

- Slack.
- universal OAuth.
- marketplace connector installation.
- multi-repo bulk onboarding.
- hard deployment blocking.
- deployment-provider matrix.
- OPA/Rego execution.
- advanced risk scoring.
- regulatory framework mapping.

Those remain separate roadmap items or existing primitives, not hidden behavior in this setup MVP.
