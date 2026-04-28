param(
  [string[]]$EnvFiles = @("gitgov\.env", "gitgov\gitgov-server\.env"),
  [string]$GitGovUrl = "https://gitgov-api.onrender.com",
  [string]$ApiKey = "",
  [string]$RepoFullName = "yohandry10/Git-Gov",
  [string]$Branch = "main",
  [string]$OrgName = "yohandry10",
  [int]$Hours = 720,
  [int]$CorrelationLimit = 500,
  [double]$MinCoverage = 0,
  [switch]$RefreshCorrelations,
  [string]$OutputPath = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")

function Load-DotEnvNoPrint {
  param([string]$Path)

  $resolved = if ([System.IO.Path]::IsPathRooted($Path)) {
    $Path
  } else {
    Join-Path $repoRoot $Path
  }

  if (!(Test-Path $resolved)) { return }

  Get-Content $resolved | ForEach-Object {
    $line = $_.Trim()
    if (!$line -or $line.StartsWith("#") -or !$line.Contains("=")) { return }
    $parts = $line -split "=", 2
    $name = $parts[0].Trim()
    $value = $parts[1].Trim().Trim('"')
    if (![string]::IsNullOrWhiteSpace($name)) {
      [Environment]::SetEnvironmentVariable($name, $value, "Process")
    }
  }
}

function Invoke-GitGovJson {
  param(
    [string]$Method,
    [string]$Path,
    [object]$Body = $null
  )

  $headers = @{
    Authorization = "Bearer $ApiKey"
    Accept = "application/json"
    "Content-Type" = "application/json"
  }

  $uri = "$($GitGovUrl.TrimEnd('/'))$Path"
  if ($null -eq $Body) {
    return Invoke-RestMethod -Method $Method -Uri $uri -Headers $headers -TimeoutSec 60
  }

  return Invoke-RestMethod -Method $Method -Uri $uri -Headers $headers -Body ($Body | ConvertTo-Json -Depth 10) -TimeoutSec 60
}

foreach ($envFile in $EnvFiles) {
  Load-DotEnvNoPrint $envFile
}

if ([string]::IsNullOrWhiteSpace($ApiKey)) {
  $ApiKey = $env:GITGOV_API_KEY
}

if ([string]::IsNullOrWhiteSpace($ApiKey)) {
  throw "Missing ApiKey or GITGOV_API_KEY."
}
if ([string]::IsNullOrWhiteSpace($GitGovUrl)) {
  throw "Missing GitGovUrl."
}
if ([string]::IsNullOrWhiteSpace($RepoFullName)) {
  throw "Missing RepoFullName."
}
if ([string]::IsNullOrWhiteSpace($Branch)) {
  throw "Missing Branch."
}
if ($Hours -lt 1) {
  throw "-Hours must be >= 1."
}
if ($CorrelationLimit -lt 1) {
  throw "-CorrelationLimit must be >= 1."
}
if ($MinCoverage -lt 0 -or $MinCoverage -gt 100) {
  throw "-MinCoverage must be between 0 and 100."
}

$correlationRefresh = $null
if ($RefreshCorrelations) {
  $payload = @{
    repo_full_name = $RepoFullName
    hours = $Hours
    limit = $CorrelationLimit
  }
  if (-not [string]::IsNullOrWhiteSpace($OrgName)) {
    $payload["org_name"] = $OrgName
  }

  $correlationRefresh = Invoke-GitGovJson -Method "POST" -Path "/integrations/jira/correlate" -Body $payload
}

$encodedRepo = [Uri]::EscapeDataString($RepoFullName)
$encodedBranch = [Uri]::EscapeDataString($Branch)
$coveragePath = "/integrations/jira/ticket-coverage?repo_full_name=$encodedRepo&branch=$encodedBranch&hours=$Hours"
if (-not [string]::IsNullOrWhiteSpace($OrgName)) {
  $coveragePath += "&org_name=$([Uri]::EscapeDataString($OrgName))"
}

$coverage = Invoke-GitGovJson -Method "GET" -Path $coveragePath
$coveragePercent = 0.0
if ($coverage.PSObject.Properties.Name -contains "coverage_percentage") {
  $coveragePercent = [double]$coverage.coverage_percentage
}

$passedThreshold = $coveragePercent -ge $MinCoverage
$result = [ordered]@{
  ok = $passedThreshold
  checked_at_utc = (Get-Date).ToUniversalTime().ToString("o")
  gitgov_url = $GitGovUrl.TrimEnd("/")
  repo_full_name = $RepoFullName
  branch = $Branch
  org_name = $OrgName
  hours = $Hours
  min_coverage = $MinCoverage
  refresh_correlations = $RefreshCorrelations.IsPresent
  correlation_refresh = $correlationRefresh
  coverage = $coverage
  coverage_percentage = [Math]::Round($coveragePercent, 2)
}

$json = $result | ConvertTo-Json -Depth 12
if (![string]::IsNullOrWhiteSpace($OutputPath)) {
  $outDir = Split-Path -Parent $OutputPath
  if (![string]::IsNullOrWhiteSpace($outDir) -and !(Test-Path $outDir)) {
    New-Item -ItemType Directory -Force -Path $outDir | Out-Null
  }
  Set-Content -Path $OutputPath -Value $json -Encoding UTF8
}

Write-Output $json

if (!$passedThreshold) {
  exit 1
}
