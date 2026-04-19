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

$tokenCandidates = @(@($GitHubToken, $env:GITHUB_TOKEN, $env:GH_TOKEN, $env:GITHUB_PAT, $env:GITHUB_PERSONAL_ACCESS_TOKEN) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
if ($tokenCandidates.Count -eq 0) {
  Write-Error "Missing GitHub token. Provide -GitHubToken or set GITHUB_TOKEN/GH_TOKEN/GITHUB_PAT/GITHUB_PERSONAL_ACCESS_TOKEN."
  exit 1
}
$token = $tokenCandidates[0]

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
    $exists = $false
  }

  $payload = @{ name = $Name; value = $Value } | ConvertTo-Json
  if ($exists) {
    Invoke-RestMethod -Method Patch -Uri "$base/$Name" -Headers $Headers -ContentType "application/json" -Body $payload | Out-Null
    Write-Host "UPDATED variable: $Name"
  } else {
    Invoke-RestMethod -Method Post -Uri $base -Headers $Headers -ContentType "application/json" -Body $payload | Out-Null
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
