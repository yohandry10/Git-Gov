param(
  [string[]]$EnvFiles = @("gitgov\.env", "gitgov\gitgov-server\.env"),
  [string]$GitGovUrl = "https://gitgov-api.onrender.com",
  [string]$LocalGitGovUrl = "http://127.0.0.1:3001",
  [string]$RepoFullName = "yohandry10/Git-Gov",
  [string]$Branch = "main",
  [int]$Hours = 720,
  [switch]$SkipLocalGitGov,
  [switch]$SkipSonar,
  [switch]$SkipJenkins,
  [switch]$SkipJira,
  [switch]$IncludeReleaseReadiness,
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

  if (!(Test-Path $resolved)) {
    return
  }

  Get-Content $resolved | ForEach-Object {
    $line = $_.Trim()
    if (!$line -or $line.StartsWith("#") -or !$line.Contains("=")) {
      return
    }
    $parts = $line -split "=", 2
    $name = $parts[0].Trim()
    $value = $parts[1].Trim().Trim('"')
    if (![string]::IsNullOrWhiteSpace($name)) {
      [Environment]::SetEnvironmentVariable($name, $value, "Process")
    }
  }
}

function New-CheckResult {
  param(
    [string]$Name,
    [bool]$Ok,
    [hashtable]$Details = @{},
    [string]$ErrorMessage = ""
  )

  [ordered]@{
    name = $Name
    ok = $Ok
    details = $Details
    error = $ErrorMessage
  }
}

function Invoke-JsonGet {
  param(
    [string]$Uri,
    [hashtable]$Headers = @{}
  )

  Invoke-RestMethod -Uri $Uri -Headers $Headers -TimeoutSec 30
}

foreach ($envFile in $EnvFiles) {
  Load-DotEnvNoPrint $envFile
}

$checks = New-Object System.Collections.Generic.List[object]

try {
  if ([string]::IsNullOrWhiteSpace($env:GITGOV_API_KEY)) {
    throw "GITGOV_API_KEY is not loaded."
  }
  $headers = @{ Authorization = "Bearer $($env:GITGOV_API_KEY)"; Accept = "application/json" }
  $health = Invoke-JsonGet -Uri "$($GitGovUrl.TrimEnd('/'))/health"
  $statsResponse = Invoke-WebRequest -Uri "$($GitGovUrl.TrimEnd('/'))/stats" -Headers $headers -TimeoutSec 30 -UseBasicParsing
  $checks.Add((New-CheckResult -Name "gitgov-production" -Ok $true -Details @{
        url = $GitGovUrl
        health = $health.status
        stats_status = $statsResponse.StatusCode
      }))
} catch {
  $checks.Add((New-CheckResult -Name "gitgov-production" -Ok $false -ErrorMessage $_.Exception.Message))
}

if (!$SkipLocalGitGov) {
  try {
    $health = Invoke-JsonGet -Uri "$($LocalGitGovUrl.TrimEnd('/'))/health"
    $checks.Add((New-CheckResult -Name "gitgov-local" -Ok $true -Details @{
          url = $LocalGitGovUrl
          health = $health.status
          version = $health.version
        }))
  } catch {
    $checks.Add((New-CheckResult -Name "gitgov-local" -Ok $false -ErrorMessage $_.Exception.Message))
  }
}

if (!$SkipSonar) {
  try {
    if ([string]::IsNullOrWhiteSpace($env:SONAR_HOST_URL) -or [string]::IsNullOrWhiteSpace($env:SONAR_TOKEN) -or [string]::IsNullOrWhiteSpace($env:SONAR_PROJECT_KEY)) {
      throw "SONAR_HOST_URL, SONAR_TOKEN, or SONAR_PROJECT_KEY is not loaded."
    }
    $sonarAuth = [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes("$($env:SONAR_TOKEN):"))
    $headers = @{ Authorization = "Basic $sonarAuth"; Accept = "application/json" }
    $status = Invoke-JsonGet -Uri "$($env:SONAR_HOST_URL.TrimEnd('/'))/api/system/status" -Headers $headers
    $qg = Invoke-JsonGet -Uri "$($env:SONAR_HOST_URL.TrimEnd('/'))/api/qualitygates/project_status?projectKey=$($env:SONAR_PROJECT_KEY)" -Headers $headers
    $checks.Add((New-CheckResult -Name "sonarqube" -Ok $true -Details @{
          url = $env:SONAR_HOST_URL
          system = $status.status
          project = $env:SONAR_PROJECT_KEY
          quality_gate = $qg.projectStatus.status
        }))
  } catch {
    $checks.Add((New-CheckResult -Name "sonarqube" -Ok $false -ErrorMessage $_.Exception.Message))
  }
}

if (!$SkipJenkins) {
  try {
    if ([string]::IsNullOrWhiteSpace($env:JENKINS_SERVER_URL) -or [string]::IsNullOrWhiteSpace($env:JENKINS_USER) -or [string]::IsNullOrWhiteSpace($env:JENKINS_API_TOKEN) -or [string]::IsNullOrWhiteSpace($env:JENKINS_JOB_NAME)) {
      throw "JENKINS_SERVER_URL, JENKINS_USER, JENKINS_API_TOKEN, or JENKINS_JOB_NAME is not loaded."
    }
    $jenkinsAuth = [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes("$($env:JENKINS_USER):$($env:JENKINS_API_TOKEN)"))
    $headers = @{ Authorization = "Basic $jenkinsAuth"; Accept = "application/json" }
    $who = Invoke-JsonGet -Uri "$($env:JENKINS_SERVER_URL.TrimEnd('/'))/whoAmI/api/json" -Headers $headers
    $job = Invoke-JsonGet -Uri "$($env:JENKINS_SERVER_URL.TrimEnd('/'))/job/$($env:JENKINS_JOB_NAME)/api/json?tree=lastBuild[number,result,building,url]" -Headers $headers
    $checks.Add((New-CheckResult -Name "jenkins" -Ok $true -Details @{
          url = $env:JENKINS_SERVER_URL
          user = $who.name
          job = $env:JENKINS_JOB_NAME
          last_build = $job.lastBuild.number
          result = $job.lastBuild.result
          building = $job.lastBuild.building
        }))
  } catch {
    $checks.Add((New-CheckResult -Name "jenkins" -Ok $false -ErrorMessage $_.Exception.Message))
  }
}

if (!$SkipJira) {
  try {
    if ([string]::IsNullOrWhiteSpace($env:JIRA_BASE_URL) -or [string]::IsNullOrWhiteSpace($env:JIRA_EMAIL) -or [string]::IsNullOrWhiteSpace($env:JIRA_API_TOKEN) -or [string]::IsNullOrWhiteSpace($env:JIRA_PROJECT_KEY)) {
      throw "JIRA_BASE_URL, JIRA_EMAIL, JIRA_API_TOKEN, or JIRA_PROJECT_KEY is not loaded."
    }
    $jiraAuth = [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes("$($env:JIRA_EMAIL):$($env:JIRA_API_TOKEN)"))
    $headers = @{ Authorization = "Basic $jiraAuth"; Accept = "application/json" }
    $project = Invoke-JsonGet -Uri "$($env:JIRA_BASE_URL.TrimEnd('/'))/rest/api/3/project/$($env:JIRA_PROJECT_KEY)" -Headers $headers
    $checks.Add((New-CheckResult -Name "jira" -Ok $true -Details @{
          url = $env:JIRA_BASE_URL
          key = $project.key
          id = $project.id
          name = $project.name
        }))
  } catch {
    $checks.Add((New-CheckResult -Name "jira" -Ok $false -ErrorMessage $_.Exception.Message))
  }
}

if ($IncludeReleaseReadiness) {
  try {
    if ([string]::IsNullOrWhiteSpace($env:GITGOV_API_KEY)) {
      throw "GITGOV_API_KEY is not loaded."
    }
    $readinessScript = Join-Path $repoRoot "scripts\jenkins\validate_release_readiness_gate.ps1"
    $tmp = Join-Path $env:TEMP ("gitgov-readiness-" + [guid]::NewGuid().ToString("N") + ".json")
    & $readinessScript -GitGovUrl $GitGovUrl -ApiKey $env:GITGOV_API_KEY -RepoFullName $RepoFullName -Branch $Branch -Tier standard -MinReadiness 75 -Hours $Hours -OutputPath $tmp | Out-Host
    $readiness = Get-Content $tmp -Raw | ConvertFrom-Json
    Remove-Item -LiteralPath $tmp -Force
    $checks.Add((New-CheckResult -Name "release-readiness" -Ok $true -Details @{
          repo = $RepoFullName
          branch = $Branch
          readiness = $readiness.readiness_score
          target = $readiness.target_readiness
          signal_coverage = $readiness.signal_coverage
          pipeline_success_rate = $readiness.metrics.pipeline_success_rate
          jira_ticket_coverage = $readiness.metrics.jira_ticket_coverage
          sonar_pass_rate = $readiness.metrics.sonar_pass_rate
        }))
  } catch {
    $checks.Add((New-CheckResult -Name "release-readiness" -Ok $false -ErrorMessage $_.Exception.Message))
  }
}

$failed = @($checks | Where-Object { $_.ok -ne $true })
$result = [ordered]@{
  ok = ($failed.Count -eq 0)
  checked_at_utc = (Get-Date).ToUniversalTime().ToString("o")
  checks = $checks
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

if ($failed.Count -gt 0) {
  exit 1
}
