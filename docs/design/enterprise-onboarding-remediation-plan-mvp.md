# Enterprise Onboarding Remediation Plan MVP

Updated: 2026-05-02

Ticket: `KAN-57`

## Purpose

KAN-57 turns Enterprise Onboarding Readiness output into a concrete remediation plan.

KAN-52 tells the customer whether onboarding is ready. KAN-53 through KAN-56 automate, monitor, trend, and watch that readiness evidence. KAN-57 answers the next operator question: what should we do next, who owns it, and how do we validate that the action worked?

## Script

```text
scripts/control-plane/generate_enterprise_onboarding_remediation_plan.ps1
```

## Inputs

| Input | Default | Purpose |
| --- | --- | --- |
| `ReadinessPath` | `out/enterprise-onboarding-readiness/enterprise-onboarding-readiness.json` | KAN-52 readiness JSON. |
| `AdoptionPackPath` | inferred from readiness when available | Optional adoption pack JSON for variable/secret names. |
| `OutputDir` | `out/enterprise-onboarding-remediation-plan` | Output directory. |
| `FailOnBlocked` | disabled | Optional strict mode for blocked remediation states. |

## Outputs

```text
enterprise-onboarding-remediation-plan.md
enterprise-onboarding-remediation-plan.json
```

The plan includes:

- customer/repository context.
- current readiness status and score.
- remediation status.
- prioritized action items.
- suggested owner per action.
- validation evidence required per action.
- GitHub Actions variable/secret setup commands with placeholders only.
- safety flags proving the plan is non-mutating and does not contain secret values.

## Priority Model

| Priority | Stage | Reason |
| ---: | --- | --- |
| `0` | Any `blocked` stage | Fix profile or consistency blockers first. |
| `1` | Adoption profile | A valid profile is the base for every generated artifact. |
| `2` | Provider connections | Provider reachability determines evidence quality. |
| `3` | Workflow template pack | Templates define what should be installed. |
| `4` | Remote workflow readiness | Installed workflows must match reviewed templates. |
| `5` | GitHub Actions configuration | Required variable/secret names must exist before workflows can run. |
| `6` | Release governance policy | Confirm policy is intentional, especially when stricter than record-only. |

## Safety Boundaries

The remediation plan generator:

- reads readiness JSON.
- optionally reads adoption pack JSON for configuration names only.
- does not read `.env` files.
- does not read provider tokens.
- does not print secret values.
- does not create GitHub Actions variables or secrets.
- does not mutate customer repositories.
- does not mutate provider settings.
- does not dispatch workflows.
- does not alter branch protection.
- does not make onboarding readiness or release governance blocking by default.

## Non-Goals

- creating variables or secrets automatically.
- storing customer secret values.
- changing branch protection.
- opening remote PRs.
- replacing provider validation.
- replacing workflow readiness validation.
- making release governance blocking by default.

## Acceptance Criteria

- Script produces Markdown and JSON from KAN-52 readiness JSON.
- Script can infer the adoption pack path from readiness output when available.
- Script emits placeholder-only `gh variable set` and `gh secret set` commands.
- Script output contains no secret values.
- Documentation explains that the plan is advisory/non-mutating by default.
