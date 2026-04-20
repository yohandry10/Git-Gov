param(
  [Parameter(Mandatory = $true)][string]$Version,
  [Parameter(Mandatory = $true)][string]$Url,
  [Parameter(Mandatory = $true)][string]$Signature,
  [string]$Notes = "",
  [string]$MinSupportedVersion = "",
  [switch]$ForceUpdate,
  [string]$ForceUpdateReason = "",
  [switch]$CriticalUpdate,
  [string]$Platform = "windows-x86_64",
  [string]$PubDateUtc = "",
  [Parameter(Mandatory = $true)][string]$OutputPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($PubDateUtc)) {
  $PubDateUtc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
}

try {
  [void][datetime]::Parse($PubDateUtc)
} catch {
  Write-Error "Invalid -PubDateUtc value: $PubDateUtc"
  exit 1
}

$manifest = @{
  version = $Version
  notes = $Notes
  pub_date = $PubDateUtc
  platforms = @{
    $Platform = @{
      signature = $Signature
      url = $Url
    }
  }
}

if (-not [string]::IsNullOrWhiteSpace($MinSupportedVersion)) {
  $manifest.min_supported_version = $MinSupportedVersion
}

$forceEnabled = $ForceUpdate.IsPresent -or $CriticalUpdate.IsPresent
if ($forceEnabled) {
  $manifest.force_update = $true
}

if (-not [string]::IsNullOrWhiteSpace($ForceUpdateReason)) {
  $manifest.force_update_reason = $ForceUpdateReason
}

if ($CriticalUpdate.IsPresent) {
  $manifest.critical_update = $true
}

$outputDir = Split-Path -Parent $OutputPath
if (-not [string]::IsNullOrWhiteSpace($outputDir) -and -not (Test-Path $outputDir)) {
  New-Item -ItemType Directory -Path $outputDir | Out-Null
}

$manifest | ConvertTo-Json -Depth 10 | Set-Content -Path $OutputPath -Encoding UTF8

Write-Host "PASS: tauri updater manifest generated"
Write-Host "  version:  $Version"
Write-Host "  platform: $Platform"
if (-not [string]::IsNullOrWhiteSpace($MinSupportedVersion)) {
  Write-Host "  min_supported_version: $MinSupportedVersion"
}
if ($forceEnabled) {
  Write-Host "  force_update: true"
}
Write-Host "  output:   $OutputPath"
