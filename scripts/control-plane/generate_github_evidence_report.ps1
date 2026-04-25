param(
  [string]$GitGovUrl = $env:GITGOV_URL,
  [string]$ApiKey = $env:GITGOV_API_KEY,
  [string]$StatsJsonPath = "",
  [string]$OutputPath = "",
  [string]$OrgName = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

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

function Get-MapValue {
  param(
    [Parameter(Mandatory = $true)][object]$Map,
    [Parameter(Mandatory = $true)][string]$Key
  )
  if ($null -eq $Map) { return 0 }
  if ($Map -is [System.Collections.IDictionary]) {
    if ($Map.Contains($Key)) { return Get-Number -Value $Map[$Key] }
    return 0
  }
  $property = $Map.PSObject.Properties[$Key]
  if ($null -eq $property) { return 0 }
  return Get-Number -Value $property.Value
}

function Get-EventCount {
  param(
    [Parameter(Mandatory = $true)][object]$ByType,
    [Parameter(Mandatory = $true)][string[]]$Keys
  )
  $total = 0.0
  foreach ($key in $Keys) {
    $total += Get-MapValue -Map $ByType -Key $key
  }
  return [int]$total
}

function Read-Stats {
  if (-not [string]::IsNullOrWhiteSpace($StatsJsonPath)) {
    if (-not (Test-Path -LiteralPath $StatsJsonPath)) {
      throw "Stats JSON file not found: $StatsJsonPath"
    }
    return Get-Content -LiteralPath $StatsJsonPath -Raw | ConvertFrom-Json
  }

  if ([string]::IsNullOrWhiteSpace($GitGovUrl)) {
    throw "Missing -GitGovUrl or GITGOV_URL."
  }
  if ([string]::IsNullOrWhiteSpace($ApiKey)) {
    throw "Missing -ApiKey or GITGOV_API_KEY."
  }

  $base = $GitGovUrl.TrimEnd('/')
  $headers = @{ Authorization = "Bearer $ApiKey" }
  return Invoke-RestMethod -Uri "$base/stats" -Method GET -Headers $headers
}

function Get-DefaultOutputPath {
  $stamp = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHHmmssZ")
  return Join-Path "docs/reports" "github-evidence-executive-report-$stamp.md"
}

function Escape-MarkdownCell {
  param([string]$Value)
  return ($Value -replace '\|', '\|')
}

$stats = Read-Stats
$byType = $stats.github_events.by_type

$signals = @(
  [pscustomobject]@{ Label = "PR lifecycle"; Count = Get-EventCount -ByType $byType -Keys @("pull_request"); Source = "pull_request" }
  [pscustomobject]@{ Label = "Reviews"; Count = Get-EventCount -ByType $byType -Keys @("pull_request_review"); Source = "pull_request_review" }
  [pscustomobject]@{ Label = "Comentarios PR"; Count = Get-EventCount -ByType $byType -Keys @("pull_request_review_comment", "issue_comment"); Source = "pull_request_review_comment + issue_comment" }
  [pscustomobject]@{ Label = "Checks/status"; Count = Get-EventCount -ByType $byType -Keys @("check_run", "check_suite", "status"); Source = "check_run + check_suite + status" }
)

$activeSignals = @($signals | Where-Object { $_.Count -gt 0 }).Count
$totalSignals = $signals.Count
$executiveStatus = if ($activeSignals -eq $totalSignals) {
  "Completo"
} elseif ($activeSignals -gt 0) {
  "Parcial"
} else {
  "Sin evidencia"
}
$missingSignals = @($signals | Where-Object { $_.Count -eq 0 } | ForEach-Object { $_.Label })

$topEventRows = @()
if ($null -ne $byType) {
  $properties = $byType.PSObject.Properties
  foreach ($property in $properties) {
    $topEventRows += [pscustomobject]@{
      EventType = [string]$property.Name
      Count = [int](Get-Number -Value $property.Value)
    }
  }
}
$topEventRows = @($topEventRows | Sort-Object -Property Count -Descending | Select-Object -First 10)

if ([string]::IsNullOrWhiteSpace($OutputPath)) {
  $OutputPath = Get-DefaultOutputPath
}

$outputDir = Split-Path -Parent $OutputPath
if (-not [string]::IsNullOrWhiteSpace($outputDir)) {
  New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
}

$generatedAt = (Get-Date).ToUniversalTime().ToString("o")
$source = if (-not [string]::IsNullOrWhiteSpace($StatsJsonPath)) {
  "stats JSON: $StatsJsonPath"
} else {
  "$($GitGovUrl.TrimEnd('/'))/stats"
}

$lines = New-Object System.Collections.Generic.List[string]
$lines.Add("# GitHub Evidence Executive Report") | Out-Null
$lines.Add("") | Out-Null
$lines.Add(('Generated: `{0}`' -f $generatedAt)) | Out-Null
$lines.Add("") | Out-Null
$lines.Add(('Source: `{0}`' -f $source)) | Out-Null
if (-not [string]::IsNullOrWhiteSpace($OrgName)) {
  $lines.Add("") | Out-Null
  $lines.Add(('Organization: `{0}`' -f $OrgName)) | Out-Null
}
$lines.Add("") | Out-Null
$lines.Add("## Executive Summary") | Out-Null
$lines.Add("") | Out-Null
$lines.Add(('- Status: `{0}`' -f $executiveStatus)) | Out-Null
$lines.Add(('- Coverage: `{0}/{1} signals`' -f $activeSignals, $totalSignals)) | Out-Null
$missingSignalsText = if ($missingSignals.Count -gt 0) { $missingSignals -join ', ' } else { 'none' }
$lines.Add(('- Missing signals: `{0}`' -f $missingSignalsText)) | Out-Null
$lines.Add("") | Out-Null
$lines.Add("## Signal Coverage") | Out-Null
$lines.Add("") | Out-Null
$lines.Add("| Signal | Count | Source event types |") | Out-Null
$lines.Add("|---|---:|---|") | Out-Null
foreach ($signal in $signals) {
  $lines.Add(('| {0} | {1} | `{2}` |' -f (Escape-MarkdownCell $signal.Label), $signal.Count, (Escape-MarkdownCell $signal.Source))) | Out-Null
}
$lines.Add("") | Out-Null
$lines.Add("## Top GitHub Event Types") | Out-Null
$lines.Add("") | Out-Null
if ($topEventRows.Count -eq 0) {
  $lines.Add("No GitHub event type counts were present in the stats payload.") | Out-Null
} else {
  $lines.Add("| Event type | Count |") | Out-Null
  $lines.Add("|---|---:|") | Out-Null
  foreach ($row in $topEventRows) {
    $lines.Add(('| `{0}` | {1} |' -f (Escape-MarkdownCell $row.EventType), $row.Count)) | Out-Null
  }
}
$lines.Add("") | Out-Null
$lines.Add("## Interpretation") | Out-Null
$lines.Add("") | Out-Null
$lines.Add('- `Completo` means GitGov has evidence across PR lifecycle, reviews, PR comments, and checks/status.') | Out-Null
$lines.Add('- `Parcial` means at least one evidence family is missing and operators should verify webhook event selection or recent repo activity.') | Out-Null
$lines.Add('- Counts come from `/stats.github_events.by_type`; this report does not expose provider secrets or raw webhook payloads.') | Out-Null

Set-Content -LiteralPath $OutputPath -Value $lines -Encoding UTF8
Write-Output "Wrote GitHub evidence executive report: $OutputPath"
