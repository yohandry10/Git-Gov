param(
  [string]$Owner = "yohandry10",
  [string]$Repo = "Git-Gov",
  [string]$Branch = "main",
  [string]$GitHubToken = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
. (Join-Path $scriptRoot "_token_helpers.ps1")

$token = Resolve-GitHubToken -ExplicitToken $GitHubToken -ScriptRoot $scriptRoot
if ([string]::IsNullOrWhiteSpace($token)) {
  Write-Error "Missing GitHub token. Provide -GitHubToken, set GITHUB_TOKEN/GH_TOKEN/GITHUB_PAT/GITHUB_PERSONAL_ACCESS_TOKEN, or define GITHUB_PERSONAL_ACCESS_TOKEN in gitgov/gitgov-server/.env."
  exit 1
}

$headers = @{
  Authorization = "Bearer $token"
  Accept = "application/vnd.github+json"
  "X-GitHub-Api-Version" = "2022-11-28"
  "User-Agent" = "gitgov-token-permissions-check"
}

function Invoke-EndpointCheck {
  param(
    [Parameter(Mandatory = $true)][string]$Label,
    [Parameter(Mandatory = $true)][string]$Uri
  )

  try {
    $resp = Invoke-WebRequest -Method Get -Uri $Uri -Headers $headers
    return [pscustomobject]@{
      Label = $Label
      Status = [int]$resp.StatusCode
      AcceptedPermissions = [string]$resp.Headers["x-accepted-github-permissions"]
      Result = "OK"
    }
  } catch {
    $response = $_.Exception.Response
    if ($null -eq $response) {
      return [pscustomobject]@{
        Label = $Label
        Status = -1
        AcceptedPermissions = ""
        Result = "ERROR: $($_.Exception.Message)"
      }
    }

    return [pscustomobject]@{
      Label = $Label
      Status = [int]$response.StatusCode.value__
      AcceptedPermissions = [string]$response.Headers["x-accepted-github-permissions"]
      Result = if ([int]$response.StatusCode.value__ -eq 403) { "FORBIDDEN" } else { "HTTP_$([int]$response.StatusCode.value__)" }
    }
  }
}

$checks = @(
  @{ label = "Repo metadata"; uri = "https://api.github.com/repos/$Owner/$Repo" }
  @{ label = "Actions secrets"; uri = "https://api.github.com/repos/$Owner/$Repo/actions/secrets?per_page=1" }
  @{ label = "Actions variables"; uri = "https://api.github.com/repos/$Owner/$Repo/actions/variables?per_page=1" }
  @{ label = "Branch protection"; uri = "https://api.github.com/repos/$Owner/$Repo/branches/$Branch/protection" }
)

$results = foreach ($check in $checks) {
  Invoke-EndpointCheck -Label $check.label -Uri $check.uri
}

Write-Host ("Token permission check for {0}/{1}" -f $Owner, $Repo)
Write-Host ""
Write-Host ("{0,-20} {1,-8} {2,-10} {3}" -f "Endpoint", "Status", "Result", "Accepted permissions hint")
Write-Host ("{0,-20} {1,-8} {2,-10} {3}" -f "--------", "------", "------", "-------------------------")
foreach ($row in $results) {
  Write-Host ("{0,-20} {1,-8} {2,-10} {3}" -f $row.Label, $row.Status, $row.Result, $row.AcceptedPermissions)
}

$forbidden = @($results | Where-Object { $_.Status -eq 403 })
if ($forbidden.Count -gt 0) {
  Write-Host ""
  Write-Error ("FAIL: token does not have required permissions for: {0}" -f (($forbidden | ForEach-Object { $_.Label }) -join ", "))
  exit 1
}

Write-Host ""
Write-Host "PASS: token can access required GitHub endpoints."
