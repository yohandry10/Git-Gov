param(
  [string]$Owner = "yohandry10",
  [string]$Repo = "Git-Gov",
  [string]$Branch = "main",
  [string]$GitHubToken = "",
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

if (-not $SkipTokenPermissionsCheck) {
  Write-Host "[1/4] Checking GitHub token permissions..."
  & $checkTokenPermissionsScript -Owner $Owner -Repo $Repo -Branch $Branch -GitHubToken $token
  Write-Host ""
} else {
  Write-Host "[1/4] Skipped token permissions check."
  Write-Host ""
}

if (-not $SkipCiConfigCheck) {
  Write-Host "[2/4] Checking CI repository config (secrets/variables)..."
  & $checkCiScript -Owner $Owner -Repo $Repo -GitHubToken $token -AllowMissingSonar:$AllowMissingSonar -RequireGitGovTelemetry:$RequireGitGovTelemetry
  Write-Host ""
} else {
  Write-Host "[2/4] Skipped CI repository config check."
  Write-Host ""
}

if ($ApplyBranchProtection) {
  Write-Host "[3/4] Applying branch protection required checks..."
  & $setChecksScript -Owner $Owner -Repo $Repo -Branch $Branch -GitHubToken $token -RequiredApprovals $RequiredApprovals
  Write-Host ""
} else {
  Write-Host "[3/4] Dry run mode (branch protection not applied)."
  Write-Host "       Use -ApplyBranchProtection to apply required checks."
  Write-Host ""
}

Write-Host "[4/4] Validating branch protection required checks..."
& $checkProtectionScript -Owner $Owner -Repo $Repo -Branch $Branch -GitHubToken $token
Write-Host ""
Write-Host "Done."
