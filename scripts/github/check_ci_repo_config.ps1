param(
  [string]$Owner = "yohandry10",
  [string]$Repo = "Git-Gov",
  [string]$GitHubToken = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$tokenCandidates = @(@($GitHubToken, $env:GITHUB_TOKEN, $env:GH_TOKEN, $env:GITHUB_PAT) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
if ($tokenCandidates.Count -eq 0) {
  Write-Error "Missing GitHub token. Provide -GitHubToken or set GITHUB_TOKEN/GH_TOKEN/GITHUB_PAT with repo/actions read access."
  exit 1
}
$token = $tokenCandidates[0]

$headers = @{
  Authorization = "Bearer $token"
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
  try {
    $response = Invoke-RestMethod -Method Get -Uri $Uri -Headers $Headers
  } catch {
    if ($_.Exception.Response) {
      $reader = New-Object IO.StreamReader($_.Exception.Response.GetResponseStream())
      $body = $reader.ReadToEnd()
      throw "GitHub API request failed ($Uri): $body"
    }
    throw
  }

  $names = @()
  if ($null -eq $response) {
    return $names
  }

  $items = @()
  if ($response.PSObject.Properties.Name -contains $CollectionField) {
    $items = @($response.$CollectionField)
  }

  foreach ($item in @($items)) {
    if ($null -ne $item -and $item.PSObject.Properties.Name -contains "name" -and -not [string]::IsNullOrWhiteSpace([string]$item.name)) {
      $names += ([string]$item.name).Trim().ToUpperInvariant()
    }
  }
  return @($names | Select-Object -Unique)
}

$base = "https://api.github.com/repos/$Owner/$Repo/actions"
$secretNames = Get-NameSet -Uri "$base/secrets?per_page=100" -CollectionField "secrets" -Headers $headers
$variableNames = Get-NameSet -Uri "$base/variables?per_page=100" -CollectionField "variables" -Headers $headers
if ($null -eq $secretNames) { $secretNames = @() }
if ($null -eq $variableNames) { $variableNames = @() }

function Set-ContainsName {
  param(
    [Parameter()][object]$SetObject,
    [Parameter(Mandatory = $true)][string]$Name
  )
  if ($null -eq $SetObject) { return $false }
  $normalized = $Name.Trim().ToUpperInvariant()
  return @($SetObject) -contains $normalized
}

$missingRequiredSecrets = @()
foreach ($name in $requiredSecrets) {
  if (-not (Set-ContainsName -SetObject $secretNames -Name $name)) {
    $missingRequiredSecrets += $name
  }
}

$missingRequiredVariables = @()
foreach ($name in $requiredVariables) {
  if (-not (Set-ContainsName -SetObject $variableNames -Name $name)) {
    $missingRequiredVariables += $name
  }
}

$missingOptionalSecrets = @()
foreach ($name in $optionalSecrets) {
  if (-not (Set-ContainsName -SetObject $secretNames -Name $name)) {
    $missingOptionalSecrets += $name
  }
}

$missingOptionalVariables = @()
foreach ($name in $optionalVariables) {
  if (-not (Set-ContainsName -SetObject $variableNames -Name $name)) {
    $missingOptionalVariables += $name
  }
}

Write-Host "Repository: $Owner/$Repo"
Write-Host ""
Write-Host "Required secrets:"
foreach ($name in $requiredSecrets) {
  $status = if (Set-ContainsName -SetObject $secretNames -Name $name) { "OK" } else { "MISSING" }
  Write-Host ("  [{0}] {1}" -f $status, $name)
}
Write-Host ""
Write-Host "Optional secrets:"
foreach ($name in $optionalSecrets) {
  $status = if (Set-ContainsName -SetObject $secretNames -Name $name) { "OK" } else { "MISSING" }
  Write-Host ("  [{0}] {1}" -f $status, $name)
}
Write-Host ""
Write-Host "Required variables:"
foreach ($name in $requiredVariables) {
  $status = if (Set-ContainsName -SetObject $variableNames -Name $name) { "OK" } else { "MISSING" }
  Write-Host ("  [{0}] {1}" -f $status, $name)
}
Write-Host ""
Write-Host "Optional variables:"
foreach ($name in $optionalVariables) {
  $status = if (Set-ContainsName -SetObject $variableNames -Name $name) { "OK" } else { "MISSING" }
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
