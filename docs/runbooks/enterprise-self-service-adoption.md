# Enterprise Self-Service Adoption

Updated: 2026-05-01

Tickets: `KAN-29`, `KAN-30`, `KAN-31`, `KAN-32`, `KAN-33`, `KAN-34`, `KAN-35`, `KAN-36`, `KAN-50`, `KAN-51`, `KAN-52`, `KAN-53`

## Purpose

Use this runbook to generate the first GitGov adoption pack for a customer or internal demo tenant.

The adoption pack is a planning artifact. It lists what to connect, which workflows to install, what policy preset applies, and which evidence modules are expected.

It must not contain provider tokens or secret values.

## Example Profile

```text
docs/examples/enterprise-adoption-profile.example.json
```

## Generate A Pack

Run from the repository root:

```powershell
.\scripts\control-plane\generate_enterprise_adoption_pack.ps1 -ProfilePath docs/examples/enterprise-adoption-profile.example.json -OutputDir out/enterprise-adoption-pack
```

Expected outputs:

```text
out/enterprise-adoption-pack/enterprise-adoption-pack.md
out/enterprise-adoption-pack/enterprise-adoption-pack.json
```

## Generate Workflow Templates

Run from the repository root:

```powershell
.\scripts\control-plane\generate_enterprise_workflow_templates.ps1 -ProfilePath docs/examples/enterprise-adoption-profile.example.json -OutputDir out/enterprise-workflow-templates -Force
```

Expected outputs:

```text
out/enterprise-workflow-templates/README.md
out/enterprise-workflow-templates/workflow-template-manifest.json
out/enterprise-workflow-templates/.github/workflows/*.yml
```

The workflow template pack is an onboarding artifact. It is meant to be reviewed before copying files into a customer repository.

## Download Workflow Templates From Dashboard

In the GitGov Enterprise Adoption panel:

1. Load or edit the adoption profile.
2. Save the profile if it should persist for the org.
3. Use `Workflows` to download the workflow template pack JSON.

The dashboard pack contains a manifest, README text, and generated workflow file contents. It keeps the same safety boundary as the PowerShell generator.

It does not:

- install workflows automatically.
- mutate customer repositories.
- read local `.env` files.
- include provider token values.
- generate secret values.

## Install Workflow Templates With Review

Use this only after reviewing the generated workflow pack. The installer is dry-run by default and writes files only when `-Apply` is passed.

Install from the CLI-generated pack directory:

```powershell
.\scripts\control-plane\install_enterprise_workflow_templates.ps1 -PackDir out/enterprise-workflow-templates -TargetRepoPath C:\path\to\customer-repo -OutputPlanPath out/workflow-install-plan.json
```

Install from the dashboard JSON pack:

```powershell
.\scripts\control-plane\install_enterprise_workflow_templates.ps1 -PackPath C:\path\to\workflow-template-pack.json -TargetRepoPath C:\path\to\customer-repo -OutputPlanPath out/workflow-install-plan.json
```

The plan reports each workflow file as:

- `create`: new workflow file would be added.
- `update`: existing workflow file would be replaced, only when `-Overwrite` is also used.
- `skip`: existing workflow file already matches.
- `blocked`: existing workflow file differs and needs review before overwrite.

After reviewing the plan, apply it:

```powershell
.\scripts\control-plane\install_enterprise_workflow_templates.ps1 -PackDir out/enterprise-workflow-templates -TargetRepoPath C:\path\to\customer-repo -OutputPlanPath out/workflow-install-plan-apply.json -Apply
```

Use `-Overwrite` only after reviewing replacements:

```powershell
.\scripts\control-plane\install_enterprise_workflow_templates.ps1 -PackDir out/enterprise-workflow-templates -TargetRepoPath C:\path\to\customer-repo -OutputPlanPath out/workflow-install-plan-overwrite.json -Apply -Overwrite
```

Safety boundaries:

- target path must be a git checkout with a `.git` marker.
- writes are limited to `.github/workflows/*.yml` and `.github/workflows/*.yaml`.
- unsafe paths such as `..`, rooted paths, drive-qualified paths, nested workflow paths, and non-YAML files are rejected.
- the installer does not read `.env` files, provider tokens, or secret values.
- the installer does not call GitHub APIs or mutate remote repositories.

## Open A Remote Workflow Installation PR

KAN-50 adds the remote PR path for customers who want GitGov to prepare the workflow installation change directly in GitHub.

Dry-run first:

```powershell
.\scripts\control-plane\open_enterprise_workflow_template_pr.ps1 `
  -PackDir out/enterprise-workflow-templates `
  -Repository owner/repo `
  -TicketId EX-123 `
  -OutputPlanPath out/remote-workflow-pr-plan.json
```

Open a draft PR after reviewing the plan:

```powershell
.\scripts\control-plane\open_enterprise_workflow_template_pr.ps1 `
  -PackDir out/enterprise-workflow-templates `
  -Repository owner/repo `
  -TicketId EX-123 `
  -Apply `
  -OutputPlanPath out/remote-workflow-pr-apply.json
```

Use `-Overwrite` only after reviewing differing existing workflow files:

```powershell
.\scripts\control-plane\open_enterprise_workflow_template_pr.ps1 `
  -PackDir out/enterprise-workflow-templates `
  -Repository owner/repo `
  -TicketId EX-123 `
  -Apply `
  -Overwrite `
  -OutputPlanPath out/remote-workflow-pr-overwrite.json
```

Use `-ReadyForReview` only when the PR should be opened as a normal non-draft PR.

Remote PR safety boundaries:

- dry-run is the default.
- remote branch, commit, and PR creation require `-Apply`.
- PRs are draft by default.
- writes are limited to workflow files directly under `.github/workflows`.
- existing differing workflow files are `blocked` unless `-Overwrite` is passed.
- the script does not create GitHub Actions variables or secrets.
- the script does not modify branch protection or required checks.
- the script does not merge the PR.
- token values are never printed.

## Validate Remote Workflow Readiness

KAN-51 adds a read-only validator for checking whether the target GitHub repository has the expected workflow files and required GitHub Actions configuration names after local install or remote PR merge.

Run in report-only mode first:

```powershell
.\scripts\control-plane\validate_enterprise_workflow_installation_readiness.ps1 `
  -PackDir out/enterprise-workflow-templates `
  -Repository owner/repo `
  -Ref main `
  -ReportOnly `
  -OutputPath out/workflow-readiness.json
```

Use the dashboard pack JSON if the templates came from the UI:

```powershell
.\scripts\control-plane\validate_enterprise_workflow_installation_readiness.ps1 `
  -PackPath C:\path\to\workflow-template-pack.json `
  -Repository owner/repo `
  -Ref main `
  -ReportOnly `
  -OutputPath out/workflow-readiness.json
```

Remove `-ReportOnly` only when the validation should fail the calling process if anything is missing or different.

Status values:

- `ready`: all workflows match and all required variable/secret names are present.
- `needs-action`: at least one workflow is missing/different, or a required variable/secret name is missing.

Readiness safety boundaries:

- read-only GitHub API calls only.
- no `.env` reads.
- no secret values are read; GitHub Actions secrets are checked by name only.
- no GitHub Actions variables or secrets are created.
- no branch, file, PR, branch protection, workflow dispatch, or provider mutation.

## Generate Enterprise Onboarding Readiness

KAN-52 consolidates the adoption profile, provider connection report, workflow template plan, remote workflow readiness, GitHub Actions configuration names, and release governance posture into one Markdown/JSON readiness report.

Generate a non-blocking report from only the profile:

```powershell
.\scripts\control-plane\generate_enterprise_onboarding_readiness_report.ps1 `
  -ProfilePath docs/examples/enterprise-adoption-profile.example.json `
  -OutputDir out/enterprise-onboarding-readiness `
  -ReportOnly
```

Generate a fuller report after running provider and workflow readiness checks:

```powershell
.\scripts\control-plane\generate_enterprise_onboarding_readiness_report.ps1 `
  -AdoptionPackPath out/enterprise-adoption-pack/enterprise-adoption-pack.json `
  -ProviderConnectionsPath out/provider-connections-report-only.json `
  -WorkflowReadinessPath out/workflow-readiness.json `
  -OutputDir out/enterprise-onboarding-readiness `
  -ReportOnly
```

Expected outputs:

```text
out/enterprise-onboarding-readiness/enterprise-onboarding-readiness.md
out/enterprise-onboarding-readiness/enterprise-onboarding-readiness.json
```

Status values:

- `ready`: all readiness stages are ready.
- `needs-action`: at least one stage needs configuration, evidence, or validation.
- `blocked`: the profile is invalid or internally inconsistent.

Remove `-ReportOnly` only when the caller should fail if onboarding is not fully ready.

Onboarding readiness safety boundaries:

- no secret values are read or printed.
- no GitHub Actions variables or secrets are created.
- no customer repository, provider, branch protection, or workflow dispatch mutation is performed.
- release blocking remains customer opt-in only; record-only remains the safe default.

## Automate Onboarding Readiness Evidence

KAN-53 adds a GitHub Actions workflow that generates the KAN-52 readiness report as a reusable evidence artifact.

Workflow:

```text
.github/workflows/enterprise-onboarding-readiness.yml
```

Manual run defaults:

- customer name: `GitGov`.
- repository: current GitHub repository.
- branch: `main`.
- Jira key: `KAN`.
- policy preset: `moderate`.
- report-only: enabled.
- remote workflow readiness: disabled unless explicitly selected.

The workflow uploads:

```text
enterprise-onboarding-readiness-{run_id}
```

Use `include_remote_workflow_readiness=true` only when the workflow should also run the KAN-51 read-only repository comparison with the GitHub Actions run token.

Automation safety boundaries:

- scheduled/default runs do not require provider secrets.
- no `.env` files are read.
- no secret values are printed.
- no GitHub Actions variables or secrets are created.
- no branch, PR, provider, workflow dispatch, or branch-protection mutation occurs.
- not-ready output is non-blocking by default through `report_only=true`.

## Monitor Onboarding Readiness Artifacts

KAN-54 adds a GitHub Actions monitor that validates the latest KAN-53 onboarding readiness artifact is still present, fresh, and not expired.

Workflow:

```text
.github/workflows/enterprise-onboarding-readiness-artifact-monitor.yml
```

Manual run input:

- `max_age_hours`: maximum accepted artifact age. Default is `192`.

The monitor checks for artifacts with this prefix:

```text
enterprise-onboarding-readiness-
```

It uploads:

```text
enterprise-onboarding-readiness-artifact-monitor
```

Run the same check locally when GitHub API access is available:

```powershell
$token = & C:\Users\PC\Tools\gh\bin\gh.exe auth token
.\scripts\control-plane\validate_github_evidence_report_artifact.ps1 `
  -Repository yohandry10/Git-Gov `
  -WorkflowFile enterprise-onboarding-readiness.yml `
  -ArtifactNamePrefix enterprise-onboarding-readiness- `
  -MaxAgeHours 192 `
  -GitHubToken $token `
  -OutputPath out/enterprise-onboarding-readiness-artifact-monitor.json
```

Monitor safety boundaries:

- no `.env` files are read.
- no provider secret values are read or printed.
- GitHub artifact metadata is read only through the GitHub token available to the caller.
- no GitHub Actions variables or secrets are created.
- no customer repository, provider, branch protection, or workflow dispatch mutation is performed.
- this monitor validates artifact freshness only; it does not fail because onboarding readiness is `needs-action`.

## Trend Onboarding Readiness Artifacts

KAN-55 adds a GitHub Actions trend report for recent KAN-53 onboarding readiness artifacts.

Workflow:

```text
.github/workflows/enterprise-onboarding-readiness-trend-report.yml
```

Manual run input:

- `max_reports`: maximum parseable onboarding readiness artifacts to include. Default is `12`.

The trend reads artifacts with this prefix:

```text
enterprise-onboarding-readiness-
```

It uploads:

```text
enterprise-onboarding-readiness-trend-report
```

Run the same trend locally when GitHub API access is available:

```powershell
$token = & C:\Users\PC\Tools\gh\bin\gh.exe auth token
.\scripts\control-plane\generate_enterprise_onboarding_readiness_trend_report.ps1 `
  -Repository yohandry10/Git-Gov `
  -WorkflowFile enterprise-onboarding-readiness.yml `
  -ArtifactNamePrefix enterprise-onboarding-readiness- `
  -MaxReports 12 `
  -GitHubToken $token `
  -OutputMarkdownPath out/enterprise-onboarding-readiness-trend-report.md `
  -OutputJsonPath out/enterprise-onboarding-readiness-trend-report.json
```

Trend safety boundaries:

- no `.env` files are read.
- no provider secret values are read or printed.
- only GitHub Actions artifact metadata and sanitized readiness JSON are read.
- no GitHub Actions variables or secrets are created.
- no customer repository, provider, branch protection, or workflow dispatch mutation is performed.
- this trend reports whether readiness is improving, declining, or stable; it does not make `needs-action` release-blocking by default.

## Monitor Onboarding Readiness Trend

KAN-56 adds a GitHub Actions monitor for the KAN-55 trend artifact.

Workflow:

```text
.github/workflows/enterprise-onboarding-readiness-trend-monitor.yml
```

Manual run inputs:

- `max_age_hours`: maximum accepted trend artifact age. Default is `192`.
- `min_latest_score`: minimum accepted latest onboarding readiness score. Default is `75`.
- `report_only`: keep the monitor non-blocking. Default is `true`.

The monitor reads this trend artifact:

```text
enterprise-onboarding-readiness-trend-report
```

It uploads:

```text
enterprise-onboarding-readiness-trend-monitor
```

Run the same monitor locally when GitHub API access is available:

```powershell
$token = & C:\Users\PC\Tools\gh\bin\gh.exe auth token
.\scripts\control-plane\validate_enterprise_onboarding_readiness_trend_monitor.ps1 `
  -Repository yohandry10/Git-Gov `
  -WorkflowFile enterprise-onboarding-readiness-trend-report.yml `
  -ArtifactName enterprise-onboarding-readiness-trend-report `
  -MaxAgeHours 192 `
  -MinLatestScore 75 `
  -GitHubToken $token `
  -ReportOnly `
  -OutputMarkdownPath out/enterprise-onboarding-readiness-trend-monitor.md `
  -OutputJsonPath out/enterprise-onboarding-readiness-trend-monitor.json
```

Monitor status values:

- `ready`: the trend artifact is fresh and no deterioration rule fired.
- `needs-action`: the trend artifact is parseable, but onboarding readiness changed in a way an operator should review.
- `blocked`: the trend artifact is missing, stale, expired, or not parseable.

Trend monitor safety boundaries:

- no `.env` files are read.
- no provider secret values are read or printed.
- only GitHub Actions artifact metadata and sanitized trend JSON are read.
- no GitHub Actions variables or secrets are created.
- no customer repository, provider, branch protection, or workflow dispatch mutation is performed.
- `report_only=true` is the default; remove it only when an operator intentionally wants the monitor to fail the calling workflow.

## Validate Direct Provider Connections

Use this only when the customer or local operator has explicitly provided provider credentials through ignored env files or process environment variables.

Run a strict validation for selected providers:

```powershell
.\scripts\control-plane\validate_enterprise_provider_connections.ps1 -Providers github,jira -RepositoryFullName owner/repo -JiraProjectKey EX -OutputPath out/provider-connections.json
```

Run a non-blocking onboarding report for the current profile:

```powershell
.\scripts\control-plane\validate_enterprise_provider_connections.ps1 -ProfilePath docs/examples/enterprise-adoption-profile.example.json -ReportOnly -OutputPath out/provider-connections-report-only.json
```

Status values:

- `ready`: config exists and the provider API probe succeeded.
- `missing-config`: required variable or auth source is missing.
- `failed`: config exists, but the provider probe failed.

Provider config names:

- GitHub: `GITHUB_TOKEN`, `GH_TOKEN`, or authenticated `gh` CLI.
- Jira: `JIRA_BASE_URL`, `JIRA_EMAIL`, `JIRA_API_TOKEN`, and Jira project key.
- Jenkins: `JENKINS_SERVER_URL`, `JENKINS_USER`, `JENKINS_API_TOKEN`, and optional `JENKINS_JOB_NAME`.
- SonarQube: `SONAR_HOST_URL`, `SONAR_TOKEN`, and Sonar project key.
- Render: `RENDER_API_KEY`.
- Vercel: `VERCEL_TOKEN`.

Safety boundaries:

- the validator reads provider credentials only from ignored env files or the process environment.
- the validator reports config names, never secret values.
- the validator does not mutate provider settings.
- the validator does not create webhooks.
- the validator does not create GitHub Actions variables or secrets.

## Policy Presets

`audit-only`:

- gathers evidence.
- avoids release blocking.

`moderate`:

- requires ticket traceability.
- requires fresh evidence artifacts.
- blocks reachable critical/high vulnerabilities.
- targets release readiness score `75`.

`strict`:

- requires ticket traceability.
- requires PR review evidence.
- requires fresh evidence artifacts.
- blocks reachable critical/high vulnerabilities.
- requires medium-risk acceptance.
- targets release readiness score `85`.
- enables vulnerability trend enforcement.

## Safe Handling

- Use placeholder examples in reusable docs.
- Store provider tokens only in customer secret stores or GitHub Actions secrets.
- Do not paste `.env` values into adoption profiles.
- Treat generated packs as customer-specific planning evidence, not as a secret store.
- Treat generated workflow templates as customer-specific installation candidates, not as automatically approved production CI.
