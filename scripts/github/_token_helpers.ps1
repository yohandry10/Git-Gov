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
