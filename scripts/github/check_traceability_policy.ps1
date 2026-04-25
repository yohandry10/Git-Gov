param(
  [string]$PullRequestTitle = "",
  [string]$CommitRange = "",
  [string]$TicketPattern = "[A-Z][A-Z0-9]+-[0-9]+",
  [switch]$SkipPullRequestTitleCheck,
  [switch]$SkipCommitCheck
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Assert-ContainsTicketId {
  param(
    [Parameter(Mandatory = $true)][string]$Label,
    [Parameter(Mandatory = $true)][string]$Value
  )

  if ([string]::IsNullOrWhiteSpace($Value)) {
    throw "$Label is empty and must include a Jira ticket ID matching '$TicketPattern'."
  }

  if ($Value -notmatch $TicketPattern) {
    throw "$Label must include a Jira ticket ID matching '$TicketPattern'. value='$Value'"
  }
}

if (-not $SkipPullRequestTitleCheck.IsPresent -and -not [string]::IsNullOrWhiteSpace($PullRequestTitle)) {
  Assert-ContainsTicketId -Label "Pull request title" -Value $PullRequestTitle
}

if (-not $SkipCommitCheck.IsPresent) {
  $commits = @()
  if (-not [string]::IsNullOrWhiteSpace($CommitRange)) {
    if ($CommitRange -like "*..*") {
      $commits = @(git log --format='%H%x09%s' $CommitRange)
    } else {
      $commits = @(git log -n 1 --format='%H%x09%s' $CommitRange)
    }
  } else {
    $commits = @(git log --format='%H%x09%s' -n 1)
  }

  foreach ($line in $commits) {
    if ([string]::IsNullOrWhiteSpace($line)) { continue }
    $parts = $line.Split("`t", 2)
    $sha = $parts[0].Trim()
    $subject = if ($parts.Count -gt 1) { $parts[1].Trim() } else { "" }
    Assert-ContainsTicketId -Label "Commit subject ($sha)" -Value $subject
  }
}

Write-Host "PASS: traceability policy check completed"
if (-not [string]::IsNullOrWhiteSpace($PullRequestTitle)) {
  Write-Host "  pull request title checked"
}
if (-not [string]::IsNullOrWhiteSpace($CommitRange)) {
  Write-Host "  commit range: $CommitRange"
} elseif (-not $SkipCommitCheck.IsPresent) {
  Write-Host "  commit range: HEAD"
}

exit 0
