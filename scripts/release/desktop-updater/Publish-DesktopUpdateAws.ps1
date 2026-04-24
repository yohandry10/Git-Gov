param(
  [Parameter(Mandatory = $true)][string]$ExePath,
  [Parameter(Mandatory = $true)][string]$SigPath,
  [Parameter(Mandatory = $true)][string]$ManifestPath,
  [Parameter(Mandatory = $true)][string]$Bucket,
  [string]$Channel = "stable",
  [string]$Prefix = "desktop",
  [string]$CloudFrontDistributionId = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

foreach ($path in @($ExePath, $SigPath, $ManifestPath)) {
  if (-not (Test-Path $path)) {
    Write-Error "Missing file: $path"
    exit 1
  }
}

$aws = Get-Command aws -ErrorAction SilentlyContinue
if ($null -eq $aws) {
  Write-Error "AWS CLI not found in PATH. Install/configure aws cli first."
  exit 1
}

$exeName = Split-Path -Leaf $ExePath
$sigName = Split-Path -Leaf $SigPath
$base = "s3://$Bucket/$Prefix/$Channel"

Write-Host "Uploading release assets..."
& aws s3 cp "$ExePath" "$base/$exeName"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

& aws s3 cp "$SigPath" "$base/$sigName"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

& aws s3 cp "$ManifestPath" "$base/latest.json"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

if (-not [string]::IsNullOrWhiteSpace($CloudFrontDistributionId)) {
  Write-Host "Creating CloudFront invalidation..."
  & aws cloudfront create-invalidation `
    --distribution-id "$CloudFrontDistributionId" `
    --paths "/$Prefix/$Channel/*"
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

Write-Host "PASS: desktop updater assets published"
Write-Host "  bucket:  $Bucket"
Write-Host "  channel: $Channel"
Write-Host "  base:    $base"
