param(
  [switch]$SkipLegacyScan
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Set-Location $repoRoot

$failed = $false

Write-Host "[1/5] Checking restricted tracked files..."
$tracked = @(git ls-files)
$restrictedPattern = '^docs/ENTERPRISE_READINESS\.md$|^docs/ENTERPRISE_READINESS_DECISION\.md$|^docs/AUDIT_.*\.md$|^docs/INTEGRATIONS_AUDIT_.*\.md$|^skills/|^skills-lock\.json$|^gitgov-video/'
$restrictedHits = @($tracked | Where-Object { $_ -match $restrictedPattern })

$allowedHiddenTopLevel = @(".github", ".githooks", ".gitignore")
$hiddenTopLevelEntries = @($tracked | Where-Object {
  $top = ($_ -split '/')[0]
  ($top -match '^\..+') -and ($allowedHiddenTopLevel -notcontains $top)
})
if ($hiddenTopLevelEntries.Count -gt 0) {
  $restrictedHits += $hiddenTopLevelEntries
}
if ($restrictedHits.Count -gt 0) {
  Write-Host "[FAIL] Restricted files detected:"
  $restrictedHits | ForEach-Object { Write-Host "  - $_" }
  $failed = $true
} else {
  Write-Host "[PASS] No restricted tracked files."
}

Write-Host ""
Write-Host "[2/5] Checking tracked .env files..."
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
  Write-Host "[3/5] Checking legacy repository markers..."
  # Build the regex dynamically to avoid self-matching the literal marker text in this file.
  $legacyRegex = @(
    ("ma" + "pfrepe"),
    ("ma" + "pfre")
  ) -join "|"
  $legacyResult = git grep -n -i -E "$legacyRegex" -- . ':!scripts/security/publication_guard.ps1' 2>$null
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
  Write-Host "[3/5] Skipped legacy marker scan."
}

Write-Host ""
Write-Host "[4/5] Checking neutral naming policy (branch + recent commits)..."
$namingPolicyRegex = @(
  ("co" + "dex"),
  ("cl" + "aude"),
  ("ai[-_ ]?agent"),
  ("ai[-_ ]?assistant")
) -join "|"

$currentBranch = ""
try {
  $currentBranch = (git rev-parse --abbrev-ref HEAD 2>$null).Trim()
} catch {
  $currentBranch = ""
}
if (-not [string]::IsNullOrWhiteSpace($currentBranch) -and $currentBranch -match $namingPolicyRegex) {
  Write-Host ("[FAIL] Branch name violates neutral naming policy: {0}" -f $currentBranch)
  $failed = $true
} else {
  Write-Host "[PASS] Branch naming policy."
}

$recentCommitMessages = @()
try {
  $recentCommitMessages = @(git log -n 30 --pretty=format:%s 2>$null)
} catch {
  $recentCommitMessages = @()
}
$messageViolations = @($recentCommitMessages | Where-Object { $_ -match $namingPolicyRegex })
if ($messageViolations.Count -gt 0) {
  Write-Host "[FAIL] Recent commit messages violate neutral naming policy:"
  $messageViolations | ForEach-Object { Write-Host ("  - {0}" -f $_) }
  $failed = $true
} else {
  Write-Host "[PASS] Commit message naming policy."
}

Write-Host ""
Write-Host "[5/5] Checking Jira traceability policy (branch + HEAD commit)..."
try {
  & (Join-Path $repoRoot "scripts\github\check_traceability_policy.ps1") `
    -BranchName $currentBranch `
    -CommitRange "HEAD" `
    -SkipPullRequestTitleCheck
  if ($LASTEXITCODE -ne 0) {
    $failed = $true
  }
} catch {
  Write-Host ("[FAIL] Jira traceability policy failed: {0}" -f $_.Exception.Message)
  $failed = $true
}

Write-Host ""
if ($failed) {
  Write-Error "Publication guard failed."
  exit 1
}

Write-Host "PASS: publication guard completed successfully."
