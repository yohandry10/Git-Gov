param(
  [string]$Owner = "yohandry10",
  [string]$Repo = "Git-Gov"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not $env:GITHUB_TOKEN -or [string]::IsNullOrWhiteSpace($env:GITHUB_TOKEN)) {
  Write-Error "Missing GITHUB_TOKEN. Export a token with repo/actions read access."
  exit 1
}

$headers = @{
  Authorization = "Bearer $($env:GITHUB_TOKEN)"
  Accept = "application/vnd.github+json"
  "X-GitHub-Api-Version" = "2022-11-28"
  "User-Agent" = "gitgov-ci-config-check"
}

$requiredSecrets = @("SONAR_TOKEN", "GITGOV_API_KEY")
$optionalSecrets = @("GITGOV_JENKINS_SECRET")
$requiredVariables = @("SONAR_PROJECT_KEY")
$optionalVariables = @("SONAR_HOST_URL", "GITGOV_URL")

function Get-NameSet {
  param(
    [string]$Uri,
    [string]$CollectionField,
    [hashtable]$Headers
  )
  $response = Invoke-RestMethod -Method Get -Uri $Uri -Headers $Headers
  $items = @()
  if ($null -ne $response -and $null -ne $response.$CollectionField) {
    $items = @($response.$CollectionField)
  }
  $set = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::OrdinalIgnoreCase)
  foreach ($item in $items) {
    if ($item.name) {
      [void]$set.Add([string]$item.name)
    }
  }
  return $set
}

$base = "https://api.github.com/repos/$Owner/$Repo/actions"
$secretNames = Get-NameSet -Uri "$base/secrets?per_page=100" -CollectionField "secrets" -Headers $headers
$variableNames = Get-NameSet -Uri "$base/variables?per_page=100" -CollectionField "variables" -Headers $headers

$missingRequiredSecrets = @($requiredSecrets | Where-Object { -not $secretNames.Contains($_) })
$missingRequiredVariables = @($requiredVariables | Where-Object { -not $variableNames.Contains($_) })
$missingOptionalSecrets = @($optionalSecrets | Where-Object { -not $secretNames.Contains($_) })
$missingOptionalVariables = @($optionalVariables | Where-Object { -not $variableNames.Contains($_) })

Write-Host "Repository: $Owner/$Repo"
Write-Host ""
Write-Host "Required secrets:"
foreach ($name in $requiredSecrets) {
  $status = if ($secretNames.Contains($name)) { "OK" } else { "MISSING" }
  Write-Host ("  [{0}] {1}" -f $status, $name)
}
Write-Host ""
Write-Host "Optional secrets:"
foreach ($name in $optionalSecrets) {
  $status = if ($secretNames.Contains($name)) { "OK" } else { "MISSING" }
  Write-Host ("  [{0}] {1}" -f $status, $name)
}
Write-Host ""
Write-Host "Required variables:"
foreach ($name in $requiredVariables) {
  $status = if ($variableNames.Contains($name)) { "OK" } else { "MISSING" }
  Write-Host ("  [{0}] {1}" -f $status, $name)
}
Write-Host ""
Write-Host "Optional variables:"
foreach ($name in $optionalVariables) {
  $status = if ($variableNames.Contains($name)) { "OK" } else { "MISSING" }
  Write-Host ("  [{0}] {1}" -f $status, $name)
}

if ($missingRequiredSecrets.Count -gt 0 -or $missingRequiredVariables.Count -gt 0) {
  Write-Host ""
  Write-Error ("FAIL: Missing required repo CI config. Secrets: [{0}] Variables: [{1}]" -f ($missingRequiredSecrets -join ", "), ($missingRequiredVariables -join ", "))
  exit 1
}

Write-Host ""
if ($missingOptionalSecrets.Count -gt 0 -or $missingOptionalVariables.Count -gt 0) {
  Write-Host ("PASS (required complete, optional missing). Optional secrets missing: [{0}] | Optional variables missing: [{1}]" -f ($missingOptionalSecrets -join ", "), ($missingOptionalVariables -join ", "))
} else {
  Write-Host "PASS (all required and optional CI repo config present)."
}
