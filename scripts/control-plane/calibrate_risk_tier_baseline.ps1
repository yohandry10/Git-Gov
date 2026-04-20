param(
  [string]$GitGovUrl = "http://127.0.0.1:3001",
  [string]$ApiKey,
  [ValidateSet("critical", "standard", "internal")][string]$Tier = "standard",
  [string]$OrgName = "",
  [int]$Hours = 168,
  [int]$CorrelationLimit = 500,
  [string]$OutputPath = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($ApiKey)) {
  Write-Error "Missing -ApiKey."
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

function Get-Number {
  param(
    [Parameter()][object]$Value,
    [double]$Default = 0
  )
  if ($null -eq $Value) { return $Default }
  try {
    return [double]$Value
  } catch {
    return $Default
  }
}

function Clamp-Percent {
  param([double]$Value)
  if ([double]::IsNaN($Value) -or [double]::IsInfinity($Value)) { return 0.0 }
  if ($Value -lt 0) { return 0.0 }
  if ($Value -gt 100) { return 100.0 }
  return $Value
}

function Compute-WeightedScore {
  param([array]$Signals)
  $active = @($Signals | Where-Object { $_.available -eq $true })
  if ($active.Count -eq 0) {
    return @{ score = 0; available = 0; total = $Signals.Count }
  }

  $totalWeight = 0.0
  $weightedSum = 0.0
  foreach ($signal in $active) {
    $weight = Get-Number -Value $signal.weight
    $value = Get-Number -Value $signal.value
    $totalWeight += $weight
    $weightedSum += ($value * $weight)
  }
  if ($totalWeight -le 0) {
    return @{ score = 0; available = $active.Count; total = $Signals.Count }
  }

  return @{
    score = [int][Math]::Round($weightedSum / $totalWeight)
    available = $active.Count
    total = $Signals.Count
  }
}

function Invoke-GitGovJson {
  param(
    [Parameter(Mandatory = $true)][string]$Path
  )
  $base = $GitGovUrl.TrimEnd('/')
  $uri = "$base$Path"
  $headers = @{
    Authorization = "Bearer $ApiKey"
    "Content-Type" = "application/json"
  }
  try {
    return Invoke-RestMethod -Uri $uri -Method GET -Headers $headers
  } catch {
    if ($_.Exception.Response) {
      $reader = New-Object IO.StreamReader($_.Exception.Response.GetResponseStream())
      $body = $reader.ReadToEnd()
      throw "HTTP error calling $uri -> $body"
    }
    throw
  }
}

function Try-Invoke-GitGovJson {
  param(
    [Parameter(Mandatory = $true)][string]$Path,
    [Parameter(Mandatory = $true)][string]$Label
  )
  try {
    return @{ ok = $true; value = (Invoke-GitGovJson -Path $Path); warning = $null }
  } catch {
    return @{ ok = $false; value = $null; warning = "$Label unavailable: $($_.Exception.Message)" }
  }
}

$profiles = @{
  critical = @{
    label = "Critical"
    readiness = @{ pipeline = 0.5; traceability = 0.2; sonar = 0.3; healthy = 90; watch = 78 }
    risk = @{
      blockedPush = 0.25
      ticketGap = 0.25
      pipelineFailure = 0.2
      sonarFailure = 0.2
      unresolved = 0.1
      lowUpper = 30
      mediumUpper = 50
      sla = @{ blockedPush = 5; ticketGap = 15; pipelineFailure = 10; sonarFailure = 12; unresolved = 30; readiness = 85 }
    }
  }
  standard = @{
    label = "Standard"
    readiness = @{ pipeline = 0.45; traceability = 0.25; sonar = 0.3; healthy = 85; watch = 70 }
    risk = @{
      blockedPush = 0.2
      ticketGap = 0.2
      pipelineFailure = 0.2
      sonarFailure = 0.2
      unresolved = 0.2
      lowUpper = 35
      mediumUpper = 60
      sla = @{ blockedPush = 10; ticketGap = 25; pipelineFailure = 20; sonarFailure = 20; unresolved = 40; readiness = 75 }
    }
  }
  internal = @{
    label = "Internal"
    readiness = @{ pipeline = 0.4; traceability = 0.2; sonar = 0.4; healthy = 80; watch = 65 }
    risk = @{
      blockedPush = 0.15
      ticketGap = 0.15
      pipelineFailure = 0.25
      sonarFailure = 0.2
      unresolved = 0.25
      lowUpper = 40
      mediumUpper = 65
      sla = @{ blockedPush = 15; ticketGap = 35; pipelineFailure = 30; sonarFailure = 30; unresolved = 50; readiness = 65 }
    }
  }
}

$profile = $profiles[$Tier]

$warnings = New-Object System.Collections.Generic.List[string]
$stats = Invoke-GitGovJson -Path "/stats"

$ticketCoveragePath = "/integrations/jira/ticket-coverage?hours=$Hours"
if (-not [string]::IsNullOrWhiteSpace($OrgName)) {
  $ticketCoveragePath += "&org_name=$([Uri]::EscapeDataString($OrgName))"
}
$ticketCoverageResponse = Try-Invoke-GitGovJson -Path $ticketCoveragePath -Label "Jira ticket coverage"
if (-not $ticketCoverageResponse.ok) {
  $warnings.Add([string]$ticketCoverageResponse.warning)
}
$ticketCoverage = $ticketCoverageResponse.value

$correlationsPath = "/integrations/jenkins/correlations?limit=$CorrelationLimit&offset=0"
$correlationsResponse = Try-Invoke-GitGovJson -Path $correlationsPath -Label "Jenkins correlations"
if (-not $correlationsResponse.ok) {
  $warnings.Add([string]$correlationsResponse.warning)
}
$correlationsPayload = $correlationsResponse.value
$correlations = @()
if ($null -ne $correlationsPayload -and $correlationsPayload.PSObject.Properties.Name -contains "correlations") {
  $correlations = @($correlationsPayload.correlations)
}

$githubPushesToday = [int](Get-Number -Value $stats.github_events.pushes_today)
$desktopPushesToday = [int](Get-Number -Value $stats.client_events.desktop_pushes_today)
$trackedPushesToday = $githubPushesToday + $desktopPushesToday
$blockedPushesToday = [int](Get-Number -Value $stats.client_events.blocked_today)
$pushAttempts = $trackedPushesToday + $blockedPushesToday

$trustedPathRate = if ($pushAttempts -gt 0) { Clamp-Percent ((100.0 * $trackedPushesToday) / $pushAttempts) } else { 100.0 }
$blockedPushRate = if ($pushAttempts -gt 0) { Clamp-Percent ((100.0 * $blockedPushesToday) / $pushAttempts) } else { 0.0 }

$ticketCoveragePercent = Clamp-Percent (Get-Number -Value $ticketCoverage.coverage_percentage)
$ticketTotalCommits = [int](Get-Number -Value $ticketCoverage.total_commits)
$ticketGapRate = Clamp-Percent (100.0 - $ticketCoveragePercent)

$pipelineTotal = [int](Get-Number -Value $stats.pipeline.total_7d)
$pipelineSuccess = [int](Get-Number -Value $stats.pipeline.success_7d)
$pipelineFailure = [int](Get-Number -Value $stats.pipeline.failure_7d)
$pipelineSuccessRate = if ($pipelineTotal -gt 0) { Clamp-Percent ((100.0 * $pipelineSuccess) / $pipelineTotal) } else { 0.0 }
$pipelineFailureRate = if ($pipelineTotal -gt 0) { Clamp-Percent ((100.0 * $pipelineFailure) / $pipelineTotal) } else { 0.0 }

$sonarRuns = @($correlations | Where-Object {
  $pipeline = $_.pipeline
  if ($null -eq $pipeline) { return $false }
  $jobName = [string]$pipeline.job_name
  return $jobName.ToLowerInvariant().Contains("sonar")
})
$sonarTotal = $sonarRuns.Count
$sonarPassed = @($sonarRuns | Where-Object { [string]$_.pipeline.status -eq "success" }).Count
$sonarFailed = @($sonarRuns | Where-Object { [string]$_.pipeline.status -eq "failure" }).Count
$sonarPassRate = if ($sonarTotal -gt 0) { Clamp-Percent ((100.0 * $sonarPassed) / $sonarTotal) } else { 0.0 }
$sonarFailureRate = if ($sonarTotal -gt 0) { Clamp-Percent ((100.0 * $sonarFailed) / $sonarTotal) } else { 0.0 }

$totalViolations = [int](Get-Number -Value $stats.violations.total)
$unresolvedViolations = [int](Get-Number -Value $stats.violations.unresolved)
$criticalViolations = [int](Get-Number -Value $stats.violations.critical)
$unresolvedViolationRate = if ($totalViolations -gt 0) { Clamp-Percent ((100.0 * $unresolvedViolations) / $totalViolations) } else { 0.0 }

$readiness = Compute-WeightedScore -Signals @(
  @{ value = $pipelineSuccessRate; weight = $profile.readiness.pipeline; available = ($pipelineTotal -gt 0) }
  @{ value = $ticketCoveragePercent; weight = $profile.readiness.traceability; available = ($ticketTotalCommits -gt 0) }
  @{ value = $sonarPassRate; weight = $profile.readiness.sonar; available = ($sonarTotal -gt 0) }
)

$readinessBand = if ($readiness.available -eq 0) {
  "Insuficiente"
} elseif ($readiness.score -ge [int]$profile.readiness.healthy) {
  "Fuerte"
} elseif ($readiness.score -ge [int]$profile.readiness.watch) {
  "Vigilancia"
} else {
  "Crítico"
}

$risk = Compute-WeightedScore -Signals @(
  @{ value = $blockedPushRate; weight = $profile.risk.blockedPush; available = ($pushAttempts -gt 0) }
  @{ value = $ticketGapRate; weight = $profile.risk.ticketGap; available = $true }
  @{ value = $pipelineFailureRate; weight = $profile.risk.pipelineFailure; available = ($pipelineTotal -gt 0) }
  @{ value = $sonarFailureRate; weight = $profile.risk.sonarFailure; available = ($sonarTotal -gt 0) }
  @{ value = $unresolvedViolationRate; weight = $profile.risk.unresolved; available = ($totalViolations -gt 0) }
)

$riskBand = if ($risk.available -eq 0) {
  "Insuficiente"
} elseif ($risk.score -ge [int]$profile.risk.mediumUpper) {
  "Alto"
} elseif ($risk.score -ge [int]$profile.risk.lowUpper) {
  "Medio"
} else {
  "Bajo"
}

$slaBreaches = New-Object System.Collections.Generic.List[string]
if ($blockedPushRate -gt [double]$profile.risk.sla.blockedPush) { $slaBreaches.Add("blocked_push_rate") }
if ($ticketGapRate -gt [double]$profile.risk.sla.ticketGap) { $slaBreaches.Add("traceability_gap") }
if ($pipelineFailureRate -gt [double]$profile.risk.sla.pipelineFailure) { $slaBreaches.Add("pipeline_failure_rate") }
if ($sonarTotal -gt 0 -and $sonarFailureRate -gt [double]$profile.risk.sla.sonarFailure) { $slaBreaches.Add("sonar_failure_rate") }
if ($totalViolations -gt 0 -and $unresolvedViolationRate -gt [double]$profile.risk.sla.unresolved) { $slaBreaches.Add("unresolved_violation_rate") }
if ($readiness.available -gt 0 -and $readiness.score -lt [int]$profile.risk.sla.readiness) { $slaBreaches.Add("release_readiness") }

$timestamp = [DateTimeOffset]::UtcNow
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
  $reportName = "risk-tier-baseline-$($timestamp.ToString("yyyyMMdd-HHmmss")).md"
  $OutputPath = Join-Path -Path "docs/reports" -ChildPath $reportName
}
$outputDirectory = Split-Path -Path $OutputPath -Parent
if (-not [string]::IsNullOrWhiteSpace($outputDirectory)) {
  New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
}

$warningMarkdown = if ($warnings.Count -gt 0) {
  ($warnings | ForEach-Object { "- $_" }) -join "`n"
} else {
  "- none"
}

$breachMarkdown = if ($slaBreaches.Count -gt 0) {
  ($slaBreaches | ForEach-Object { "- $_" }) -join "`n"
} else {
  "- none"
}

$report = @"
# Risk Tier Baseline Report

Generated (UTC): $($timestamp.ToString("yyyy-MM-dd HH:mm:ss"))

## Context

- Tier profile: $($profile.label) ($Tier)
- GitGov URL: $GitGovUrl
- Window hours: $Hours
- Org filter: $(if ([string]::IsNullOrWhiteSpace($OrgName)) { "none" } else { $OrgName })

## Composite Scores

- Release readiness: **$($readiness.score)/100** ($readinessBand, signals $($readiness.available)/$($readiness.total))
- Composite risk: **$($risk.score)/100** ($riskBand, signals $($risk.available)/$($risk.total))

## KPI Snapshot

- trusted_path_rate: $([Math]::Round($trustedPathRate, 1))%
- blocked_push_rate: $([Math]::Round($blockedPushRate, 1))%
- traceability_gap: $([Math]::Round($ticketGapRate, 1))%
- pipeline_failure_rate_7d: $([Math]::Round($pipelineFailureRate, 1))%
- sonar_failure_rate_sample: $(if ($sonarTotal -gt 0) { "$([Math]::Round($sonarFailureRate, 1))%" } else { "N/A" })
- unresolved_violation_rate: $(if ($totalViolations -gt 0) { "$([Math]::Round($unresolvedViolationRate, 1))%" } else { "N/A" })
- critical_violations: $criticalViolations

## SLA Targets ($($profile.label))

- readiness >= $($profile.risk.sla.readiness)
- blocked_push_rate <= $($profile.risk.sla.blockedPush)%
- traceability_gap <= $($profile.risk.sla.ticketGap)%
- pipeline_failure_rate <= $($profile.risk.sla.pipelineFailure)%
- sonar_failure_rate <= $($profile.risk.sla.sonarFailure)% (when sonar data exists)
- unresolved_violation_rate <= $($profile.risk.sla.unresolved)% (when violations data exists)

## SLA Breaches

$breachMarkdown

## Data Warnings

$warningMarkdown
"@

Set-Content -Path $OutputPath -Value $report -NoNewline

Write-Host "PASS: risk tier baseline report generated"
Write-Host "  tier:                 $($profile.label)"
Write-Host "  release readiness:    $($readiness.score)/100 ($readinessBand)"
Write-Host "  composite risk:       $($risk.score)/100 ($riskBand)"
Write-Host "  sla breaches:         $($slaBreaches.Count)"
Write-Host "  output:               $OutputPath"
