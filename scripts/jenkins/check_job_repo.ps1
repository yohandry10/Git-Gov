param(
  [string]$JenkinsUrl = "http://127.0.0.1:8096",
  [string]$JobName = "gitgov-demo-pipeline",
  [string]$ExpectedRepoUrl = "",
  [string]$Username = "",
  [string]$ApiTokenOrPassword = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
. (Join-Path $scriptRoot "..\github\_token_helpers.ps1")

if ([string]::IsNullOrWhiteSpace($ExpectedRepoUrl)) {
  $repoInfo = Resolve-GitHubRepoCoordinates -ScriptRoot $scriptRoot
  if (-not [string]::IsNullOrWhiteSpace($repoInfo.Owner) -and -not [string]::IsNullOrWhiteSpace($repoInfo.Repo)) {
    $ExpectedRepoUrl = "https://github.com/$($repoInfo.Owner)/$($repoInfo.Repo).git"
  }
}
if ([string]::IsNullOrWhiteSpace($ExpectedRepoUrl)) {
  Write-Error "Missing -ExpectedRepoUrl and repository coordinates could not be auto-resolved."
  exit 1
}

if ([string]::IsNullOrWhiteSpace($Username) -or [string]::IsNullOrWhiteSpace($ApiTokenOrPassword)) {
  Write-Error "Missing credentials. Provide -Username and -ApiTokenOrPassword."
  exit 1
}

$baseUrl = $JenkinsUrl.TrimEnd('/')
$jobConfigUrl = "$baseUrl/job/$JobName/config.xml"

$pair = "{0}:{1}" -f $Username, $ApiTokenOrPassword
$encoded = [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes($pair))
$headers = @{
  Authorization = "Basic $encoded"
}

try {
  $response = Invoke-WebRequest -Uri $jobConfigUrl -Method Get -Headers $headers -UseBasicParsing
  $xmlRaw = [string]$response.Content
} catch {
  Write-Error "Could not fetch Jenkins job config from $jobConfigUrl"
  exit 1
}

$normalizedXml = [regex]::Replace(
  $xmlRaw,
  '<\?xml\s+version=(["''])1\.1\1',
  '<?xml version="1.0"',
  [System.Text.RegularExpressions.RegexOptions]::IgnoreCase
)

try {
[xml]$xml = $normalizedXml
} catch {
  Write-Error "Could not parse Jenkins job config XML for $jobConfigUrl"
  exit 1
}
$remoteUrls = @()

try {
  $urlNodes = $xml.SelectNodes("//*[local-name()='userRemoteConfigs']/*[local-name()='hudson.plugins.git.UserRemoteConfig']/*[local-name()='url']")
  if ($urlNodes) {
    foreach ($node in @($urlNodes)) {
      $value = [string]$node.InnerText
      if (-not [string]::IsNullOrWhiteSpace($value)) {
        $remoteUrls += $value.Trim()
      }
    }
  }
} catch {
  Write-Error "Could not query SCM remote URLs from Jenkins job XML."
  exit 1
}

if ($remoteUrls.Count -eq 0) {
  Write-Error "No SCM remote URL found in Jenkins job config (Pipeline script from SCM expected)."
  exit 1
}

$uniqueUrls = $remoteUrls | Select-Object -Unique
$expectedFound = @($uniqueUrls | Where-Object { $_ -eq $ExpectedRepoUrl })
$unexpectedRemotes = @($uniqueUrls | Where-Object { $_ -ne $ExpectedRepoUrl })

Write-Host "Jenkins job: $JobName"
Write-Host "Detected SCM remotes:"
foreach ($url in $uniqueUrls) {
  Write-Host "  - $url"
}

if ($unexpectedRemotes.Count -gt 0) {
  Write-Error ("FAIL: Unexpected SCM remote(s) detected: {0}" -f ($unexpectedRemotes -join ", "))
  exit 1
}

if ($expectedFound.Count -eq 0) {
  Write-Error ("FAIL: Expected repo URL not found: {0}" -f $ExpectedRepoUrl)
  exit 1
}

Write-Host "PASS: Jenkins job is aligned to the expected repository URL."
