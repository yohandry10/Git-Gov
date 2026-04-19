param(
  [string]$Owner = "yohandry10",
  [string]$Repo = "Git-Gov",
  [string]$Branch = "main",
  [string]$GitHubToken = "",
  # Continue with best-effort checks when token lacks admin permissions.
  [switch]$BestEffort,
  [switch]$SkipTokenPermissionsCheck,
  [switch]$ApplyBranchProtection,
  [switch]$SkipCiConfigCheck,
  [switch]$AllowMissingSonar,
  [switch]$RequireGitGovTelemetry,
  [int]$RequiredApprovals = 1
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$checkTokenPermissionsScript = Join-Path $scriptRoot "check_token_permissions.ps1"
$checkCiScript = Join-Path $scriptRoot "check_ci_repo_config.ps1"
$setChecksScript = Join-Path $scriptRoot "set_required_checks.ps1"
$checkProtectionScript = Join-Path $scriptRoot "check_branch_protection.ps1"
. (Join-Path $scriptRoot "_token_helpers.ps1")

$token = Resolve-GitHubToken -ExplicitToken $GitHubToken -ScriptRoot $scriptRoot
if ([string]::IsNullOrWhiteSpace($token)) {
  Write-Error "Missing GitHub token. Provide -GitHubToken, set GITHUB_TOKEN/GH_TOKEN/GITHUB_PAT/GITHUB_PERSONAL_ACCESS_TOKEN, or define GITHUB_PERSONAL_ACCESS_TOKEN in gitgov/gitgov-server/.env before running governance hardening."
  exit 1
}

if ($RequiredApprovals -lt 0) {
  Write-Error "RequiredApprovals must be >= 0."
  exit 1
}

Write-Host "GitGov governance hardening"
Write-Host ("Target repo: {0}/{1} (branch: {2})" -f $Owner, $Repo, $Branch)
Write-Host ""

$warnings = New-Object System.Collections.Generic.List[string]
$skipCiByPermission = $false
$skipBranchByPermission = $false

if (-not $SkipTokenPermissionsCheck) {
  Write-Host "[1/4] Checking GitHub token permissions..."
  $tokenCheckJson = & $checkTokenPermissionsScript -Owner $Owner -Repo $Repo -Branch $Branch -GitHubToken $token -EmitJson -NoFailOnForbidden
  $tokenCheck = $null
  try {
    $tokenCheck = ($tokenCheckJson -join "`n") | ConvertFrom-Json
  } catch {
    $message = "Unable to parse token permission summary JSON from check_token_permissions.ps1."
    if ($BestEffort) {
      Write-Warning $message
      $warnings.Add($message)
    } else {
      Write-Error $message
      exit 1
    }
  }

  if ($null -ne $tokenCheck -and [int]$tokenCheck.forbidden_count -gt 0) {
    $forbiddenEndpoints = @($tokenCheck.forbidden_endpoints | ForEach-Object { [string]$_ })
    $skipCiByPermission = $forbiddenEndpoints -contains "Actions secrets" -or $forbiddenEndpoints -contains "Actions variables"
    $skipBranchByPermission = $forbiddenEndpoints -contains "Branch protection"
    $permissionMessage = "Token lacks permissions for endpoints: {0}" -f ($forbiddenEndpoints -join ", ")

    if ($BestEffort) {
      Write-Warning ($permissionMessage + " | Continuing in best-effort mode.")
      $warnings.Add($permissionMessage)
    } else {
      Write-Error ($permissionMessage + " | Re-run with a stronger token or use -BestEffort.")
      exit 1
    }
  }
  Write-Host ""
} else {
  Write-Host "[1/4] Skipped token permissions check."
  Write-Host ""
}

if ($SkipCiConfigCheck) {
  Write-Host "[2/4] Skipped CI repository config check."
  Write-Host ""
} elseif ($skipCiByPermission) {
  Write-Host "[2/4] Skipped CI repository config check (token missing Actions secrets/variables read)."
  Write-Host ""
} else {
  Write-Host "[2/4] Checking CI repository config (secrets/variables)..."
  try {
    & $checkCiScript -Owner $Owner -Repo $Repo -GitHubToken $token -AllowMissingSonar:$AllowMissingSonar -RequireGitGovTelemetry:$RequireGitGovTelemetry -NoFailOnForbidden:$BestEffort
  } catch {
    if ($BestEffort) {
      $message = "CI repository config check failed: $($_.Exception.Message)"
      Write-Warning $message
      $warnings.Add($message)
    } else {
      throw
    }
  }
  Write-Host ""
}

if ($ApplyBranchProtection -and -not $skipBranchByPermission) {
  Write-Host "[3/4] Applying branch protection required checks..."
  try {
    & $setChecksScript -Owner $Owner -Repo $Repo -Branch $Branch -GitHubToken $token -RequiredApprovals $RequiredApprovals
  } catch {
    if ($BestEffort) {
      $message = "Branch protection apply failed: $($_.Exception.Message)"
      Write-Warning $message
      $warnings.Add($message)
    } else {
      throw
    }
  }
  Write-Host ""
} elseif ($ApplyBranchProtection -and $skipBranchByPermission) {
  Write-Host "[3/4] Skipped branch protection apply (token missing Branch protection permission)."
  Write-Host ""
} else {
  Write-Host "[3/4] Dry run mode (branch protection not applied)."
  Write-Host "       Use -ApplyBranchProtection to apply required checks."
  Write-Host ""
}

if ($skipBranchByPermission) {
  Write-Host "[4/4] Skipped branch protection validation (token missing Branch protection permission)."
} else {
  Write-Host "[4/4] Validating branch protection required checks..."
  try {
    & $checkProtectionScript -Owner $Owner -Repo $Repo -Branch $Branch -GitHubToken $token
  } catch {
    if ($BestEffort) {
      $message = "Branch protection validation failed: $($_.Exception.Message)"
      Write-Warning $message
      $warnings.Add($message)
    } else {
      throw
    }
  }
}
Write-Host ""

if ($warnings.Count -gt 0) {
  Write-Host "Best-effort warnings:"
  foreach ($item in $warnings) {
    Write-Host ("  - {0}" -f $item)
  }
  Write-Host ""
}

Write-Host "Done."
