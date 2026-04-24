function Get-GitHubTokenFromEnvFile {
  param(
    [Parameter(Mandatory = $true)][string]$Path
  )

  if ([string]::IsNullOrWhiteSpace($Path) -or -not (Test-Path -LiteralPath $Path)) {
    return ""
  }

  try {
    $line = Get-Content -LiteralPath $Path | Where-Object { $_ -match '^\s*GITHUB_PERSONAL_ACCESS_TOKEN\s*=' } | Select-Object -First 1
    if ([string]::IsNullOrWhiteSpace($line)) {
      return ""
    }
    $raw = ($line -split '=', 2)[1].Trim()
    if ($raw.StartsWith('"') -and $raw.EndsWith('"') -and $raw.Length -ge 2) {
      return $raw.Substring(1, $raw.Length - 2).Trim()
    }
    return $raw
  } catch {
    return ""
  }
}

function Resolve-GitHubToken {
  param(
    [string]$ExplicitToken = "",
    [string]$ScriptRoot = ""
  )

  $candidates = @()
  if (-not [string]::IsNullOrWhiteSpace($ExplicitToken)) { $candidates += $ExplicitToken }
  if (-not [string]::IsNullOrWhiteSpace($env:GITHUB_TOKEN)) { $candidates += $env:GITHUB_TOKEN }
  if (-not [string]::IsNullOrWhiteSpace($env:GH_TOKEN)) { $candidates += $env:GH_TOKEN }
  if (-not [string]::IsNullOrWhiteSpace($env:GITHUB_PAT)) { $candidates += $env:GITHUB_PAT }
  if (-not [string]::IsNullOrWhiteSpace($env:GITHUB_PERSONAL_ACCESS_TOKEN)) { $candidates += $env:GITHUB_PERSONAL_ACCESS_TOKEN }

  if (-not [string]::IsNullOrWhiteSpace($ScriptRoot)) {
    $repoRoot = Split-Path -Parent (Split-Path -Parent $ScriptRoot)
    $repoEnvPath = Join-Path $repoRoot "gitgov\gitgov-server\.env"
    $fromRepoEnv = Get-GitHubTokenFromEnvFile -Path $repoEnvPath
    if (-not [string]::IsNullOrWhiteSpace($fromRepoEnv)) { $candidates += $fromRepoEnv }
  }

  $cwdEnvPath = Join-Path (Get-Location).Path "gitgov\gitgov-server\.env"
  $fromCwdEnv = Get-GitHubTokenFromEnvFile -Path $cwdEnvPath
  if (-not [string]::IsNullOrWhiteSpace($fromCwdEnv)) { $candidates += $fromCwdEnv }

  return @($candidates | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })[0]
}

function Get-GitRepoRootPath {
  param(
    [string]$ScriptRoot = ""
  )

  $candidates = @()
  if (-not [string]::IsNullOrWhiteSpace($ScriptRoot)) {
    $candidates += (Split-Path -Parent (Split-Path -Parent $ScriptRoot))
  }
  $candidates += (Get-Location).Path

  foreach ($candidate in $candidates) {
    if ([string]::IsNullOrWhiteSpace($candidate)) { continue }
    try {
      $resolved = (git -C $candidate rev-parse --show-toplevel 2>$null).Trim()
      if (-not [string]::IsNullOrWhiteSpace($resolved)) {
        return $resolved
      }
    } catch {
      # best effort
    }
  }

  return ""
}

function Convert-GitRemoteToGitHubRepo {
  param(
    [string]$RemoteUrl = ""
  )

  if ([string]::IsNullOrWhiteSpace($RemoteUrl)) {
    return [pscustomobject]@{ Owner = ""; Repo = ""; Source = "" }
  }

  $trimmed = $RemoteUrl.Trim()
  $patterns = @(
    '^https?://github\.com/(?<owner>[^/]+)/(?<repo>[^/]+?)(?:\.git)?$',
    '^git@github\.com:(?<owner>[^/]+)/(?<repo>[^/]+?)(?:\.git)?$',
    '^ssh://git@github\.com/(?<owner>[^/]+)/(?<repo>[^/]+?)(?:\.git)?$'
  )

  foreach ($pattern in $patterns) {
    $m = [regex]::Match($trimmed, $pattern, [System.Text.RegularExpressions.RegexOptions]::IgnoreCase)
    if ($m.Success) {
      return [pscustomobject]@{
        Owner = $m.Groups["owner"].Value
        Repo = $m.Groups["repo"].Value
        Source = "git_remote_origin"
      }
    }
  }

  return [pscustomobject]@{ Owner = ""; Repo = ""; Source = "" }
}

function Resolve-GitHubRepoCoordinates {
  param(
    [string]$Owner = "",
    [string]$Repo = "",
    [string]$ScriptRoot = ""
  )

  $resolvedOwner = $Owner
  $resolvedRepo = $Repo
  $source = "explicit"

  if ([string]::IsNullOrWhiteSpace($resolvedOwner) -or [string]::IsNullOrWhiteSpace($resolvedRepo)) {
    $envRepo = $env:GITHUB_REPOSITORY
    if (-not [string]::IsNullOrWhiteSpace($envRepo) -and $envRepo.Contains("/")) {
      $parts = $envRepo.Split("/", 2)
      if ([string]::IsNullOrWhiteSpace($resolvedOwner)) { $resolvedOwner = $parts[0] }
      if ([string]::IsNullOrWhiteSpace($resolvedRepo)) { $resolvedRepo = $parts[1] }
      $source = "env:GITHUB_REPOSITORY"
    }
  }

  if ([string]::IsNullOrWhiteSpace($resolvedOwner) -or [string]::IsNullOrWhiteSpace($resolvedRepo)) {
    $repoRoot = Get-GitRepoRootPath -ScriptRoot $ScriptRoot
    if (-not [string]::IsNullOrWhiteSpace($repoRoot)) {
      $remoteUrl = ""
      try {
        $remoteUrl = (git -C $repoRoot config --get remote.origin.url 2>$null).Trim()
      } catch {
        $remoteUrl = ""
      }

      $converted = Convert-GitRemoteToGitHubRepo -RemoteUrl $remoteUrl
      if ([string]::IsNullOrWhiteSpace($resolvedOwner)) { $resolvedOwner = $converted.Owner }
      if ([string]::IsNullOrWhiteSpace($resolvedRepo)) { $resolvedRepo = $converted.Repo }
      if (-not [string]::IsNullOrWhiteSpace($converted.Source)) {
        $source = $converted.Source
      }
    }
  }

  return [pscustomobject]@{
    Owner = $resolvedOwner
    Repo = $resolvedRepo
    Source = $source
  }
}
