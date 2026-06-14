# KAN-85 Deployment Gate Provider Examples MVP

Updated: 2026-06-14

## Summary

KAN-85 adds provider-specific examples for calling GitGov Deployment Gates from common CI/CD systems.

Examples live under:

```text
docs/examples/deployment-gates/
```

Included providers:

- GitHub Actions
- Jenkins Pipeline
- GitLab CI

Each example calls:

```text
POST /deployment-gates/authorize
```

## Product Intent

KAN-83 created the stable Deployment Gates API and KAN-84 connected generated workflow templates to it.
KAN-85 makes the integration concrete for customer delivery teams that do not use GitGov-generated
workflow packs directly.

The examples are intentionally copy-review-install artifacts. GitGov does not mutate customer CI/CD
providers in this slice.

## Required Contract

Every provider example must send:

- `release_id`
- `repository_full_name`
- `branch`
- `target_sha`
- `environment`
- `deployer`
- `evidence_packet_hash`
- `requested_by`
- `deployment_run_id`
- `metadata`

The evidence hash must be for a release-bound Evidence Packet that matches the release id, repository,
branch, target SHA, and environment.

## Safety

- No provider token is committed.
- `GITGOV_API_KEY` is read from the provider secret store.
- Authorization responses are saved as evidence artifacts.
- `blocking=true` fails deployment.
- `would_block=true` fails only when the provider job explicitly opts into advisory failure.
- Missing GitGov config or missing evidence hash skips only when enforcement is disabled.

## Validation

`scripts/control-plane/validate_deployment_gate_provider_examples.ps1` checks that examples:

- exist;
- call `/deployment-gates/authorize`;
- do not call `/enterprise/release-governance/evaluate`;
- include required request fields;
- preserve evidence artifacts;
- handle `blocking` and `would_block`;
- do not hardcode bearer tokens or API key values.

## Non-Goals

- No GitHub/Jenkins/GitLab mutation.
- No automatic secret creation.
- No branch protection updates.
- No automatic break-glass request generation in provider examples; KAN-88 documents the required prior approval route.
- No OPA/Rego execution.
