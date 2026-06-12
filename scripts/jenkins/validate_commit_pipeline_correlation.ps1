param(
  [string]$GitGovUrl = "http://127.0.0.1:3001",
  [string]$ApiKey,
  [string]$JenkinsSecret = "",
  [string]$RepoFullName = "",
  [string]$OrgName = "",
  [string]$CommitSha = "",
  [string]$Branch = "main",
  [string]$UserLogin = "jenkins",
  [switch]$InjectPipelineIfMissing
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

if ([string]::IsNullOrWhiteSpace($OrgName) -and $RepoFullName.Contains('/')) {
  $OrgName = $RepoFullName.Split('/')[0]
}

if ([string]::IsNullOrWhiteSpace($ApiKey)) {
  Write-Error "Missing -ApiKey."
  exit 1
}

function Invoke-GitGovJson {
  param(
    [Parameter(Mandatory = $true)][string]$Method,
    [Parameter(Mandatory = $true)][string]$Path,
    [Parameter()][object]$Body
  )

  $base = $GitGovUrl.TrimEnd('/')
  $uri = "$base$Path"
  $headers = @{
    Authorization = "Bearer $ApiKey"
    "Content-Type" = "application/json"
  }
  if (
    -not [string]::IsNullOrWhiteSpace($JenkinsSecret) `
    -and $Path.StartsWith("/integrations/jenkins", [System.StringComparison]::OrdinalIgnoreCase)
  ) {
    $headers["x-gitgov-jenkins-secret"] = $JenkinsSecret
  }

  try {
    if ($PSBoundParameters.ContainsKey('Body')) {
      return Invoke-RestMethod -Uri $uri -Method $Method -Headers $headers -Body ($Body | ConvertTo-Json -Depth 20)
    }
    return Invoke-RestMethod -Uri $uri -Method $Method -Headers $headers
  } catch {
    if ($_.Exception.Response) {
      $response = $_.Exception.Response
      $payload = ""
      if ($null -ne $_.ErrorDetails -and ($_.ErrorDetails.PSObject.Properties.Name -contains "Message")) {
        $payload = [string]$_.ErrorDetails.Message
      }
      if ([string]::IsNullOrWhiteSpace($payload)) {
        try {
          if ($response.PSObject.Properties.Name -contains "Content" -and $null -ne $response.Content) {
            $payload = $response.Content.ReadAsStringAsync().GetAwaiter().GetResult()
          } elseif ($response.PSObject.Methods.Name -contains "GetResponseStream") {
            $reader = New-Object IO.StreamReader($response.GetResponseStream())
            $payload = $reader.ReadToEnd()
          }
        } catch {
          $payload = $_.Exception.Message
        }
      }
      $status = ""
      if ($response.PSObject.Properties.Name -contains "StatusCode") {
        $status = " status=$($response.StatusCode)"
      }
      throw "HTTP error calling $Method $uri$status -> $payload"
    }
    throw
  }
}

$effectiveCommitSha = $CommitSha
if ([string]::IsNullOrWhiteSpace($effectiveCommitSha)) {
  $effectiveCommitSha = (git rev-parse HEAD 2>$null).Trim()
}
if ([string]::IsNullOrWhiteSpace($effectiveCommitSha)) {
  Write-Error "Could not resolve commit SHA. Provide -CommitSha explicitly."
  exit 1
}

if ($InjectPipelineIfMissing) {
  $pipelineId = "smoke-correlation-$([DateTimeOffset]::UtcNow.ToUnixTimeSeconds())"
  $pipelineEvent = @{
    org_name = if ([string]::IsNullOrWhiteSpace($OrgName)) { $null } else { $OrgName }
    pipeline_id = $pipelineId
    job_name = "sonar-smoke-correlation"
    status = "success"
    commit_sha = $effectiveCommitSha
    branch = $Branch
    repo_full_name = $RepoFullName
    duration_ms = 1000
    triggered_by = "smoke-script"
    stages = @(@{ name = "quality_gate"; status = "OK"; duration_ms = 500 })
    artifacts = @()
    timestamp = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
  }
  $ingestPipeline = Invoke-GitGovJson -Method "POST" -Path "/integrations/jenkins" -Body $pipelineEvent
  if (-not $ingestPipeline.accepted) {
    Write-Error "Pipeline ingest was not accepted."
    exit 1
  }
}

$eventUuid = [Guid]::NewGuid().ToString()
$commitEvent = @{
  events = @(
    @{
      event_uuid = $eventUuid
      event_type = "commit"
      org_name = if ([string]::IsNullOrWhiteSpace($OrgName)) { $null } else { $OrgName }
      repo_full_name = $RepoFullName
      user_login = $UserLogin
      user_name = $UserLogin
      branch = $Branch
      commit_sha = $effectiveCommitSha
      files = @()
      status = "success"
      reason = $null
      metadata = @{ commit_message = "smoke: correlation validation" }
      timestamp = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    }
  )
  client_version = "smoke-correlation-script"
}

$ingestCommit = Invoke-GitGovJson -Method "POST" -Path "/events" -Body $commitEvent
if (-not ($ingestCommit.accepted -contains $eventUuid)) {
  Write-Error "Commit event ingest failed for event_uuid $eventUuid"
  exit 1
}

$userParam = [Uri]::EscapeDataString($UserLogin)
$repoParam = [Uri]::EscapeDataString($RepoFullName)
$branchParam = [Uri]::EscapeDataString($Branch)
$correlationsPath = "/integrations/jenkins/correlations?limit=50&offset=0&repo_full_name=$repoParam&branch=$branchParam&user_login=$userParam"
if (-not [string]::IsNullOrWhiteSpace($OrgName)) {
  $orgParam = [Uri]::EscapeDataString($OrgName)
  $correlationsPath += "&org_name=$orgParam"
}
$correlations = Invoke-GitGovJson -Method "GET" -Path $correlationsPath

$match = $null
foreach ($item in @($correlations.correlations)) {
  if ($item.commit_sha -eq $effectiveCommitSha -and $item.repo_name -eq $RepoFullName) {
    $match = $item
    break
  }
}

if ($null -eq $match) {
  Write-Error "FAIL: No commit/pipeline correlation found for commit $effectiveCommitSha in repo $RepoFullName"
  exit 1
}

if ($null -eq $match.pipeline) {
  Write-Error "FAIL: Correlation found but pipeline payload is null for commit $effectiveCommitSha"
  exit 1
}

Write-Host "PASS: Correlation validated"
Write-Host "  commit_sha: $($match.commit_sha)"
Write-Host "  repo_name:  $($match.repo_name)"
Write-Host "  pipeline:   $($match.pipeline.pipeline_id)"
Write-Host "  job_name:   $($match.pipeline.job_name)"
Write-Host "  status:     $($match.pipeline.status)"
