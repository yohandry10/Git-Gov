param(
  [switch]$SkipLegacyScan
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Set-Location $repoRoot

$failed = $false

function Test-EnvExamplePlaceholderValue {
  param(
    [AllowEmptyString()][string]$Value
  )

  $normalized = $Value.Trim().Trim('"').Trim("'")
  if ([string]::IsNullOrWhiteSpace($normalized)) {
    return $true
  }

  $placeholderPattern = @(
    '<[^>]+>',
    '\$\{[^}]+\}',
    '(?i)your[-_]',
    '(?i)example',
    '(?i)sample',
    '(?i)placeholder',
    '(?i)change[-_]?me',
    '(?i)replace',
    '(?i)dummy',
    '(?i)fake',
    '(?i)localhost',
    '(?i)127\.0\.0\.1',
    '(?i)user:password@host',
    '(?i)host:5432'
  ) -join "|"

  return ($normalized -match $placeholderPattern)
}

Write-Host "[1/6] Checking restricted tracked files..."
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
Write-Host "[2/6] Checking tracked .env files..."
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

Write-Host ""
Write-Host "[3/6] Checking .env.example placeholder values..."
$envExamples = @($tracked | Where-Object { $_ -match '(^|/)\.env\.example$' })
$sensitiveEnvNameRegex = '(?i)(TOKEN|SECRET|PASSWORD|API_KEY|PRIVATE_KEY|DATABASE_URL|SUPABASE|JIRA|JENKINS|SONAR|RENDER|WEBHOOK|JWT)'
$envExampleViolations = @()
foreach ($envExample in $envExamples) {
  $lines = @(Get-Content -LiteralPath $envExample)
  for ($i = 0; $i -lt $lines.Count; $i++) {
    $line = $lines[$i].Trim()
    if ([string]::IsNullOrWhiteSpace($line) -or $line.StartsWith("#")) {
      continue
    }
    if ($line -notmatch '^([A-Za-z_][A-Za-z0-9_]*)=(.*)$') {
      continue
    }
    $name = $Matches[1]
    $value = $Matches[2]
    if ($name -match $sensitiveEnvNameRegex -and -not (Test-EnvExamplePlaceholderValue -Value $value)) {
      $envExampleViolations += ("{0}:{1} {2}=<non-placeholder>" -f $envExample, ($i + 1), $name)
    }
  }
}
if ($envExampleViolations.Count -gt 0) {
  Write-Host "[FAIL] .env.example contains non-placeholder values for sensitive keys:"
  $envExampleViolations | ForEach-Object { Write-Host "  - $_" }
  $failed = $true
} else {
  Write-Host "[PASS] .env.example sensitive values are placeholder-only."
}

if (-not $SkipLegacyScan) {
  Write-Host ""
  Write-Host "[4/6] Checking legacy repository markers..."
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
  Write-Host "[4/6] Skipped legacy marker scan."
}

Write-Host ""
Write-Host "[5/6] Checking neutral naming policy (branch + recent commits)..."
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
Write-Host "[6/6] Checking Jira traceability policy (branch + HEAD commit)..."
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
