# Policy-as-Code Flexible Source MVP

Updated: 2026-06-12

## Executive Decision

GitGov should support Policy-as-Code as a first-class governance capability, but it should not
force a single customer-facing file format.

The correct product shape is:

- One internal policy model: `GitGovConfig`.
- Multiple customer-facing policy formats: TOML, YAML, and JSON.
- Multiple source modes: Control Plane managed, repo Policy-as-Code, and hybrid advisory.
- Deterministic GitGov policy evaluation by default.
- Optional external OPA/Rego adapter for customers that explicitly need it.

In plain language: DevSecOps can ask for YAML/JSON in the repo and GitGov should support that, but
GitGov should still normalize every format into one policy model so Desktop, CI, Governance,
Evidence, drift, and release workflows all speak the same language.

## Why This Came Up

The current repository already has important policy pieces, but it does not yet fully implement the
specific idea: "a policy file committed in the repo is reviewed like code, merged, and then becomes
the active GitGov policy."

The gap list below is not a rejection of the feature. It is the exact implementation checklist for
making the feature real.

## What Exists Today

### Desktop Local Policy

Desktop can read a repository-local `gitgov.toml`:

```text
repo/gitgov.toml -> Tauri load_config -> GitGovConfig
```

Current local policy behavior includes:

- branch pattern validation.
- protected branch checks.
- group membership checks.
- allowed branch/path checks.
- basic commit message validation helpers.

This is the local workstation side of Policy-as-Code, but currently only TOML is supported locally.

### Control Plane Policy Store

The backend stores policy as normalized JSON in the `policies` table:

```text
GitGovConfig -> canonical-ish JSON -> checksum -> policies/history
```

Current backend policy capabilities include:

- `GET /policy/{repo_name}`.
- `PUT /policy/{repo_name}/override`.
- `GET /policy/{repo_name}/history`.
- `POST /policy/check`.
- checksum and history for stored policies.
- quality gate downgrade protection with explicit exception metadata.

This gives GitGov a central operational policy snapshot.

### Policy Review APIs

The backend already has a policy request workflow:

- create policy change request.
- list policy change requests.
- approve request.
- reject request.
- reject self-approval.
- store decisions append-only.

This is useful review infrastructure, but it is not yet connected to repository pull requests.

### Policy Drift Evidence

The backend has append-only drift evidence through:

```text
POST /policy/drift-events
GET /policy/drift-events
```

It can record actions such as:

- `sync_local`.
- `push_local`.
- `drift_snapshot`.

This is the right foundation for detecting that the active Control Plane policy and repo policy
file have diverged.

### Governance UI

Governance has policy editing surfaces that load and save policy through the Control Plane.

This is useful for `control-plane-managed` mode, but it becomes risky in pure repo Policy-as-Code
mode unless Save becomes "create a proposed file change" instead of "silently override DB policy."

## Original Gap List And Current Status

These were the points missing in the previous explanation. The MVP implementation now covers the
shared model/parser/checksum path, repo policy activation foundation, and optional external OPA
adapter. Remaining items are called out explicitly below.

### 1. YAML/JSON As Committed Policy Formats

Current documented repo policy format started as `gitgov.toml`.

Implemented:

- `.gitgov/policy.yml`.
- `.gitgov/policy.yaml`.
- `.gitgov/policy.json`.
- shared parser support in Desktop/backend/tooling.

Still pending:

- examples and schema docs for all supported formats.

### 2. Real Policy File Committed In This Repo

This repository documents `gitgov.toml` as the repo-local policy pattern, but there is no real
tracked `gitgov.toml` acting as this repo's active policy source.

That means the project demonstrates the concept in docs/code, but this repo is not itself dogfooding
repo Policy-as-Code yet.

### 3. Repo File As Control Plane Source Of Truth

Today, the Control Plane operationally uses the policy stored in the DB.

Missing:

```text
merged policy file blob -> parse -> normalize -> checksum -> activate policy in Control Plane
```

The MVP path now activates the exact merged policy blob from GitHub when the webhook has enough
context and the backend has a GitHub token. Remaining hardening is a controlled GitHub API
activation test and periodic drift comparison.

### 4. PR Review As The Required Policy Change Path

Today, admins can use `override_policy` directly.

That is acceptable for Control Plane managed mode, but it is not enough for DevSecOps Policy-as-Code
because policy changes should go through pull request review.

Implemented:

- PR check for policy file changes.
- schema validation on the PR.
- semantic diff on the PR.

Still pending:

- risky-change warning/blocking depending on customer policy.
- approval evidence tied to the merged PR.

### 5. Policy Change Requests Integrated With PRs

The backend policy request workflow exists, but it is not yet the same thing as:

```text
PR changes policy file -> GitGov validates -> reviewers approve -> merge activates policy
```

Missing:

- mapping policy request records to PR number/commit/blob.
- using the request workflow as review evidence for repo-file policy changes.
- automatic closure/activation when the PR merges.

### 6. OPA/Rego Runtime

GitGov does not embed OPA/Rego and should keep the native deterministic GitGov policy engine as the
default.

Implemented MVP shape: GitGov can call an external OPA Data API server when `adapters.opa.enabled`
is configured. The adapter supports advisory and required effects, fail-open/fail-closed behavior,
safe env-var based bearer tokens, customer result mapping, `allow` booleans, and common Rego
`deny` collections.

Remaining work: persisted OPA decision audit history/export and a real `opa run --server` smoke
script.

## Product Decision: Flexible But Not Fragmented

The user/customer should be able to choose how policy is stored:

| Choice | Meaning |
| --- | --- |
| TOML | Keep current `gitgov.toml` compatibility. Good for simple human-edited repo policy. |
| YAML | DevSecOps-friendly `.gitgov/policy.yml`; good for PR review and platform conventions. |
| JSON | Schema/tooling-friendly `.gitgov/policy.json`; good for generated config and validation. |

But the system should not implement three different policy semantics.

All formats should normalize into:

```text
GitGovConfig
```

Then every product surface uses the same config:

- Desktop local checks.
- `/policy/check`.
- Governance UI.
- Evidence Packets.
- drift reports.
- Action Center guidance.
- release governance where relevant.

## Source Modes

GitGov should support three source modes per repository.

| Mode | Source of truth | UI behavior | Intended customer |
| --- | --- | --- | --- |
| `control-plane-managed` | DB/UI/API policy snapshot | Save writes policy directly to Control Plane | Fast setup, central admin management |
| `repo-policy-as-code` | Committed policy file on activation branch | Save creates/proposes a file change, not a DB override | DevSecOps review-required workflows |
| `hybrid-advisory` | Repo file preferred, overrides allowed | Save marks override and creates drift/reconcile guidance | Migration, incident response, phased rollout |

Existing customers should default to `control-plane-managed` so the new feature does not break
current behavior.

New DevSecOps-led customers can choose `repo-policy-as-code`.

## File Discovery And Settings

Each repo should have explicit policy source settings:

- `source_mode`.
- `policy_path`.
- `policy_format`.
- `activation_branch`.
- `allow_emergency_override`.
- `require_policy_pr_check`.

If a repo has no explicit `policy_path`, discovery can use this order:

1. `.gitgov/policy.yml`
2. `.gitgov/policy.yaml`
3. `.gitgov/policy.json`
4. `gitgov.toml`

If more than one policy file exists, GitGov should return an ambiguity error unless migration mode
is explicitly enabled.

## Canonicalization

Every format must produce a stable canonical checksum:

```text
source file -> parse -> GitGovConfig -> canonical JSON -> SHA-256 checksum
```

Checksum rules:

- ignore whitespace.
- ignore key order.
- ignore YAML comments.
- ignore TOML formatting.
- include only normalized policy semantics.

The activated policy record should store:

- normalized config.
- canonical checksum.
- source mode.
- source path.
- source format.
- repo full name.
- activation branch/ref.
- commit SHA.
- blob SHA when available.
- PR number when available.
- actor/reviewer metadata when available.
- activation timestamp.

## PR Flow

For `repo-policy-as-code` mode:

1. A PR changes the configured policy file.
2. GitGov detects the policy-file change.
3. GitGov parses and validates the file.
4. GitGov computes a semantic diff.
5. GitGov reports changed enforcement levels, branches, groups, paths, traceability, quality gates,
   and risky downgrades.
6. GitGov can fail the PR check only when the customer explicitly enables blocking policy review.
7. After merge to the activation branch, GitGov reads the exact merged blob.
8. GitGov activates that exact normalized snapshot in the Control Plane.
9. `/policy/check` uses the activated snapshot.
10. Evidence records point back to commit, PR, checksum, source file, and reviewers.

Activation should be idempotent by:

```text
repo + source_path + commit_sha + checksum
```

## Manual Override Flow

Manual override behavior depends on source mode.

### Control Plane Managed

Current behavior is acceptable:

- UI/API can save policy directly.
- History/audit records the change.
- `/policy/check` uses the updated DB snapshot.

### Repo Policy-as-Code

Direct DB override should not be silent.

Allowed behavior:

- UI edits generate a file patch/proposal.
- Emergency override requires reason, ticket, actor, expiration, previous checksum, and source
  checksum.
- Active emergency override appears as drift.
- Action Center/Governance guides operator to reconcile by committing a policy-file change.

### Hybrid Advisory

Overrides are allowed but always labeled:

- `source = manual_override`.
- `repo_checksum = ...`.
- `active_checksum = ...`.
- `reconcile_required = true`.

## OPA/Rego External Adapter

OPA/Rego is not the default GitGov engine and is not embedded in the backend. GitGov can call an
external OPA server for customers that already operate Rego policies.

Why this shape:

- GitGov keeps one deterministic native `GitGovConfig` model for common governance.
- DevSecOps customers with existing Rego investments can connect OPA without forking GitGov policy
  semantics.
- OPA remains operationally owned by the customer through their sidecar/service, bundles, auth, and
  decision logs.

The optional config shape is:

```yaml
enforcement:
  external_policy: block

adapters:
  opa:
    enabled: true
    connection: default
    decision_path: /v1/data/gitgov/allow
    effect: required
    failure_mode: fail-closed
    timeout_ms: 1500
    input_profile: policy-check-v1
    token_env_var: OPA_AUTH_TOKEN
```

Rules:

- `adapters.opa.base_url` can be committed only when it contains no credentials; otherwise use
  `GITGOV_OPA_URL` or `GITGOV_OPA_<CONNECTION>_URL`.
- Remote OPA URLs must use `https://`; `http://` is accepted only for parsed loopback hosts such
  as `localhost`, `127.0.0.1`, other `127.0.0.0/8` loopback addresses, or `[::1]`.
- `token_env_var` is only an environment variable name. Secret values never belong in policy files.
- `effect: advisory` can only add warnings/evidence.
- `effect: required` plus `enforcement.external_policy: block` can block `/policy/check`.
- `failure_mode: fail-open` records a warning and continues; `fail-closed` denies when required.

The native GitGov path remains:

```text
policy file -> GitGovConfig -> GitGov decision -> evidence
```

## Implementation Plan

### Phase 1: Shared Policy Parser

- Add shared parser support for TOML/YAML/JSON.
- Add JSON schema for `GitGovConfig`.
- Add examples for all three formats.
- Add canonical checksum function.
- Add unit tests proving equivalent TOML/YAML/JSON produce the same normalized config/checksum.

### Phase 2: Policy Source Metadata

- Add repo/org source settings: mode, path, format, activation branch.
- Extend policy history with source metadata.
- Return source metadata from `GET /policy/{repo_name}`.
- Update Governance UI to display source mode, file path, checksum, and activation commit.

### Phase 3: PR Validation

- Add workflow/script to validate policy files in PRs.
- Output semantic diff.
- Flag risky downgrades such as `block -> warn/off`, removing protected branches, or reducing
  quality gate enforcement.
- Keep blocking opt-in by customer policy.

### Phase 4: Post-Merge Activation

- On merged PR/push to activation branch, read the exact policy file blob.
- Normalize and checksum the blob.
- Activate the Control Plane snapshot idempotently.
- Link activation to PR, commit, file path, blob SHA, actor/reviewers.

### Phase 5: UI And Override Behavior

- In `repo-policy-as-code` mode, Governance Save creates a file proposal/patch instead of direct DB
  override.
- Add explicit emergency override flow.
- Show drift/reconcile status in Governance and Action Center.

### Phase 6: Drift And Evidence

- Compare active Control Plane checksum with repo file checksum.
- Emit policy drift events when they differ.
- Include policy source/checksum in exports and Evidence Packets.
- Add Action Center next action for unresolved policy drift.

## What "Done" Means

This feature is done when:

- A customer can choose TOML, YAML, or JSON.
- All formats normalize to the same `GitGovConfig`.
- A PR policy-file change gets validation and semantic diff.
- A merge activates the exact policy file snapshot in the Control Plane.
- `/policy/check` and Desktop connected checks use the activated snapshot.
- Governance UI shows source mode/path/commit/checksum.
- UI overrides cannot silently bypass PR review in `repo-policy-as-code` mode.
- Drift is visible when repo policy and active Control Plane policy diverge.
- Existing Control Plane managed customers continue working unchanged.

## Non-Goals

- No embedded OPA/Rego runtime in the MVP.
- No automatic mutation of customer repos without explicit operator action.
- No branch protection mutation as part of policy activation.
- No secret values in policy files.
- No breaking change for current Control Plane managed policy behavior.

## Implementation Status - 2026-06-12

Initial implementation has started. The current code now includes:

- shared Rust policy core for TOML/YAML/JSON parsing, discovery, canonical JSON, and checksum.
- backend/Tauri use of the shared policy model.
- source metadata storage through `supabase_schema_v31.sql`.
- canonical checksum generation for overrides and policy change requests.
- PR policy validation CLI/script/workflow with semantic downgrade detection.
- merged PR webhook activation path when a GitHub token can fetch the exact merged policy blob.
- Governance UI source display.
- store-level guard that prevents silent direct overrides when the active policy source is
  `repo-policy-as-code`.

Remaining implementation work is tracked in
`docs/reports/policy-as-code-flexible-source-implementation-2026-06-12.md`.
