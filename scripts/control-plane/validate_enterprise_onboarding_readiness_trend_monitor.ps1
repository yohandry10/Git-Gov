param(
  [string]$Repository = $env:GITHUB_REPOSITORY,
  [string]$WorkflowFile = "enterprise-onboarding-readiness-trend-report.yml",
  [string]$ArtifactName = "enterprise-onboarding-readiness-trend-report",
  [string]$TrendJsonFileName = "enterprise-onboarding-readiness-trend-report.json",
  [int]$MaxAgeHours = 192,
  [int]$MinLatestScore = 75,
  [string]$GitHubToken = $env:GITHUB_TOKEN,
  [string]$OutputMarkdownPath = "out/enterprise-onboarding-readiness-trend-monitor.md",
  [string]$OutputJsonPath = "out/enterprise-onboarding-readiness-trend-monitor.json",
  [switch]$ReportOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Fail-Monitor {
  param([Parameter(Mandatory = $true)][string]$Message)
  Write-Host "[FAIL] $Message"
  exit 1
}

function Invoke-GitHubApi {
  param([Parameter(Mandatory = $true)][string]$Path)

  $headers = @{
    Authorization          = "Bearer $GitHubToken"
    Accept                 = "application/vnd.github+json"
    "X-GitHub-Api-Version" = "2022-11-28"
  }

  return Invoke-RestMethod -Uri "https://api.github.com$Path" -Headers $headers -Method GET
}

function Download-GitHubArtifact {
  param(
    [Parameter(Mandatory = $true)][string]$DownloadUrl,
    [Parameter(Mandatory = $true)][string]$OutputPath
  )

  $headers = @{
    Authorization          = "Bearer $GitHubToken"
    Accept                 = "application/vnd.github+json"
    "X-GitHub-Api-Version" = "2022-11-28"
  }

  Invoke-WebRequest -Uri $DownloadUrl -Headers $headers -OutFile $OutputPath | Out-Null
}

function Add-Finding {
  param(
    [Parameter(Mandatory = $true)]$List,
    [Parameter(Mandatory = $true)][string]$Severity,
    [Parameter(Mandatory = $true)][string]$Code,
    [Parameter(Mandatory = $true)][string]$Message,
    [string]$Evidence = ""
  )

  $List.Add([pscustomobject]@{
    severity = $Severity
    code     = $Code
    message  = $Message
    evidence = $Evidence
  }) | Out-Null
}

function Escape-MarkdownCell {
  param([string]$Value)
  if ([string]::IsNullOrEmpty($Value)) {
    return ""
  }
  return ($Value -replace '\|', '\|')
}

function Get-ObjectPropertyValue {
  param(
    [Parameter(Mandatory = $true)]$Object,
    [Parameter(Mandatory = $true)][string]$Name
  )

  if ($null -eq $Object) {
    return $null
  }

  $property = $Object.PSObject.Properties[$Name]
  if ($null -eq $property) {
    return $null
  }

  return $property.Value
}

function Get-NumberOrNull {
  param($Value)

  if ($null -eq $Value -or [string]::IsNullOrWhiteSpace([string]$Value)) {
    return $null
  }

  return [int64]$Value
}

if ([string]::IsNullOrWhiteSpace($Repository)) {
  Fail-Monitor "Missing -Repository or GITHUB_REPOSITORY."
}

if ([string]::IsNullOrWhiteSpace($GitHubToken)) {
  Fail-Monitor "Missing -GitHubToken or GITHUB_TOKEN."
}

if ($MaxAgeHours -le 0) {
  Fail-Monitor "-MaxAgeHours must be greater than zero."
}

if ($MinLatestScore -lt 0 -or $MinLatestScore -gt 100) {
  Fail-Monitor "-MinLatestScore must be between 0 and 100."
}

foreach ($path in @($OutputMarkdownPath, $OutputJsonPath)) {
  $dir = Split-Path -Parent $path
  if (-not [string]::IsNullOrWhiteSpace($dir)) {
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
  }
}

$generatedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
$findings = New-Object System.Collections.Generic.List[object]
$latestRun = $null
$selectedArtifact = $null
$artifactAgeHours = $null
$trendSummary = $null
$trendJsonFound = $false

$encodedWorkflow = [System.Uri]::EscapeDataString($WorkflowFile)
$runs = Invoke-GitHubApi -Path "/repos/$Repository/actions/workflows/$encodedWorkflow/runs?status=success&per_page=10"
$successfulRuns = @($runs.workflow_runs | Where-Object { $_.status -eq "completed" -and $_.conclusion -eq "success" } | Sort-Object -Property created_at -Descending)

if ($successfulRuns.Count -eq 0) {
  Add-Finding -List $findings -Severity "blocked" -Code "missing-successful-trend-run" -Message "No successful completed trend workflow run was found." -Evidence $WorkflowFile
} else {
  $latestRun = $successfulRuns[0]
  $artifacts = Invoke-GitHubApi -Path "/repos/$Repository/actions/runs/$($latestRun.id)/artifacts?per_page=100"
  $artifactMatches = @($artifacts.artifacts | Where-Object { $_.name -eq $ArtifactName } | Sort-Object -Property created_at -Descending)

  if ($artifactMatches.Count -eq 0) {
    Add-Finding -List $findings -Severity "blocked" -Code "missing-trend-artifact" -Message "Latest successful trend workflow run does not contain the expected artifact." -Evidence ("run={0}; artifact={1}" -f $latestRun.id, $ArtifactName)
  } else {
    $selectedArtifact = $artifactMatches[0]

    if ($selectedArtifact.expired) {
      Add-Finding -List $findings -Severity "blocked" -Code "expired-trend-artifact" -Message "The latest trend artifact is expired." -Evidence ("run={0}; artifact_id={1}" -f $latestRun.id, $selectedArtifact.id)
    }

    $now = (Get-Date).ToUniversalTime()
    $artifactCreatedAt = [DateTime]::Parse([string]$selectedArtifact.created_at).ToUniversalTime()
    $artifactAgeHours = [math]::Round(($now - $artifactCreatedAt).TotalHours, 2)

    if ($artifactAgeHours -gt $MaxAgeHours) {
      Add-Finding -List $findings -Severity "blocked" -Code "stale-trend-artifact" -Message "The latest trend artifact is older than the accepted freshness window." -Evidence ("age_hours={0}; max_age_hours={1}" -f $artifactAgeHours, $MaxAgeHours)
    }

    if (-not $selectedArtifact.expired) {
      $tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("gitgov-onboarding-readiness-trend-monitor-" + [System.Guid]::NewGuid().ToString("N"))
      New-Item -ItemType Directory -Force -Path $tempRoot | Out-Null

      try {
        $zipPath = Join-Path $tempRoot "artifact.zip"
        $expandPath = Join-Path $tempRoot "expanded"
        New-Item -ItemType Directory -Force -Path $expandPath | Out-Null

        Download-GitHubArtifact -DownloadUrl ([string]$selectedArtifact.archive_download_url) -OutputPath $zipPath
        Expand-Archive -Path $zipPath -DestinationPath $expandPath -Force

        $trendFile = Get-ChildItem -Path $expandPath -Recurse -File -Filter $TrendJsonFileName | Select-Object -First 1
        if ($null -eq $trendFile) {
          Add-Finding -List $findings -Severity "blocked" -Code "missing-trend-json" -Message "The trend artifact did not contain the expected trend JSON file." -Evidence $TrendJsonFileName
        } else {
          $trendJsonFound = $true
          $trendSummary = Get-Content -Raw -LiteralPath $trendFile.FullName | ConvertFrom-Json
        }
      } finally {
        Remove-Item -LiteralPath $tempRoot -Recurse -Force -ErrorAction SilentlyContinue
      }
    }
  }
}

$latestStatus = $null
$latestScore = $null
$latestReadyCount = $null
$latestNeedsActionCount = $null
$latestBlockedCount = $null
$scoreDelta = $null
$blockedDelta = $null
$trendDirection = $null
$latestSuccessfulRunArtifactStatus = $null
$reportsAnalyzed = $null
$sourceLatestSuccessfulRunId = $null

if ($null -ne $trendSummary) {
  $latestStatus = [string](Get-ObjectPropertyValue $trendSummary "latest_status")
  $latestScore = Get-NumberOrNull (Get-ObjectPropertyValue $trendSummary "latest_score")
  $latestReadyCount = Get-NumberOrNull (Get-ObjectPropertyValue $trendSummary "latest_ready_count")
  $latestNeedsActionCount = Get-NumberOrNull (Get-ObjectPropertyValue $trendSummary "latest_needs_action_count")
  $latestBlockedCount = Get-NumberOrNull (Get-ObjectPropertyValue $trendSummary "latest_blocked_count")
  $scoreDelta = Get-NumberOrNull (Get-ObjectPropertyValue $trendSummary "score_delta_vs_oldest")
  $blockedDelta = Get-NumberOrNull (Get-ObjectPropertyValue $trendSummary "blocked_delta_vs_oldest")
  $trendDirection = [string](Get-ObjectPropertyValue $trendSummary "trend_direction")
  $latestSuccessfulRunArtifactStatus = [string](Get-ObjectPropertyValue $trendSummary "latest_successful_run_artifact_status")
  $reportsAnalyzed = Get-NumberOrNull (Get-ObjectPropertyValue $trendSummary "reports_analyzed")
  $sourceLatestSuccessfulRunId = Get-NumberOrNull (Get-ObjectPropertyValue $trendSummary "latest_successful_run_id")

  if ($latestSuccessfulRunArtifactStatus -ne "parsed") {
    Add-Finding -List $findings -Severity "blocked" -Code "latest-readiness-artifact-not-parsed" -Message "The trend report could not parse the latest successful onboarding readiness artifact." -Evidence ("status={0}; source_run={1}" -f $latestSuccessfulRunArtifactStatus, $sourceLatestSuccessfulRunId)
  }

  if ($latestStatus -eq "blocked") {
    Add-Finding -List $findings -Severity "needs-action" -Code "latest-readiness-blocked" -Message "The latest onboarding readiness report is blocked." -Evidence ("source_run={0}" -f $sourceLatestSuccessfulRunId)
  }

  if ($null -ne $latestBlockedCount -and $latestBlockedCount -gt 0) {
    Add-Finding -List $findings -Severity "needs-action" -Code "blocked-stage-present" -Message "The latest readiness report contains one or more blocked stages." -Evidence ("blocked_count={0}" -f $latestBlockedCount)
  }

  if ($null -ne $latestScore -and $latestScore -lt $MinLatestScore) {
    Add-Finding -List $findings -Severity "needs-action" -Code "latest-score-below-threshold" -Message "The latest readiness score is below the configured threshold." -Evidence ("latest_score={0}; min_latest_score={1}" -f $latestScore, $MinLatestScore)
  }

  if ($null -ne $scoreDelta -and $scoreDelta -lt 0) {
    Add-Finding -List $findings -Severity "needs-action" -Code "readiness-score-declined" -Message "The readiness score declined compared with the oldest analyzed report." -Evidence ("score_delta_vs_oldest={0}" -f $scoreDelta)
  }

  if ($null -ne $blockedDelta -and $blockedDelta -gt 0) {
    Add-Finding -List $findings -Severity "needs-action" -Code "blocked-stage-count-increased" -Message "The number of blocked stages increased compared with the oldest analyzed report." -Evidence ("blocked_delta_vs_oldest={0}" -f $blockedDelta)
  }
}

$hasBlockedFinding = @($findings.ToArray() | Where-Object { $_.severity -eq "blocked" }).Count -gt 0
$monitorStatus = if ($hasBlockedFinding) {
  "blocked"
} elseif ($findings.Count -gt 0) {
  "needs-action"
} else {
  "ready"
}

$summary = [pscustomobject]@{
  generated_at          = $generatedAtUtc
  status                = $monitorStatus
  report_only           = [bool]$ReportOnly
  release_blocking_default = $false
  repository            = $Repository
  workflow_file         = $WorkflowFile
  workflow_run_id       = if ($null -ne $latestRun) { [int64]$latestRun.id } else { $null }
  workflow_run_url      = if ($null -ne $latestRun) { [string]$latestRun.html_url } else { $null }
  workflow_run_created_at = if ($null -ne $latestRun) { [string]$latestRun.created_at } else { $null }
  artifact_name         = if ($null -ne $selectedArtifact) { [string]$selectedArtifact.name } else { $ArtifactName }
  artifact_id           = if ($null -ne $selectedArtifact) { [int64]$selectedArtifact.id } else { $null }
  artifact_created_at   = if ($null -ne $selectedArtifact) { [string]$selectedArtifact.created_at } else { $null }
  artifact_expired      = if ($null -ne $selectedArtifact) { [bool]$selectedArtifact.expired } else { $null }
  artifact_age_hours    = $artifactAgeHours
  max_age_hours         = $MaxAgeHours
  min_latest_score      = $MinLatestScore
  trend_json_found      = $trendJsonFound
  trend                 = [pscustomobject]@{
    latest_status                         = $latestStatus
    latest_score                          = $latestScore
    latest_ready_count                    = $latestReadyCount
    latest_needs_action_count             = $latestNeedsActionCount
    latest_blocked_count                  = $latestBlockedCount
    score_delta_vs_oldest                 = $scoreDelta
    blocked_delta_vs_oldest               = $blockedDelta
    trend_direction                       = $trendDirection
    latest_successful_run_artifact_status = $latestSuccessfulRunArtifactStatus
    latest_successful_run_id              = $sourceLatestSuccessfulRunId
    reports_analyzed                      = $reportsAnalyzed
  }
  findings              = @($findings.ToArray())
}

$summary | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $OutputJsonPath -Encoding UTF8

$markdown = New-Object System.Collections.Generic.List[string]
$markdown.Add("# Enterprise Onboarding Readiness Trend Monitor") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add(('Generated: `{0}`' -f $generatedAtUtc)) | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add(('Repository: `{0}`' -f $Repository)) | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("## Executive Summary") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add(('- Monitor status: `{0}`' -f $monitorStatus)) | Out-Null
$markdown.Add(('- Report-only mode: `{0}`' -f [bool]$ReportOnly)) | Out-Null
$markdown.Add(('- Release blocking by default: `{0}`' -f $false)) | Out-Null
$markdown.Add(('- Trend workflow run: `{0}`' -f $(if ($null -ne $latestRun) { $latestRun.id } else { "not-found" }))) | Out-Null
$markdown.Add(('- Trend artifact: `{0}`' -f $(if ($null -ne $selectedArtifact) { $selectedArtifact.name } else { $ArtifactName }))) | Out-Null
$markdown.Add(('- Artifact age hours: `{0}` / max `{1}`' -f $(if ($null -ne $artifactAgeHours) { $artifactAgeHours } else { "unknown" }), $MaxAgeHours)) | Out-Null
$markdown.Add(('- Latest readiness status: `{0}`' -f $(if (-not [string]::IsNullOrWhiteSpace($latestStatus)) { $latestStatus } else { "unknown" }))) | Out-Null
$markdown.Add(('- Latest readiness score: `{0}` / min `{1}`' -f $(if ($null -ne $latestScore) { $latestScore } else { "unknown" }), $MinLatestScore)) | Out-Null
$markdown.Add(('- Trend direction: `{0}`' -f $(if (-not [string]::IsNullOrWhiteSpace($trendDirection)) { $trendDirection } else { "unknown" }))) | Out-Null
$markdown.Add(('- Latest source artifact status: `{0}`' -f $(if (-not [string]::IsNullOrWhiteSpace($latestSuccessfulRunArtifactStatus)) { $latestSuccessfulRunArtifactStatus } else { "unknown" }))) | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("## Findings") | Out-Null
$markdown.Add("") | Out-Null
if ($findings.Count -eq 0) {
  $markdown.Add("No monitor findings.") | Out-Null
} else {
  $markdown.Add("| Severity | Code | Message | Evidence |") | Out-Null
  $markdown.Add("|---|---|---|---|") | Out-Null
  foreach ($finding in $findings) {
    $markdown.Add(('| `{0}` | `{1}` | {2} | `{3}` |' -f (Escape-MarkdownCell $finding.severity), (Escape-MarkdownCell $finding.code), (Escape-MarkdownCell $finding.message), (Escape-MarkdownCell $finding.evidence))) | Out-Null
  }
}

$markdown.Add("") | Out-Null
$markdown.Add("## Trend Snapshot") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("| Field | Value |") | Out-Null
$markdown.Add("|---|---|") | Out-Null
$markdown.Add(('| Latest status | `{0}` |' -f $(if (-not [string]::IsNullOrWhiteSpace($latestStatus)) { $latestStatus } else { "unknown" }))) | Out-Null
$markdown.Add(('| Latest score | `{0}` |' -f $(if ($null -ne $latestScore) { $latestScore } else { "unknown" }))) | Out-Null
$markdown.Add(('| Ready stages | `{0}` |' -f $(if ($null -ne $latestReadyCount) { $latestReadyCount } else { "unknown" }))) | Out-Null
$markdown.Add(('| Needs-action stages | `{0}` |' -f $(if ($null -ne $latestNeedsActionCount) { $latestNeedsActionCount } else { "unknown" }))) | Out-Null
$markdown.Add(('| Blocked stages | `{0}` |' -f $(if ($null -ne $latestBlockedCount) { $latestBlockedCount } else { "unknown" }))) | Out-Null
$markdown.Add(('| Score delta vs oldest | `{0}` |' -f $(if ($null -ne $scoreDelta) { $scoreDelta } else { "unknown" }))) | Out-Null
$markdown.Add(('| Blocked delta vs oldest | `{0}` |' -f $(if ($null -ne $blockedDelta) { $blockedDelta } else { "unknown" }))) | Out-Null
$markdown.Add(('| Reports analyzed | `{0}` |' -f $(if ($null -ne $reportsAnalyzed) { $reportsAnalyzed } else { "unknown" }))) | Out-Null

$markdown.Add("") | Out-Null
$markdown.Add("## Interpretation") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add('- `ready` means the latest trend artifact is fresh and no deterioration rule fired.') | Out-Null
$markdown.Add('- `needs-action` means customer onboarding evidence changed in a way an operator should review.') | Out-Null
$markdown.Add('- `blocked` means the monitor could not trust the trend evidence, usually because the artifact is missing, stale, expired, or not parseable.') | Out-Null
$markdown.Add("- This monitor is report-only by default and does not make release governance blocking unless an operator explicitly runs it without report-only mode.") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("## Safety") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("- Reads GitHub Actions run and artifact metadata only.") | Out-Null
$markdown.Add("- Downloads the sanitized trend artifact emitted by GitGov workflows.") | Out-Null
$markdown.Add('- Does not read `.env` files or provider tokens.') | Out-Null
$markdown.Add("- Does not print Authorization headers.") | Out-Null
$markdown.Add("- Does not create or update GitHub Actions variables or secrets.") | Out-Null
$markdown.Add("- Does not mutate customer repositories, branch protection, workflows, or provider settings.") | Out-Null

Set-Content -LiteralPath $OutputMarkdownPath -Value $markdown -Encoding UTF8
Write-Host "Wrote enterprise onboarding readiness trend monitor: $OutputMarkdownPath"
Write-Host "Wrote enterprise onboarding readiness trend monitor JSON: $OutputJsonPath"

if (-not $ReportOnly -and $monitorStatus -ne "ready") {
  foreach ($finding in $findings) {
    Write-Host ("[FAIL] {0}: {1}" -f $finding.code, $finding.message)
  }
  exit 1
}

Write-Host ("[{0}] Enterprise onboarding readiness trend monitor completed. report_only={1}" -f $monitorStatus.ToUpperInvariant(), [bool]$ReportOnly)
exit 0
