param(
  [string[]]$EnvFiles = @("gitgov\.env", "gitgov\gitgov-server\.env"),
  [string]$CopilotUrl = "https://www.gitgov.cloud/api/copilot/governance",
  [string]$OrgName = "yohandry10",
  [string]$RepositoryFullName = "yohandry10/Git-Gov",
  [string]$Branch = "main",
  [string]$TicketId = "KAN-39",
  [string]$ReleaseId = "",
  [string]$Environment = "production",
  [int]$Hours = 720,
  [string]$Question = "Is this GitGov change ready for production based on the available evidence?",
  [int]$MinSources = 3,
  [int]$MinOkSources = 2,
  [int]$MinCitations = 3,
  [int]$TimeoutSeconds = 45,
  [string]$OutputPath = "",
  [switch]$RequireAiMode
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")

function Resolve-RepoPath {
  param([Parameter(Mandatory = $true)][string]$Path)

  if ([System.IO.Path]::IsPathRooted($Path)) {
    return $Path
  }
  return Join-Path $repoRoot $Path
}

function Load-DotEnvNoPrint {
  param([Parameter(Mandatory = $true)][string]$Path)

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

function Get-SecretValues {
  $secretValues = New-Object System.Collections.Generic.List[string]
  foreach ($entry in [Environment]::GetEnvironmentVariables("Process").GetEnumerator()) {
    $name = [string]$entry.Key
    $value = [string]$entry.Value
    if ($value.Length -lt 6) {
      continue
    }
    if ($name -match "(TOKEN|SECRET|PASSWORD|API_KEY|DATABASE_URL|PRIVATE_KEY)") {
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

function Get-StringHash {
  param([string]$Value)

  $bytes = [Text.Encoding]::UTF8.GetBytes($Value)
  $hash = [Security.Cryptography.SHA256]::Create().ComputeHash($bytes)
  return ([BitConverter]::ToString($hash) -replace "-", "").ToLowerInvariant()
}

function ConvertTo-Count {
  param($Value)

  if ($null -eq $Value) {
    return 0
  }
  return @($Value).Count
}

function Test-SafeCopilotUrl {
  param([Parameter(Mandatory = $true)][string]$Url)

  $uri = [Uri]$Url
  if ($uri.Scheme -notin @("http", "https")) {
    throw "Copilot URL must use http or https."
  }
  if (-not [string]::IsNullOrWhiteSpace($uri.UserInfo)) {
    throw "Copilot URL must not contain embedded credentials."
  }
  if ($uri.Scheme -eq "http") {
    $hostName = $uri.Host.ToLowerInvariant()
    if ($hostName -notin @("127.0.0.1", "localhost", "::1")) {
      throw "Plain HTTP copilot validation is allowed only for loopback hosts."
    }
  }
  return $uri.AbsoluteUri
}

foreach ($envFile in $EnvFiles) {
  Load-DotEnvNoPrint $envFile
}
$script:SecretValues = Get-SecretValues

if ([string]::IsNullOrWhiteSpace($env:GITGOV_API_KEY)) {
  throw "GITGOV_API_KEY is not loaded from the configured env files or process environment."
}

if ([string]::IsNullOrWhiteSpace($ReleaseId)) {
  $ReleaseId = $TicketId
}

$safeUrl = Test-SafeCopilotUrl $CopilotUrl
$body = [ordered]@{
  question = $Question
  org_name = $OrgName
  repository_full_name = $RepositoryFullName
  branch = $Branch
  ticket_id = $TicketId
  release_id = $ReleaseId
  environment = $Environment
  hours = $Hours
} | ConvertTo-Json -Depth 6

$started = Get-Date
$statusCode = 0
$responseJson = $null
$errorMessage = ""

try {
  $response = Invoke-WebRequest `
    -Method POST `
    -Uri $safeUrl `
    -Headers @{
      Authorization = "Bearer $($env:GITGOV_API_KEY)"
      Accept = "application/json"
      "Content-Type" = "application/json"
    } `
    -Body $body `
    -TimeoutSec $TimeoutSeconds `
    -UseBasicParsing

  $statusCode = [int]$response.StatusCode
  $responseJson = $response.Content | ConvertFrom-Json
} catch {
  $errorMessage = Protect-SecretText $_.Exception.Message
}

$durationMs = [int]((Get-Date) - $started).TotalMilliseconds
$mode = ""
$model = ""
$success = $false
$answerLength = 0
$answerSha256 = ""
$citationsCount = 0
$sourcesCount = 0
$okSourcesCount = 0
$sourceStatuses = @{}
$warnings = @()

if ($null -ne $responseJson) {
  $success = [bool]$responseJson.success
  $mode = [string]$responseJson.mode
  if ($responseJson.PSObject.Properties["model"]) {
    $model = [string]$responseJson.model
  }
  if ($responseJson.PSObject.Properties["answer"] -and $null -ne $responseJson.answer) {
    $answerText = [string]$responseJson.answer
    $answerLength = $answerText.Length
    $answerSha256 = Get-StringHash $answerText
  }

  $citationsCount = ConvertTo-Count $responseJson.citations
  $sourcesCount = ConvertTo-Count $responseJson.sources
  if ($null -ne $responseJson.sources) {
    foreach ($source in @($responseJson.sources)) {
      $id = [string]$source.id
      $status = [string]$source.status
      if (-not [string]::IsNullOrWhiteSpace($id)) {
        $sourceStatuses[$id] = $status
      }
      if ($status -eq "ok") {
        $okSourcesCount += 1
      }
    }
  }
  if ($null -ne $responseJson.warnings) {
    $warnings = @($responseJson.warnings | ForEach-Object { Protect-SecretText ([string]$_) })
  }
}

$failures = New-Object System.Collections.Generic.List[string]
if ($statusCode -lt 200 -or $statusCode -ge 300) {
  $failures.Add("copilot route did not return a 2xx response") | Out-Null
}
if (-not $success) {
  $failures.Add("copilot response did not report success=true") | Out-Null
}
if ($sourcesCount -lt $MinSources) {
  $failures.Add("copilot returned fewer sources than required") | Out-Null
}
if ($okSourcesCount -lt $MinOkSources) {
  $failures.Add("copilot returned fewer ok evidence sources than required") | Out-Null
}
if ($citationsCount -lt $MinCitations) {
  $failures.Add("copilot returned fewer citations than required") | Out-Null
}
if ($RequireAiMode -and $mode -ne "ai") {
  $failures.Add("copilot did not return mode=ai while RequireAiMode was set") | Out-Null
}
if (-not [string]::IsNullOrWhiteSpace($errorMessage)) {
  $failures.Add($errorMessage) | Out-Null
}

$status = if ($failures.Count -gt 0) {
  "failed"
} elseif ($mode -eq "ai") {
  "ai"
} else {
  "fallback"
}

$result = [ordered]@{
  generated_at = (Get-Date).ToUniversalTime().ToString("o")
  status = $status
  ok = ($failures.Count -eq 0)
  require_ai_mode = [bool]$RequireAiMode
  request = [ordered]@{
    copilot_url = $safeUrl
    org_name = $OrgName
    repository_full_name = $RepositoryFullName
    branch = $Branch
    ticket_id = $TicketId
    release_id = $ReleaseId
    environment = $Environment
    hours = $Hours
  }
  response = [ordered]@{
    http_status = $statusCode
    success = $success
    mode = $mode
    model = $model
    answer_length = $answerLength
    answer_sha256 = $answerSha256
    citations_count = $citationsCount
    sources_count = $sourcesCount
    ok_sources_count = $okSourcesCount
    source_statuses = $sourceStatuses
    warnings_count = $warnings.Count
    warnings = $warnings
    duration_ms = $durationMs
  }
  thresholds = [ordered]@{
    min_sources = $MinSources
    min_ok_sources = $MinOkSources
    min_citations = $MinCitations
  }
  failures = @($failures.ToArray())
  safety = [ordered]@{
    prints_secret_values = $false
    prints_authorization_header = $false
    stores_raw_answer = $false
  }
}

$json = $result | ConvertTo-Json -Depth 10
if (-not [string]::IsNullOrWhiteSpace($OutputPath)) {
  $parent = Split-Path -Parent $OutputPath
  if (-not [string]::IsNullOrWhiteSpace($parent)) {
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
  }
  Set-Content -LiteralPath $OutputPath -Value $json -Encoding UTF8
}

Write-Output $json

if ($failures.Count -gt 0) {
  exit 1
}
