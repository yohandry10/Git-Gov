param(
  [string]$JenkinsUrl = "http://127.0.0.1:8096",
  [string]$JobName = "gitgov-demo-pipeline",
  [string]$ExpectedRepoUrl = "https://github.com/yohandry10/Git-Gov.git",
  [string]$Username = "",
  [string]$ApiTokenOrPassword = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

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
  $xmlRaw = Invoke-RestMethod -Uri $jobConfigUrl -Method Get -Headers $headers
} catch {
  Write-Error "Could not fetch Jenkins job config from $jobConfigUrl"
  exit 1
}

[xml]$xml = $xmlRaw
$remoteUrls = @()

if ($xml.flowdefinition.definition -and $xml.flowdefinition.definition.scm -and $xml.flowdefinition.definition.scm.userRemoteConfigs) {
  $nodes = $xml.flowdefinition.definition.scm.userRemoteConfigs.'hudson.plugins.git.UserRemoteConfig'
  if ($nodes) {
    foreach ($node in @($nodes)) {
      if ($node.url) {
        $remoteUrls += [string]$node.url
      }
    }
  }
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
