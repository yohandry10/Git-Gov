param(
  [string]$BranchName = "",
  [string]$CommitRange = "",
  [switch]$SkipBranchCheck
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$blockedPattern = '(?i)(codex|claude|chatgpt|openai|anthropic)'

function Assert-CleanValue {
  param(
    [Parameter(Mandatory = $true)][string]$Label,
    [Parameter(Mandatory = $true)][string]$Value
  )
  if ([string]::IsNullOrWhiteSpace($Value)) { return }
  if ($Value -match $blockedPattern) {
    throw "$Label contains blocked marker. value='$Value'"
  }
}

if (-not $SkipBranchCheck.IsPresent) {
  if ([string]::IsNullOrWhiteSpace($BranchName)) {
    try {
      $BranchName = (git rev-parse --abbrev-ref HEAD).Trim()
    } catch {
      $BranchName = ""
    }
  }
  Assert-CleanValue -Label "Branch name" -Value $BranchName
}

$commits = @()
if (-not [string]::IsNullOrWhiteSpace($CommitRange)) {
  $commits = @(git log --format='%H%x09%s' $CommitRange)
} else {
  $commits = @(git log --format='%H%x09%s' -n 20)
}

foreach ($line in $commits) {
  if ([string]::IsNullOrWhiteSpace($line)) { continue }
  $parts = $line.Split("`t", 2)
  $sha = $parts[0]
  $subject = if ($parts.Count -gt 1) { $parts[1] } else { "" }
  Assert-CleanValue -Label "Commit subject ($sha)" -Value $subject
}

Write-Host "PASS: public naming policy check completed"
if (-not [string]::IsNullOrWhiteSpace($BranchName)) {
  Write-Host "  branch: $BranchName"
}
if (-not [string]::IsNullOrWhiteSpace($CommitRange)) {
  Write-Host "  commit range: $CommitRange"
} else {
  Write-Host "  commit range: latest 20 commits"
}

exit 0
