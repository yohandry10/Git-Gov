param(
  [string]$Channel = "stable",
  [Parameter(Mandatory = $true)][string]$BaseUrl,
  [Parameter(Mandatory = $true)][string]$PubKey,
  [Parameter(Mandatory = $true)][string]$OutputPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$normalized = $BaseUrl.TrimEnd("/")
$endpoint = "$normalized/$Channel/latest.json"

$snippet = @{
  plugins = @{
    updater = @{
      endpoints = @($endpoint)
      pubkey = $PubKey
    }
  }
}

$outputDir = Split-Path -Parent $OutputPath
if (-not [string]::IsNullOrWhiteSpace($outputDir) -and -not (Test-Path $outputDir)) {
  New-Item -ItemType Directory -Path $outputDir | Out-Null
}

$snippet | ConvertTo-Json -Depth 10 | Set-Content -Path $OutputPath -Encoding UTF8

Write-Host "PASS: tauri updater config snippet generated"
Write-Host "  endpoint: $endpoint"
Write-Host "  output:   $OutputPath"
