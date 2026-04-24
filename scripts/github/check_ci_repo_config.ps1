param(
  [string]$Owner = "",
  [string]$Repo = "",
  [string]$GitHubToken = "",
  # When set, SONAR_TOKEN and SONAR_PROJECT_KEY become optional.
  [switch]$AllowMissingSonar,
  # When set, require telemetry publish config for GitGov ingest.
  [switch]$RequireGitGovTelemetry,
  # When set, 403 from GitHub Actions secrets/variables endpoints is reported as warning instead of hard fail.
  [switch]$NoFailOnForbidden
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
. (Join-Path $scriptRoot "_token_helpers.ps1")

$repoInfo = Resolve-GitHubRepoCoordinates -Owner $Owner -Repo $Repo -ScriptRoot $scriptRoot
$Owner = $repoInfo.Owner
$Repo = $repoInfo.Repo
if ([string]::IsNullOrWhiteSpace($Owner) -or [string]::IsNullOrWhiteSpace($Repo)) {
  Write-Error "Could not resolve GitHub repository coordinates. Provide -Owner and -Repo, set GITHUB_REPOSITORY, or configure git remote origin to github.com/<owner>/<repo>."
  exit 1
}

$token = Resolve-GitHubToken -ExplicitToken $GitHubToken -ScriptRoot $scriptRoot
if ([string]::IsNullOrWhiteSpace($token)) {
  Write-Error "Missing GitHub token. Provide -GitHubToken, set GITHUB_TOKEN/GH_TOKEN/GITHUB_PAT/GITHUB_PERSONAL_ACCESS_TOKEN, or define GITHUB_PERSONAL_ACCESS_TOKEN in gitgov/gitgov-server/.env (repo/actions read access required)."
  exit 1
}

$headers = @{
  Authorization = "Bearer $token"
  Accept = "application/vnd.github+json"
  "X-GitHub-Api-Version" = "2022-11-28"
  "User-Agent" = "gitgov-ci-config-check"
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

$requiredSecrets = @()
$optionalSecrets = @("GITGOV_JENKINS_SECRET")
$requiredVariables = @()
$optionalVariables = @("SONAR_HOST_URL")

if (-not $AllowMissingSonar) {
  $requiredSecrets += "SONAR_TOKEN"
  $requiredVariables += "SONAR_PROJECT_KEY"
} else {
  $optionalSecrets += "SONAR_TOKEN"
  $optionalVariables += "SONAR_PROJECT_KEY"
}

if ($RequireGitGovTelemetry) {
  $requiredSecrets += "GITGOV_API_KEY"
  $requiredVariables += "GITGOV_URL"
} else {
  $optionalSecrets += "GITGOV_API_KEY"
  $optionalVariables += "GITGOV_URL"
}

function Get-NameSet {
  param(
    [string]$Uri,
    [string]$CollectionField,
    [hashtable]$Headers,
    [string]$Kind
  )
  $result = [pscustomobject]@{
    Names = @()
    PermissionDenied = $false
  }
  try {
    $response = Invoke-RestMethod -Method Get -Uri $Uri -Headers $Headers
  } catch {
    $response = $_.Exception.Response
    $statusCode = if ($null -ne $response) { [int]$response.StatusCode.value__ } else { -1 }
    if ($statusCode -eq 403 -and $NoFailOnForbidden) {
      $result.PermissionDenied = $true
      Write-Warning ("Skipping {0} visibility check due to token permission limits (403). Use a token with Actions {0} read access for strict validation." -f $Kind)
      return $result
    }
    throw (Get-GitHubApiFailureMessage -ErrorRecord $_ -Uri $Uri)
  }

  if ($null -eq $response) {
    return $result
  }

  $items = @()
  if ($response.PSObject.Properties.Name -contains $CollectionField) {
    $items = @($response.$CollectionField)
  }

  foreach ($item in @($items)) {
    if ($null -ne $item -and $item.PSObject.Properties.Name -contains "name" -and -not [string]::IsNullOrWhiteSpace([string]$item.name)) {
      $result.Names += ([string]$item.name).Trim().ToUpperInvariant()
    }
  }
  $result.Names = @($result.Names | Select-Object -Unique)
  return $result
}

$base = "https://api.github.com/repos/$Owner/$Repo/actions"
$secretResult = Get-NameSet -Uri "$base/secrets?per_page=100" -CollectionField "secrets" -Headers $headers -Kind "secrets"
$variableResult = Get-NameSet -Uri "$base/variables?per_page=100" -CollectionField "variables" -Headers $headers -Kind "variables"
$secretNames = $secretResult.Names
$variableNames = $variableResult.Names
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
if (-not $secretResult.PermissionDenied) {
  foreach ($name in $requiredSecrets) {
    if (-not (Set-ContainsName -SetObject $secretNames -Name $name)) {
      $missingRequiredSecrets += $name
    }
  }
}

$missingRequiredVariables = @()
if (-not $variableResult.PermissionDenied) {
  foreach ($name in $requiredVariables) {
    if (-not (Set-ContainsName -SetObject $variableNames -Name $name)) {
      $missingRequiredVariables += $name
    }
  }
}

$missingOptionalSecrets = @()
if (-not $secretResult.PermissionDenied) {
  foreach ($name in $optionalSecrets) {
    if (-not (Set-ContainsName -SetObject $secretNames -Name $name)) {
      $missingOptionalSecrets += $name
    }
  }
}

$missingOptionalVariables = @()
if (-not $variableResult.PermissionDenied) {
  foreach ($name in $optionalVariables) {
    if (-not (Set-ContainsName -SetObject $variableNames -Name $name)) {
      $missingOptionalVariables += $name
    }
  }
}

Write-Host "Repository: $Owner/$Repo"
Write-Host ""
Write-Host "Required secrets:"
if ($secretResult.PermissionDenied) {
  Write-Host "  [UNKNOWN] Skipped (token cannot read Actions secrets)."
} else {
  foreach ($name in $requiredSecrets) {
    $status = if (Set-ContainsName -SetObject $secretNames -Name $name) { "OK" } else { "MISSING" }
    Write-Host ("  [{0}] {1}" -f $status, $name)
  }
}
Write-Host ""
Write-Host "Optional secrets:"
if ($secretResult.PermissionDenied) {
  Write-Host "  [UNKNOWN] Skipped (token cannot read Actions secrets)."
} else {
  foreach ($name in $optionalSecrets) {
    $status = if (Set-ContainsName -SetObject $secretNames -Name $name) { "OK" } else { "MISSING" }
    Write-Host ("  [{0}] {1}" -f $status, $name)
  }
}
Write-Host ""
Write-Host "Required variables:"
if ($variableResult.PermissionDenied) {
  Write-Host "  [UNKNOWN] Skipped (token cannot read Actions variables)."
} else {
  foreach ($name in $requiredVariables) {
    $status = if (Set-ContainsName -SetObject $variableNames -Name $name) { "OK" } else { "MISSING" }
    Write-Host ("  [{0}] {1}" -f $status, $name)
  }
}
Write-Host ""
Write-Host "Optional variables:"
if ($variableResult.PermissionDenied) {
  Write-Host "  [UNKNOWN] Skipped (token cannot read Actions variables)."
} else {
  foreach ($name in $optionalVariables) {
    $status = if (Set-ContainsName -SetObject $variableNames -Name $name) { "OK" } else { "MISSING" }
    Write-Host ("  [{0}] {1}" -f $status, $name)
  }
}

if ($missingRequiredSecrets.Count -gt 0 -or $missingRequiredVariables.Count -gt 0) {
  Write-Host ""
  Write-Error ("FAIL: Missing required repo CI config. Secrets: [{0}] Variables: [{1}]" -f ($missingRequiredSecrets -join ", "), ($missingRequiredVariables -join ", "))
  exit 1
}

Write-Host ""
if ($secretResult.PermissionDenied -or $variableResult.PermissionDenied) {
  Write-Host "PASS (best-effort): required validation completed with limited token visibility on Actions config."
} elseif ($missingOptionalSecrets.Count -gt 0 -or $missingOptionalVariables.Count -gt 0) {
  Write-Host ("PASS (required complete, optional missing). Optional secrets missing: [{0}] | Optional variables missing: [{1}]" -f ($missingOptionalSecrets -join ", "), ($missingOptionalVariables -join ", "))
} else {
  Write-Host "PASS (all required and optional CI repo config present)."
}
