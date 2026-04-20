param(
  [string]$GitGovUrl = "http://127.0.0.1:3001",
  [string]$ApiKey,
  [string]$TargetsPath = "ops/slo/domain-slo-targets.json",
  [string]$DomainName = "",
  [int]$Hours = 168,
  [int]$CorrelationLimit = 500,
  [string]$OutputDir = "",
  [switch]$FailOnBreach
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($ApiKey)) {
  Write-Error "Missing -ApiKey."
  exit 1
}

if (-not (Test-Path $TargetsPath)) {
  Write-Error "Targets file not found: $TargetsPath"
  exit 1
}

if ($Hours -lt 1) {
  Write-Error "-Hours must be >= 1."
  exit 1
}

if ($CorrelationLimit -lt 1) {
  Write-Error "-CorrelationLimit must be >= 1."
  exit 1
}

$targetsRaw = Get-Content -Path $TargetsPath -Raw | ConvertFrom-Json
if ($null -eq $targetsRaw -or $null -eq $targetsRaw.domains) {
  Write-Error "Invalid targets format: expected { domains: [...] }"
  exit 1
}

$domains = @($targetsRaw.domains)
if (-not [string]::IsNullOrWhiteSpace($DomainName)) {
  $domains = @($domains | Where-Object { [string]$_.name -eq $DomainName })
  if ($domains.Count -eq 0) {
    $available = @($targetsRaw.domains | ForEach-Object { [string]$_.name }) -join ", "
    Write-Error "Domain '$DomainName' not found. Available: $available"
    exit 1
  }
}

if ($domains.Count -eq 0) {
  Write-Error "No domains configured in $TargetsPath"
  exit 1
}

$timestamp = (Get-Date).ToUniversalTime()
if ([string]::IsNullOrWhiteSpace($OutputDir)) {
  $OutputDir = "docs/reports/domain-slo-validation-$($timestamp.ToString("yyyy-MM-ddTHHmmssZ"))"
}
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

function Parse-FirstMatch {
  param(
    [Parameter(Mandatory = $true)][string]$Text,
    [Parameter(Mandatory = $true)][string]$Pattern
  )
  $m = [regex]::Match($Text, $Pattern)
  if (-not $m.Success) { return $null }
  return $m.Groups[1].Value
}

function Parse-PercentOrNull {
  param([string]$Raw)
  if ([string]::IsNullOrWhiteSpace($Raw)) { return $null }
  if ($Raw -match '^\s*N/A\s*$') { return $null }
  $clean = $Raw.Trim().TrimEnd('%')
  try {
    return [double]$clean
  } catch {
    return $null
  }
}

function Evaluate-MaxThreshold {
  param(
    [Parameter(Mandatory = $true)][string]$Name,
    [AllowNull()][double]$Actual,
    [double]$MaxAllowed
  )
  if ($null -eq $Actual) {
    return [pscustomobject]@{
      metric = $Name
      status = "SKIP"
      actual = "N/A"
      target = "<= $MaxAllowed"
      detail = "No data"
      breached = $false
    }
  }

  $ok = $Actual -le $MaxAllowed
  return [pscustomobject]@{
    metric = $Name
    status = $(if ($ok) { "PASS" } else { "FAIL" })
    actual = ("{0:N1}%" -f $Actual)
    target = "<= $MaxAllowed%"
    detail = $(if ($ok) { "Within target" } else { "Exceeded target" })
    breached = (-not $ok)
  }
}

function Evaluate-MinThreshold {
  param(
    [Parameter(Mandatory = $true)][string]$Name,
    [AllowNull()][double]$Actual,
    [double]$MinAllowed
  )
  if ($null -eq $Actual) {
    return [pscustomobject]@{
      metric = $Name
      status = "SKIP"
      actual = "N/A"
      target = ">= $MinAllowed"
      detail = "No data"
      breached = $false
    }
  }

  $ok = $Actual -ge $MinAllowed
  return [pscustomobject]@{
    metric = $Name
    status = $(if ($ok) { "PASS" } else { "FAIL" })
    actual = ("{0:N1}" -f $Actual)
    target = ">= $MinAllowed"
    detail = $(if ($ok) { "Within target" } else { "Below target" })
    breached = (-not $ok)
  }
}

$results = New-Object System.Collections.Generic.List[object]

foreach ($domain in $domains) {
  $domainName = [string]$domain.name
  $tier = [string]$domain.tier
  $orgName = [string]$domain.org_name
  $slo = $domain.slo

  if ([string]::IsNullOrWhiteSpace($domainName) -or [string]::IsNullOrWhiteSpace($tier) -or $null -eq $slo) {
    Write-Error "Invalid domain entry in targets file. Each domain requires name, tier, and slo."
    exit 1
  }

  $baselinePath = Join-Path -Path $OutputDir -ChildPath ("domain-{0}-baseline.md" -f $domainName)
  $calibrationArgs = @{
    GitGovUrl = $GitGovUrl
    ApiKey = $ApiKey
    Tier = $tier
    Hours = $Hours
    CorrelationLimit = $CorrelationLimit
    OutputPath = $baselinePath
  }
  if (-not [string]::IsNullOrWhiteSpace($orgName)) {
    $calibrationArgs.OrgName = $orgName
  }

  & ./scripts/control-plane/calibrate_risk_tier_baseline.ps1 @calibrationArgs
  $calibrationExit = if (Test-Path variable:LASTEXITCODE) { [int]$LASTEXITCODE } else { if ($?) { 0 } else { 1 } }
  if ($calibrationExit -ne 0) {
    Write-Error "Baseline calibration failed for domain '$domainName' (tier=$tier)."
    exit $calibrationExit
  }

  $baselineText = Get-Content -Path $baselinePath -Raw

  $readinessRaw = Parse-FirstMatch -Text $baselineText -Pattern 'Release readiness:\s+\*\*([0-9]+)\/100\*\*'
  $blockedPushRaw = Parse-FirstMatch -Text $baselineText -Pattern 'blocked_push_rate:\s+([0-9\.]+%)'
  $traceabilityGapRaw = Parse-FirstMatch -Text $baselineText -Pattern 'traceability_gap:\s+([0-9\.]+%)'
  $pipelineFailureRaw = Parse-FirstMatch -Text $baselineText -Pattern 'pipeline_failure_rate_7d:\s+([0-9\.]+%)'
  $sonarFailureRaw = Parse-FirstMatch -Text $baselineText -Pattern 'sonar_failure_rate_sample:\s+([0-9\.]+%|N/A)'
  $unresolvedRaw = Parse-FirstMatch -Text $baselineText -Pattern 'unresolved_violation_rate:\s+([0-9\.]+%|N/A)'

  $readiness = if ($null -ne $readinessRaw) { [double]$readinessRaw } else { $null }
  $blockedPush = Parse-PercentOrNull -Raw $blockedPushRaw
  $traceabilityGap = Parse-PercentOrNull -Raw $traceabilityGapRaw
  $pipelineFailure = Parse-PercentOrNull -Raw $pipelineFailureRaw
  $sonarFailure = Parse-PercentOrNull -Raw $sonarFailureRaw
  $unresolved = Parse-PercentOrNull -Raw $unresolvedRaw

  $checks = @(
    (Evaluate-MinThreshold -Name "release_readiness" -Actual $readiness -MinAllowed ([double]$slo.readiness_min)),
    (Evaluate-MaxThreshold -Name "blocked_push_rate" -Actual $blockedPush -MaxAllowed ([double]$slo.blocked_push_rate_max)),
    (Evaluate-MaxThreshold -Name "traceability_gap" -Actual $traceabilityGap -MaxAllowed ([double]$slo.traceability_gap_max)),
    (Evaluate-MaxThreshold -Name "pipeline_failure_rate_7d" -Actual $pipelineFailure -MaxAllowed ([double]$slo.pipeline_failure_rate_max)),
    (Evaluate-MaxThreshold -Name "sonar_failure_rate_sample" -Actual $sonarFailure -MaxAllowed ([double]$slo.sonar_failure_rate_max)),
    (Evaluate-MaxThreshold -Name "unresolved_violation_rate" -Actual $unresolved -MaxAllowed ([double]$slo.unresolved_violation_rate_max))
  )

  $breaches = @($checks | Where-Object { $_.breached -eq $true })
  $status = if ($breaches.Count -eq 0) { "PASS" } else { "FAIL" }
  $results.Add([pscustomobject]@{
    domain = $domainName
    tier = $tier
    org_name = $orgName
    status = $status
    baseline_report = $baselinePath
    checks = $checks
    breach_count = $breaches.Count
  }) | Out-Null
}

$summaryLines = $results | ForEach-Object {
  $domain = $_
  "| $($domain.domain) | $($domain.tier) | $($domain.status) | $($domain.breach_count) |"
}

$detailBlocks = $results | ForEach-Object {
  $domain = $_
  $rows = $domain.checks | ForEach-Object {
    "| $($_.metric) | $($_.status) | $($_.actual) | $($_.target) | $($_.detail) |"
  }
@"
## Domain: $($domain.domain)

- Tier: $($domain.tier)
- Org filter: $(if ([string]::IsNullOrWhiteSpace([string]$domain.org_name)) { "none" } else { $domain.org_name })
- Status: **$($domain.status)**
- Baseline report: $($domain.baseline_report)

| Metric | Status | Actual | Target | Detail |
|---|---|---|---|---|
$(($rows -join "`n"))
"@
}

$total = $results.Count
$failed = @($results | Where-Object { $_.status -eq "FAIL" }).Count
$passed = $total - $failed
$overall = if ($failed -gt 0) { "FAIL" } else { "PASS" }

$summaryPath = Join-Path -Path $OutputDir -ChildPath "domain-slo-summary.md"
$report = @"
# Domain SLO Validation Report

Generated (UTC): $($timestamp.ToString("yyyy-MM-dd HH:mm:ss"))
GitGov URL: $GitGovUrl
Targets file: $TargetsPath
Window hours: $Hours

Overall status: **$overall**

## Summary

| Domain | Tier | Status | Breaches |
|---|---|---|---|
$(($summaryLines -join "`n"))

## Totals

- Domains validated: $total
- Passed: $passed
- Failed: $failed

$(($detailBlocks -join "`n`n"))
"@

Set-Content -Path $summaryPath -Value $report -Encoding UTF8

Write-Host "${overall}: domain SLO validation completed"
Write-Host "  output dir: $OutputDir"
Write-Host "  summary:    $summaryPath"

if ($overall -eq "FAIL" -and $FailOnBreach.IsPresent) {
  exit 2
}
exit 0
