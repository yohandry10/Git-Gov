param(
  [string]$TargetsPath = "ops/slo/domain-slo-targets.json",
  [switch]$RequireOrgName,
  [switch]$RequireRepoFullName,
  [switch]$RequireBranch
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$failed = $false

function Add-ConfigError {
  param([Parameter(Mandatory = $true)][string]$Message)
  Write-Host "[FAIL] $Message"
  $script:failed = $true
}

function Test-StringValue {
  param([object]$Value)
  return -not [string]::IsNullOrWhiteSpace([string]$Value)
}

function Test-NumberValue {
  param([object]$Value)
  if ($null -eq $Value) { return $false }
  try {
    [void][double]$Value
    return $true
  } catch {
    return $false
  }
}

function Test-PercentThreshold {
  param([object]$Value)
  if (-not (Test-NumberValue -Value $Value)) { return $false }
  $numeric = [double]$Value
  return $numeric -ge 0 -and $numeric -le 100
}

if (-not (Test-Path $TargetsPath)) {
  Write-Error "Targets file not found: $TargetsPath"
  exit 1
}

try {
  $targets = Get-Content -Path $TargetsPath -Raw | ConvertFrom-Json
} catch {
  Write-Error "Targets file is not valid JSON: $TargetsPath"
  exit 1
}

if ($null -eq $targets -or $null -eq $targets.domains) {
  Write-Error "Invalid targets format: expected { domains: [...] }"
  exit 1
}

$domains = @($targets.domains)
if ($domains.Count -eq 0) {
  Add-ConfigError "No domains configured in $TargetsPath."
}

$seenNames = @{}
$allowedTiers = @("critical", "standard", "internal")
$requiredSloFields = @(
  "readiness_min",
  "blocked_push_rate_max",
  "traceability_gap_max",
  "pipeline_failure_rate_max",
  "sonar_failure_rate_max",
  "unresolved_violation_rate_max"
)

foreach ($domain in $domains) {
  $name = [string]$domain.name
  $label = if (Test-StringValue -Value $name) { $name } else { "<unnamed>" }

  if (-not (Test-StringValue -Value $domain.name)) {
    Add-ConfigError "A domain is missing name."
  } elseif ($seenNames.ContainsKey($name)) {
    Add-ConfigError "Duplicate domain name '$name'."
  } else {
    $seenNames[$name] = $true
  }

  if (-not (Test-StringValue -Value $domain.tier)) {
    Add-ConfigError "Domain '$label' is missing tier."
  } elseif ($allowedTiers -notcontains ([string]$domain.tier)) {
    Add-ConfigError "Domain '$label' has unsupported tier '$($domain.tier)'. Allowed: $($allowedTiers -join ', ')."
  }

  if ($RequireOrgName.IsPresent -and -not (Test-StringValue -Value $domain.org_name)) {
    Add-ConfigError "Domain '$label' must define org_name when -RequireOrgName is used."
  }

  if ($RequireRepoFullName.IsPresent -and -not (Test-StringValue -Value $domain.repo_full_name)) {
    Add-ConfigError "Domain '$label' must define repo_full_name when -RequireRepoFullName is used."
  }

  if ($RequireBranch.IsPresent -and -not (Test-StringValue -Value $domain.branch)) {
    Add-ConfigError "Domain '$label' must define branch when -RequireBranch is used."
  }

  if ($null -eq $domain.slo) {
    Add-ConfigError "Domain '$label' is missing slo."
    continue
  }

  foreach ($field in $requiredSloFields) {
    if ($domain.slo.PSObject.Properties.Name -notcontains $field) {
      Add-ConfigError "Domain '$label' slo is missing '$field'."
      continue
    }

    $value = $domain.slo.$field
    if ($field -eq "readiness_min") {
      if (-not (Test-PercentThreshold -Value $value)) {
        Add-ConfigError "Domain '$label' slo '$field' must be a number from 0 to 100."
      }
    } elseif (-not (Test-PercentThreshold -Value $value)) {
      Add-ConfigError "Domain '$label' slo '$field' must be a percentage number from 0 to 100."
    }
  }
}

if ($failed) {
  exit 1
}

Write-Host "[PASS] Domain SLO target config is valid: $TargetsPath"
exit 0
