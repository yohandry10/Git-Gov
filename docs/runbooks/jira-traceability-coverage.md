# Jira Traceability Coverage Runbook

Date: 2026-04-28

## Purpose

Operate Jira traceability coverage separately from the full release readiness gate.

Release readiness combines pipeline, Sonar, and Jira signals. This runbook focuses only on the Jira ticket coverage path:

- branch names
- commit messages
- pull request titles
- pull request comments
- merged PR evidence materialized by GitHub webhooks

## Local Preflight

Before committing or opening a PR:

```powershell
.\scripts\security\publication_guard.ps1
```

For a direct traceability-only check:

```powershell
.\scripts\github\check_traceability_policy.ps1
```

## Production Coverage Validation

Read-only coverage check:

```powershell
.\scripts\control-plane\validate_jira_traceability_coverage.ps1
```

Refresh PR/Jira correlations before measuring:

```powershell
.\scripts\control-plane\validate_jira_traceability_coverage.ps1 -RefreshCorrelations
```

Require a minimum coverage threshold:

```powershell
.\scripts\control-plane\validate_jira_traceability_coverage.ps1 -RefreshCorrelations -MinCoverage 50
```

Persist JSON evidence:

```powershell
.\scripts\control-plane\validate_jira_traceability_coverage.ps1 -RefreshCorrelations -OutputPath .\docs\reports\local\jira-traceability-coverage.json
```

The script uses ignored local env files and does not print API keys.

## Operating Rules

- Every branch should include a Jira ID such as `KAN-19`.
- Every PR title should include the same Jira ID.
- Every commit subject should include a Jira ID.
- PR comments that establish operational evidence should include the Jira ID when relevant.
- Do not push non-traceable commits from `main`; recreate the diff on a Jira-tagged branch.

## Interpretation

Coverage can improve when:

- new commits include Jira IDs
- PR titles include Jira IDs
- PR comment/title correlations are refreshed
- GitHub webhook delivery materializes merged PR evidence

Low coverage is not automatically a pipeline outage. Treat it as an operational traceability gap unless release readiness also falls below target.

## Related Commands

Full provider validation:

```powershell
.\scripts\control-plane\validate_provider_access.ps1 -IncludeReleaseReadiness
```

Full release readiness:

```powershell
.\scripts\jenkins\validate_release_readiness_gate.ps1 -GitGovUrl https://gitgov-api.onrender.com -ApiKey $env:GITGOV_API_KEY -RepoFullName yohandry10/Git-Gov -Branch main -Tier standard -Hours 720 -MinReadiness 75
```
