param(
  [string]$GitGovUrl = "http://127.0.0.1:3001",
  [string]$ApiKey,
  [string]$RepoFullName,
  [string]$Branch = "main",
  [string]$FailingCommitSha,
  [string]$GreenCommitSha,
  [string]$UserLogin = "jenkins",
  [string]$OutputPath = "",
  [switch]$LeavePolicyAsIs
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($ApiKey)) {
  Write-Error "Missing -ApiKey."
  exit 1
}
if ([string]::IsNullOrWhiteSpace($RepoFullName)) {
  Write-Error "Missing -RepoFullName (<owner>/<repo>)."
  exit 1
}
if ([string]::IsNullOrWhiteSpace($FailingCommitSha)) {
  Write-Error "Missing -FailingCommitSha."
  exit 1
}
if ([string]::IsNullOrWhiteSpace($GreenCommitSha)) {
  Write-Error "Missing -GreenCommitSha."
  exit 1
}

$baseUrl = $GitGovUrl.TrimEnd("/")
$repoPath = [System.Uri]::EscapeDataString($RepoFullName)
$headers = @{
  Authorization = "Bearer $ApiKey"
  "Content-Type" = "application/json"
}

if ([string]::IsNullOrWhiteSpace($OutputPath)) {
  $stamp = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHHmmssZ")
  $OutputPath = "docs/reports/quality-gate-policy-matrix-$stamp.md"
}

function Get-JsonClone {
  param([Parameter(Mandatory = $true)][object]$InputObject)
  return ($InputObject | ConvertTo-Json -Depth 30 | ConvertFrom-Json)
}

function Invoke-GitGovJson {
  param(
    [Parameter(Mandatory = $true)][ValidateSet("GET", "POST", "PUT")][string]$Method,
    [Parameter(Mandatory = $true)][string]$Path,
    [Parameter()][object]$Body
  )
  $uri = "$baseUrl$Path"
  try {
    if ($Method -eq "GET") {
      return Invoke-RestMethod -Uri $uri -Method Get -Headers $headers
    }
    $json = if ($null -eq $Body) { "" } else { $Body | ConvertTo-Json -Depth 30 }
    return Invoke-RestMethod -Uri $uri -Method $Method -Headers $headers -Body $json
  } catch {
    if ($_.Exception.Response) {
      $response = $_.Exception.Response
      $responseBody = ""
      if ($response.PSObject.Properties.Name -contains "Content" -and $null -ne $response.Content) {
        $responseBody = $response.Content.ReadAsStringAsync().GetAwaiter().GetResult()
      } elseif ($response.PSObject.Methods.Name -contains "GetResponseStream") {
        $reader = New-Object IO.StreamReader($response.GetResponseStream())
        $responseBody = $reader.ReadToEnd()
      }
      throw "HTTP error calling $Method $uri -> $responseBody"
    }
    throw
  }
}

function New-PolicyCheckBody {
  param([Parameter(Mandatory = $true)][string]$CommitSha)
  return @{
    repo = $RepoFullName
    branch = $Branch
    commit = $CommitSha
    user_login = $UserLogin
  }
}

function Has-QualityGateViolation {
  param(
    [Parameter(Mandatory = $true)][object]$PolicyCheckResponse,
    [Parameter(Mandatory = $true)][string]$ExpectedEnforcement
  )
  $violations = @($PolicyCheckResponse.violations)
  foreach ($violation in $violations) {
    if ([string]$violation.rule -eq "quality_gate_green" -and [string]$violation.enforcement -eq $ExpectedEnforcement) {
      return $true
    }
  }
  return $false
}

function New-TemporaryQualityGateException {
  return @{
    enabled = $true
    reason = "temporary quality gate matrix validation restore"
    ticket_id = "GITGOV-CI"
    approved_by = $UserLogin
    expires_at = [DateTimeOffset]::UtcNow.AddMinutes(30).ToUnixTimeMilliseconds()
    created_at = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
  }
}

function Invoke-PolicyOverride {
  param(
    [Parameter(Mandatory = $true)][object]$Config,
    [Parameter()][object]$QualityGateException = $null,
    [switch]$Governed
  )

  if ($Governed.IsPresent) {
    return Invoke-GitGovJson -Method "PUT" -Path "/policy/$repoPath/override" -Body @{
      config = $Config
      quality_gate_exception = $QualityGateException
    }
  }

  return Invoke-GitGovJson -Method "PUT" -Path "/policy/$repoPath/override" -Body $Config
}

function Set-PolicyForMatrix {
  param(
    [Parameter(Mandatory = $true)][object]$Config
  )

  if ($Config.PSObject.Properties.Name -contains "quality_gate_exception") {
    $Config.quality_gate_exception = $null
  } else {
    $Config | Add-Member -NotePropertyName "quality_gate_exception" -NotePropertyValue $null
  }

  try {
    [void](Invoke-PolicyOverride -Config $Config)
    return
  } catch {
    if (-not ([string]$_.Exception.Message).Contains("quality gate enforcement downgrade requires active quality_gate_exception")) {
      throw
    }
  }

  $exceptionPolicy = Get-JsonClone -InputObject $Config
  if ($exceptionPolicy.PSObject.Properties.Name -contains "quality_gate_exception") {
    $exceptionPolicy.quality_gate_exception = New-TemporaryQualityGateException
  } else {
    $exceptionPolicy | Add-Member -NotePropertyName "quality_gate_exception" -NotePropertyValue (New-TemporaryQualityGateException)
  }
  [void](Invoke-PolicyOverride -Config $exceptionPolicy)
  [void](Invoke-PolicyOverride -Config $Config)
}

function Restore-OriginalPolicy {
  param(
    [Parameter(Mandatory = $true)][object]$Config
  )

  try {
    [void](Invoke-PolicyOverride -Config $Config)
    return
  } catch {
    if (-not ([string]$_.Exception.Message).Contains("quality gate enforcement downgrade requires active quality_gate_exception")) {
      throw
    }
  }

  $exceptionPolicy = Get-JsonClone -InputObject $Config
  if ($exceptionPolicy.PSObject.Properties.Name -contains "quality_gate_exception") {
    $exceptionPolicy.quality_gate_exception = New-TemporaryQualityGateException
  } else {
    $exceptionPolicy | Add-Member -NotePropertyName "quality_gate_exception" -NotePropertyValue (New-TemporaryQualityGateException)
  }
  [void](Invoke-PolicyOverride -Config $exceptionPolicy)
  if (($Config.PSObject.Properties.Name -notcontains "quality_gate_exception") -or $null -eq $Config.quality_gate_exception) {
    [void](Invoke-PolicyOverride -Config $Config)
  }
}

if (!(Test-Path (Split-Path -Parent $OutputPath))) {
  New-Item -ItemType Directory -Path (Split-Path -Parent $OutputPath) | Out-Null
}

$originalPolicy = $null
$warnResultFail = $null
$blockResultFail = $null
$blockResultGreen = $null
$restoreOutcome = "NOT_ATTEMPTED"
$errors = New-Object System.Collections.Generic.List[string]

try {
  $policyResponse = Invoke-GitGovJson -Method "GET" -Path "/policy/$repoPath"
  if ($null -eq $policyResponse.config) {
    throw "Policy not found for repo '$RepoFullName'."
  }
  $originalPolicy = Get-JsonClone -InputObject $policyResponse.config

  # 1) quality_gates=warn
  $warnPolicy = Get-JsonClone -InputObject $originalPolicy
  $warnPolicy.enforcement.quality_gates = "warn"
  Set-PolicyForMatrix -Config $warnPolicy

  $warnResultFail = Invoke-GitGovJson -Method "POST" -Path "/policy/check" -Body (New-PolicyCheckBody -CommitSha $FailingCommitSha)
  if (-not $warnResultFail.allowed) {
    $errors.Add("Expected warn mode to allow failing commit, but allowed=false.")
  }
  if (-not $warnResultFail.advisory) {
    $errors.Add("Expected warn mode to be advisory=true, but advisory=false.")
  }
  if (-not (Has-QualityGateViolation -PolicyCheckResponse $warnResultFail -ExpectedEnforcement "warn")) {
    $errors.Add("Expected quality_gate_green violation with enforcement=warn in warn mode.")
  }

  # 2) quality_gates=block
  $blockPolicy = Get-JsonClone -InputObject $originalPolicy
  $blockPolicy.enforcement.quality_gates = "block"
  Set-PolicyForMatrix -Config $blockPolicy

  $blockResultFail = Invoke-GitGovJson -Method "POST" -Path "/policy/check" -Body (New-PolicyCheckBody -CommitSha $FailingCommitSha)
  if ($blockResultFail.allowed) {
    $errors.Add("Expected block mode to deny failing commit, but allowed=true.")
  }
  if ($blockResultFail.advisory) {
    $errors.Add("Expected block mode to be advisory=false, but advisory=true.")
  }
  if (-not (Has-QualityGateViolation -PolicyCheckResponse $blockResultFail -ExpectedEnforcement "block")) {
    $errors.Add("Expected quality_gate_green violation with enforcement=block in block mode.")
  }

  # 3) green commit under block should pass
  $blockResultGreen = Invoke-GitGovJson -Method "POST" -Path "/policy/check" -Body (New-PolicyCheckBody -CommitSha $GreenCommitSha)
  if (-not $blockResultGreen.allowed) {
    $errors.Add("Expected green commit to pass under block mode, but allowed=false.")
  }
} finally {
  if ($null -ne $originalPolicy -and -not $LeavePolicyAsIs.IsPresent) {
    try {
      Restore-OriginalPolicy -Config $originalPolicy
      $restoreOutcome = "RESTORED"
    } catch {
      $restoreOutcome = "FAILED"
      $errors.Add("Failed to restore original policy: $($_.Exception.Message)")
    }
  } elseif ($LeavePolicyAsIs.IsPresent) {
    $restoreOutcome = "SKIPPED_BY_FLAG"
  }
}

$status = if ($errors.Count -eq 0) { "PASS" } else { "FAIL" }
$generatedUtc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-dd HH:mm:ss")
$warnJson = if ($null -eq $warnResultFail) { "{}" } else { $warnResultFail | ConvertTo-Json -Depth 20 }
$blockFailJson = if ($null -eq $blockResultFail) { "{}" } else { $blockResultFail | ConvertTo-Json -Depth 20 }
$blockGreenJson = if ($null -eq $blockResultGreen) { "{}" } else { $blockResultGreen | ConvertTo-Json -Depth 20 }
$errorsText = if ($errors.Count -eq 0) { "- none" } else { ($errors | ForEach-Object { "- $_" }) -join "`n" }

$report = @"
# Quality Gate Policy Matrix Validation Report

Generated (UTC): $generatedUtc
Status: **$status**

## Context

- GitGov URL: $GitGovUrl
- Repo: $RepoFullName
- Branch: $Branch
- Failing commit (non-green): $FailingCommitSha
- Green commit: $GreenCommitSha
- Policy restore: $restoreOutcome

## Step 1 - quality_gates=warn + failing commit

~~~json
$warnJson
~~~

## Step 2 - quality_gates=block + failing commit

~~~json
$blockFailJson
~~~

## Step 3 - quality_gates=block + green commit

~~~json
$blockGreenJson
~~~

## Validation errors

$errorsText
"@

Set-Content -Path $OutputPath -Value $report -Encoding UTF8

if ($status -eq "PASS") {
  Write-Host "PASS: quality gate matrix validation completed"
  Write-Host "  output: $OutputPath"
  exit 0
}

Write-Error "FAIL: quality gate matrix validation failed. See report: $OutputPath"
exit 1
