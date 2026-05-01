param(
  [string]$Repository = $env:GITHUB_REPOSITORY,
  [string]$WorkflowFile = "enterprise-onboarding-readiness.yml",
  [string]$ArtifactNamePrefix = "enterprise-onboarding-readiness-",
  [int]$MaxRuns = 30,
  [int]$MaxReports = 12,
  [string]$GitHubToken = $env:GITHUB_TOKEN,
  [string]$OutputMarkdownPath = "out/enterprise-onboarding-readiness-trend-report.md",
  [string]$OutputJsonPath = "out/enterprise-onboarding-readiness-trend-report.json"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Fail-Trend {
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

function Get-Number {
  param($Value)

  if ($null -eq $Value) {
    return 0
  }

  return [Math]::Max(0, [int]$Value)
}

if ([string]::IsNullOrWhiteSpace($Repository)) {
  Fail-Trend "Missing -Repository or GITHUB_REPOSITORY."
}

if ([string]::IsNullOrWhiteSpace($GitHubToken)) {
  Fail-Trend "Missing -GitHubToken or GITHUB_TOKEN."
}

if ($MaxRuns -le 0) {
  Fail-Trend "-MaxRuns must be greater than zero."
}

if ($MaxReports -le 0) {
  Fail-Trend "-MaxReports must be greater than zero."
}

foreach ($path in @($OutputMarkdownPath, $OutputJsonPath)) {
  $dir = Split-Path -Parent $path
  if (-not [string]::IsNullOrWhiteSpace($dir)) {
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
  }
}

$encodedWorkflow = [System.Uri]::EscapeDataString($WorkflowFile)
$runs = Invoke-GitHubApi -Path "/repos/$Repository/actions/workflows/$encodedWorkflow/runs?status=success&per_page=$MaxRuns"
$successfulRuns = @($runs.workflow_runs | Where-Object { $_.status -eq "completed" -and $_.conclusion -eq "success" } | Sort-Object -Property created_at -Descending)

if ($successfulRuns.Count -eq 0) {
  Fail-Trend "No successful completed runs found for workflow '$WorkflowFile' in '$Repository'."
}

$latestSuccessfulRun = $successfulRuns[0]
$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("gitgov-onboarding-readiness-trend-" + [System.Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $tempRoot | Out-Null

$records = New-Object System.Collections.Generic.List[object]
$skipped = New-Object System.Collections.Generic.List[object]

try {
  foreach ($run in $successfulRuns) {
    if ($records.Count -ge $MaxReports) {
      break
    }

    $artifacts = Invoke-GitHubApi -Path "/repos/$Repository/actions/runs/$($run.id)/artifacts?per_page=100"
    $artifact = @($artifacts.artifacts | Where-Object { [string]$_.name -like "$ArtifactNamePrefix*" } | Sort-Object -Property created_at -Descending | Select-Object -First 1)

    if ($artifact.Count -eq 0) {
      $skipped.Add([pscustomobject]@{ workflow_run_id = [int64]$run.id; reason = "missing_artifact" }) | Out-Null
      continue
    }

    $selectedArtifact = $artifact[0]
    if ($selectedArtifact.expired) {
      $skipped.Add([pscustomobject]@{ workflow_run_id = [int64]$run.id; artifact_id = [int64]$selectedArtifact.id; reason = "expired_artifact" }) | Out-Null
      continue
    }

    $runDir = Join-Path $tempRoot ([string]$run.id)
    New-Item -ItemType Directory -Force -Path $runDir | Out-Null
    $zipPath = Join-Path $runDir "artifact.zip"
    $expandPath = Join-Path $runDir "expanded"
    New-Item -ItemType Directory -Force -Path $expandPath | Out-Null

    Download-GitHubArtifact -DownloadUrl ([string]$selectedArtifact.archive_download_url) -OutputPath $zipPath
    Expand-Archive -Path $zipPath -DestinationPath $expandPath -Force

    $summaryFile = Get-ChildItem -Path $expandPath -Recurse -File -Filter "enterprise-onboarding-readiness.json" | Select-Object -First 1
    if ($null -eq $summaryFile) {
      $skipped.Add([pscustomobject]@{ workflow_run_id = [int64]$run.id; artifact_id = [int64]$selectedArtifact.id; reason = "missing_enterprise_onboarding_readiness_json" }) | Out-Null
      continue
    }

    $summary = Get-Content -Raw -LiteralPath $summaryFile.FullName | ConvertFrom-Json
    $stageCounts = Get-ObjectPropertyValue $summary "stage_counts"
    $releaseGovernance = Get-ObjectPropertyValue $summary "release_governance"
    $stages = @(Get-ObjectPropertyValue $summary "stages")

    $records.Add([pscustomobject]@{
      workflow_run_id                = [int64]$run.id
      workflow_run_url               = [string]$run.html_url
      workflow_created_at            = [string]$run.created_at
      artifact_id                    = [int64]$selectedArtifact.id
      artifact_name                  = [string]$selectedArtifact.name
      artifact_created_at            = [string]$selectedArtifact.created_at
      report_generated_at            = [string](Get-ObjectPropertyValue $summary "generated_at")
      customer_name                  = [string](Get-ObjectPropertyValue $summary "customer_name")
      repository_full_name           = [string](Get-ObjectPropertyValue $summary "repository_full_name")
      default_branch                 = [string](Get-ObjectPropertyValue $summary "default_branch")
      policy_preset                  = [string](Get-ObjectPropertyValue $summary "policy_preset")
      status                         = [string](Get-ObjectPropertyValue $summary "status")
      readiness_score                = Get-Number (Get-ObjectPropertyValue $summary "readiness_score")
      ready_count                    = Get-Number (Get-ObjectPropertyValue $stageCounts "ready")
      needs_action_count             = Get-Number (Get-ObjectPropertyValue $stageCounts "needs-action")
      blocked_count                  = Get-Number (Get-ObjectPropertyValue $stageCounts "blocked")
      stage_count                    = $stages.Count
      release_governance_mode        = [string](Get-ObjectPropertyValue $releaseGovernance "mode")
      release_governance_environment = [string](Get-ObjectPropertyValue $releaseGovernance "environment")
      release_governance_enforcement = [string](Get-ObjectPropertyValue $releaseGovernance "enforcement")
    }) | Out-Null
  }
} finally {
  Remove-Item -LiteralPath $tempRoot -Recurse -Force -ErrorAction SilentlyContinue
}

if ($records.Count -eq 0) {
  Fail-Trend "No parseable '$ArtifactNamePrefix*' artifacts found in the latest $MaxRuns successful '$WorkflowFile' runs."
}

$orderedRecords = @($records | Sort-Object -Property workflow_created_at -Descending)
$latest = $orderedRecords[0]
$oldest = $orderedRecords[$orderedRecords.Count - 1]
$scoreDelta = [int]$latest.readiness_score - [int]$oldest.readiness_score
$readyDelta = [int]$latest.ready_count - [int]$oldest.ready_count
$needsActionDelta = [int]$latest.needs_action_count - [int]$oldest.needs_action_count
$blockedDelta = [int]$latest.blocked_count - [int]$oldest.blocked_count
$runsReady = @($orderedRecords | Where-Object { $_.status -eq "ready" }).Count
$runsNeedsAction = @($orderedRecords | Where-Object { $_.status -eq "needs-action" }).Count
$runsBlocked = @($orderedRecords | Where-Object { $_.status -eq "blocked" }).Count
$generatedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
$latestSuccessfulRunId = [int64]$latestSuccessfulRun.id
$latestSuccessfulRunRecord = @($orderedRecords | Where-Object { $_.workflow_run_id -eq $latestSuccessfulRunId } | Select-Object -First 1)
$latestSuccessfulRunSkip = @($skipped | Where-Object { $_.workflow_run_id -eq $latestSuccessfulRunId } | Select-Object -First 1)
$latestSuccessfulRunArtifactStatus = if ($latestSuccessfulRunRecord.Count -gt 0) {
  "parsed"
} elseif ($latestSuccessfulRunSkip.Count -gt 0) {
  [string]$latestSuccessfulRunSkip[0].reason
} else {
  "not_scanned"
}

$trendDirection = if ($scoreDelta -gt 0) {
  "improving"
} elseif ($scoreDelta -lt 0) {
  "declining"
} else {
  "stable"
}

$summaryOutput = [pscustomobject]@{
  generated_at                             = $generatedAtUtc
  repository                               = $Repository
  workflow_file                            = $WorkflowFile
  artifact_name_prefix                     = $ArtifactNamePrefix
  reports_analyzed                         = $orderedRecords.Count
  successful_runs_scanned                  = $successfulRuns.Count
  latest_successful_run_id                 = $latestSuccessfulRunId
  latest_successful_run_artifact_status    = $latestSuccessfulRunArtifactStatus
  skipped_artifacts                        = @($skipped.ToArray())
  latest_status                            = $latest.status
  latest_score                             = $latest.readiness_score
  latest_ready_count                       = $latest.ready_count
  latest_needs_action_count                = $latest.needs_action_count
  latest_blocked_count                     = $latest.blocked_count
  score_delta_vs_oldest                    = $scoreDelta
  ready_delta_vs_oldest                    = $readyDelta
  needs_action_delta_vs_oldest             = $needsActionDelta
  blocked_delta_vs_oldest                  = $blockedDelta
  runs_ready                               = $runsReady
  runs_needs_action                        = $runsNeedsAction
  runs_blocked                             = $runsBlocked
  trend_direction                          = $trendDirection
  release_blocking_default                 = $false
  reports                                  = @($orderedRecords)
}

$summaryOutput | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $OutputJsonPath -Encoding UTF8

$markdown = New-Object System.Collections.Generic.List[string]
$markdown.Add("# Enterprise Onboarding Readiness Trend Report") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add(('Generated: `{0}`' -f $generatedAtUtc)) | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add(('Repository: `{0}`' -f $Repository)) | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("## Executive Summary") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add(('- Latest status: `{0}`' -f $latest.status)) | Out-Null
$markdown.Add(('- Latest readiness score: `{0}`' -f $latest.readiness_score)) | Out-Null
$markdown.Add(('- Latest stage counts: `{0}` ready, `{1}` needs-action, `{2}` blocked' -f $latest.ready_count, $latest.needs_action_count, $latest.blocked_count)) | Out-Null
$markdown.Add(('- Reports analyzed: `{0}`' -f $orderedRecords.Count)) | Out-Null
$markdown.Add(('- Latest successful workflow run: `{0}`' -f $latestSuccessfulRunId)) | Out-Null
$markdown.Add(('- Latest successful run artifact status: `{0}`' -f $latestSuccessfulRunArtifactStatus)) | Out-Null
$markdown.Add(('- Trend direction: `{0}`' -f $trendDirection)) | Out-Null
$markdown.Add(('- Score delta vs oldest report: `{0}`' -f $scoreDelta)) | Out-Null
$markdown.Add(('- Ready-stage delta vs oldest report: `{0}`' -f $readyDelta)) | Out-Null
$markdown.Add(('- Needs-action-stage delta vs oldest report: `{0}`' -f $needsActionDelta)) | Out-Null
$markdown.Add(('- Blocked-stage delta vs oldest report: `{0}`' -f $blockedDelta)) | Out-Null
$markdown.Add(('- Runs ready: `{0}`' -f $runsReady)) | Out-Null
$markdown.Add(('- Runs needs-action: `{0}`' -f $runsNeedsAction)) | Out-Null
$markdown.Add(('- Runs blocked: `{0}`' -f $runsBlocked)) | Out-Null

$markdown.Add("") | Out-Null
$markdown.Add("## Report History") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("| Run | Report generated | Customer | Target repo | Status | Score | Ready | Needs-action | Blocked | Artifact |") | Out-Null
$markdown.Add("|---:|---|---|---|---|---:|---:|---:|---:|---|") | Out-Null
foreach ($record in $orderedRecords) {
  $markdown.Add(('| [{0}]({1}) | `{2}` | {3} | `{4}` | `{5}` | {6} | {7} | {8} | {9} | `{10}` |' -f $record.workflow_run_id, $record.workflow_run_url, $record.report_generated_at, (Escape-MarkdownCell $record.customer_name), (Escape-MarkdownCell $record.repository_full_name), (Escape-MarkdownCell $record.status), $record.readiness_score, $record.ready_count, $record.needs_action_count, $record.blocked_count, (Escape-MarkdownCell $record.artifact_name))) | Out-Null
}

if ($skipped.Count -gt 0) {
  $markdown.Add("") | Out-Null
  $markdown.Add("## Skipped Runs") | Out-Null
  $markdown.Add("") | Out-Null
  $markdown.Add("| Run | Reason |") | Out-Null
  $markdown.Add("|---:|---|") | Out-Null
  foreach ($item in $skipped) {
    $markdown.Add(('| {0} | `{1}` |' -f $item.workflow_run_id, $item.reason)) | Out-Null
  }
}

$markdown.Add("") | Out-Null
$markdown.Add("## Interpretation") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("- Trend data comes from Enterprise Onboarding Readiness artifacts, not from provider secrets.") | Out-Null
$markdown.Add("- needs-action means onboarding setup or evidence is incomplete; it is not a release-blocking result by default.") | Out-Null
$markdown.Add("- A declining score or new blocked stage should be triaged before customer production onboarding continues.") | Out-Null
$markdown.Add("- Release blocking remains a customer-selected policy, not a default of this trend report.") | Out-Null

Set-Content -LiteralPath $OutputMarkdownPath -Value $markdown -Encoding UTF8
Write-Host "Wrote enterprise onboarding readiness trend report: $OutputMarkdownPath"
Write-Host "Wrote enterprise onboarding readiness trend JSON: $OutputJsonPath"
