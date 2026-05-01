# Enterprise Onboarding Readiness Automation MVP

Updated: 2026-05-01

Ticket: `KAN-53`

## Purpose

KAN-52 created the Enterprise onboarding readiness report. KAN-53 operationalizes it as GitHub Actions evidence so the adoption state can be generated on demand or on a schedule.

The workflow is for GitGov operators and customer onboarding teams that want a repeatable artifact showing whether a customer adoption profile is ready, needs action, or blocked.

## Workflow

File:

```text
.github/workflows/enterprise-onboarding-readiness.yml
```

Triggers:

- manual `workflow_dispatch`.
- weekly schedule on Wednesday at `13:37 UTC`.

Artifact:

```text
enterprise-onboarding-readiness-{run_id}
```

Artifact contents:

```text
enterprise-onboarding-readiness.md
enterprise-onboarding-readiness.json
```

## Inputs

Manual inputs:

| Input | Default | Purpose |
|---|---:|---|
| `customer_name` | `GitGov` | Customer/tenant label in the report. |
| `repository_full_name` | current GitHub repository | Repository in the temporary adoption profile. |
| `default_branch` | `main` | Branch in the temporary adoption profile. |
| `jira_project_key` | `KAN` | Jira project key used for traceability examples. |
| `policy_preset` | `moderate` | Adoption preset: `audit-only`, `moderate`, or `strict`. |
| `providers` | `github,jira,jenkins,sonarqube` | Comma-separated provider ids. |
| `modules` | standard evidence modules | Comma-separated module ids. |
| `include_remote_workflow_readiness` | `false` | Runs KAN-51 read-only workflow readiness validation. |
| `report_only` | `true` | Keeps the workflow non-blocking when readiness is not fully ready. |

## Behavior

The workflow:

1. Builds a temporary adoption profile under the runner temp directory.
2. Generates an adoption pack from that profile.
3. Optionally generates workflow templates and runs the KAN-51 remote workflow readiness validator in `-ReportOnly` mode.
4. Runs the KAN-52 onboarding readiness report generator.
5. Uploads the Markdown/JSON evidence artifact.

## Safety Boundaries

The workflow:

- does not read `.env` files.
- does not print secret values.
- does not create GitHub Actions variables or secrets.
- does not mutate customer repositories.
- does not mutate provider settings.
- does not open PRs.
- does not dispatch customer workflows.
- does not change branch protection.
- does not make release blocking the default.

When `include_remote_workflow_readiness=true`, the workflow uses the GitHub run token only for read-only repository content/config-name checks.

## Non-Goals

- Direct provider credential validation; KAN-36 remains the explicit provider credential validator.
- GitHub Actions variable or secret creation.
- Remote workflow installation; KAN-50 remains the reviewed PR creation path.
- Release blocking enforcement.
- Direct GitHub App installation.

## Acceptance Criteria

- Workflow can run manually with safe defaults.
- Workflow uploads a parseable onboarding readiness artifact.
- Optional remote workflow readiness remains read-only and report-only.
- Scheduled run does not require provider secrets.
- Documentation explains that readiness evidence is non-mutating and release blocking stays opt-in.
