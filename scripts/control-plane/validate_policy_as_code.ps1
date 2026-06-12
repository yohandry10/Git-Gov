param(
  [string]$RepoPath = ".",
  [Parameter(Mandatory = $true)]
  [string]$BaseRef,
  [Parameter(Mandatory = $true)]
  [string]$HeadRef,
  [switch]$Blocking,
  [switch]$Json
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repo = Resolve-Path -LiteralPath $RepoPath
$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$workspaceRoot = Resolve-Path -LiteralPath (Join-Path $scriptRoot "..\..")
$manifestPath = Join-Path $workspaceRoot "gitgov\policy-core\Cargo.toml"

if (-not (Test-Path -LiteralPath $manifestPath)) {
  throw "Policy core manifest not found at $manifestPath"
}

$cargoArgs = @(
  "run",
  "--quiet",
  "--manifest-path",
  $manifestPath,
  "--bin",
  "gitgov-policy",
  "--",
  "validate",
  "--repo",
  $repo.Path,
  "--base-ref",
  $BaseRef,
  "--head-ref",
  $HeadRef
)

if ($Blocking) {
  $cargoArgs += "--blocking"
}
if ($Json) {
  $cargoArgs += "--json"
}

& cargo @cargoArgs
exit $LASTEXITCODE
