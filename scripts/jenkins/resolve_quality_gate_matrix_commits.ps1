param(
  [string]$GitGovUrl = "http://127.0.0.1:3001",
  [string]$ApiKey,
  [string]$RepoFullName,
  [string]$OrgName = "",
  [string]$Branch = "main",
  [int]$Limit = 500,
  [string]$OutputPath = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($ApiKey)) {
  Write-Error "Missing -ApiKey."
  exit 1
}
if ([string]::IsNullOrWhiteSpace($RepoFullName)) {
  Write-Error "Missing -RepoFullName (<owner>/<repo>)."
  exit 1
}
if ([string]::IsNullOrWhiteSpace($OrgName) -and $RepoFullName.Contains("/")) {
  $OrgName = $RepoFullName.Split("/", 2)[0]
}
if ($Limit -lt 1) {
  Write-Error "-Limit must be >= 1."
  exit 1
}

function Get-HttpErrorBody {
  param(
    [Parameter(Mandatory = $true)]
    [object]$Response
  )

  if ($Response -and $Response.PSObject.Properties.Name -contains "Content" -and $Response.Content) {
    try {
      return $Response.Content.ReadAsStringAsync().GetAwaiter().GetResult()
    } catch {
      return ""
    }
  }

  if ($Response -and $Response.PSObject.Methods.Name -contains "GetResponseStream") {
    try {
      $reader = New-Object IO.StreamReader($Response.GetResponseStream())
      return $reader.ReadToEnd()
    } catch {
      return ""
    }
  }

  return ""
}

$baseUrl = $GitGovUrl.TrimEnd("/")
$orgQuery = ""
if (-not [string]::IsNullOrWhiteSpace($OrgName)) {
  $orgQuery = "&org_name=$([System.Uri]::EscapeDataString($OrgName))"
}
$uri = "{0}/integrations/jenkins/correlations?limit={1}&offset=0{2}" -f $baseUrl, $Limit, $orgQuery
$headers = @{
  Authorization = "Bearer $ApiKey"
  "Content-Type" = "application/json"
}

try {
  $resp = Invoke-RestMethod -Uri $uri -Method Get -Headers $headers
} catch {
  if ($_.Exception.Response) {
    $body = Get-HttpErrorBody -Response $_.Exception.Response
    Write-Error "HTTP error resolving correlations: $body"
    exit 1
  }
  throw
}

$all = @($resp.correlations)
if ($all.Count -eq 0) {
  Write-Error "No correlations found."
  exit 1
}

$repoFiltered = @($all | Where-Object { [string]$_.repo_name -eq $RepoFullName -and $null -ne $_.pipeline })
$branchFiltered = if ([string]::IsNullOrWhiteSpace($Branch)) { $repoFiltered } else { @($repoFiltered | Where-Object { [string]$_.branch -eq $Branch }) }
$candidates = @($branchFiltered | Where-Object {
  $job = [string]$_.pipeline.job_name
  return $job.ToLowerInvariant().Contains("sonar")
})

if ($candidates.Count -eq 0) {
  Write-Error "No Sonar-related correlations found for repo '$RepoFullName' branch '$Branch'."
  exit 1
}

$successStates = @("success", "passed", "ok")
$failureStates = @("failure", "failed", "error", "unstable", "timeout", "scan_failed")

$green = $null
$failing = $null

foreach ($item in $candidates) {
  $status = ([string]$item.pipeline.status).ToLowerInvariant()
  if ($null -eq $green -and $successStates -contains $status) {
    $green = $item
  }
  if ($null -eq $failing -and $failureStates -contains $status) {
    $failing = $item
  }
  if ($null -ne $green -and $null -ne $failing) {
    break
  }
}

if ($null -eq $failing) {
  # Fallback: infer failing commit from quality-gate policy violation signals
  $signalsUri = "{0}/signals?signal_type=policy_violation&limit={1}&offset=0{2}" -f $baseUrl, $Limit, $orgQuery
  try {
    $signalsResp = Invoke-RestMethod -Uri $signalsUri -Method Get -Headers $headers
    $signalCandidates = @($signalsResp.signals | Where-Object {
      $rule = [string]$_.evidence.rule
      $repo = [string]$_.evidence.repo_name
      $signalBranch = [string]$_.branch
      $gate = ([string]$_.evidence.gate_status).ToLowerInvariant()
      $hasCommit = -not [string]::IsNullOrWhiteSpace([string]$_.commit_sha)
      $repoMatch = $repo -eq $RepoFullName
      $branchMatch = [string]::IsNullOrWhiteSpace($Branch) -or $signalBranch -eq $Branch
      $nonGreen = ($gate -and $gate -ne "success" -and $gate -ne "passed" -and $gate -ne "ok" -and $gate -ne "green")
      return $hasCommit -and $repoMatch -and $branchMatch -and $rule -eq "quality_gate_green" -and $nonGreen
    })
    if ($signalCandidates.Count -gt 0) {
      $failing = [pscustomobject]@{
        commit_sha = [string]$signalCandidates[0].commit_sha
        branch = [string]$signalCandidates[0].branch
        pipeline = [pscustomobject]@{
          status = [string]$signalCandidates[0].evidence.gate_status
          job_name = [string]$signalCandidates[0].evidence.job_name
        }
      }
    }
  } catch {
    # best effort fallback; keep existing behavior if still unresolved
  }
}

if ($null -eq $failing) {
  Write-Error "No failing Sonar evidence found for repo '$RepoFullName' branch '$Branch' (correlations/signals)."
  exit 1
}
if ($null -eq $green) {
  Write-Error "No green Sonar correlation found for repo '$RepoFullName' branch '$Branch'."
  exit 1
}

$result = [pscustomobject]@{
  repo = $RepoFullName
  org_name = $OrgName
  branch = $Branch
  failing_commit_sha = [string]$failing.commit_sha
  failing_status = [string]$failing.pipeline.status
  failing_job_name = [string]$failing.pipeline.job_name
  green_commit_sha = [string]$green.commit_sha
  green_status = [string]$green.pipeline.status
  green_job_name = [string]$green.pipeline.job_name
}

if ([string]::IsNullOrWhiteSpace($OutputPath)) {
  $stamp = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHHmmssZ")
  $OutputPath = "docs/reports/quality-gate-matrix-commit-resolution-$stamp.json"
}

if (!(Test-Path (Split-Path -Parent $OutputPath))) {
  New-Item -ItemType Directory -Path (Split-Path -Parent $OutputPath) | Out-Null
}

$result | ConvertTo-Json -Depth 10 | Set-Content -Path $OutputPath -Encoding UTF8

Write-Host "PASS: resolved matrix commits"
Write-Host "  repo:               $RepoFullName"
Write-Host "  branch:             $Branch"
Write-Host "  failing commit:     $($result.failing_commit_sha) ($($result.failing_status), $($result.failing_job_name))"
Write-Host "  green commit:       $($result.green_commit_sha) ($($result.green_status), $($result.green_job_name))"
Write-Host "  output:             $OutputPath"
