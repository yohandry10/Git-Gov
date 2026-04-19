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

$tokenCandidates = @(@($GitHubToken, $env:GITHUB_TOKEN, $env:GH_TOKEN, $env:GITHUB_PAT, $env:GITHUB_PERSONAL_ACCESS_TOKEN) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
if ($tokenCandidates.Count -eq 0) {
  Write-Error "Missing GitHub token. Provide -GitHubToken or set GITHUB_TOKEN/GH_TOKEN/GITHUB_PAT/GITHUB_PERSONAL_ACCESS_TOKEN with repo administration read access."
  exit 1
}
$token = $tokenCandidates[0]

$headers = @{
  Authorization = "Bearer $token"
  Accept = "application/vnd.github+json"
  "X-GitHub-Api-Version" = "2022-11-28"
  "User-Agent" = "gitgov-branch-protection-check"
}

$uri = "https://api.github.com/repos/$Owner/$Repo/branches/$Branch/protection"

try {
  $protection = Invoke-RestMethod -Method Get -Uri $uri -Headers $headers
} catch {
  Write-Error "Could not read branch protection for ${Owner}/${Repo}:$Branch. Ensure protection exists and token has access."
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
