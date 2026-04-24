param(
  [string]$GitGovUrl = "http://127.0.0.1:3001",
  [string]$PublicBaseUrl = "http://127.0.0.1:3001",
  [string]$RepoFullName = "",
  [string]$Branch = "main",
  [string]$ApiKey = "",
  [string]$GitHubToken = "",
  [int]$Hours = 168,
  [string]$OutputRoot = "",
  [switch]$ProbeUpdaterEndpoint,
  [switch]$FailFast
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Resolve-RepoFromGit {
  try {
    $origin = (git config --get remote.origin.url).Trim()
    if ([string]::IsNullOrWhiteSpace($origin)) { return "" }

    if ($origin -match "github\.com[:/](.+?)(?:\.git)?$") {
      return $matches[1]
    }
    return ""
  } catch {
    return ""
  }
}

function Resolve-EnvValue {
  param(
    [string]$CurrentValue,
    [string]$Key
  )
  if (-not [string]::IsNullOrWhiteSpace($CurrentValue)) {
    return $CurrentValue
  }
  $envPath = "gitgov/gitgov-server/.env"
  if (Test-Path $envPath) {
    $line = Get-Content $envPath | Where-Object { $_ -match "^\s*$Key\s*=\s*.+$" } | Select-Object -First 1
    if ($line) {
      return ($line -replace "^\s*$Key\s*=\s*","").Trim().Trim('"').Trim("'")
    }
  }
  return ""
}

function Run-Step {
  param(
    [Parameter(Mandatory = $true)][string]$Name,
    [Parameter(Mandatory = $true)][scriptblock]$Action
  )

  try {
    & $Action
    return [pscustomobject]@{ name = $Name; status = "PASS"; details = "Completed" }
  } catch {
    return [pscustomobject]@{ name = $Name; status = "FAIL"; details = $_.Exception.Message }
  }
}

$ApiKey = Resolve-EnvValue -CurrentValue $ApiKey -Key "GITGOV_API_KEY"
$GitHubToken = Resolve-EnvValue -CurrentValue $GitHubToken -Key "GITHUB_PERSONAL_ACCESS_TOKEN"
if ([string]::IsNullOrWhiteSpace($RepoFullName)) {
  $RepoFullName = Resolve-RepoFromGit
}
if ([string]::IsNullOrWhiteSpace($RepoFullName)) {
  $RepoFullName = "yohandry10/Git-Gov"
}

if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
  $stamp = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHHmmssZ")
  $outDir = "docs/reports/readiness-bundle-$stamp"
} else {
  $outDir = $OutputRoot
}
New-Item -ItemType Directory -Path $outDir -Force | Out-Null

$steps = New-Object System.Collections.Generic.List[object]

# 1) Public infra
$requireHttps = $true
try {
  $publicUri = [Uri]$PublicBaseUrl
  if ($publicUri.Scheme -ne "https") { $requireHttps = $false }
} catch {
  $requireHttps = $false
}
$steps.Add((Run-Step -Name "public_infra" -Action {
  & ./scripts/deploy/validate_public_infra.ps1 `
    -BaseUrl $PublicBaseUrl `
    -ApiKey $ApiKey `
    -RequireHttps:$requireHttps `
    -OutputPath "$outDir/public-infra.md"
  if ($LASTEXITCODE -ne 0) { throw "validate_public_infra failed with exit code $LASTEXITCODE" }
})) | Out-Null
if ($FailFast -and $steps[-1].status -eq "FAIL") { throw "FailFast: public_infra" }

# 2) Updater readiness
$steps.Add((Run-Step -Name "desktop_updater_readiness" -Action {
  if ($ProbeUpdaterEndpoint.IsPresent) {
    & ./scripts/deploy/validate_desktop_updater_readiness.ps1 `
      -TauriConfigPath "gitgov/src-tauri/tauri.conf.json" `
      -OutputPath "$outDir/desktop-updater.md"
  } else {
    & ./scripts/deploy/validate_desktop_updater_readiness.ps1 `
      -TauriConfigPath "gitgov/src-tauri/tauri.conf.json" `
      -SkipEndpointProbe `
      -OutputPath "$outDir/desktop-updater.md"
  }
  if ($LASTEXITCODE -ne 0) { throw "validate_desktop_updater_readiness failed with exit code $LASTEXITCODE" }
})) | Out-Null
if ($FailFast -and $steps[-1].status -eq "FAIL") { throw "FailFast: desktop_updater_readiness" }

# 3) Quality gate matrix (requires ApiKey)
if (-not [string]::IsNullOrWhiteSpace($ApiKey)) {
  $steps.Add((Run-Step -Name "quality_gate_matrix" -Action {
    & ./scripts/jenkins/resolve_quality_gate_matrix_commits.ps1 `
      -GitGovUrl $GitGovUrl `
      -ApiKey $ApiKey `
      -RepoFullName $RepoFullName `
      -Branch $Branch `
      -OutputPath "$outDir/quality-gate-matrix-resolution.json"
    if ($LASTEXITCODE -ne 0) { throw "resolve_quality_gate_matrix_commits failed with exit code $LASTEXITCODE" }

    $resolved = Get-Content "$outDir/quality-gate-matrix-resolution.json" -Raw | ConvertFrom-Json
    & ./scripts/jenkins/validate_quality_gate_policy_matrix.ps1 `
      -GitGovUrl $GitGovUrl `
      -ApiKey $ApiKey `
      -RepoFullName $RepoFullName `
      -Branch $Branch `
      -FailingCommitSha ([string]$resolved.failing_commit_sha) `
      -GreenCommitSha ([string]$resolved.green_commit_sha) `
      -OutputPath "$outDir/quality-gate-matrix.md"
    if ($LASTEXITCODE -ne 0) { throw "validate_quality_gate_policy_matrix failed with exit code $LASTEXITCODE" }
  })) | Out-Null
  if ($FailFast -and $steps[-1].status -eq "FAIL") { throw "FailFast: quality_gate_matrix" }

  # 4) Tier baseline standard (requires ApiKey)
  $steps.Add((Run-Step -Name "tier_baseline_standard" -Action {
    & ./scripts/control-plane/calibrate_risk_tier_baseline.ps1 `
      -GitGovUrl $GitGovUrl `
      -ApiKey $ApiKey `
      -Tier "standard" `
      -Hours $Hours `
      -OutputPath "$outDir/risk-tier-baseline-standard.md"
    if ($LASTEXITCODE -ne 0) { throw "calibrate_risk_tier_baseline failed with exit code $LASTEXITCODE" }
  })) | Out-Null
  if ($FailFast -and $steps[-1].status -eq "FAIL") { throw "FailFast: tier_baseline_standard" }
} else {
  $steps.Add([pscustomobject]@{ name = "quality_gate_matrix"; status = "SKIP"; details = "Missing ApiKey" }) | Out-Null
  $steps.Add([pscustomobject]@{ name = "tier_baseline_standard"; status = "SKIP"; details = "Missing ApiKey" }) | Out-Null
}

# 5) GitHub cloud visibility checks (optional)
if (-not [string]::IsNullOrWhiteSpace($GitHubToken)) {
  $steps.Add((Run-Step -Name "github_token_permissions" -Action {
    $json = & ./scripts/github/check_token_permissions.ps1 `
      -GitHubToken $GitHubToken `
      -Owner ($RepoFullName.Split('/')[0]) `
      -Repo ($RepoFullName.Split('/')[1]) `
      -Branch $Branch `
      -EmitJson `
      -NoFailOnForbidden `
      -Quiet
    if ($LASTEXITCODE -ne 0) { throw "check_token_permissions failed with exit code $LASTEXITCODE" }
    $json | Set-Content -Path "$outDir/github-token-permissions.json" -Encoding UTF8
  })) | Out-Null

  $steps.Add((Run-Step -Name "github_ci_config_precheck" -Action {
    $text = & ./scripts/github/check_ci_repo_config.ps1 `
      -GitHubToken $GitHubToken `
      -Owner ($RepoFullName.Split('/')[0]) `
      -Repo ($RepoFullName.Split('/')[1]) `
      -AllowMissingSonar `
      -NoFailOnForbidden
    if ($LASTEXITCODE -ne 0) { throw "check_ci_repo_config failed with exit code $LASTEXITCODE" }
    $text | Set-Content -Path "$outDir/github-ci-config-precheck.txt" -Encoding UTF8
  })) | Out-Null
} else {
  $steps.Add([pscustomobject]@{ name = "github_token_permissions"; status = "SKIP"; details = "Missing GitHubToken" }) | Out-Null
  $steps.Add([pscustomobject]@{ name = "github_ci_config_precheck"; status = "SKIP"; details = "Missing GitHubToken" }) | Out-Null
}

$passCount = @($steps | Where-Object { $_.status -eq "PASS" }).Count
$failCount = @($steps | Where-Object { $_.status -eq "FAIL" }).Count
$skipCount = @($steps | Where-Object { $_.status -eq "SKIP" }).Count
$summary = if ($failCount -gt 0) { "FAIL" } elseif ($skipCount -gt 0) { "WARN" } else { "PASS" }

$rows = $steps | ForEach-Object { "| $($_.name) | $($_.status) | $($_.details -replace '\|','\\|') |" }
$generatedUtc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-dd HH:mm:ss")
$reportPath = "$outDir/readiness-bundle-summary.md"

$report = @"
# Enterprise Readiness Bundle Report

Generated (UTC): $generatedUtc
Summary: **$summary**

## Context

- GitGov URL: $GitGovUrl
- Public base URL: $PublicBaseUrl
- Repo: $RepoFullName
- Branch: $Branch

## Steps

| Step | Status | Details |
|---|---|---|
$(($rows -join "`n"))

## Totals

- PASS: $passCount
- FAIL: $failCount
- SKIP: $skipCount
"@

Set-Content -Path $reportPath -Value $report -Encoding UTF8

Write-Host "${summary}: enterprise readiness bundle completed"
Write-Host "  report: $reportPath"
Write-Host "  folder: $outDir"

if ($summary -eq "FAIL") {
  exit 1
}
exit 0
