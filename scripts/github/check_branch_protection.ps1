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
  )
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
. (Join-Path $scriptRoot "_token_helpers.ps1")

$token = Resolve-GitHubToken -ExplicitToken $GitHubToken -ScriptRoot $scriptRoot
if ([string]::IsNullOrWhiteSpace($token)) {
  Write-Error "Missing GitHub token. Provide -GitHubToken, set GITHUB_TOKEN/GH_TOKEN/GITHUB_PAT/GITHUB_PERSONAL_ACCESS_TOKEN, or define GITHUB_PERSONAL_ACCESS_TOKEN in gitgov/gitgov-server/.env (repo administration read access required)."
  exit 1
}

$headers = @{
  Authorization = "Bearer $token"
  Accept = "application/vnd.github+json"
  "X-GitHub-Api-Version" = "2022-11-28"
  "User-Agent" = "gitgov-branch-protection-check"
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

$uri = "https://api.github.com/repos/$Owner/$Repo/branches/$Branch/protection"

try {
  $protection = Invoke-RestMethod -Method Get -Uri $uri -Headers $headers
} catch {
  $detail = Get-GitHubApiFailureMessage -ErrorRecord $_ -Uri $uri
  Write-Error ("Could not read branch protection for ${Owner}/${Repo}:$Branch. {0}" -f $detail)
  exit 1
}

$contexts = @()
if ($protection.required_status_checks -and $protection.required_status_checks.contexts) {
  $contexts = @($protection.required_status_checks.contexts)
}

$contextSet = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::OrdinalIgnoreCase)
foreach ($ctx in $contexts) {
  if (-not [string]::IsNullOrWhiteSpace($ctx)) {
    [void]$contextSet.Add($ctx)
  }
}

$missing = @($RequiredChecks | Where-Object { -not $contextSet.Contains($_) })

Write-Host "Branch protection: ${Owner}/${Repo}:$Branch"
Write-Host ("Strict checks: {0}" -f ($protection.required_status_checks.strict -as [string]))
Write-Host ("Enforce admins: {0}" -f ($protection.enforce_admins.enabled -as [string]))
Write-Host ""
Write-Host "Configured required checks:"
foreach ($ctx in ($contexts | Sort-Object)) {
  Write-Host "  - $ctx"
}

Write-Host ""
if ($missing.Count -gt 0) {
  Write-Error ("FAIL: Missing required checks: {0}" -f ($missing -join ", "))
  exit 1
}

Write-Host "PASS: All expected required checks are configured."
