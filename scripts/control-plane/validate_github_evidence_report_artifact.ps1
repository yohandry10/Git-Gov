param(
  [string]$Repository = $env:GITHUB_REPOSITORY,
  [string]$WorkflowFile = "github-evidence-report.yml",
  [string]$ArtifactName = "github-evidence-executive-report",
  [string]$ArtifactNamePrefix = "",
  [int]$MaxAgeHours = 192,
  [string]$GitHubToken = $env:GITHUB_TOKEN,
  [string]$OutputPath = ""
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

if ([string]::IsNullOrWhiteSpace($Repository)) {
  Fail-Monitor "Missing -Repository or GITHUB_REPOSITORY."
}

if ([string]::IsNullOrWhiteSpace($GitHubToken)) {
  Fail-Monitor "Missing -GitHubToken or GITHUB_TOKEN."
}

if ($MaxAgeHours -le 0) {
  Fail-Monitor "-MaxAgeHours must be greater than zero."
}

$encodedWorkflow = [System.Uri]::EscapeDataString($WorkflowFile)
$runs = Invoke-GitHubApi -Path "/repos/$Repository/actions/workflows/$encodedWorkflow/runs?status=success&per_page=10"
$successfulRuns = @($runs.workflow_runs | Where-Object { $_.status -eq "completed" -and $_.conclusion -eq "success" })

if ($successfulRuns.Count -eq 0) {
  Fail-Monitor "No successful completed runs found for workflow '$WorkflowFile' in '$Repository'."
}

$latestRun = $successfulRuns | Sort-Object -Property created_at -Descending | Select-Object -First 1
$artifacts = Invoke-GitHubApi -Path "/repos/$Repository/actions/runs/$($latestRun.id)/artifacts?per_page=100"
$artifactMatches = if (-not [string]::IsNullOrWhiteSpace($ArtifactNamePrefix)) {
  @($artifacts.artifacts | Where-Object { [string]$_.name -like "$ArtifactNamePrefix*" })
} else {
  @($artifacts.artifacts | Where-Object { $_.name -eq $ArtifactName })
}
$artifact = @($artifactMatches | Sort-Object -Property created_at -Descending | Select-Object -First 1)

if ($artifact.Count -eq 0) {
  $expectedArtifact = if (-not [string]::IsNullOrWhiteSpace($ArtifactNamePrefix)) {
    "artifact with prefix '$ArtifactNamePrefix'"
  } else {
    "artifact '$ArtifactName'"
  }
  Fail-Monitor "Latest successful run $($latestRun.id) does not contain $expectedArtifact."
}

$selectedArtifact = $artifact[0]
if ($selectedArtifact.expired) {
  Fail-Monitor "Artifact '$ArtifactName' from run $($latestRun.id) is expired."
}

$now = (Get-Date).ToUniversalTime()
$artifactCreatedAt = [DateTime]::Parse([string]$selectedArtifact.created_at).ToUniversalTime()
$ageHours = ($now - $artifactCreatedAt).TotalHours

$status = if ($ageHours -le $MaxAgeHours) { "PASS" } else { "FAIL" }
$summary = [pscustomobject]@{
  status                  = $status
  repository              = $Repository
  workflow_file           = $WorkflowFile
  workflow_run_id         = [int64]$latestRun.id
  workflow_run_url        = [string]$latestRun.html_url
  workflow_run_created_at = [string]$latestRun.created_at
  artifact_name           = [string]$selectedArtifact.name
  artifact_name_prefix    = [string]$ArtifactNamePrefix
  artifact_id             = [int64]$selectedArtifact.id
  artifact_created_at     = [string]$selectedArtifact.created_at
  artifact_expired        = [bool]$selectedArtifact.expired
  max_age_hours           = $MaxAgeHours
  artifact_age_hours      = [math]::Round($ageHours, 2)
}

if (-not [string]::IsNullOrWhiteSpace($OutputPath)) {
  $outputDir = Split-Path -Parent $OutputPath
  if (-not [string]::IsNullOrWhiteSpace($outputDir)) {
    New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
  }
  $summary | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $OutputPath -Encoding UTF8
}

if ($status -eq "FAIL") {
  Write-Host ($summary | ConvertTo-Json -Depth 5)
  Fail-Monitor "Artifact '$($selectedArtifact.name)' is stale: $([math]::Round($ageHours, 2))h old, max $MaxAgeHours h."
}

Write-Host "[PASS] Workflow artifact is fresh: workflow '$WorkflowFile', run $($latestRun.id), artifact '$($selectedArtifact.name)' ($($selectedArtifact.id)), age $([math]::Round($ageHours, 2))h."
exit 0
