param(
  [string]$Owner = "yohandry10",
  [string]$Repo = "Git-Gov",
  [string]$GitHubToken = "",
  [string]$SonarProjectKey = "yohandry10_git-gov",
  [string]$SonarHostUrl = "",
  [string]$GitGovUrl = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
. (Join-Path $scriptRoot "_token_helpers.ps1")

$token = Resolve-GitHubToken -ExplicitToken $GitHubToken -ScriptRoot $scriptRoot
if ([string]::IsNullOrWhiteSpace($token)) {
  Write-Error "Missing GitHub token. Provide -GitHubToken, set GITHUB_TOKEN/GH_TOKEN/GITHUB_PAT/GITHUB_PERSONAL_ACCESS_TOKEN, or define GITHUB_PERSONAL_ACCESS_TOKEN in gitgov/gitgov-server/.env."
  exit 1
}

if ([string]::IsNullOrWhiteSpace($SonarProjectKey)) {
  Write-Error "SonarProjectKey is required."
  exit 1
}

$headers = @{
  Authorization = "Bearer $token"
  Accept = "application/vnd.github+json"
  "X-GitHub-Api-Version" = "2022-11-28"
  "User-Agent" = "gitgov-ci-variable-bootstrap"
}

function Get-GitHubApiFailureMessage {
  param(
    [Parameter(Mandatory = $true)][object]$ErrorRecord,
    [Parameter(Mandatory = $true)][string]$Uri
  )

  $response = $ErrorRecord.Exception.Response
  if ($null -eq $response) {
    return "GitHub API request failed ($Uri): $($ErrorRecord.Exception.Message)"
  }

  $statusCode = $response.StatusCode.value__
  $acceptedPerms = $response.Headers["x-accepted-github-permissions"]
  $body = ""
  try {
    $stream = $response.GetResponseStream()
    if ($null -ne $stream) {
      $reader = New-Object IO.StreamReader($stream)
      $body = $reader.ReadToEnd()
    }
  } catch {
    # best effort
  }

  $parts = @("GitHub API request failed ($Uri): status=$statusCode")
  if (-not [string]::IsNullOrWhiteSpace($acceptedPerms)) {
    $parts += "accepted_permissions=$acceptedPerms"
  }
  if (-not [string]::IsNullOrWhiteSpace($body)) {
    $parts += "body=$body"
  }
  return ($parts -join " | ")
}

function Upsert-RepoVariable {
  param(
    [Parameter(Mandatory = $true)][string]$Owner,
    [Parameter(Mandatory = $true)][string]$Repo,
    [Parameter(Mandatory = $true)][string]$Name,
    [Parameter(Mandatory = $true)][string]$Value,
    [Parameter(Mandatory = $true)][hashtable]$Headers
  )

  $base = "https://api.github.com/repos/$Owner/$Repo/actions/variables"
  $exists = $false
  try {
    Invoke-RestMethod -Method Get -Uri "$base/$Name" -Headers $Headers | Out-Null
    $exists = $true
  } catch {
    if ($_.Exception.Response -and $_.Exception.Response.StatusCode.value__ -ne 404) {
      throw (Get-GitHubApiFailureMessage -ErrorRecord $_ -Uri "$base/$Name")
    }
    $exists = $false
  }

  $payload = @{ name = $Name; value = $Value } | ConvertTo-Json
  if ($exists) {
    try {
      Invoke-RestMethod -Method Patch -Uri "$base/$Name" -Headers $Headers -ContentType "application/json" -Body $payload | Out-Null
    } catch {
      throw (Get-GitHubApiFailureMessage -ErrorRecord $_ -Uri "$base/$Name")
    }
    Write-Host "UPDATED variable: $Name"
  } else {
    try {
      Invoke-RestMethod -Method Post -Uri $base -Headers $Headers -ContentType "application/json" -Body $payload | Out-Null
    } catch {
      throw (Get-GitHubApiFailureMessage -ErrorRecord $_ -Uri $base)
    }
    Write-Host "CREATED variable: $Name"
  }
}

Upsert-RepoVariable -Owner $Owner -Repo $Repo -Name "SONAR_PROJECT_KEY" -Value $SonarProjectKey -Headers $headers

if (-not [string]::IsNullOrWhiteSpace($SonarHostUrl)) {
  Upsert-RepoVariable -Owner $Owner -Repo $Repo -Name "SONAR_HOST_URL" -Value $SonarHostUrl -Headers $headers
}

if (-not [string]::IsNullOrWhiteSpace($GitGovUrl)) {
  Upsert-RepoVariable -Owner $Owner -Repo $Repo -Name "GITGOV_URL" -Value $GitGovUrl -Headers $headers
}

Write-Host "PASS: CI variables bootstrap completed."
