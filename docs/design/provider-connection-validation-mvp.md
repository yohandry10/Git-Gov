# Provider Connection Validation MVP

Updated: 2026-04-30

Ticket: `KAN-36`

## Goal

Add direct, secret-safe provider connection checks for Enterprise Self-Service onboarding.

KAN-32 shows provider health from evidence already ingested into GitGov. KAN-36 is different: it validates whether explicitly supplied provider credentials can reach the selected provider APIs during setup.

## Scope

Implemented in:

```text
scripts/control-plane/validate_enterprise_provider_connections.ps1
```

The validator reads an adoption profile and checks selected providers:

- GitHub.
- Jira.
- Jenkins.
- SonarQube.
- Render.
- Vercel.

It supports overrides for local validation:

- `-Providers`.
- `-RepositoryFullName`.
- `-JiraProjectKey`.
- `-JenkinsJobName`.
- `-SonarProjectKey`.

It can write a sanitized JSON report through `-OutputPath`.

## Status Model

Every provider returns one status:

- `ready`: required config exists and the provider API probe succeeded.
- `missing-config`: required environment variable or local auth source is missing.
- `failed`: config exists, but the provider API probe failed.

The overall report status is:

- `ready` when all selected providers are ready.
- `missing-config` when at least one selected provider lacks config and none failed.
- `failed` when at least one selected provider has config but cannot be reached or authenticated.

By default, the script exits non-zero unless the overall status is `ready`.

Use `-ReportOnly` to produce evidence without failing the process. This is useful during onboarding when a customer has not granted all credentials yet.

## Credential Sources

Credentials come from ignored local env files or process environment variables. The adoption profile stores only intent and names, not secret values.

Supported provider config:

- GitHub:
  - `GITHUB_TOKEN`, or
  - `GH_TOKEN`, or
  - authenticated `gh` CLI.
- Jira:
  - `JIRA_BASE_URL`.
  - `JIRA_EMAIL`.
  - `JIRA_API_TOKEN`.
  - profile/parameter/env Jira project key.
- Jenkins:
  - `JENKINS_SERVER_URL`.
  - `JENKINS_USER`.
  - `JENKINS_API_TOKEN`.
  - optional `JENKINS_JOB_NAME`.
- SonarQube:
  - `SONAR_HOST_URL`.
  - `SONAR_TOKEN`.
  - profile/parameter/env Sonar project key.
- Render:
  - `RENDER_API_KEY`.
- Vercel:
  - `VERCEL_TOKEN`.

## Safety Model

The validator:

- does not print token values.
- does not write token values.
- does not mutate provider state.
- does not mutate customer repositories.
- does not install webhooks.
- does not create GitHub Actions variables or secrets.
- does not create provider resources.

The report includes only sanitized status, provider names, selected config variable names, and non-secret probe metadata.

## Non-Goals

- No dashboard UI for direct credential entry.
- No webhook creation.
- No GitHub Actions secret/variable creation.
- No remote workflow installation.
- No formal enterprise release approval.
- No Vercel AI SDK Copilot.

## Next Product Steps

1. Add formal enterprise release approval with approvers, expiration, risk acceptance, and evidence binding.
2. Optionally add a dashboard wrapper that shows direct provider connection reports without storing credentials.
3. Start Vercel AI SDK Copilot after onboarding and approval evidence are complete enough to explain.
