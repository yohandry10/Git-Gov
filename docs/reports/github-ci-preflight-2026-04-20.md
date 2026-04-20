# GitHub CI Preflight Report

Date: 2026-04-20
Repository: yohandry10/Git-Gov

## Scope

Validate what remains for cloud CI closure (Sonar + GitGov telemetry) without leaking secrets.

## Token Permission Diagnostic

Command:

`powershell
powershell -ExecutionPolicy Bypass -File scripts/github/check_token_permissions.ps1 -Branch main -EmitJson -NoFailOnForbidden -Quiet
`

Result:

`json
{
    "owner":  "yohandry10",
    "repo":  "Git-Gov",
    "branch":  "main",
    "forbidden_count":  3,
    "forbidden_endpoints":  [
                                "Actions secrets",
                                "Actions variables",
                                "Branch protection"
                            ],
    "results":  [
                    {
                        "Label":  "Repo metadata",
                        "Status":  200,
                        "AcceptedPermissions":  "metadata=read",
                        "Result":  "OK"
                    },
                    {
                        "Label":  "Actions secrets",
                        "Status":  403,
                        "AcceptedPermissions":  "secrets=read",
                        "Result":  "FORBIDDEN"
                    },
                    {
                        "Label":  "Actions variables",
                        "Status":  403,
                        "AcceptedPermissions":  "actions_variables=read",
                        "Result":  "FORBIDDEN"
                    },
                    {
                        "Label":  "Branch protection",
                        "Status":  403,
                        "AcceptedPermissions":  "administration=read",
                        "Result":  "FORBIDDEN"
                    }
                ]
}

`

Interpretation:

- The active PAT can read repo metadata.
- The active PAT cannot currently read Actions secrets, Actions variables, or branch protection.
- Required permission hints from GitHub headers:
  - secrets=read
  - ctions_variables=read
  - dministration=read

## CI Config Visibility Diagnostic

Command:

`powershell
powershell -ExecutionPolicy Bypass -File scripts/github/check_ci_repo_config.ps1 -AllowMissingSonar -NoFailOnForbidden
`

Result:

`	ext
ADVERTENCIA: Skipping secrets visibility check due to token permission limits (403). Use a token with Actions secrets 
read access for strict validation.
ADVERTENCIA: Skipping variables visibility check due to token permission limits (403). Use a token with Actions 
variables read access for strict validation.
Repository: yohandry10/Git-Gov

Required secrets:
  [UNKNOWN] Skipped (token cannot read Actions secrets).

Optional secrets:
  [UNKNOWN] Skipped (token cannot read Actions secrets).

Required variables:
  [UNKNOWN] Skipped (token cannot read Actions variables).

Optional variables:
  [UNKNOWN] Skipped (token cannot read Actions variables).

PASS (best-effort): required validation completed with limited token visibility on Actions config.

`

Interpretation:

- Best-effort check completes, but strict validation is blocked by PAT scope.
- Until token scope is expanded, cloud-side secret/variable status remains UNKNOWN from automation.

## Next Action

1. Expand PAT permissions (secrets=read, ctions_variables=read, dministration=read).
2. Re-run strict checks (without -NoFailOnForbidden) to get PASS/FAIL for real config.
3. Complete GitHub-hosted quality gate matrix (quality_gates=warn and quality_gates=block) once strict check passes.
