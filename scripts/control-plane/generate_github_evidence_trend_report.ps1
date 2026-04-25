param(
  [string]$Repository = $env:GITHUB_REPOSITORY,
  [string]$WorkflowFile = "github-evidence-report.yml",
  [string]$ArtifactName = "github-evidence-executive-report",
  [int]$MaxRuns = 30,
  [int]$MaxReports = 12,
  [string]$GitHubToken = $env:GITHUB_TOKEN,
  [string]$OutputMarkdownPath = "out/github-evidence-trend-report.md",
  [string]$OutputJsonPath = "out/github-evidence-trend-report.json"
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

function Read-ReportField {
  param(
    [Parameter(Mandatory = $true)][AllowEmptyString()][string[]]$Lines,
    [Parameter(Mandatory = $true)][string]$Pattern,
    [Parameter(Mandatory = $true)][string]$GroupName,
    [string]$Default = ""
  )

  foreach ($line in $Lines) {
    $match = [regex]::Match($line, $Pattern)
    if ($match.Success) {
      return [string]$match.Groups[$GroupName].Value
    }
  }
  return $Default
}

function Escape-MarkdownCell {
  param([string]$Value)
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

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("gitgov-evidence-trend-" + [System.Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $tempRoot | Out-Null

$records = New-Object System.Collections.Generic.List[object]
$skipped = New-Object System.Collections.Generic.List[object]

try {
  foreach ($run in $successfulRuns) {
    if ($records.Count -ge $MaxReports) {
      break
    }

    $artifacts = Invoke-GitHubApi -Path "/repos/$Repository/actions/runs/$($run.id)/artifacts?per_page=100"
    $artifact = @($artifacts.artifacts | Where-Object { $_.name -eq $ArtifactName } | Sort-Object -Property created_at -Descending | Select-Object -First 1)

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

    $markdownFile = Get-ChildItem -Path $expandPath -Recurse -File -Filter "*.md" | Select-Object -First 1
    if ($null -eq $markdownFile) {
      $skipped.Add([pscustomobject]@{ workflow_run_id = [int64]$run.id; artifact_id = [int64]$selectedArtifact.id; reason = "missing_markdown" }) | Out-Null
      continue
    }

    $lines = Get-Content -LiteralPath $markdownFile.FullName
    $status = Read-ReportField -Lines $lines -Pattern '^- Status:\s+`(?<value>[^`]+)`' -GroupName "value" -Default "unknown"
    $coverageRaw = Read-ReportField -Lines $lines -Pattern '^- Coverage:\s+`(?<value>[^`]+)`' -GroupName "value" -Default "0/0 signals"
    $coverageMatch = [regex]::Match($coverageRaw, '^(?<active>\d+)\/(?<total>\d+)')
    $activeSignals = if ($coverageMatch.Success) { [int]$coverageMatch.Groups["active"].Value } else { 0 }
    $totalSignals = if ($coverageMatch.Success) { [int]$coverageMatch.Groups["total"].Value } else { 0 }
    $missingSignals = Read-ReportField -Lines $lines -Pattern '^- Missing signals:\s+`(?<value>[^`]+)`' -GroupName "value" -Default "unknown"
    $generatedAt = Read-ReportField -Lines $lines -Pattern '^Generated:\s+`(?<value>[^`]+)`' -GroupName "value" -Default ([string]$run.created_at)

    $records.Add([pscustomobject]@{
      workflow_run_id     = [int64]$run.id
      workflow_run_url    = [string]$run.html_url
      workflow_created_at = [string]$run.created_at
      report_generated_at = $generatedAt
      artifact_id         = [int64]$selectedArtifact.id
      artifact_created_at = [string]$selectedArtifact.created_at
      status              = $status
      active_signals      = $activeSignals
      total_signals       = $totalSignals
      coverage            = $coverageRaw
      missing_signals     = $missingSignals
    }) | Out-Null
  }
} finally {
  Remove-Item -LiteralPath $tempRoot -Recurse -Force -ErrorAction SilentlyContinue
}

if ($records.Count -eq 0) {
  Fail-Trend "No parseable '$ArtifactName' report artifacts found in the latest $MaxRuns successful '$WorkflowFile' runs."
}

$orderedRecords = @($records | Sort-Object -Property workflow_created_at -Descending)
$latest = $orderedRecords[0]
$oldest = $orderedRecords[$orderedRecords.Count - 1]
$coverageDelta = [int]$latest.active_signals - [int]$oldest.active_signals
$completeCount = @($orderedRecords | Where-Object { $_.status -eq "Completo" }).Count
$generatedAtUtc = (Get-Date).ToUniversalTime().ToString("o")

$trendStatus = if ($latest.total_signals -gt 0 -and $latest.active_signals -eq $latest.total_signals) {
  "Completo"
} elseif ($latest.active_signals -gt 0) {
  "Parcial"
} else {
  "Sin evidencia"
}

$summary = [pscustomobject]@{
  generated_at              = $generatedAtUtc
  repository                = $Repository
  workflow_file             = $WorkflowFile
  artifact_name             = $ArtifactName
  reports_analyzed          = $orderedRecords.Count
  successful_runs_scanned   = $successfulRuns.Count
  skipped_artifacts         = @($skipped.ToArray())
  latest_status             = $latest.status
  latest_coverage           = $latest.coverage
  coverage_delta_vs_oldest  = $coverageDelta
  complete_report_count     = $completeCount
  trend_status              = $trendStatus
  reports                   = @($orderedRecords)
}

$summary | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $OutputJsonPath -Encoding UTF8

$markdown = New-Object System.Collections.Generic.List[string]
$markdown.Add("# GitHub Evidence Trend Report") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add(('Generated: `{0}`' -f $generatedAtUtc)) | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add(('Repository: `{0}`' -f $Repository)) | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("## Executive Summary") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add(('- Latest status: `{0}`' -f $latest.status)) | Out-Null
$markdown.Add(('- Latest coverage: `{0}`' -f $latest.coverage)) | Out-Null
$markdown.Add(('- Reports analyzed: `{0}`' -f $orderedRecords.Count)) | Out-Null
$markdown.Add(('- Complete reports: `{0}`' -f $completeCount)) | Out-Null
$markdown.Add(('- Coverage delta vs oldest report: `{0}` signals' -f $coverageDelta)) | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("## Report History") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("| Run | Report generated | Status | Coverage | Missing signals |") | Out-Null
$markdown.Add("|---:|---|---|---|---|") | Out-Null
foreach ($record in $orderedRecords) {
  $markdown.Add(('| [{0}]({1}) | `{2}` | `{3}` | `{4}` | `{5}` |' -f $record.workflow_run_id, $record.workflow_run_url, $record.report_generated_at, (Escape-MarkdownCell $record.status), (Escape-MarkdownCell $record.coverage), (Escape-MarkdownCell $record.missing_signals))) | Out-Null
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
$markdown.Add("- Trend data comes from uploaded GitHub Actions artifacts, not from provider secrets.") | Out-Null
$markdown.Add("- A negative coverage delta indicates the latest report has fewer evidence signal families than the oldest report in the window.") | Out-Null
$markdown.Add("- Missing or expired artifacts are listed separately so operators can distinguish data gaps from workflow retention limits.") | Out-Null

Set-Content -LiteralPath $OutputMarkdownPath -Value $markdown -Encoding UTF8
Write-Host "Wrote GitHub evidence trend report: $OutputMarkdownPath"
Write-Host "Wrote GitHub evidence trend JSON: $OutputJsonPath"
