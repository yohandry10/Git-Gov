param(
  [string]$Owner = "",
  [string]$Repo = "",
  [string]$Branch = "main",
  [string]$GitHubToken = "",
  [string[]]$RequiredChecks = @(
    "Security Guard",
    "Server Clippy + Check",
    "Desktop Rust Clippy",
    "Frontend Lint + Typecheck",
    "Website Lint + Typecheck + Build",
    "Validate quality_gates warn/block matrix"
  ),
  [int]$RequiredApprovals = 1
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
. (Join-Path $scriptRoot "_token_helpers.ps1")

$repoInfo = Resolve-GitHubRepoCoordinates -Owner $Owner -Repo $Repo -ScriptRoot $scriptRoot
$Owner = $repoInfo.Owner
$Repo = $repoInfo.Repo
if ([string]::IsNullOrWhiteSpace($Owner) -or [string]::IsNullOrWhiteSpace($Repo)) {
  Write-Error "Could not resolve GitHub repository coordinates. Provide -Owner and -Repo, set GITHUB_REPOSITORY, or configure git remote origin to github.com/<owner>/<repo>."
  exit 1
}

$token = Resolve-GitHubToken -ExplicitToken $GitHubToken -ScriptRoot $scriptRoot
if ([string]::IsNullOrWhiteSpace($token)) {
  Write-Error "Missing GitHub token. Provide -GitHubToken, set GITHUB_TOKEN/GH_TOKEN/GITHUB_PAT/GITHUB_PERSONAL_ACCESS_TOKEN, or define GITHUB_PERSONAL_ACCESS_TOKEN in gitgov/gitgov-server/.env (repository administration permissions required)."
  exit 1
}

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

function Get-GitHubApiFailureMessage {
  param(
    [Parameter(Mandatory = $true)][object]$ErrorRecord,
    [Parameter(Mandatory = $true)][string]$Uri
  )

  $response = $ErrorRecord.Exception.Response
  if ($null -eq $response) {
    return "GitHub API request failed ($Uri): $($ErrorRecord.Exception.Message)"
  }

  $statusCode = $response.StatusCode.value__
  $acceptedPerms = $response.Headers["x-accepted-github-permissions"]
  $body = ""
  try {
    $stream = $response.GetResponseStream()
    if ($null -ne $stream) {
      $reader = New-Object IO.StreamReader($stream)
      $body = $reader.ReadToEnd()
    }
  } catch {
    # best effort
  }

  $parts = @("GitHub API request failed ($Uri): status=$statusCode")
  if (-not [string]::IsNullOrWhiteSpace($acceptedPerms)) {
    $parts += "accepted_permissions=$acceptedPerms"
  }
  if (-not [string]::IsNullOrWhiteSpace($body)) {
    $parts += "body=$body"
  }
  return ($parts -join " | ")
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

try {
  $response = Invoke-RestMethod -Method Put -Uri $uri -Headers $headers -ContentType "application/json" -Body $json
} catch {
  $detail = Get-GitHubApiFailureMessage -ErrorRecord $_ -Uri $uri
  Write-Error ("Failed to apply branch protection. {0}" -f $detail)
  exit 1
}

Write-Host "Done."
Write-Host ("Enforce admins: {0}" -f $response.enforce_admins.enabled)
Write-Host ("Strict checks: {0}" -f $response.required_status_checks.strict)
Write-Host ("Checks count: {0}" -f $response.required_status_checks.contexts.Count)
