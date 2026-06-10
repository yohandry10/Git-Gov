param(
  [string]$GitGovUrl = "http://127.0.0.1:3001",
  [string]$ApiKey,
  [string]$RepoFullName = "",
  [string]$Branch = "main",
  [ValidateSet("critical", "standard", "internal")][string]$Tier = "standard",
  [string]$OrgName = "",
  [int]$Hours = 168,
  [int]$CorrelationLimit = 500,
  [int]$MinReadiness = 0,
  [switch]$FailOnMissingSignals,
  [string]$OutputPath = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
. (Join-Path $scriptRoot "..\github\_token_helpers.ps1")

if ([string]::IsNullOrWhiteSpace($RepoFullName)) {
  $repoInfo = Resolve-GitHubRepoCoordinates -ScriptRoot $scriptRoot
  if (-not [string]::IsNullOrWhiteSpace($repoInfo.Owner) -and -not [string]::IsNullOrWhiteSpace($repoInfo.Repo)) {
    $RepoFullName = "$($repoInfo.Owner)/$($repoInfo.Repo)"
  }
}
if ([string]::IsNullOrWhiteSpace($RepoFullName)) {
  Write-Error "Missing -RepoFullName and repository coordinates could not be auto-resolved."
  exit 1
}

if ([string]::IsNullOrWhiteSpace($ApiKey)) {
  Write-Error "Missing -ApiKey."
  exit 1
}

if ([string]::IsNullOrWhiteSpace($RepoFullName)) {
  Write-Error "Missing -RepoFullName."
  exit 1
}

if ([string]::IsNullOrWhiteSpace($Branch)) {
  Write-Error "Missing -Branch."
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
      $payload = $reader.ReadToEnd()
      throw "HTTP error calling $uri -> $payload"
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
    risk = @{ sla = @{ readiness = 85 } }
  }
  standard = @{
    label = "Standard"
    readiness = @{ pipeline = 0.45; traceability = 0.25; sonar = 0.3; healthy = 85; watch = 70 }
    risk = @{ sla = @{ readiness = 75 } }
  }
  internal = @{
    label = "Internal"
    readiness = @{ pipeline = 0.4; traceability = 0.2; sonar = 0.4; healthy = 80; watch = 65 }
    risk = @{ sla = @{ readiness = 65 } }
  }
}

$profile = $profiles[$Tier]
$warnings = New-Object System.Collections.Generic.List[string]

$encodedRepo = [Uri]::EscapeDataString($RepoFullName)
$encodedBranch = [Uri]::EscapeDataString($Branch)

$ticketCoveragePath = "/integrations/jira/ticket-coverage?repo_full_name=$encodedRepo&branch=$encodedBranch&hours=$Hours"
if (-not [string]::IsNullOrWhiteSpace($OrgName)) {
  $ticketCoveragePath += "&org_name=$([Uri]::EscapeDataString($OrgName))"
}
$ticketCoverageResponse = Try-Invoke-GitGovJson -Path $ticketCoveragePath -Label "Jira ticket coverage"
if (-not $ticketCoverageResponse.ok) {
  $warnings.Add([string]$ticketCoverageResponse.warning)
}
$ticketCoverage = $ticketCoverageResponse.value

$correlationsPath = "/integrations/jenkins/correlations?repo_full_name=$encodedRepo&branch=$encodedBranch&limit=$CorrelationLimit&offset=0"
$correlationsResponse = Try-Invoke-GitGovJson -Path $correlationsPath -Label "Jenkins correlations"
if (-not $correlationsResponse.ok) {
  $warnings.Add([string]$correlationsResponse.warning)
}
$correlationsPayload = $correlationsResponse.value

$correlations = @()
if ($null -ne $correlationsPayload -and $correlationsPayload.PSObject.Properties.Name -contains "correlations") {
  $correlations = @($correlationsPayload.correlations)
}

$pipelineRuns = @(
  $correlations | Where-Object {
    $null -ne $_.pipeline -and -not [string]::IsNullOrWhiteSpace([string]$_.pipeline.status)
  } | ForEach-Object { $_.pipeline }
)

$pipelineTotal = $pipelineRuns.Count
$pipelineSuccess = @($pipelineRuns | Where-Object { ([string]$_.status).Trim().ToLowerInvariant() -eq "success" }).Count
$pipelineFailure = @($pipelineRuns | Where-Object { ([string]$_.status).Trim().ToLowerInvariant() -eq "failure" }).Count
$pipelineSuccessRate = if ($pipelineTotal -gt 0) { Clamp-Percent ((100.0 * $pipelineSuccess) / $pipelineTotal) } else { 0.0 }
$pipelineFailureRate = if ($pipelineTotal -gt 0) { Clamp-Percent ((100.0 * $pipelineFailure) / $pipelineTotal) } else { 0.0 }

$sonarRuns = @($pipelineRuns | Where-Object {
  $jobName = [string]$_.job_name
  -not [string]::IsNullOrWhiteSpace($jobName) -and $jobName.ToLowerInvariant().Contains("sonar")
})
$sonarTotal = $sonarRuns.Count
$sonarPassed = @($sonarRuns | Where-Object { ([string]$_.status).Trim().ToLowerInvariant() -eq "success" }).Count
$sonarFailed = @($sonarRuns | Where-Object { ([string]$_.status).Trim().ToLowerInvariant() -eq "failure" }).Count
$sonarPassRate = if ($sonarTotal -gt 0) { Clamp-Percent ((100.0 * $sonarPassed) / $sonarTotal) } else { 0.0 }
$sonarFailureRate = if ($sonarTotal -gt 0) { Clamp-Percent ((100.0 * $sonarFailed) / $sonarTotal) } else { 0.0 }

$ticketCoveragePercent = 0.0
$ticketTotalCommits = 0
$ticketUnverifiedPercent = 0.0
$ticketUnverifiedCommits = 0
if ($null -ne $ticketCoverage) {
  # coverage_percentage is verified-only: a detected ticket id counts only when
  # it matches an ingested Jira ticket. Pattern-only matches are reported
  # separately and never raise readiness.
  if ($ticketCoverage.PSObject.Properties.Name -contains "coverage_percentage") {
    $ticketCoveragePercent = Clamp-Percent (Get-Number -Value $ticketCoverage.coverage_percentage)
  }
  if ($ticketCoverage.PSObject.Properties.Name -contains "total_commits") {
    $ticketTotalCommits = [int](Get-Number -Value $ticketCoverage.total_commits)
  }
  if ($ticketCoverage.PSObject.Properties.Name -contains "unverified_coverage_percentage") {
    $ticketUnverifiedPercent = Clamp-Percent (Get-Number -Value $ticketCoverage.unverified_coverage_percentage)
  }
  if ($ticketCoverage.PSObject.Properties.Name -contains "detected_unverified_commits") {
    $ticketUnverifiedCommits = [int](Get-Number -Value $ticketCoverage.detected_unverified_commits)
  }
}

if ($ticketUnverifiedCommits -gt 0) {
  $warnings.Add("jira_ticket_coverage_has_$($ticketUnverifiedCommits)_unverified_detections") | Out-Null
}

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

$targetReadiness = if ($MinReadiness -gt 0) {
  [Math]::Max(0, [Math]::Min(100, $MinReadiness))
} else {
  [int]$profile.risk.sla.readiness
}

$failReasons = New-Object System.Collections.Generic.List[string]
if ($readiness.available -eq 0) {
  $failReasons.Add("no_release_readiness_signals")
}
if ($FailOnMissingSignals -and $readiness.available -lt $readiness.total) {
  $failReasons.Add("missing_signals_strict_mode")
}
if ($readiness.available -gt 0 -and $readiness.score -lt $targetReadiness) {
  $failReasons.Add("readiness_below_target")
}

$passed = $failReasons.Count -eq 0

$result = [ordered]@{
  passed = $passed
  repo_full_name = $RepoFullName
  branch = $Branch
  tier = $Tier
  target_readiness = $targetReadiness
  readiness_score = $readiness.score
  readiness_band = $readinessBand
  signal_coverage = "$($readiness.available)/$($readiness.total)"
  metrics = @{
    pipeline_total = $pipelineTotal
    pipeline_success_rate = [Math]::Round($pipelineSuccessRate, 2)
    pipeline_failure_rate = [Math]::Round($pipelineFailureRate, 2)
    jira_ticket_coverage = [Math]::Round($ticketCoveragePercent, 2)
    jira_ticket_coverage_unverified = [Math]::Round($ticketUnverifiedPercent, 2)
    jira_ticket_unverified_commits = $ticketUnverifiedCommits
    sonar_total = $sonarTotal
    sonar_pass_rate = [Math]::Round($sonarPassRate, 2)
    sonar_failure_rate = [Math]::Round($sonarFailureRate, 2)
  }
  fail_reasons = @($failReasons)
  warnings = @($warnings)
  generated_at = [DateTimeOffset]::UtcNow.ToString("o")
}

if (-not [string]::IsNullOrWhiteSpace($OutputPath)) {
  $outputDirectory = Split-Path -Path $OutputPath -Parent
  if (-not [string]::IsNullOrWhiteSpace($outputDirectory)) {
    New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null
  }
  ($result | ConvertTo-Json -Depth 10) | Out-File -FilePath $OutputPath -Encoding UTF8
}

Write-Host "Release readiness gate:"
Write-Host "  repo:            $RepoFullName"
Write-Host "  branch:          $Branch"
Write-Host "  tier:            $($profile.label)"
Write-Host "  readiness:       $($readiness.score) ($readinessBand)"
Write-Host "  target:          $targetReadiness"
Write-Host "  signal coverage: $($readiness.available)/$($readiness.total)"
Write-Host "  pipeline:        total=$pipelineTotal success_rate=$([Math]::Round($pipelineSuccessRate,2))%"
Write-Host "  jira coverage:   $([Math]::Round($ticketCoveragePercent,2))% verified (unverified $([Math]::Round($ticketUnverifiedPercent,2))% / $ticketUnverifiedCommits commits)"
Write-Host "  sonar:           total=$sonarTotal pass_rate=$([Math]::Round($sonarPassRate,2))%"

if ($warnings.Count -gt 0) {
  Write-Host "Warnings:"
  $warnings | ForEach-Object { Write-Host "  - $_" }
}

if (-not $passed) {
  Write-Host "FAIL reasons:"
  $failReasons | ForEach-Object { Write-Host "  - $_" }
  exit 1
}

Write-Host "PASS: release readiness gate satisfied."
exit 0
