param(
  [string[]]$EnvFiles = @("gitgov\.env", "gitgov\gitgov-server\.env"),
  [string]$GitGovUrl = "",
  [string]$ApiKey = "",
  [string]$OrgName = "",
  [string]$RepositoryFullName = "",
  [string]$ReleaseId = "",
  [string]$Environment = "production",
  [string]$EvidencePacketHash = "",
  [int]$TimeoutSeconds = 45,
  [string]$OutputPath = "",
  [switch]$Enforce,
  [switch]$FailOnWouldBlock,
  [switch]$RequirePolicySatisfied
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

function Test-SafeGitGovUrl {
  param([Parameter(Mandatory = $true)][string]$Url)

  $uri = [Uri]$Url
  if ($uri.Scheme -notin @("http", "https")) {
    throw "GitGov URL must use http or https."
  }
  if (-not [string]::IsNullOrWhiteSpace($uri.UserInfo)) {
    throw "GitGov URL must not contain embedded credentials."
  }
  if ($uri.Scheme -eq "http") {
    $hostName = $uri.Host.ToLowerInvariant()
    if ($hostName -notin @("127.0.0.1", "localhost", "::1")) {
      throw "Plain HTTP GitGov validation is allowed only for loopback hosts."
    }
  }
  return $uri.AbsoluteUri.TrimEnd("/")
}

function Add-QueryParam {
  param(
    [System.Collections.Generic.List[string]]$QueryParams,
    [Parameter(Mandatory = $true)][string]$Name,
    [AllowEmptyString()][string]$Value
  )

  if ([string]::IsNullOrWhiteSpace($Value)) {
    return
  }
  $QueryParams.Add(("{0}={1}" -f [Uri]::EscapeDataString($Name), [Uri]::EscapeDataString($Value))) | Out-Null
}

foreach ($envFile in $EnvFiles) {
  Load-DotEnvNoPrint $envFile
}
$script:SecretValues = Get-SecretValues

if ([string]::IsNullOrWhiteSpace($GitGovUrl)) {
  $GitGovUrl = $env:GITGOV_URL
}
if ([string]::IsNullOrWhiteSpace($GitGovUrl)) {
  $GitGovUrl = "https://gitgov-api.onrender.com"
}
if ([string]::IsNullOrWhiteSpace($ApiKey)) {
  $ApiKey = $env:GITGOV_API_KEY
}
if ([string]::IsNullOrWhiteSpace($OrgName)) {
  $OrgName = $env:GITGOV_ORG_NAME
}

$failures = New-Object System.Collections.Generic.List[string]
if ([string]::IsNullOrWhiteSpace($ApiKey)) {
  $failures.Add("missing_gitgov_api_key") | Out-Null
}
if ([string]::IsNullOrWhiteSpace($RepositoryFullName) -or $RepositoryFullName -notmatch "^[^/\s]+/[^/\s]+$") {
  $failures.Add("repository_full_name_must_look_like_owner_repo") | Out-Null
}
if ([string]::IsNullOrWhiteSpace($ReleaseId)) {
  $failures.Add("release_id_is_required") | Out-Null
}
if ([string]::IsNullOrWhiteSpace($Environment)) {
  $failures.Add("environment_is_required") | Out-Null
}
if (-not [string]::IsNullOrWhiteSpace($EvidencePacketHash) -and $EvidencePacketHash -notmatch "^[A-Fa-f0-9]{64}$") {
  $failures.Add("evidence_packet_hash_must_be_64_hex_characters") | Out-Null
}
if ($TimeoutSeconds -lt 1) {
  $failures.Add("timeout_seconds_must_be_positive") | Out-Null
}

$started = Get-Date
$statusCode = 0
$responseJson = $null
$errorMessage = ""
$safeUrl = ""

if ($failures.Count -eq 0) {
  try {
    $safeUrl = Test-SafeGitGovUrl $GitGovUrl
    $queryParams = New-Object System.Collections.Generic.List[string]
    Add-QueryParam -QueryParams $queryParams -Name "org_name" -Value $OrgName
    Add-QueryParam -QueryParams $queryParams -Name "repository_full_name" -Value $RepositoryFullName.Trim()
    Add-QueryParam -QueryParams $queryParams -Name "release_id" -Value $ReleaseId.Trim()
    Add-QueryParam -QueryParams $queryParams -Name "environment" -Value $Environment.Trim().ToLowerInvariant()
    Add-QueryParam -QueryParams $queryParams -Name "evidence_packet_hash" -Value $EvidencePacketHash.Trim()
    $uri = "{0}/enterprise/release-governance/evaluate?{1}" -f $safeUrl, ($queryParams -join "&")

    $response = Invoke-WebRequest `
      -Method GET `
      -Uri $uri `
      -Headers @{
        Authorization = "Bearer $ApiKey"
        Accept = "application/json"
      } `
      -TimeoutSec $TimeoutSeconds `
      -UseBasicParsing

    $statusCode = [int]$response.StatusCode
    $responseJson = $response.Content | ConvertFrom-Json
  } catch {
    $errorMessage = Protect-SecretText $_.Exception.Message
    $failures.Add("release_governance_evaluator_request_failed") | Out-Null
  }
}

$durationMs = [int]((Get-Date) - $started).TotalMilliseconds
$evaluationStatus = ""
$policyMode = ""
$policyEnforcement = ""
$policyApplies = $false
$policySatisfied = $false
$blocking = $false
$wouldBlock = $false
$validApprovalCount = 0
$requiredApprovalCount = 0
$issues = @()
$nextSteps = @()
$quorumRules = @()
$matchingApprovalCount = 0

if ($null -ne $responseJson) {
  $evaluationStatus = [string]$responseJson.status
  $policySatisfied = [bool]$responseJson.policy_satisfied
  $blocking = [bool]$responseJson.blocking
  $wouldBlock = [bool]$responseJson.would_block
  $validApprovalCount = [int]$responseJson.valid_approval_count
  $requiredApprovalCount = [int]$responseJson.required_approval_count
  if ($responseJson.PSObject.Properties["policy"] -and $null -ne $responseJson.policy) {
    $policyMode = [string]$responseJson.policy.mode
    $policyEnforcement = [string]$responseJson.policy.enforcement
    $policyApplies = [bool]$responseJson.policy.policy_applies
    $quorumRules = @($responseJson.policy.quorum_rules)
  }
  if ($responseJson.PSObject.Properties["issues"] -and $null -ne $responseJson.issues) {
    $issues = @($responseJson.issues | ForEach-Object { Protect-SecretText ([string]$_) })
  }
  if ($responseJson.PSObject.Properties["next_steps"] -and $null -ne $responseJson.next_steps) {
    $nextSteps = @($responseJson.next_steps | ForEach-Object { Protect-SecretText ([string]$_) })
  }
  if ($responseJson.PSObject.Properties["approvals"] -and $null -ne $responseJson.approvals) {
    $matchingApprovalCount = @($responseJson.approvals).Count
  }
}

if ($statusCode -ne 0 -and ($statusCode -lt 200 -or $statusCode -ge 300)) {
  $failures.Add("release_governance_evaluator_returned_non_2xx") | Out-Null
}
if ($Enforce -and $blocking) {
  $failures.Add("release_governance_blocking_policy_not_satisfied") | Out-Null
}
if ($FailOnWouldBlock -and $wouldBlock) {
  $failures.Add("release_governance_would_block") | Out-Null
}
if ($RequirePolicySatisfied -and -not $policySatisfied) {
  $failures.Add("release_governance_policy_not_satisfied") | Out-Null
}

$gateStatus = if ($failures.Count -gt 0) {
  "failed"
} elseif ($blocking) {
  "blocking-observed"
} elseif ($wouldBlock) {
  "would-block-observed"
} else {
  "passed"
}

$result = [ordered]@{
  generated_at = (Get-Date).ToUniversalTime().ToString("o")
  status = $gateStatus
  passed = ($failures.Count -eq 0)
  enforce = [bool]$Enforce
  fail_on_would_block = [bool]$FailOnWouldBlock
  require_policy_satisfied = [bool]$RequirePolicySatisfied
  request = [ordered]@{
    gitgov_url = $safeUrl
    org_name = $OrgName
    repository_full_name = $RepositoryFullName
    release_id = $ReleaseId
    environment = $Environment
    evidence_packet_hash_present = -not [string]::IsNullOrWhiteSpace($EvidencePacketHash)
  }
  evaluation = [ordered]@{
    http_status = $statusCode
    status = $evaluationStatus
    policy_mode = $policyMode
    policy_enforcement = $policyEnforcement
    policy_applies = $policyApplies
    policy_satisfied = $policySatisfied
    blocking = $blocking
    would_block = $wouldBlock
    valid_approval_count = $validApprovalCount
    required_approval_count = $requiredApprovalCount
    matching_approval_count = $matchingApprovalCount
    quorum_rules = @($quorumRules)
    issues = @($issues)
    next_steps = @($nextSteps)
    duration_ms = $durationMs
  }
  failures = @($failures.ToArray())
  error = $errorMessage
  safety = [ordered]@{
    prints_secret_values = $false
    prints_authorization_header = $false
    stores_raw_secret_values = $false
    omits_approval_identity_details = $true
  }
}

$json = $result | ConvertTo-Json -Depth 12
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
