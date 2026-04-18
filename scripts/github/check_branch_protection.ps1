param(
  [string]$Owner = "yohandry10",
  [string]$Repo = "Git-Gov",
  [string]$Branch = "main",
  [string[]]$RequiredChecks = @(
    "server-lint",
    "desktop-lint",
    "frontend-lint",
    "website-lint",
    "Security Guard"
  )
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not $env:GITHUB_TOKEN -or [string]::IsNullOrWhiteSpace($env:GITHUB_TOKEN)) {
  Write-Error "Missing GITHUB_TOKEN. Export a token with repo administration read access."
  exit 1
}

$headers = @{
  Authorization = "Bearer $($env:GITHUB_TOKEN)"
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
