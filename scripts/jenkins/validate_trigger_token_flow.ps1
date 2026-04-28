param(
  [string[]]$EnvFiles = @("gitgov\.env", "gitgov\gitgov-server\.env"),
  [string]$JenkinsUrl = "",
  [string]$JobName = "",
  [string]$Username = "",
  [string]$ApiTokenOrPassword = "",
  [string]$BuildTriggerToken = "",
  [switch]$RequireTriggerToken,
  [switch]$Trigger,
  [string]$OutputPath = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")

function Load-DotEnvNoPrint {
  param([string]$Path)

  $resolved = if ([System.IO.Path]::IsPathRooted($Path)) {
    $Path
  } else {
    Join-Path $repoRoot $Path
  }

  if (!(Test-Path $resolved)) { return }

  Get-Content $resolved | ForEach-Object {
    $line = $_.Trim()
    if (!$line -or $line.StartsWith("#") -or !$line.Contains("=")) { return }
    $parts = $line -split "=", 2
    $name = $parts[0].Trim()
    $value = $parts[1].Trim().Trim('"')
    if (![string]::IsNullOrWhiteSpace($name)) {
      [Environment]::SetEnvironmentVariable($name, $value, "Process")
    }
  }
}

function Get-BasicAuthHeader {
  param([string]$User, [string]$Token)

  $pair = "{0}:{1}" -f $User, $Token
  $encoded = [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes($pair))
  @{ Authorization = "Basic $encoded"; Accept = "application/json" }
}

function ConvertTo-JenkinsJobPath {
  param([string]$Name)

  $parts = $Name -split "/"
  $encodedParts = @($parts | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | ForEach-Object {
      "job/" + [uri]::EscapeDataString($_)
    })
  return ($encodedParts -join "/")
}

foreach ($envFile in $EnvFiles) {
  Load-DotEnvNoPrint $envFile
}

if ([string]::IsNullOrWhiteSpace($JenkinsUrl)) { $JenkinsUrl = $env:JENKINS_SERVER_URL }
if ([string]::IsNullOrWhiteSpace($JobName)) { $JobName = $env:JENKINS_JOB_NAME }
if ([string]::IsNullOrWhiteSpace($Username)) { $Username = $env:JENKINS_USER }
if ([string]::IsNullOrWhiteSpace($ApiTokenOrPassword)) { $ApiTokenOrPassword = $env:JENKINS_API_TOKEN }
if ([string]::IsNullOrWhiteSpace($BuildTriggerToken)) { $BuildTriggerToken = $env:JENKINS_BUILD_TRIGGER_TOKEN }

$warnings = New-Object System.Collections.Generic.List[string]

if ([string]::IsNullOrWhiteSpace($JenkinsUrl)) { throw "Missing JenkinsUrl or JENKINS_SERVER_URL." }
if ([string]::IsNullOrWhiteSpace($JobName)) { throw "Missing JobName or JENKINS_JOB_NAME." }
if ([string]::IsNullOrWhiteSpace($Username) -or [string]::IsNullOrWhiteSpace($ApiTokenOrPassword)) {
  throw "Missing Jenkins API credentials. JENKINS_USER and JENKINS_API_TOKEN are required for safe inspection."
}

$baseUrl = $JenkinsUrl.TrimEnd("/")
$jobPath = ConvertTo-JenkinsJobPath $JobName
$headers = Get-BasicAuthHeader -User $Username -Token $ApiTokenOrPassword

$job = Invoke-RestMethod -Uri "$baseUrl/$jobPath/api/json?tree=name,fullName,lastBuild[number,result,building,url]" -Headers $headers -TimeoutSec 30
$configResponse = Invoke-WebRequest -Uri "$baseUrl/$jobPath/config.xml" -Headers $headers -TimeoutSec 30 -UseBasicParsing
$xmlRaw = [string]$configResponse.Content
$normalizedXml = [regex]::Replace(
  $xmlRaw,
  '<\?xml\s+version=(["''])1\.1\1',
  '<?xml version="1.0"',
  [System.Text.RegularExpressions.RegexOptions]::IgnoreCase
)

[xml]$xml = $normalizedXml
$authTokenNodes = @($xml.SelectNodes("//*[local-name()='authToken']"))
$jobTriggerTokenConfigured = $authTokenNodes.Count -gt 0 -and @($authTokenNodes | Where-Object { -not [string]::IsNullOrWhiteSpace($_.InnerText) }).Count -gt 0
$triggerTokenLoaded = -not [string]::IsNullOrWhiteSpace($BuildTriggerToken)
$triggerTokenMatchesConfig = $false

if ($triggerTokenLoaded -and $jobTriggerTokenConfigured) {
  $triggerTokenMatchesConfig = @($authTokenNodes | Where-Object { $_.InnerText -eq $BuildTriggerToken }).Count -gt 0
}

if (!$triggerTokenLoaded) {
  $warnings.Add("JENKINS_BUILD_TRIGGER_TOKEN is not loaded; trigger-only URL validation is dry-run metadata only.")
}
if ($triggerTokenLoaded -and !$jobTriggerTokenConfigured) {
  $warnings.Add("A trigger token is loaded locally, but the Jenkins job config did not expose an authToken node.")
}
if ($triggerTokenLoaded -and $jobTriggerTokenConfigured -and !$triggerTokenMatchesConfig) {
  $warnings.Add("A trigger token is loaded locally, but it does not match the Jenkins job authToken node.")
}

$triggerReady = $triggerTokenLoaded -and (!$jobTriggerTokenConfigured -or $triggerTokenMatchesConfig)
$triggerUrlRedacted = "$baseUrl/$jobPath/build?token=***"
$triggerAttempted = $false
$triggerAccepted = $false
$queueLocation = ""
$triggerStatusCode = $null

if ($Trigger) {
  $triggerAttempted = $true
  if (!$triggerReady) {
    throw "Trigger requested but trigger token is missing or does not match the Jenkins job configuration."
  }

  $triggerUrl = "$baseUrl/$jobPath/build?token=$([uri]::EscapeDataString($BuildTriggerToken))"
  try {
    $response = Invoke-WebRequest -Method Post -Uri $triggerUrl -TimeoutSec 30 -MaximumRedirection 0 -ErrorAction Stop -UseBasicParsing
    $triggerStatusCode = [int]$response.StatusCode
    $triggerAccepted = $triggerStatusCode -in @(200, 201, 202, 302)
    if ($response.Headers["Location"]) {
      $queueLocation = [string]$response.Headers["Location"]
    }
  } catch {
    if ($_.Exception.Response -and $_.Exception.Response.StatusCode) {
      $triggerStatusCode = [int]$_.Exception.Response.StatusCode
    }
    throw "Trigger-only build request failed with status $triggerStatusCode."
  }
}

$ok = $true
if ($RequireTriggerToken -and !$triggerReady) { $ok = $false }
if ($Trigger -and !$triggerAccepted) { $ok = $false }

$result = [ordered]@{
  ok = $ok
  checked_at_utc = (Get-Date).ToUniversalTime().ToString("o")
  jenkins_url = $baseUrl
  job_name = $JobName
  api_inspection = [ordered]@{
    ok = $true
    last_build = if ($job.lastBuild) { $job.lastBuild.number } else { $null }
    last_result = if ($job.lastBuild) { $job.lastBuild.result } else { $null }
    building = if ($job.lastBuild) { $job.lastBuild.building } else { $null }
  }
  trigger_only = [ordered]@{
    trigger_token_loaded = $triggerTokenLoaded
    job_auth_token_configured = $jobTriggerTokenConfigured
    token_matches_job_config = $triggerTokenMatchesConfig
    trigger_ready = $triggerReady
    trigger_url = $triggerUrlRedacted
    trigger_attempted = $triggerAttempted
    trigger_accepted = $triggerAccepted
    trigger_status_code = $triggerStatusCode
    queue_location = $queueLocation
  }
  warnings = @($warnings)
}

$json = $result | ConvertTo-Json -Depth 10
if (![string]::IsNullOrWhiteSpace($OutputPath)) {
  $outDir = Split-Path -Parent $OutputPath
  if (![string]::IsNullOrWhiteSpace($outDir) -and !(Test-Path $outDir)) {
    New-Item -ItemType Directory -Force -Path $outDir | Out-Null
  }
  Set-Content -Path $OutputPath -Value $json -Encoding UTF8
}

Write-Output $json

if (!$ok) { exit 1 }
