param(
  [string]$TauriConfigPath = "gitgov/src-tauri/tauri.conf.json",
  [int]$TimeoutSeconds = 20,
  [switch]$SkipEndpointProbe,
  [string]$OutputPath = "",
  [switch]$FailOnWarnings
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Add-CheckResult {
  param(
    [System.Collections.Generic.List[object]]$Bag,
    [string]$Check,
    [string]$Status,
    [string]$Details
  )
  $Bag.Add([pscustomobject]@{
    Check = $Check
    Status = $Status
    Details = $Details
  }) | Out-Null
}

function Invoke-EndpointProbe {
  param([Parameter(Mandatory = $true)][string]$Url)
  try {
    $resp = Invoke-WebRequest -Uri $Url -Method Get -TimeoutSec $TimeoutSeconds
    return @{
      code = [int]$resp.StatusCode
      content = $resp.Content
      error = $null
    }
  } catch {
    if ($_.Exception.Response) {
      $response = $_.Exception.Response
      $statusCode = [int]$response.StatusCode
      $reader = New-Object IO.StreamReader($response.GetResponseStream())
      $content = $reader.ReadToEnd()
      return @{
        code = $statusCode
        content = $content
        error = $_.Exception.Message
      }
    }
    return @{
      code = 0
      content = ""
      error = $_.Exception.Message
    }
  }
}

function Test-VersionLike {
  param([string]$Value)
  if ([string]::IsNullOrWhiteSpace($Value)) { return $false }
  return $Value -match '^[vV]?\d+\.\d+\.\d+([\-+][0-9A-Za-z\.\-]+)?$'
}

$results = New-Object System.Collections.Generic.List[object]
$warnings = New-Object System.Collections.Generic.List[string]
$failures = New-Object System.Collections.Generic.List[string]

if (-not (Test-Path $TauriConfigPath)) {
  Write-Error "Missing tauri config path: $TauriConfigPath"
  exit 1
}

$raw = Get-Content $TauriConfigPath -Raw
$tauri = $raw | ConvertFrom-Json

$updater = $tauri.plugins.updater
if ($null -eq $updater) {
  Add-CheckResult -Bag $results -Check "plugins.updater" -Status "FAIL" -Details "Missing updater config block."
  $failures.Add("plugins.updater missing in tauri.conf.json") | Out-Null
} else {
  Add-CheckResult -Bag $results -Check "plugins.updater" -Status "PASS" -Details "Updater block exists."
}

$pubkey = ""
$endpoints = @()
if ($null -ne $updater) {
  $pubkey = [string]$updater.pubkey
  $endpoints = @($updater.endpoints)
}

if ([string]::IsNullOrWhiteSpace($pubkey)) {
  Add-CheckResult -Bag $results -Check "updater.pubkey" -Status "FAIL" -Details "pubkey is empty."
  $failures.Add("updater.pubkey missing") | Out-Null
} elseif ($pubkey -match "TU_PUBLIC_KEY|PUBLIC_KEY|REPLACE_ME") {
  Add-CheckResult -Bag $results -Check "updater.pubkey" -Status "WARN" -Details "pubkey looks like placeholder."
  $warnings.Add("updater.pubkey appears to be placeholder") | Out-Null
} else {
  Add-CheckResult -Bag $results -Check "updater.pubkey" -Status "PASS" -Details "pubkey configured."
}

if ($endpoints.Count -eq 0) {
  Add-CheckResult -Bag $results -Check "updater.endpoints" -Status "FAIL" -Details "No updater endpoints configured."
  $failures.Add("updater.endpoints missing") | Out-Null
} else {
  Add-CheckResult -Bag $results -Check "updater.endpoints" -Status "PASS" -Details ("Configured endpoints: " + ($endpoints -join ", "))
}

foreach ($endpoint in $endpoints) {
  $label = "endpoint syntax: $endpoint"
  try {
    $uri = [Uri]$endpoint
    if ($uri.Scheme -ne "https") {
      Add-CheckResult -Bag $results -Check $label -Status "WARN" -Details "Endpoint is not HTTPS."
      $warnings.Add("Endpoint '$endpoint' is not HTTPS") | Out-Null
    } else {
      Add-CheckResult -Bag $results -Check $label -Status "PASS" -Details "HTTPS endpoint syntax OK."
    }
  } catch {
    Add-CheckResult -Bag $results -Check $label -Status "FAIL" -Details "Invalid URL."
    $failures.Add("Invalid updater endpoint URL: $endpoint") | Out-Null
  }
}

if (-not $SkipEndpointProbe.IsPresent) {
  foreach ($endpoint in $endpoints) {
    $label = "endpoint probe: $endpoint"
    $probe = Invoke-EndpointProbe -Url $endpoint
    if ($probe.code -ne 200) {
      Add-CheckResult -Bag $results -Check $label -Status "WARN" -Details "HTTP $($probe.code). $($probe.error)"
      $warnings.Add("Endpoint probe returned HTTP $($probe.code) for $endpoint") | Out-Null
      continue
    }

    $manifest = $null
    try {
      $manifest = $probe.content | ConvertFrom-Json
    } catch {
      Add-CheckResult -Bag $results -Check $label -Status "WARN" -Details "HTTP 200 but payload is not valid JSON."
      $warnings.Add("Endpoint payload is not valid JSON: $endpoint") | Out-Null
      continue
    }

    $hasVersion = $manifest.PSObject.Properties.Name -contains "version"
    $hasPlatforms = $manifest.PSObject.Properties.Name -contains "platforms"
    if ($hasVersion -and $hasPlatforms) {
      Add-CheckResult -Bag $results -Check $label -Status "PASS" -Details "HTTP 200 and manifest has version+platforms."
    } else {
      Add-CheckResult -Bag $results -Check $label -Status "WARN" -Details "HTTP 200 but manifest misses version/platforms."
      $warnings.Add("Manifest shape incomplete for $endpoint") | Out-Null
    }

    if ($manifest.PSObject.Properties.Name -contains "min_supported_version") {
      $minSupportedVersion = [string]$manifest.min_supported_version
      if (Test-VersionLike -Value $minSupportedVersion) {
        Add-CheckResult -Bag $results -Check "manifest min_supported_version: $endpoint" -Status "PASS" -Details "min_supported_version present and semver-like."
      } else {
        Add-CheckResult -Bag $results -Check "manifest min_supported_version: $endpoint" -Status "WARN" -Details "min_supported_version present but format is not semver-like."
        $warnings.Add("min_supported_version format is not semver-like for $endpoint") | Out-Null
      }
    } else {
      Add-CheckResult -Bag $results -Check "manifest min_supported_version: $endpoint" -Status "WARN" -Details "Not present (forced update policy cannot enforce minimum supported version)."
      $warnings.Add("Manifest without min_supported_version: $endpoint") | Out-Null
    }

    $forceFlagPresent = $manifest.PSObject.Properties.Name -contains "force_update"
    if ($forceFlagPresent -and ($manifest.force_update -eq $true)) {
      $forceReason = if ($manifest.PSObject.Properties.Name -contains "force_update_reason") { [string]$manifest.force_update_reason } else { "" }
      if ([string]::IsNullOrWhiteSpace($forceReason)) {
        Add-CheckResult -Bag $results -Check "manifest force_update_reason: $endpoint" -Status "WARN" -Details "force_update=true without explicit reason."
        $warnings.Add("force_update=true without force_update_reason: $endpoint") | Out-Null
      } else {
        Add-CheckResult -Bag $results -Check "manifest force_update_reason: $endpoint" -Status "PASS" -Details "force_update reason configured."
      }
    } else {
      Add-CheckResult -Bag $results -Check "manifest force_update flag: $endpoint" -Status "PASS" -Details "force_update not active."
    }
  }
} else {
  Add-CheckResult -Bag $results -Check "endpoint probe" -Status "WARN" -Details "Skipped by -SkipEndpointProbe."
  $warnings.Add("Endpoint probe skipped") | Out-Null
}

$summary = if ($failures.Count -gt 0) { "FAIL" } elseif ($warnings.Count -gt 0) { "WARN" } else { "PASS" }
if ($FailOnWarnings.IsPresent -and $summary -eq "WARN") {
  $summary = "FAIL"
  $failures.Add("FailOnWarnings enabled and warnings were found") | Out-Null
}

if ([string]::IsNullOrWhiteSpace($OutputPath)) {
  $stamp = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHHmmssZ")
  $OutputPath = "docs/reports/desktop-updater-readiness-$stamp.md"
}
if (!(Test-Path (Split-Path -Parent $OutputPath))) {
  New-Item -ItemType Directory -Path (Split-Path -Parent $OutputPath) | Out-Null
}

$generatedUtc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-dd HH:mm:ss")
$rows = $results | ForEach-Object { "| $($_.Check) | $($_.Status) | $($_.Details -replace '\|','\\|') |" }
$warningText = if ($warnings.Count -eq 0) { "- none" } else { ($warnings | ForEach-Object { "- $_" }) -join "`n" }
$failureText = if ($failures.Count -eq 0) { "- none" } else { ($failures | ForEach-Object { "- $_" }) -join "`n" }

$report = @"
# Desktop Updater Readiness Report

Generated (UTC): $generatedUtc
Summary: **$summary**
Tauri config: $TauriConfigPath

## Checks

| Check | Status | Details |
|---|---|---|
$(($rows -join "`n"))

## Warnings

$warningText

## Failures

$failureText
"@

Set-Content -Path $OutputPath -Value $report -Encoding UTF8

Write-Host "${summary}: desktop updater readiness validation completed"
Write-Host "  output: $OutputPath"

if ($summary -eq "FAIL") {
  exit 1
}
exit 0
