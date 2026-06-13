param(
  [string[]]$EnvFiles = @("gitgov\.env", "gitgov\gitgov-server\.env"),
  [string]$GitGovUrl = "",
  [string]$ApiKey = "",
  [string]$OrgName = "",
  [string]$RepositoryFullName = "",
  [string]$Branch = "",
  [string]$TargetSha = "",
  [string]$ReleaseId = "",
  [string]$Environment = "production",
  [string]$Deployer = "gitgov-validator",
  [string]$TicketId = "",
  [string]$EvidencePacketHash = "",
  [string]$DeploymentRunId = "",
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
if ([string]::IsNullOrWhiteSpace($Branch)) {
  $failures.Add("branch_is_required") | Out-Null
}
if ([string]::IsNullOrWhiteSpace($TargetSha) -or $TargetSha -notmatch "^[A-Fa-f0-9]{40}([A-Fa-f0-9]{24})?$") {
  $failures.Add("target_sha_must_be_full_40_or_64_hex") | Out-Null
}
if ([string]::IsNullOrWhiteSpace($ReleaseId)) {
  $failures.Add("release_id_is_required") | Out-Null
}
if ([string]::IsNullOrWhiteSpace($Environment)) {
  $failures.Add("environment_is_required") | Out-Null
}
if ([string]::IsNullOrWhiteSpace($Deployer)) {
  $failures.Add("deployer_is_required") | Out-Null
}
if ([string]::IsNullOrWhiteSpace($EvidencePacketHash) -or $EvidencePacketHash -notmatch "^[A-Fa-f0-9]{64}$") {
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
    $uri = "{0}/deployment-gates/authorize" -f $safeUrl
    $payload = [ordered]@{
      org_name = if ([string]::IsNullOrWhiteSpace($OrgName)) { $null } else { $OrgName.Trim() }
      release_id = $ReleaseId.Trim()
      repository_full_name = $RepositoryFullName.Trim()
      branch = $Branch.Trim()
      target_sha = $TargetSha.Trim().ToLowerInvariant()
      environment = $Environment.Trim().ToLowerInvariant()
      deployer = $Deployer.Trim()
      ticket_id = if ([string]::IsNullOrWhiteSpace($TicketId)) { $null } else { $TicketId.Trim().ToUpperInvariant() }
      evidence_packet_hash = $EvidencePacketHash.Trim().ToLowerInvariant()
      requested_by = "gitgov-validation-script"
      deployment_run_id = if ([string]::IsNullOrWhiteSpace($DeploymentRunId)) { $null } else { $DeploymentRunId.Trim() }
      metadata = [ordered]@{
        source = "scripts/control-plane/validate_release_governance_gate.ps1"
        enforce = [bool]$Enforce
        fail_on_would_block = [bool]$FailOnWouldBlock
        require_policy_satisfied = [bool]$RequirePolicySatisfied
      }
    }
    $body = $payload | ConvertTo-Json -Depth 8

    $response = Invoke-WebRequest `
      -Method POST `
      -Uri $uri `
      -Headers @{
        Authorization = "Bearer $ApiKey"
        Accept = "application/json"
      } `
      -ContentType "application/json" `
      -Body $body `
      -TimeoutSec $TimeoutSeconds `
      -UseBasicParsing

    $statusCode = [int]$response.StatusCode
    $responseJson = $response.Content | ConvertFrom-Json
  } catch {
    $errorMessage = Protect-SecretText $_.Exception.Message
    $failures.Add("deployment_gate_authorization_request_failed") | Out-Null
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
  $evaluationStatus = [string]$responseJson.decision
  $blocking = [bool]$responseJson.blocking
  $wouldBlock = [bool]$responseJson.would_block
  if ($responseJson.PSObject.Properties["evaluation"] -and $null -ne $responseJson.evaluation) {
    $policySatisfied = [bool]$responseJson.evaluation.policy_satisfied
    $validApprovalCount = [int]$responseJson.evaluation.valid_approval_count
    $requiredApprovalCount = [int]$responseJson.evaluation.required_approval_count
    if ($responseJson.evaluation.PSObject.Properties["policy"] -and $null -ne $responseJson.evaluation.policy) {
      $policyMode = [string]$responseJson.evaluation.policy.mode
      $policyEnforcement = [string]$responseJson.evaluation.policy.enforcement
      $policyApplies = [bool]$responseJson.evaluation.policy.policy_applies
      $quorumRules = @($responseJson.evaluation.policy.quorum_rules)
    }
    if ($responseJson.evaluation.PSObject.Properties["issues"] -and $null -ne $responseJson.evaluation.issues) {
      $issues = @($responseJson.evaluation.issues | ForEach-Object { Protect-SecretText ([string]$_) })
    }
    if ($responseJson.evaluation.PSObject.Properties["next_steps"] -and $null -ne $responseJson.evaluation.next_steps) {
      $nextSteps = @($responseJson.evaluation.next_steps | ForEach-Object { Protect-SecretText ([string]$_) })
    }
    if ($responseJson.evaluation.PSObject.Properties["approvals"] -and $null -ne $responseJson.evaluation.approvals) {
      $matchingApprovalCount = @($responseJson.evaluation.approvals).Count
    }
  }
}

if ($statusCode -ne 0 -and ($statusCode -lt 200 -or $statusCode -ge 300)) {
  $failures.Add("deployment_gate_authorization_returned_non_2xx") | Out-Null
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

$authorizationId = ""
$authorizationDecision = ""
$authorizationApproved = $false
$authorizationReason = ""
$authorizationWarnings = @()
$authorizationBlockedBy = @()
$authorizationPolicyChecksum = ""
$authorizationBreakGlassEligible = $false
if ($null -ne $responseJson) {
  if ($responseJson.PSObject.Properties["authorization_id"]) { $authorizationId = [string]$responseJson.authorization_id }
  if ($responseJson.PSObject.Properties["decision"]) { $authorizationDecision = [string]$responseJson.decision }
  if ($responseJson.PSObject.Properties["approved"]) { $authorizationApproved = [bool]$responseJson.approved }
  if ($responseJson.PSObject.Properties["reason"]) { $authorizationReason = Protect-SecretText ([string]$responseJson.reason) }
  if ($responseJson.PSObject.Properties["warnings"] -and $null -ne $responseJson.warnings) {
    $authorizationWarnings = @($responseJson.warnings | ForEach-Object { Protect-SecretText ([string]$_) })
  }
  if ($responseJson.PSObject.Properties["blocked_by"] -and $null -ne $responseJson.blocked_by) {
    $authorizationBlockedBy = @($responseJson.blocked_by | ForEach-Object { Protect-SecretText ([string]$_) })
  }
  if ($responseJson.PSObject.Properties["policy_checksum"]) { $authorizationPolicyChecksum = [string]$responseJson.policy_checksum }
  if ($responseJson.PSObject.Properties["break_glass_eligible"]) { $authorizationBreakGlassEligible = [bool]$responseJson.break_glass_eligible }
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
    branch = $Branch
    target_sha = if ([string]::IsNullOrWhiteSpace($TargetSha)) { "" } else { $TargetSha.Trim().ToLowerInvariant() }
    release_id = $ReleaseId
    environment = $Environment
    deployer = $Deployer
    ticket_id = $TicketId
    deployment_run_id = $DeploymentRunId
    evidence_packet_hash_present = -not [string]::IsNullOrWhiteSpace($EvidencePacketHash)
  }
  authorization = [ordered]@{
    authorization_id = $authorizationId
    decision = $authorizationDecision
    approved = $authorizationApproved
    reason = $authorizationReason
    warnings = @($authorizationWarnings)
    blocked_by = @($authorizationBlockedBy)
    policy_checksum = $authorizationPolicyChecksum
    break_glass_eligible = $authorizationBreakGlassEligible
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
