param(
  [string]$Repository = $env:GITHUB_REPOSITORY,
  [string]$WorkflowFile = "enterprise-route-auth-smoke.yml",
  [string]$ArtifactNamePrefix = "enterprise-route-auth-smoke-",
  [int]$MaxRuns = 30,
  [int]$MaxReports = 12,
  [string]$GitHubToken = $env:GITHUB_TOKEN,
  [string]$OutputMarkdownPath = "out/enterprise-route-auth-smoke-trend-report.md",
  [string]$OutputJsonPath = "out/enterprise-route-auth-smoke-trend-report.json"
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

  return Invoke-RestMethod -Uri "https://api.github.com$Path" -Headers $headers -Method Get
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

function Escape-MarkdownCell {
  param([string]$Value)

  if ([string]::IsNullOrEmpty($Value)) {
    return ""
  }

  return ($Value -replace '\|', '\|')
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
$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("gitgov-enterprise-route-auth-trend-" + [System.Guid]::NewGuid().ToString("N"))
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

    $summaryFile = Get-ChildItem -Path $expandPath -Recurse -File -Filter "enterprise-route-auth-smoke.json" | Select-Object -First 1
    if ($null -eq $summaryFile) {
      $skipped.Add([pscustomobject]@{ workflow_run_id = [int64]$run.id; artifact_id = [int64]$selectedArtifact.id; reason = "missing_enterprise_route_auth_smoke_json" }) | Out-Null
      continue
    }

    $summary = Get-Content -Raw -LiteralPath $summaryFile.FullName | ConvertFrom-Json
    $checks = @(Get-ObjectPropertyValue $summary "checks")
    $passedChecks = @($checks | Where-Object { $_.ok -eq $true }).Count
    $failedChecks = @($checks | Where-Object { $_.ok -ne $true }).Count
    $anonymousChecks = @($checks | Where-Object { $_.auth -eq "anonymous" }).Count
    $authenticatedChecks = @($checks | Where-Object { $_.auth -eq "bearer" }).Count

    $records.Add([pscustomobject]@{
      workflow_run_id      = [int64]$run.id
      workflow_run_url     = [string]$run.html_url
      workflow_created_at  = [string]$run.created_at
      artifact_id          = [int64]$selectedArtifact.id
      artifact_name        = [string]$selectedArtifact.name
      artifact_created_at  = [string]$selectedArtifact.created_at
      checked_at_utc       = [string](Get-ObjectPropertyValue $summary "checked_at_utc")
      gitgov_url           = [string](Get-ObjectPropertyValue $summary "gitgov_url")
      org_name             = [string](Get-ObjectPropertyValue $summary "org_name")
      repository_full_name = [string](Get-ObjectPropertyValue $summary "repository_full_name")
      release_id           = [string](Get-ObjectPropertyValue $summary "release_id")
      environment          = [string](Get-ObjectPropertyValue $summary "environment")
      status               = [string](Get-ObjectPropertyValue $summary "status")
      total_checks         = $checks.Count
      passed_checks        = $passedChecks
      failed_checks        = $failedChecks
      anonymous_checks     = $anonymousChecks
      authenticated_checks = $authenticatedChecks
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
$passedDelta = [int]$latest.passed_checks - [int]$oldest.passed_checks
$failedDelta = [int]$latest.failed_checks - [int]$oldest.failed_checks
$runsPassed = @($orderedRecords | Where-Object { $_.status -eq "passed" }).Count
$runsFailed = @($orderedRecords | Where-Object { $_.status -eq "failed" }).Count
$runsSkipped = @($orderedRecords | Where-Object { $_.status -eq "skipped" }).Count
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

$trendDirection = if ($latest.failed_checks -gt 0) {
  "failing"
} elseif ($failedDelta -lt 0) {
  "improving"
} elseif ($failedDelta -gt 0) {
  "declining"
} else {
  "stable"
}

$summaryOutput = [pscustomobject]@{
  generated_at                          = $generatedAtUtc
  repository                            = $Repository
  workflow_file                         = $WorkflowFile
  artifact_name_prefix                  = $ArtifactNamePrefix
  reports_analyzed                      = $orderedRecords.Count
  successful_runs_scanned               = $successfulRuns.Count
  latest_successful_run_id              = $latestSuccessfulRunId
  latest_successful_run_artifact_status = $latestSuccessfulRunArtifactStatus
  skipped_artifacts                     = @($skipped.ToArray())
  latest_status                         = $latest.status
  latest_total_checks                   = $latest.total_checks
  latest_passed_checks                  = $latest.passed_checks
  latest_failed_checks                  = $latest.failed_checks
  passed_delta_vs_oldest                = $passedDelta
  failed_delta_vs_oldest                = $failedDelta
  runs_passed                           = $runsPassed
  runs_failed                           = $runsFailed
  runs_skipped                          = $runsSkipped
  trend_direction                       = $trendDirection
  reports                               = @($orderedRecords)
}

$summaryOutput | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $OutputJsonPath -Encoding UTF8

$markdown = New-Object System.Collections.Generic.List[string]
$markdown.Add("# Enterprise Route Auth Smoke Trend Report") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add(('Generated: `{0}`' -f $generatedAtUtc)) | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add(('Repository: `{0}`' -f $Repository)) | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("## Executive Summary") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add(('- Latest status: `{0}`' -f $latest.status)) | Out-Null
$markdown.Add(('- Latest total checks: `{0}`' -f $latest.total_checks)) | Out-Null
$markdown.Add(('- Latest passed checks: `{0}`' -f $latest.passed_checks)) | Out-Null
$markdown.Add(('- Latest failed checks: `{0}`' -f $latest.failed_checks)) | Out-Null
$markdown.Add(('- Reports analyzed: `{0}`' -f $orderedRecords.Count)) | Out-Null
$markdown.Add(('- Latest successful workflow run: `{0}`' -f $latestSuccessfulRunId)) | Out-Null
$markdown.Add(('- Latest successful run artifact status: `{0}`' -f $latestSuccessfulRunArtifactStatus)) | Out-Null
$markdown.Add(('- Trend direction: `{0}`' -f $trendDirection)) | Out-Null
$markdown.Add(('- Passed-check delta vs oldest report: `{0}`' -f $passedDelta)) | Out-Null
$markdown.Add(('- Failed-check delta vs oldest report: `{0}`' -f $failedDelta)) | Out-Null
$markdown.Add(('- Runs passed: `{0}`' -f $runsPassed)) | Out-Null
$markdown.Add(('- Runs failed: `{0}`' -f $runsFailed)) | Out-Null
$markdown.Add(('- Runs skipped: `{0}`' -f $runsSkipped)) | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("## Report History") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("| Run | Checked at | Status | Passed | Failed | Org | Environment | Artifact |") | Out-Null
$markdown.Add("|---:|---|---|---:|---:|---|---|---|") | Out-Null
foreach ($record in $orderedRecords) {
  $markdown.Add(('| [{0}]({1}) | `{2}` | `{3}` | {4} | {5} | `{6}` | `{7}` | `{8}` |' -f $record.workflow_run_id, $record.workflow_run_url, $record.checked_at_utc, (Escape-MarkdownCell $record.status), $record.passed_checks, $record.failed_checks, (Escape-MarkdownCell $record.org_name), (Escape-MarkdownCell $record.environment), (Escape-MarkdownCell $record.artifact_name))) | Out-Null
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
$markdown.Add("- Trend data comes from uploaded enterprise route auth smoke artifacts, not from provider secrets.") | Out-Null
$markdown.Add("- A failed check means one of the expected anonymous or authenticated route responses drifted from the expected HTTP code.") | Out-Null
$markdown.Add("- This report shows whether auth-smoke behavior is stable over time; it does not make release blocking the default by itself.") | Out-Null
$markdown.Add("- Missing or expired artifacts are listed separately so operators can distinguish data-retention gaps from route regressions.") | Out-Null

Set-Content -LiteralPath $OutputMarkdownPath -Value $markdown -Encoding UTF8
Write-Host "Wrote enterprise route auth smoke trend report: $OutputMarkdownPath"
Write-Host "Wrote enterprise route auth smoke trend JSON: $OutputJsonPath"
