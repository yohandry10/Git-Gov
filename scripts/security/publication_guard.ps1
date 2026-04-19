param(
  [switch]$SkipLegacyScan
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Set-Location $repoRoot

$failed = $false

Write-Host "[1/3] Checking restricted tracked files..."
$restrictedPattern = '^docs/ENTERPRISE_READINESS\.md$|^docs/ENTERPRISE_READINESS_DECISION\.md$|^docs/AUDIT_.*\.md$|^docs/INTEGRATIONS_AUDIT_.*\.md$|^\.claude/|^CLAUDE\.md$|^\.kiro/|^\.trae/|^\.windsurf/|^\.agents/|^skills/|^skills-lock\.json$|^gitgov-video/'
$tracked = @(git ls-files)
$restrictedHits = @($tracked | Where-Object { $_ -match $restrictedPattern })
if ($restrictedHits.Count -gt 0) {
  Write-Host "[FAIL] Restricted files detected:"
  $restrictedHits | ForEach-Object { Write-Host "  - $_" }
  $failed = $true
} else {
  Write-Host "[PASS] No restricted tracked files."
}

Write-Host ""
Write-Host "[2/3] Checking tracked .env files..."
$trackedEnv = @($tracked | Where-Object {
  ($_ -match '(^|/)\.env($|[.][^/]+$)') -and ($_ -notmatch '\.env\.example$')
})
if ($trackedEnv.Count -gt 0) {
  Write-Host "[FAIL] Tracked .env files detected:"
  $trackedEnv | ForEach-Object { Write-Host "  - $_" }
  $failed = $true
} else {
  Write-Host "[PASS] No tracked .env files (except .env.example)."
}

if (-not $SkipLegacyScan) {
  Write-Host ""
  Write-Host "[3/3] Checking legacy repository markers..."
  $legacyResult = git grep -n -i -E "m'apfrepe|m'apfre" -- . 2>$null
  if ($LASTEXITCODE -eq 0 -and -not [string]::IsNullOrWhiteSpace($legacyResult)) {
    Write-Host "[FAIL] Legacy repository markers detected:"
    $legacyResult -split "`n" | ForEach-Object {
      if (-not [string]::IsNullOrWhiteSpace($_)) { Write-Host "  - $_" }
    }
    $failed = $true
  } else {
    Write-Host "[PASS] No legacy repository markers."
  }
} else {
  Write-Host ""
  Write-Host "[3/3] Skipped legacy marker scan."
}

Write-Host ""
if ($failed) {
  Write-Error "Publication guard failed."
  exit 1
}

Write-Host "PASS: publication guard completed successfully."
