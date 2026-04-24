param(
  [string]$Owner = "",
  [string]$Repo = "",
  [string]$Base = "main",
  [string]$Head = "",
  [string]$Title = "",
  [string]$Body = "",
  [string]$BodyFile = "",
  [switch]$Draft,
  [string]$GitHubToken = ""
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
  Write-Error "Missing GitHub token. Provide -GitHubToken, set GITHUB_TOKEN/GH_TOKEN/GITHUB_PAT/GITHUB_PERSONAL_ACCESS_TOKEN, or define GITHUB_PERSONAL_ACCESS_TOKEN in gitgov/gitgov-server/.env."
  exit 1
}

if ([string]::IsNullOrWhiteSpace($Head)) {
  try {
    $Head = (git rev-parse --abbrev-ref HEAD 2>$null).Trim()
  } catch {
    $Head = ""
  }
}
if ([string]::IsNullOrWhiteSpace($Head)) {
  Write-Error "Could not resolve PR head branch. Provide -Head or run from a git branch."
  exit 1
}

if ([string]::IsNullOrWhiteSpace($Title)) {
  $Title = "chore: merge $Head into $Base"
}

if ($Head -eq $Base) {
  Write-Error "Head branch cannot be the same as base branch ($Base)."
  exit 1
}

$namingPolicyRegex = @(
  ("co" + "dex"),
  ("cl" + "aude"),
  ("ai[-_ ]?agent"),
  ("ai[-_ ]?assistant")
) -join "|"
if ($Head -match $namingPolicyRegex -or $Title -match $namingPolicyRegex) {
  Write-Error "Branch/PR title violates publication naming policy. Use neutral names without internal tooling markers."
  exit 1
}

if ([string]::IsNullOrWhiteSpace($Body) -and -not [string]::IsNullOrWhiteSpace($BodyFile)) {
  if (-not (Test-Path -LiteralPath $BodyFile)) {
    Write-Error "BodyFile not found: $BodyFile"
    exit 1
  }
  $Body = Get-Content -LiteralPath $BodyFile -Raw
}

$headers = @{
  Authorization = "Bearer $token"
  Accept = "application/vnd.github+json"
  "X-GitHub-Api-Version" = "2022-11-28"
  "User-Agent" = "gitgov-pr-helper"
}

$compareUrl = "https://github.com/$Owner/$Repo/compare/$Base...$Head`?expand=1"

try {
  $open = Invoke-RestMethod -Method Get -Uri "https://api.github.com/repos/$Owner/$Repo/pulls?state=open&head=$Owner`:$Head" -Headers $headers
  if ($open.Count -gt 0) {
    $pr = $open[0]
    Write-Host ("PASS: open PR already exists -> #{0} {1}" -f $pr.number, $pr.html_url)
    exit 0
  }
} catch {
  Write-Warning ("Could not list open PRs: {0}" -f $_.Exception.Message)
}

$payload = @{
  title = $Title
  head = $Head
  base = $Base
  body = $Body
  draft = [bool]$Draft
  maintainer_can_modify = $true
} | ConvertTo-Json -Depth 6

try {
  $created = Invoke-RestMethod -Method Post -Uri "https://api.github.com/repos/$Owner/$Repo/pulls" -Headers $headers -ContentType "application/json" -Body $payload
  Write-Host ("PASS: PR created -> #{0} {1}" -f $created.number, $created.html_url)
  exit 0
} catch {
  $response = $_.Exception.Response
  $status = if ($response) { [int]$response.StatusCode.value__ } else { -1 }
  $message = $_.Exception.Message
  $accepted = ""
  $bodyText = ""
  if ($response) {
    $accepted = [string]$response.Headers["x-accepted-github-permissions"]
    try {
      $stream = $response.GetResponseStream()
      if ($null -ne $stream) {
        $reader = New-Object IO.StreamReader($stream)
        $bodyText = $reader.ReadToEnd()
      }
    } catch {}
  }

  if ($status -eq 403) {
    Write-Warning "Token cannot create pull requests via API (403)."
    if (-not [string]::IsNullOrWhiteSpace($accepted)) {
      Write-Warning ("Accepted permissions hint: {0}" -f $accepted)
    }
    if (-not [string]::IsNullOrWhiteSpace($bodyText)) {
      Write-Warning ("GitHub response: {0}" -f $bodyText)
    } else {
      Write-Warning ("GitHub response: {0}" -f $message)
    }
    Write-Host ("OPEN THIS URL TO CREATE PR: {0}" -f $compareUrl)
    exit 0
  }

  Write-Error ("Failed to create PR (status={0}): {1}" -f $status, (if([string]::IsNullOrWhiteSpace($bodyText)){ $message } else { $bodyText }))
  exit 1
}
