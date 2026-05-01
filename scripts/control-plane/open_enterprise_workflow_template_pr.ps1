param(
  [Parameter(Mandatory = $true, ParameterSetName = "PackPath")]
  [string]$PackPath,

  [Parameter(Mandatory = $true, ParameterSetName = "PackDir")]
  [string]$PackDir,

  [string]$Repository,
  [string]$BaseBranch,
  [string]$BranchName,
  [string]$TicketId,
  [string]$PullRequestTitle,
  [string]$PullRequestBody,
  [string]$CommitMessage,
  [string]$OutputPlanPath,
  [string]$GitHubToken = $env:GITHUB_TOKEN,
  [string]$GitHubCliPath = "gh",

  [switch]$Apply,
  [switch]$Overwrite,
  [switch]$ReadyForReview
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Fail-RemoteInstall {
  param([Parameter(Mandatory = $true)][string]$Message)
  Write-Host "[FAIL] $Message"
  exit 1
}

function Get-ObjectPropertyValue {
  param(
    [Parameter(Mandatory = $true)]$Object,
    [Parameter(Mandatory = $true)][string]$Name
  )

  $property = $Object.PSObject.Properties[$Name]
  if ($null -eq $property) {
    return $null
  }
  return $property.Value
}

function Resolve-ExistingDirectory {
  param([Parameter(Mandatory = $true)][string]$Path)

  if (-not (Test-Path -LiteralPath $Path)) {
    Fail-RemoteInstall "Directory not found: $Path"
  }

  $item = Get-Item -LiteralPath $Path
  if (-not $item.PSIsContainer) {
    Fail-RemoteInstall "Path is not a directory: $Path"
  }

  return $item.FullName
}

function ConvertTo-SafeWorkflowPath {
  param([Parameter(Mandatory = $true)][string]$Path)

  if ([string]::IsNullOrWhiteSpace($Path)) {
    Fail-RemoteInstall "Workflow file path is empty."
  }
  if ($Path.IndexOf([char]0) -ge 0) {
    Fail-RemoteInstall "Workflow file path contains a null byte."
  }

  $normalized = $Path.Trim().Replace("\", "/")
  while ($normalized.StartsWith("./")) {
    $normalized = $normalized.Substring(2)
  }

  if ([System.IO.Path]::IsPathRooted($Path) -or $normalized -match '^[A-Za-z]:') {
    Fail-RemoteInstall "Workflow file path must be relative: $Path"
  }
  if ($normalized -match '(^|/)\.\.($|/)') {
    Fail-RemoteInstall "Workflow file path must not contain parent directory segments: $Path"
  }
  if ($normalized -notmatch '^\.github/workflows/[A-Za-z0-9._-]+\.ya?ml$') {
    Fail-RemoteInstall "Workflow file path must be a .yml or .yaml file directly under .github/workflows: $Path"
  }

  return $normalized
}

function ConvertTo-SafeBranchName {
  param([Parameter(Mandatory = $true)][string]$Value)

  $branch = $Value.Trim()
  if ([string]::IsNullOrWhiteSpace($branch)) {
    Fail-RemoteInstall "Branch name is required."
  }
  if ($branch.IndexOf([char]0) -ge 0) {
    Fail-RemoteInstall "Branch name contains a null byte."
  }
  if ($branch -notmatch '^[A-Za-z0-9._/-]+$') {
    Fail-RemoteInstall "Branch name may contain only letters, numbers, dot, underscore, slash, and dash."
  }
  if ($branch.StartsWith("/") -or $branch.EndsWith("/") -or $branch.Contains("//") -or $branch.Contains("..") -or $branch.Contains("@{")) {
    Fail-RemoteInstall "Branch name is not a safe Git ref name: $branch"
  }
  if ($branch.EndsWith(".lock")) {
    Fail-RemoteInstall "Branch name must not end with .lock."
  }

  return $branch
}

function Assert-PackSafety {
  param($Safety)

  if ($null -eq $Safety) {
    return
  }

  $containsSecretValues = Get-ObjectPropertyValue -Object $Safety -Name "contains_secret_values"
  if ($containsSecretValues -eq $true) {
    Fail-RemoteInstall "Pack declares that it contains secret values. Refusing remote PR creation."
  }

  $mutatesRepository = Get-ObjectPropertyValue -Object $Safety -Name "mutates_customer_repository"
  if ($mutatesRepository -eq $true) {
    Fail-RemoteInstall "Pack declares repository mutation behavior. This script accepts static workflow templates only."
  }
}

function New-WorkflowTemplateFile {
  param(
    [Parameter(Mandatory = $true)][string]$RelativePath,
    [Parameter(Mandatory = $true)][string]$Content,
    [string]$Reason = "",
    [string]$Source = ""
  )

  if ($Content.IndexOf([char]0) -ge 0) {
    Fail-RemoteInstall "Workflow content contains a null byte: $RelativePath"
  }
  if ([string]::IsNullOrWhiteSpace($Content)) {
    Fail-RemoteInstall "Workflow content is empty: $RelativePath"
  }

  return [pscustomobject]@{
    relative_path = ConvertTo-SafeWorkflowPath $RelativePath
    reason = $Reason
    source = $Source
    content = $Content
  }
}

function Get-PackInfoFromPackPath {
  param([Parameter(Mandatory = $true)][string]$Path)

  if (-not (Test-Path -LiteralPath $Path)) {
    Fail-RemoteInstall "Pack JSON not found: $Path"
  }

  $pack = Get-Content -Raw -LiteralPath $Path | ConvertFrom-Json
  $manifest = Get-ObjectPropertyValue -Object $pack -Name "manifest"
  if ($null -ne $manifest) {
    Assert-PackSafety (Get-ObjectPropertyValue -Object $manifest -Name "safety")
  }

  $files = Get-ObjectPropertyValue -Object $pack -Name "files"
  if ($null -eq $files) {
    Fail-RemoteInstall "Pack JSON must contain a files array."
  }

  $templates = New-Object System.Collections.Generic.List[object]
  foreach ($file in @($files)) {
    $filePath = Get-ObjectPropertyValue -Object $file -Name "file"
    if ($null -eq $filePath) {
      $filePath = Get-ObjectPropertyValue -Object $file -Name "path"
    }
    $content = Get-ObjectPropertyValue -Object $file -Name "content"
    $reason = Get-ObjectPropertyValue -Object $file -Name "reason"
    if ($null -eq $filePath) {
      Fail-RemoteInstall "Every pack file must contain file or path."
    }
    if ($null -eq $content) {
      Fail-RemoteInstall "Every pack file must contain content: $filePath"
    }

    $templates.Add((New-WorkflowTemplateFile -RelativePath ([string]$filePath) -Content ([string]$content) -Reason ([string]$reason) -Source $Path)) | Out-Null
  }

  if ($templates.Count -eq 0) {
    Fail-RemoteInstall "Pack JSON contains no workflow files."
  }

  return [pscustomobject]@{
    manifest = $manifest
    templates = @($templates.ToArray())
    source_type = "PackPath"
  }
}

function Get-PackInfoFromPackDir {
  param([Parameter(Mandatory = $true)][string]$Path)

  $root = Resolve-ExistingDirectory $Path
  $manifestPath = Join-Path $root "workflow-template-manifest.json"
  $manifest = $null
  if (Test-Path -LiteralPath $manifestPath) {
    $manifest = Get-Content -Raw -LiteralPath $manifestPath | ConvertFrom-Json
    Assert-PackSafety (Get-ObjectPropertyValue -Object $manifest -Name "safety")
  }

  $reasonByFile = @{}
  if ($null -ne $manifest) {
    $workflowTemplates = Get-ObjectPropertyValue -Object $manifest -Name "workflow_templates"
    if ($null -ne $workflowTemplates) {
      foreach ($template in @($workflowTemplates)) {
        $filePath = Get-ObjectPropertyValue -Object $template -Name "file"
        $reason = Get-ObjectPropertyValue -Object $template -Name "reason"
        if ($null -ne $filePath) {
          $reasonByFile[(ConvertTo-SafeWorkflowPath ([string]$filePath))] = [string]$reason
        }
      }
    }
  }

  $workflowDir = Join-Path $root ".github\workflows"
  if (-not (Test-Path -LiteralPath $workflowDir)) {
    Fail-RemoteInstall "Pack directory must contain .github/workflows."
  }

  $templates = New-Object System.Collections.Generic.List[object]
  $workflowFiles = @(
    Get-ChildItem -LiteralPath $workflowDir -File |
      Where-Object { $_.Extension -in @(".yml", ".yaml") } |
      Sort-Object Name
  )

  foreach ($workflowFile in $workflowFiles) {
    $relativePath = ".github/workflows/$($workflowFile.Name)"
    $safeRelativePath = ConvertTo-SafeWorkflowPath $relativePath
    $reason = if ($reasonByFile.ContainsKey($safeRelativePath)) { $reasonByFile[$safeRelativePath] } else { "" }
    $content = Get-Content -Raw -LiteralPath $workflowFile.FullName
    $templates.Add((New-WorkflowTemplateFile -RelativePath $safeRelativePath -Content $content -Reason $reason -Source $root)) | Out-Null
  }

  if ($templates.Count -eq 0) {
    Fail-RemoteInstall "Pack directory contains no .yml or .yaml workflow files."
  }

  return [pscustomobject]@{
    manifest = $manifest
    templates = @($templates.ToArray())
    source_type = "PackDir"
  }
}

function Get-GitHubTokenValue {
  if (-not [string]::IsNullOrWhiteSpace($GitHubToken)) {
    return $GitHubToken
  }
  if (-not [string]::IsNullOrWhiteSpace($env:GH_TOKEN)) {
    return $env:GH_TOKEN
  }

  try {
    $token = & $GitHubCliPath auth token 2>$null
    if ($LASTEXITCODE -eq 0 -and -not [string]::IsNullOrWhiteSpace($token)) {
      return [string]$token
    }
  } catch {
    # Fall through to the explicit failure below.
  }

  Fail-RemoteInstall "Missing GitHub token. Set GITHUB_TOKEN/GH_TOKEN, pass -GitHubToken, or authenticate gh."
}

function Invoke-GitHubApi {
  param(
    [Parameter(Mandatory = $true)][string]$Method,
    [Parameter(Mandatory = $true)][string]$Path,
    $Body = $null,
    [switch]$AllowNotFound
  )

  $headers = @{
    Authorization = "Bearer $script:ResolvedGitHubToken"
    Accept = "application/vnd.github+json"
    "X-GitHub-Api-Version" = "2022-11-28"
  }

  $uri = "https://api.github.com$Path"
  try {
    if ($null -eq $Body) {
      return Invoke-RestMethod -Method $Method -Uri $uri -Headers $headers
    }

    return Invoke-RestMethod -Method $Method -Uri $uri -Headers $headers -ContentType "application/json" -Body ($Body | ConvertTo-Json -Depth 20)
  } catch {
    $statusCode = $null
    if ($_.Exception.Response) {
      $statusCode = [int]$_.Exception.Response.StatusCode
    }
    if ($AllowNotFound -and $statusCode -eq 404) {
      return $null
    }
    $statusText = if ($null -eq $statusCode) { "unknown status" } else { "HTTP $statusCode" }
    Fail-RemoteInstall "GitHub API request failed: $Method $Path ($statusText)."
  }
}

function ConvertTo-GitHubContentPath {
  param([Parameter(Mandatory = $true)][string]$RelativePath)
  return (($RelativePath -split "/") | ForEach-Object { [System.Uri]::EscapeDataString($_) }) -join "/"
}

function Get-ContentSha256 {
  param([Parameter(Mandatory = $true)][string]$Value)
  $sha = [System.Security.Cryptography.SHA256]::Create()
  try {
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($Value)
    return ([System.BitConverter]::ToString($sha.ComputeHash($bytes)) -replace "-", "").ToLowerInvariant()
  } finally {
    $sha.Dispose()
  }
}

function Write-JsonFile {
  param(
    [Parameter(Mandatory = $true)]$Value,
    [Parameter(Mandatory = $true)][string]$Path
  )

  $parent = Split-Path -Parent $Path
  if (-not [string]::IsNullOrWhiteSpace($parent)) {
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
  }
  $Value | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $Path -Encoding UTF8
}

$packInfo = if ($PSCmdlet.ParameterSetName -eq "PackPath") {
  Get-PackInfoFromPackPath -Path $PackPath
} else {
  Get-PackInfoFromPackDir -Path $PackDir
}

$manifest = $packInfo.manifest
if ([string]::IsNullOrWhiteSpace($Repository) -and $null -ne $manifest) {
  $manifestRepository = Get-ObjectPropertyValue -Object $manifest -Name "repository_full_name"
  if ($null -ne $manifestRepository) {
    $Repository = [string]$manifestRepository
  }
}
if ([string]::IsNullOrWhiteSpace($BaseBranch) -and $null -ne $manifest) {
  $manifestBranch = Get-ObjectPropertyValue -Object $manifest -Name "default_branch"
  if ($null -ne $manifestBranch) {
    $BaseBranch = [string]$manifestBranch
  }
}

if ([string]::IsNullOrWhiteSpace($Repository)) {
  Fail-RemoteInstall "Missing -Repository and pack manifest repository_full_name."
}
if ($Repository -notmatch '^[^/\s]+/[^/\s]+$') {
  Fail-RemoteInstall "-Repository must look like owner/repo."
}
if ([string]::IsNullOrWhiteSpace($BaseBranch)) {
  $BaseBranch = "main"
}

$ticketPrefix = if ([string]::IsNullOrWhiteSpace($TicketId)) { "" } else { "$($TicketId.Trim()) " }
$timestamp = (Get-Date).ToUniversalTime().ToString("yyyyMMddHHmmss")
if ([string]::IsNullOrWhiteSpace($BranchName)) {
  $branchTicket = if ([string]::IsNullOrWhiteSpace($TicketId)) { "" } else { "$($TicketId.Trim())-" }
  $BranchName = "gitgov/$($branchTicket)workflow-template-install-$timestamp"
}
$BranchName = ConvertTo-SafeBranchName $BranchName

if ([string]::IsNullOrWhiteSpace($PullRequestTitle)) {
  $PullRequestTitle = "$ticketPrefix" + "Install GitGov workflow templates"
}
if ([string]::IsNullOrWhiteSpace($CommitMessage)) {
  $CommitMessage = "$ticketPrefix" + "install GitGov workflow templates"
}
if ([string]::IsNullOrWhiteSpace($PullRequestBody)) {
  $PullRequestBody = @"
## Summary
- Install reviewed GitGov workflow templates under `.github/workflows`.
- Keep generated workflow values secret-safe; variable and secret values must be configured separately.
- Review this PR before enabling schedules or treating any workflow as blocking.

## Safety
- Generated by GitGov remote workflow installation.
- Writes only workflow YAML files under `.github/workflows`.
- Does not include provider secret values.
"@
}

$templates = @($packInfo.templates)
$seen = @{}
foreach ($template in $templates) {
  if ($seen.ContainsKey($template.relative_path)) {
    Fail-RemoteInstall "Duplicate workflow template path: $($template.relative_path)"
  }
  $seen[$template.relative_path] = $true
}

$script:ResolvedGitHubToken = Get-GitHubTokenValue
$baseRef = Invoke-GitHubApi -Method GET -Path "/repos/$Repository/git/ref/heads/$BaseBranch"
if ($null -eq $baseRef -or $null -eq $baseRef.object -or [string]::IsNullOrWhiteSpace([string]$baseRef.object.sha)) {
  Fail-RemoteInstall "Could not resolve base branch '$BaseBranch' in '$Repository'."
}
$baseSha = [string]$baseRef.object.sha

$branchRef = Invoke-GitHubApi -Method GET -Path "/repos/$Repository/git/ref/heads/$BranchName" -AllowNotFound
if ($null -ne $branchRef -and $Apply) {
  Fail-RemoteInstall "Remote branch already exists: $BranchName. Choose a new -BranchName."
}

$planFiles = New-Object System.Collections.Generic.List[object]
foreach ($template in $templates) {
  $contentPath = ConvertTo-GitHubContentPath $template.relative_path
  $existing = Invoke-GitHubApi -Method GET -Path "/repos/$Repository/contents/$contentPath`?ref=$([System.Uri]::EscapeDataString($BaseBranch))" -AllowNotFound
  $status = "create"
  $remoteSha = $null

  if ($null -ne $existing) {
    $remoteSha = [string]$existing.sha
    $remoteContent = [System.Text.Encoding]::UTF8.GetString([System.Convert]::FromBase64String(([string]$existing.content -replace '\s', '')))
    if ($remoteContent -eq $template.content) {
      $status = "skip"
    } elseif ($Overwrite) {
      $status = "update"
    } else {
      $status = "blocked"
    }
  }

  $planFiles.Add([pscustomobject]@{
      file = $template.relative_path
      status = $status
      reason = $template.reason
      source = $template.source
      remote_sha = $remoteSha
      content_sha256 = Get-ContentSha256 $template.content
    }) | Out-Null
}

$totals = [ordered]@{
  create = @($planFiles | Where-Object { $_.status -eq "create" }).Count
  update = @($planFiles | Where-Object { $_.status -eq "update" }).Count
  skip = @($planFiles | Where-Object { $_.status -eq "skip" }).Count
  blocked = @($planFiles | Where-Object { $_.status -eq "blocked" }).Count
}

$plan = [pscustomobject]@{
  generated_at = (Get-Date).ToUniversalTime().ToString("o")
  mode = if ($Apply) { "apply" } else { "dry-run" }
  source_type = $packInfo.source_type
  repository = $Repository
  base_branch = $BaseBranch
  base_sha = $baseSha
  branch_name = $BranchName
  overwrite = [bool]$Overwrite
  pull_request_title = $PullRequestTitle
  pull_request_url = $null
  commit_sha = $null
  safety = @{
    writes_only_under_github_workflows = $true
    reads_secret_values = $false
    prints_secret_values = $false
    remote_mutation_requires_apply = $true
    creates_pull_request = [bool]$Apply
    draft_by_default = -not [bool]$ReadyForReview
  }
  totals = $totals
  files = @($planFiles.ToArray())
}

Write-Host ("Remote workflow PR plan: create={0}; update={1}; skip={2}; blocked={3}" -f $totals.create, $totals.update, $totals.skip, $totals.blocked)
Write-Host ("Mode: {0}" -f $plan.mode)

if ($totals.blocked -gt 0) {
  Write-Warning "One or more remote workflow files already exist and differ. Re-run with -Overwrite only after review."
  if ($Apply) {
    Fail-RemoteInstall "Apply refused because blocked files are present."
  }
}

if ($Apply) {
  $changeFiles = @($planFiles | Where-Object { $_.status -in @("create", "update") })
  if ($changeFiles.Count -eq 0) {
    Fail-RemoteInstall "No remote workflow changes to apply."
  }

  $baseCommit = Invoke-GitHubApi -Method GET -Path "/repos/$Repository/git/commits/$baseSha"
  $baseTreeSha = [string]$baseCommit.tree.sha
  if ([string]::IsNullOrWhiteSpace($baseTreeSha)) {
    Fail-RemoteInstall "Could not resolve base tree for '$BaseBranch'."
  }

  $treeEntries = New-Object System.Collections.Generic.List[object]
  foreach ($template in $templates) {
    $planned = @($planFiles | Where-Object { $_.file -eq $template.relative_path } | Select-Object -First 1)
    if ($planned.Count -eq 0 -or $planned[0].status -notin @("create", "update")) {
      continue
    }
    $treeEntries.Add([pscustomobject]@{
        path = $template.relative_path
        mode = "100644"
        type = "blob"
        content = $template.content
      }) | Out-Null
  }

  $tree = Invoke-GitHubApi -Method POST -Path "/repos/$Repository/git/trees" -Body @{
    base_tree = $baseTreeSha
    tree = @($treeEntries.ToArray())
  }
  $commit = Invoke-GitHubApi -Method POST -Path "/repos/$Repository/git/commits" -Body @{
    message = $CommitMessage
    tree = [string]$tree.sha
    parents = @($baseSha)
  }
  $newCommitSha = [string]$commit.sha

  Invoke-GitHubApi -Method POST -Path "/repos/$Repository/git/refs" -Body @{
    ref = "refs/heads/$BranchName"
    sha = $newCommitSha
  } | Out-Null

  $pullRequest = Invoke-GitHubApi -Method POST -Path "/repos/$Repository/pulls" -Body @{
    title = $PullRequestTitle
    head = $BranchName
    base = $BaseBranch
    body = $PullRequestBody
    draft = -not [bool]$ReadyForReview
  }

  $plan.commit_sha = $newCommitSha
  $plan.pull_request_url = [string]$pullRequest.html_url
  Write-Host "Created remote workflow installation PR: $($plan.pull_request_url)"
} else {
  Write-Host "Dry run only. Re-run with -Apply to create a branch, commit, and pull request."
}

if (-not [string]::IsNullOrWhiteSpace($OutputPlanPath)) {
  Write-JsonFile -Value $plan -Path $OutputPlanPath
  Write-Host "Wrote remote workflow PR plan: $OutputPlanPath"
}
