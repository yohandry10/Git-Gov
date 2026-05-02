param(
  [string]$GitGovUrl = "",
  [string]$OrgName = "yohandry10",
  [string]$RepoFullName = "yohandry10/Git-Gov",
  [string]$ReleaseId = "enterprise-route-auth-smoke",
  [string]$Environment = "production",
  [string]$ApiKey = "",
  [string]$OutputDir = "docs/reports/enterprise-route-auth-smoke-ci",
  [switch]$AllowMissingApiKey,
  [switch]$ReportOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($GitGovUrl)) {
  $GitGovUrl = $env:GITGOV_URL
}
if ([string]::IsNullOrWhiteSpace($GitGovUrl)) {
  $GitGovUrl = "https://gitgov-api.onrender.com"
}

if ([string]::IsNullOrWhiteSpace($ApiKey)) {
  $ApiKey = $env:GITGOV_API_KEY
}

$GitGovUrl = $GitGovUrl.TrimEnd("/")
$checkedAtUtc = (Get-Date).ToUniversalTime().ToString("o")

function ConvertTo-QueryValue {
  param([string]$Value)

  return [System.Uri]::EscapeDataString($Value)
}

function Join-GitGovUrl {
  param([string]$PathAndQuery)

  if (!$PathAndQuery.StartsWith("/")) {
    $PathAndQuery = "/" + $PathAndQuery
  }
  return "$GitGovUrl$PathAndQuery"
}

function Invoke-SmokeRequest {
  param(
    [string]$Id,
    [string]$Method,
    [string]$PathAndQuery,
    [int]$ExpectedStatus,
    [bool]$Authenticated
  )

  $headers = @{ Accept = "application/json" }
  if ($Authenticated) {
    $headers["Authorization"] = "Bearer $ApiKey"
  }

  $uri = Join-GitGovUrl $PathAndQuery
  $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
  $actualStatus = 0
  $errorMessage = ""

  try {
    $response = Invoke-WebRequest `
      -Method $Method `
      -Uri $uri `
      -Headers $headers `
      -UseBasicParsing `
      -TimeoutSec 30
    $actualStatus = [int]$response.StatusCode
  } catch {
    if ($_.Exception.Response) {
      $actualStatus = [int]$_.Exception.Response.StatusCode
    } else {
      $errorMessage = $_.Exception.Message
    }
  } finally {
    $stopwatch.Stop()
  }

  $ok = $actualStatus -eq $ExpectedStatus
  if (!$ok -and [string]::IsNullOrWhiteSpace($errorMessage)) {
    $errorMessage = "Expected HTTP $ExpectedStatus but received HTTP $actualStatus."
  }

  [ordered]@{
    id = $Id
    method = $Method
    path = $PathAndQuery
    auth = if ($Authenticated) { "bearer" } else { "anonymous" }
    expected_status = $ExpectedStatus
    actual_status = $actualStatus
    ok = $ok
    duration_ms = [math]::Round($stopwatch.Elapsed.TotalMilliseconds, 0)
    error = $errorMessage
  }
}

function Write-SmokeArtifacts {
  param([object]$Result)

  if (!(Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
  }

  $jsonPath = Join-Path $OutputDir "enterprise-route-auth-smoke.json"
  $mdPath = Join-Path $OutputDir "enterprise-route-auth-smoke.md"
  $Result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $jsonPath -Encoding UTF8

  $lines = New-Object System.Collections.Generic.List[string]
  $lines.Add("# Enterprise Route Auth Smoke")
  $lines.Add("")
  $lines.Add("- Checked at UTC: ``$($Result.checked_at_utc)``")
  $lines.Add("- Status: ``$($Result.status)``")
  $lines.Add("- GitGov URL: ``$($Result.gitgov_url)``")
  $lines.Add("- Org: ``$($Result.org_name)``")
  $lines.Add("- Repository: ``$($Result.repository_full_name)``")
  $lines.Add("- Release ID: ``$($Result.release_id)``")
  $lines.Add("- Environment: ``$($Result.environment)``")
  if (![string]::IsNullOrWhiteSpace($Result.reason)) {
    $lines.Add("- Reason: ``$($Result.reason)``")
  }
  $lines.Add("")
  $lines.Add("| Check | Auth | Expected | Actual | Result |")
  $lines.Add("| --- | --- | --- | --- | --- |")
  foreach ($check in $Result.checks) {
    $resultText = if ($check.ok) { "PASS" } else { "FAIL" }
    $lines.Add("| ``$($check.id)`` | ``$($check.auth)`` | ``$($check.expected_status)`` | ``$($check.actual_status)`` | $resultText |")
  }

  $lines | Set-Content -LiteralPath $mdPath -Encoding UTF8
}

$encodedOrg = ConvertTo-QueryValue $OrgName
$encodedRepo = ConvertTo-QueryValue $RepoFullName
$encodedRelease = ConvertTo-QueryValue $ReleaseId
$encodedEnvironment = ConvertTo-QueryValue $Environment

if ([string]::IsNullOrWhiteSpace($ApiKey)) {
  $result = [ordered]@{
    ok = $true
    status = "skipped"
    reason = "missing_gitgov_api_key"
    checked_at_utc = $checkedAtUtc
    gitgov_url = $GitGovUrl
    org_name = $OrgName
    repository_full_name = $RepoFullName
    release_id = $ReleaseId
    environment = $Environment
    checks = @()
  }
  Write-SmokeArtifacts -Result $result
  Write-Output ($result | ConvertTo-Json -Depth 8)
  if ($AllowMissingApiKey) {
    exit 0
  }
  exit 1
}

$routeChecks = @(
  @{
    Id = "health_public"
    Method = "GET"
    PathAndQuery = "/health"
    ExpectedStatus = 200
    Authenticated = $false
  },
  @{
    Id = "adoption_profile_anonymous"
    Method = "GET"
    PathAndQuery = "/enterprise/adoption-profile?org_name=$encodedOrg"
    ExpectedStatus = 401
    Authenticated = $false
  },
  @{
    Id = "onboarding_checklist_tracking_anonymous"
    Method = "GET"
    PathAndQuery = "/enterprise/onboarding-checklist-tracking?org_name=$encodedOrg"
    ExpectedStatus = 401
    Authenticated = $false
  },
  @{
    Id = "release_approvals_anonymous"
    Method = "GET"
    PathAndQuery = "/enterprise/release-approvals?org_name=$encodedOrg"
    ExpectedStatus = 401
    Authenticated = $false
  },
  @{
    Id = "release_governance_evaluate_anonymous"
    Method = "GET"
    PathAndQuery = "/enterprise/release-governance/evaluate?org_name=$encodedOrg&repository_full_name=$encodedRepo&release_id=$encodedRelease&environment=$encodedEnvironment"
    ExpectedStatus = 401
    Authenticated = $false
  },
  @{
    Id = "adoption_profile_authenticated"
    Method = "GET"
    PathAndQuery = "/enterprise/adoption-profile?org_name=$encodedOrg"
    ExpectedStatus = 200
    Authenticated = $true
  },
  @{
    Id = "onboarding_checklist_tracking_authenticated"
    Method = "GET"
    PathAndQuery = "/enterprise/onboarding-checklist-tracking?org_name=$encodedOrg"
    ExpectedStatus = 200
    Authenticated = $true
  },
  @{
    Id = "release_approvals_authenticated"
    Method = "GET"
    PathAndQuery = "/enterprise/release-approvals?org_name=$encodedOrg&limit=10&offset=0"
    ExpectedStatus = 200
    Authenticated = $true
  },
  @{
    Id = "release_governance_evaluate_authenticated"
    Method = "GET"
    PathAndQuery = "/enterprise/release-governance/evaluate?org_name=$encodedOrg&repository_full_name=$encodedRepo&release_id=$encodedRelease&environment=$encodedEnvironment"
    ExpectedStatus = 200
    Authenticated = $true
  }
)

$checks = New-Object System.Collections.Generic.List[object]
foreach ($routeCheck in $routeChecks) {
  $checks.Add((Invoke-SmokeRequest `
        -Id $routeCheck.Id `
        -Method $routeCheck.Method `
        -PathAndQuery $routeCheck.PathAndQuery `
        -ExpectedStatus $routeCheck.ExpectedStatus `
        -Authenticated $routeCheck.Authenticated))
}

$failed = @($checks | Where-Object { $_.ok -ne $true })
$result = [ordered]@{
  ok = ($failed.Count -eq 0)
  status = if ($failed.Count -eq 0) { "passed" } else { "failed" }
  reason = ""
  checked_at_utc = $checkedAtUtc
  gitgov_url = $GitGovUrl
  org_name = $OrgName
  repository_full_name = $RepoFullName
  release_id = $ReleaseId
  environment = $Environment
  checks = $checks
}

Write-SmokeArtifacts -Result $result
Write-Output ($result | ConvertTo-Json -Depth 8)

if ($failed.Count -gt 0 -and !$ReportOnly) {
  exit 1
}
