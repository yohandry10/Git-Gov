param(
  [string]$Owner = "yohandry10",
  [string]$Repo = "Git-Gov",
  [string]$Branch = "main",
  [string]$GitHubToken = "",
  [string[]]$RequiredChecks = @(
    "Server Clippy + Check",
    "Desktop Rust Clippy",
    "Frontend Lint + Typecheck",
    "Website Lint + Typecheck + Build",
    "Security Guard"
  ),
  [int]$RequiredApprovals = 1
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$tokenCandidates = @(@($GitHubToken, $env:GITHUB_TOKEN, $env:GH_TOKEN, $env:GITHUB_PAT, $env:GITHUB_PERSONAL_ACCESS_TOKEN) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
if ($tokenCandidates.Count -eq 0) {
  Write-Error "Missing GitHub token. Provide -GitHubToken or set GITHUB_TOKEN/GH_TOKEN/GITHUB_PAT/GITHUB_PERSONAL_ACCESS_TOKEN with repository administration permissions."
  exit 1
}
$token = $tokenCandidates[0]

if ($RequiredApprovals -lt 0) {
  Write-Error "RequiredApprovals must be >= 0."
  exit 1
}

$headers = @{
  Authorization = "Bearer $token"
  Accept = "application/vnd.github+json"
  "X-GitHub-Api-Version" = "2022-11-28"
  "User-Agent" = "gitgov-branch-protection-script"
}

$contexts = @($RequiredChecks | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | Select-Object -Unique)
if ($contexts.Count -eq 0) {
  Write-Error "RequiredChecks cannot be empty."
  exit 1
}

$uri = "https://api.github.com/repos/$Owner/$Repo/branches/$Branch/protection"

$payload = @{
  required_status_checks = @{
    strict = $true
    contexts = $contexts
  }
  enforce_admins = $true
  required_pull_request_reviews = @{
    dismiss_stale_reviews = $true
    require_code_owner_reviews = $false
    required_approving_review_count = $RequiredApprovals
  }
  restrictions = $null
  allow_force_pushes = $false
  allow_deletions = $false
  required_linear_history = $true
  required_conversation_resolution = $true
  lock_branch = $false
  allow_fork_syncing = $false
}

$json = $payload | ConvertTo-Json -Depth 6

Write-Host "Applying branch protection to ${Owner}/${Repo}:$Branch ..."
Write-Host "Required checks: $($contexts -join ', ')"

$response = Invoke-RestMethod -Method Put -Uri $uri -Headers $headers -ContentType "application/json" -Body $json

Write-Host "Done."
Write-Host ("Enforce admins: {0}" -f $response.enforce_admins.enabled)
Write-Host ("Strict checks: {0}" -f $response.required_status_checks.strict)
Write-Host ("Checks count: {0}" -f $response.required_status_checks.contexts.Count)
