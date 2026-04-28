# KAN-16 Provider Access Validation

Date: 2026-04-28

## Purpose

Reduce repeated manual validation work by adding one secret-safe command that checks the external and local services the agent uses.

## Script

```powershell
.\scripts\control-plane\validate_provider_access.ps1 -IncludeReleaseReadiness
```

The script loads ignored local env files by default:

- `gitgov\.env`
- `gitgov\gitgov-server\.env`

It does not print token values.

## Coverage

The validator checks:

- GitGov production `/health`
- GitGov production authenticated `/stats`
- Local GitGov backend `/health` on `http://127.0.0.1:3001`
- Local SonarQube system status and project quality gate
- Local Jenkins identity and last job build
- Jira project metadata
- Optional release readiness gate

## Latest Validation

Latest local run with `-IncludeReleaseReadiness` returned all checks `ok`.

| Area | Result |
|---|---|
| GitGov production | `/health` `ok`; `/stats` HTTP `200` |
| GitGov local | `/health` `ok`, version `0.1.0` |
| SonarQube | System `UP`, project `yohandry10_git-gov`, quality gate `OK` |
| Jenkins | User `admin`, job `gitgov-demo-pipeline`, build `#30` `SUCCESS` |
| Jira | Project `KAN`, name `GitGov`, ID `10000` |
| Release readiness | `92/100`, target `75`, signal coverage `3/3` |
| Pipeline signal | `98.81%` success |
| Jira coverage | `69.88%` |
| Sonar signal | `98.81%` pass |

## Operational Notes

- Use this script before changing GitHub Actions, Jenkins, Sonar, Jira, or Render integration code.
- Use `-SkipSonar`, `-SkipJenkins`, `-SkipJira`, or `-SkipLocalGitGov` when a local service is intentionally offline.
- Use `-OutputPath <path>` to persist JSON evidence for reports or CI artifacts.
