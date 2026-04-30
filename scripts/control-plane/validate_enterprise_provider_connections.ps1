param(
  [string]$ProfilePath = "docs/examples/enterprise-adoption-profile.example.json",
  [string[]]$EnvFiles = @("gitgov\.env", "gitgov\gitgov-server\.env"),
  [string[]]$Providers,
  [string]$RepositoryFullName,
  [string]$JiraProjectKey,
  [string]$JenkinsJobName,
  [string]$SonarProjectKey,
  [string]$OutputPath = "",
  [switch]$ReportOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")

function Fail-ProviderValidation {
  param([Parameter(Mandatory = $true)][string]$Message)
  Write-Host "[FAIL] $Message"
  exit 1
}

function Resolve-RepoPath {
  param([Parameter(Mandatory = $true)][string]$Path)

  if ([System.IO.Path]::IsPathRooted($Path)) {
    return $Path
  }
  return Join-Path $repoRoot $Path
}

function Load-DotEnvNoPrint {
  param([string]$Path)

  $resolved = Resolve-RepoPath $Path
  if (-not (Test-Path -LiteralPath $resolved)) {
    return
  }

  foreach ($line in Get-Content -LiteralPath $resolved) {
    $trimmed = $line.Trim()
    if ($trimmed.Length -eq 0 -or $trimmed.StartsWith("#") -or -not $trimmed.Contains("=")) {
      continue
    }
    $parts = $trimmed -split "=", 2
    $name = $parts[0].Trim()
    $value = $parts[1].Trim()
    if (($value.StartsWith('"') -and $value.EndsWith('"')) -or ($value.StartsWith("'") -and $value.EndsWith("'"))) {
      $value = $value.Substring(1, $value.Length - 2)
    }
    if (-not [string]::IsNullOrWhiteSpace($name)) {
      [Environment]::SetEnvironmentVariable($name, $value, "Process")
    }
  }
}

function Get-ProfileProperty {
  param(
    [Parameter(Mandatory = $true)]$Profile,
    [Parameter(Mandatory = $true)][string]$Name
  )

  $property = $Profile.PSObject.Properties[$Name]
  if ($null -eq $property) {
    return $null
  }
  return $property.Value
}

function Normalize-List {
  param([string[]]$Values)

  if ($null -eq $Values) {
    return @()
  }

  return @(
    $Values |
      ForEach-Object { [string]$_ } |
      Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
      ForEach-Object { $_.Trim().ToLowerInvariant() } |
      Sort-Object -Unique
  )
}

function Get-EnvValue {
  param([Parameter(Mandatory = $true)][string]$Name)

  return [Environment]::GetEnvironmentVariable($Name, "Process")
}

function Get-FirstEnvValue {
  param([Parameter(Mandatory = $true)][string[]]$Names)

  foreach ($name in $Names) {
    $value = Get-EnvValue $name
    if (-not [string]::IsNullOrWhiteSpace($value)) {
      return [pscustomobject]@{ Name = $name; Value = $value }
    }
  }
  return $null
}

function Get-SecretValues {
  $secretValues = New-Object System.Collections.Generic.List[string]
  foreach ($entry in [Environment]::GetEnvironmentVariables("Process").GetEnumerator()) {
    $name = [string]$entry.Key
    $value = [string]$entry.Value
    if ($value.Length -lt 6) {
      continue
    }
    if ($name -match '(TOKEN|SECRET|PASSWORD|API_KEY|DATABASE_URL|PRIVATE_KEY)') {
      $secretValues.Add($value) | Out-Null
    }
  }
  return @($secretValues.ToArray() | Sort-Object -Unique)
}

$script:SecretValues = @()

function Protect-SecretText {
  param([string]$Text)

  if ([string]::IsNullOrWhiteSpace($Text)) {
    return ""
  }

  $sanitized = $Text
  foreach ($secret in $script:SecretValues) {
    if (-not [string]::IsNullOrWhiteSpace($secret)) {
      $sanitized = $sanitized.Replace($secret, "[redacted]")
    }
  }
  return $sanitized
}

function New-ProviderResult {
  param(
    [Parameter(Mandatory = $true)][string]$Provider,
    [Parameter(Mandatory = $true)][string]$Status,
    [string[]]$RequiredConfig = @(),
    [string[]]$MissingConfig = @(),
    [hashtable]$Details = @{},
    [string]$AuthSource = "",
    [string]$ErrorMessage = ""
  )

  [ordered]@{
    provider = $Provider
    status = $Status
    auth_source = $AuthSource
    required_config = @($RequiredConfig)
    missing_config = @($MissingConfig)
    details = $Details
    error = Protect-SecretText $ErrorMessage
  }
}

function Test-RequiredConfig {
  param([Parameter(Mandatory = $true)][string[]]$Names)

  return @($Names | Where-Object { [string]::IsNullOrWhiteSpace((Get-EnvValue $_)) })
}

function Invoke-JsonGet {
  param(
    [Parameter(Mandatory = $true)][string]$Uri,
    [hashtable]$Headers = @{}
  )

  Invoke-RestMethod -Method GET -Uri $Uri -Headers $Headers -TimeoutSec 30
}

function Get-GhCommand {
  $localGh = "C:\Users\PC\Tools\gh\bin\gh.exe"
  if (Test-Path -LiteralPath $localGh) {
    return $localGh
  }
  $command = Get-Command gh -ErrorAction SilentlyContinue
  if ($null -ne $command) {
    return $command.Source
  }
  return $null
}

function Invoke-GitHubCheck {
  param([Parameter(Mandatory = $true)][string]$RepoFullName)

  $required = @("GITHUB_TOKEN or GH_TOKEN or authenticated gh CLI")
  $token = Get-FirstEnvValue @("GITHUB_TOKEN", "GH_TOKEN")
  try {
    if ($null -ne $token) {
      $headers = @{
        Authorization = "Bearer $($token.Value)"
        Accept = "application/vnd.github+json"
        "X-GitHub-Api-Version" = "2022-11-28"
      }
      $repo = Invoke-JsonGet -Uri "https://api.github.com/repos/$RepoFullName" -Headers $headers
      return New-ProviderResult -Provider "github" -Status "ready" -RequiredConfig $required -AuthSource $token.Name -Details @{
        repository = $repo.full_name
        default_branch = $repo.default_branch
        visibility = $repo.visibility
      }
    }

    $gh = Get-GhCommand
    if ([string]::IsNullOrWhiteSpace($gh)) {
      return New-ProviderResult -Provider "github" -Status "missing-config" -RequiredConfig $required -MissingConfig $required
    }

    $authOutput = & $gh auth status 2>&1
    if ($LASTEXITCODE -ne 0) {
      return New-ProviderResult -Provider "github" -Status "missing-config" -RequiredConfig $required -MissingConfig $required -ErrorMessage ($authOutput -join "`n")
    }

    $repoJson = & $gh repo view $RepoFullName --json nameWithOwner,defaultBranchRef,visibility 2>&1
    if ($LASTEXITCODE -ne 0) {
      return New-ProviderResult -Provider "github" -Status "failed" -RequiredConfig $required -AuthSource "gh-cli" -ErrorMessage ($repoJson -join "`n")
    }

    $repo = ($repoJson | Out-String) | ConvertFrom-Json
    return New-ProviderResult -Provider "github" -Status "ready" -RequiredConfig $required -AuthSource "gh-cli" -Details @{
      repository = $repo.nameWithOwner
      default_branch = $repo.defaultBranchRef.name
      visibility = $repo.visibility
    }
  } catch {
    return New-ProviderResult -Provider "github" -Status "failed" -RequiredConfig $required -AuthSource $(if ($null -ne $token) { $token.Name } else { "gh-cli" }) -ErrorMessage $_.Exception.Message
  }
}

function Invoke-JiraCheck {
  param([Parameter(Mandatory = $true)][string]$ProjectKey)

  $required = @("JIRA_BASE_URL", "JIRA_EMAIL", "JIRA_API_TOKEN", "JIRA_PROJECT_KEY or profile jira_project_key")
  $missing = @(Test-RequiredConfig @("JIRA_BASE_URL", "JIRA_EMAIL", "JIRA_API_TOKEN"))
  if ([string]::IsNullOrWhiteSpace($ProjectKey)) {
    $missing += "JIRA_PROJECT_KEY or profile jira_project_key"
  }
  if ($missing.Count -gt 0) {
    return New-ProviderResult -Provider "jira" -Status "missing-config" -RequiredConfig $required -MissingConfig $missing
  }

  try {
    $auth = [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes("$((Get-EnvValue 'JIRA_EMAIL')):$((Get-EnvValue 'JIRA_API_TOKEN'))"))
    $headers = @{ Authorization = "Basic $auth"; Accept = "application/json" }
    $baseUrl = (Get-EnvValue "JIRA_BASE_URL").TrimEnd("/")
    $project = Invoke-JsonGet -Uri "$baseUrl/rest/api/3/project/$ProjectKey" -Headers $headers
    return New-ProviderResult -Provider "jira" -Status "ready" -RequiredConfig $required -AuthSource "JIRA_EMAIL/JIRA_API_TOKEN" -Details @{
      base_url = $baseUrl
      project_key = $project.key
      project_id = $project.id
      project_name = $project.name
    }
  } catch {
    return New-ProviderResult -Provider "jira" -Status "failed" -RequiredConfig $required -AuthSource "JIRA_EMAIL/JIRA_API_TOKEN" -ErrorMessage $_.Exception.Message
  }
}

function ConvertTo-JenkinsJobPath {
  param([Parameter(Mandatory = $true)][string]$JobName)

  $segments = @($JobName -split "/" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
  return ($segments | ForEach-Object { "job/$([Uri]::EscapeDataString($_))" }) -join "/"
}

function Invoke-JenkinsCheck {
  param([string]$JobName)

  $required = @("JENKINS_SERVER_URL", "JENKINS_USER", "JENKINS_API_TOKEN")
  $missing = @(Test-RequiredConfig $required)
  if ($missing.Count -gt 0) {
    return New-ProviderResult -Provider "jenkins" -Status "missing-config" -RequiredConfig $required -MissingConfig $missing
  }

  try {
    $auth = [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes("$((Get-EnvValue 'JENKINS_USER')):$((Get-EnvValue 'JENKINS_API_TOKEN'))"))
    $headers = @{ Authorization = "Basic $auth"; Accept = "application/json" }
    $baseUrl = (Get-EnvValue "JENKINS_SERVER_URL").TrimEnd("/")
    $who = Invoke-JsonGet -Uri "$baseUrl/whoAmI/api/json" -Headers $headers
    $details = @{
      base_url = $baseUrl
      authenticated = [bool]$who.authenticated
    }

    if (-not [string]::IsNullOrWhiteSpace($JobName)) {
      $jobPath = ConvertTo-JenkinsJobPath $JobName
      $job = Invoke-JsonGet -Uri "$baseUrl/$jobPath/api/json?tree=lastBuild[number,result,building]" -Headers $headers
      $details.job = $JobName
      $details.last_build = $job.lastBuild.number
      $details.last_result = $job.lastBuild.result
      $details.last_building = $job.lastBuild.building
    }

    return New-ProviderResult -Provider "jenkins" -Status "ready" -RequiredConfig $required -AuthSource "JENKINS_USER/JENKINS_API_TOKEN" -Details $details
  } catch {
    return New-ProviderResult -Provider "jenkins" -Status "failed" -RequiredConfig $required -AuthSource "JENKINS_USER/JENKINS_API_TOKEN" -ErrorMessage $_.Exception.Message
  }
}

function Invoke-SonarQubeCheck {
  param([string]$ProjectKey)

  $required = @("SONAR_HOST_URL", "SONAR_TOKEN", "SONAR_PROJECT_KEY or parameter")
  $missing = @(Test-RequiredConfig @("SONAR_HOST_URL", "SONAR_TOKEN"))
  if ([string]::IsNullOrWhiteSpace($ProjectKey)) {
    $missing += "SONAR_PROJECT_KEY or parameter"
  }
  if ($missing.Count -gt 0) {
    return New-ProviderResult -Provider "sonarqube" -Status "missing-config" -RequiredConfig $required -MissingConfig $missing
  }

  try {
    $auth = [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes("$((Get-EnvValue 'SONAR_TOKEN')):"))
    $headers = @{ Authorization = "Basic $auth"; Accept = "application/json" }
    $baseUrl = (Get-EnvValue "SONAR_HOST_URL").TrimEnd("/")
    $status = Invoke-JsonGet -Uri "$baseUrl/api/system/status" -Headers $headers
    $qualityGate = Invoke-JsonGet -Uri "$baseUrl/api/qualitygates/project_status?projectKey=$ProjectKey" -Headers $headers
    return New-ProviderResult -Provider "sonarqube" -Status "ready" -RequiredConfig $required -AuthSource "SONAR_TOKEN" -Details @{
      base_url = $baseUrl
      system_status = $status.status
      project_key = $ProjectKey
      quality_gate = $qualityGate.projectStatus.status
    }
  } catch {
    return New-ProviderResult -Provider "sonarqube" -Status "failed" -RequiredConfig $required -AuthSource "SONAR_TOKEN" -ErrorMessage $_.Exception.Message
  }
}

function Invoke-RenderCheck {
  $required = @("RENDER_API_KEY")
  $missing = @(Test-RequiredConfig $required)
  if ($missing.Count -gt 0) {
    return New-ProviderResult -Provider "render" -Status "missing-config" -RequiredConfig $required -MissingConfig $missing
  }

  try {
    $headers = @{ Authorization = "Bearer $((Get-EnvValue 'RENDER_API_KEY'))"; Accept = "application/json" }
    $services = Invoke-JsonGet -Uri "https://api.render.com/v1/services?limit=1" -Headers $headers
    return New-ProviderResult -Provider "render" -Status "ready" -RequiredConfig $required -AuthSource "RENDER_API_KEY" -Details @{
      api_reachable = $true
      sample_count = @($services).Count
    }
  } catch {
    return New-ProviderResult -Provider "render" -Status "failed" -RequiredConfig $required -AuthSource "RENDER_API_KEY" -ErrorMessage $_.Exception.Message
  }
}

function Invoke-VercelCheck {
  $required = @("VERCEL_TOKEN")
  $missing = @(Test-RequiredConfig $required)
  if ($missing.Count -gt 0) {
    return New-ProviderResult -Provider "vercel" -Status "missing-config" -RequiredConfig $required -MissingConfig $missing
  }

  try {
    $headers = @{ Authorization = "Bearer $((Get-EnvValue 'VERCEL_TOKEN'))"; Accept = "application/json" }
    $user = Invoke-JsonGet -Uri "https://api.vercel.com/v2/user" -Headers $headers
    return New-ProviderResult -Provider "vercel" -Status "ready" -RequiredConfig $required -AuthSource "VERCEL_TOKEN" -Details @{
      api_reachable = $true
      user_id = $user.user.id
    }
  } catch {
    return New-ProviderResult -Provider "vercel" -Status "failed" -RequiredConfig $required -AuthSource "VERCEL_TOKEN" -ErrorMessage $_.Exception.Message
  }
}

foreach ($envFile in $EnvFiles) {
  Load-DotEnvNoPrint $envFile
}
$script:SecretValues = Get-SecretValues

$profile = $null
if (-not [string]::IsNullOrWhiteSpace($ProfilePath)) {
  $resolvedProfile = Resolve-RepoPath $ProfilePath
  if (-not (Test-Path -LiteralPath $resolvedProfile)) {
    Fail-ProviderValidation "Profile file not found: $ProfilePath"
  }
  $profile = Get-Content -Raw -LiteralPath $resolvedProfile | ConvertFrom-Json
}

if ($null -ne $profile) {
  if ([string]::IsNullOrWhiteSpace($RepositoryFullName)) {
    $profileRepo = Get-ProfileProperty -Profile $profile -Name "repository_full_name"
    if ($profileRepo) {
      $RepositoryFullName = [string]$profileRepo
    }
  }
  if ([string]::IsNullOrWhiteSpace($JiraProjectKey)) {
    $profileJira = Get-ProfileProperty -Profile $profile -Name "jira_project_key"
    if ($profileJira) {
      $JiraProjectKey = [string]$profileJira
    }
  }
  if ($null -eq $Providers -or $Providers.Count -eq 0) {
    $profileProviders = Get-ProfileProperty -Profile $profile -Name "providers"
    if ($profileProviders) {
      $Providers = @($profileProviders)
    }
  }
}

if ([string]::IsNullOrWhiteSpace($RepositoryFullName)) {
  $RepositoryFullName = "yohandry10/Git-Gov"
}
if ([string]::IsNullOrWhiteSpace($JiraProjectKey)) {
  $JiraProjectKey = Get-EnvValue "JIRA_PROJECT_KEY"
}
if ([string]::IsNullOrWhiteSpace($JenkinsJobName)) {
  $JenkinsJobName = Get-EnvValue "JENKINS_JOB_NAME"
}
if ([string]::IsNullOrWhiteSpace($SonarProjectKey)) {
  $SonarProjectKey = Get-EnvValue "SONAR_PROJECT_KEY"
}
if ($null -eq $Providers -or $Providers.Count -eq 0) {
  $Providers = @("github", "jira", "jenkins", "sonarqube")
}

$knownProviders = @("github", "jira", "jenkins", "sonarqube", "render", "vercel")
$selectedProviders = Normalize-List $Providers
$unknownProviders = @($selectedProviders | Where-Object { $_ -notin $knownProviders })
if ($unknownProviders.Count -gt 0) {
  Fail-ProviderValidation "Unknown provider(s): $($unknownProviders -join ', '). Known providers: $($knownProviders -join ', ')."
}

$checks = New-Object System.Collections.Generic.List[object]
foreach ($provider in $selectedProviders) {
  $result = switch ($provider) {
    "github" { Invoke-GitHubCheck -RepoFullName $RepositoryFullName }
    "jira" { Invoke-JiraCheck -ProjectKey $JiraProjectKey }
    "jenkins" { Invoke-JenkinsCheck -JobName $JenkinsJobName }
    "sonarqube" { Invoke-SonarQubeCheck -ProjectKey $SonarProjectKey }
    "render" { Invoke-RenderCheck }
    "vercel" { Invoke-VercelCheck }
  }
  $checks.Add($result) | Out-Null
}

$totals = [ordered]@{
  ready = @($checks | Where-Object { $_.status -eq "ready" }).Count
  missing_config = @($checks | Where-Object { $_.status -eq "missing-config" }).Count
  failed = @($checks | Where-Object { $_.status -eq "failed" }).Count
}
$overallStatus = if ($totals.failed -gt 0) {
  "failed"
} elseif ($totals.missing_config -gt 0) {
  "missing-config"
} else {
  "ready"
}

$report = [ordered]@{
  generated_at = (Get-Date).ToUniversalTime().ToString("o")
  status = $overallStatus
  report_only = [bool]$ReportOnly
  repository_full_name = $RepositoryFullName
  jira_project_key = $JiraProjectKey
  selected_providers = @($selectedProviders)
  totals = $totals
  safety = @{
    reads_env_files = $true
    prints_secret_values = $false
    writes_secret_values = $false
    mutates_provider_state = $false
    mutates_customer_repository = $false
  }
  checks = @($checks.ToArray())
}

$json = $report | ConvertTo-Json -Depth 10
if (-not [string]::IsNullOrWhiteSpace($OutputPath)) {
  $parent = Split-Path -Parent $OutputPath
  if (-not [string]::IsNullOrWhiteSpace($parent)) {
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
  }
  Set-Content -LiteralPath $OutputPath -Value $json -Encoding UTF8
  Write-Host "Wrote provider connection validation report: $OutputPath"
}

Write-Output $json

if (-not $ReportOnly -and $overallStatus -ne "ready") {
  exit 1
}
