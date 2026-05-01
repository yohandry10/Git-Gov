param(
  [Parameter(Mandatory = $true, ParameterSetName = "PackPath")]
  [string]$PackPath,

  [Parameter(Mandatory = $true, ParameterSetName = "PackDir")]
  [string]$PackDir,

  [string]$Repository,
  [string]$Ref,
  [string]$OutputPath = "",
  [string]$GitHubToken = $env:GITHUB_TOKEN,
  [string]$GitHubCliPath = "gh",
  [switch]$ReportOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Fail-Readiness {
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
    Fail-Readiness "Directory not found: $Path"
  }

  $item = Get-Item -LiteralPath $Path
  if (-not $item.PSIsContainer) {
    Fail-Readiness "Path is not a directory: $Path"
  }

  return $item.FullName
}

function ConvertTo-SafeWorkflowPath {
  param([Parameter(Mandatory = $true)][string]$Path)

  if ([string]::IsNullOrWhiteSpace($Path)) {
    Fail-Readiness "Workflow file path is empty."
  }
  if ($Path.IndexOf([char]0) -ge 0) {
    Fail-Readiness "Workflow file path contains a null byte."
  }

  $normalized = $Path.Trim().Replace("\", "/")
  while ($normalized.StartsWith("./")) {
    $normalized = $normalized.Substring(2)
  }

  if ([System.IO.Path]::IsPathRooted($Path) -or $normalized -match '^[A-Za-z]:') {
    Fail-Readiness "Workflow file path must be relative: $Path"
  }
  if ($normalized -match '(^|/)\.\.($|/)') {
    Fail-Readiness "Workflow file path must not contain parent directory segments: $Path"
  }
  if ($normalized -notmatch '^\.github/workflows/[A-Za-z0-9._-]+\.ya?ml$') {
    Fail-Readiness "Workflow file path must be a .yml or .yaml file directly under .github/workflows: $Path"
  }

  return $normalized
}

function Assert-PackSafety {
  param($Safety)

  if ($null -eq $Safety) {
    return
  }

  $containsSecretValues = Get-ObjectPropertyValue -Object $Safety -Name "contains_secret_values"
  if ($containsSecretValues -eq $true) {
    Fail-Readiness "Pack declares that it contains secret values. Refusing readiness validation."
  }
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

function ConvertTo-GitHubContentPath {
  param([Parameter(Mandatory = $true)][string]$RelativePath)
  return (($RelativePath -split "/") | ForEach-Object { [System.Uri]::EscapeDataString($_) }) -join "/"
}

function New-WorkflowTemplate {
  param(
    [Parameter(Mandatory = $true)][string]$RelativePath,
    [string]$Content = "",
    [string]$Reason = "",
    [string]$Source = ""
  )

  if (-not [string]::IsNullOrEmpty($Content) -and $Content.IndexOf([char]0) -ge 0) {
    Fail-Readiness "Workflow content contains a null byte: $RelativePath"
  }

  return [pscustomobject]@{
    relative_path = ConvertTo-SafeWorkflowPath $RelativePath
    reason = $Reason
    source = $Source
    content = $Content
    expected_sha256 = if ([string]::IsNullOrEmpty($Content)) { $null } else { Get-ContentSha256 $Content }
  }
}

function Add-UniqueByName {
  param(
    [System.Collections.Generic.List[object]]$List,
    [Parameter(Mandatory = $true)]$Item,
    [Parameter(Mandatory = $true)][string]$NameProperty
  )

  $name = [string](Get-ObjectPropertyValue -Object $Item -Name $NameProperty)
  if ([string]::IsNullOrWhiteSpace($name)) {
    return
  }

  $existing = @($List | Where-Object { [string](Get-ObjectPropertyValue -Object $_ -Name $NameProperty) -eq $name })
  if ($existing.Count -eq 0) {
    $List.Add($Item) | Out-Null
  }
}

function Get-PackInfoFromPackPath {
  param([Parameter(Mandatory = $true)][string]$Path)

  if (-not (Test-Path -LiteralPath $Path)) {
    Fail-Readiness "Pack JSON not found: $Path"
  }

  $pack = Get-Content -Raw -LiteralPath $Path | ConvertFrom-Json
  $manifest = Get-ObjectPropertyValue -Object $pack -Name "manifest"
  if ($null -ne $manifest) {
    Assert-PackSafety (Get-ObjectPropertyValue -Object $manifest -Name "safety")
  }

  $files = Get-ObjectPropertyValue -Object $pack -Name "files"
  if ($null -eq $files) {
    Fail-Readiness "Pack JSON must contain a files array."
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
      Fail-Readiness "Every pack file must contain file or path."
    }
    if ($null -eq $content) {
      Fail-Readiness "Every pack file must contain content: $filePath"
    }

    $templates.Add((New-WorkflowTemplate -RelativePath ([string]$filePath) -Content ([string]$content) -Reason ([string]$reason) -Source $Path)) | Out-Null
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
    Fail-Readiness "Pack directory must contain .github/workflows."
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
    $templates.Add((New-WorkflowTemplate -RelativePath $safeRelativePath -Content $content -Reason $reason -Source $root)) | Out-Null
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
    # Fall through to explicit failure.
  }

  Fail-Readiness "Missing GitHub token. Set GITHUB_TOKEN/GH_TOKEN, pass -GitHubToken, or authenticate gh."
}

function Invoke-GitHubApi {
  param(
    [Parameter(Mandatory = $true)][string]$Method,
    [Parameter(Mandatory = $true)][string]$Path,
    [switch]$AllowNotFound
  )

  $headers = @{
    Authorization = "Bearer $script:ResolvedGitHubToken"
    Accept = "application/vnd.github+json"
    "X-GitHub-Api-Version" = "2022-11-28"
  }

  try {
    return Invoke-RestMethod -Method $Method -Uri "https://api.github.com$Path" -Headers $headers
  } catch {
    $statusCode = $null
    if ($_.Exception.Response) {
      $statusCode = [int]$_.Exception.Response.StatusCode
    }
    if ($AllowNotFound -and $statusCode -eq 404) {
      return $null
    }
    $statusText = if ($null -eq $statusCode) { "unknown status" } else { "HTTP $statusCode" }
    Fail-Readiness "GitHub API request failed: $Method $Path ($statusText)."
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
if ([string]::IsNullOrWhiteSpace($Ref) -and $null -ne $manifest) {
  $manifestBranch = Get-ObjectPropertyValue -Object $manifest -Name "default_branch"
  if ($null -ne $manifestBranch) {
    $Ref = [string]$manifestBranch
  }
}
if ([string]::IsNullOrWhiteSpace($Ref)) {
  $Ref = "main"
}
if ([string]::IsNullOrWhiteSpace($Repository)) {
  Fail-Readiness "Missing -Repository and pack manifest repository_full_name."
}
if ($Repository -notmatch '^[^/\s]+/[^/\s]+$') {
  Fail-Readiness "-Repository must look like owner/repo."
}

$templates = @($packInfo.templates)
if ($templates.Count -eq 0) {
  Fail-Readiness "Pack contains no workflow templates."
}

$variables = New-Object System.Collections.Generic.List[object]
$secrets = New-Object System.Collections.Generic.List[object]
if ($null -ne $manifest) {
  $manifestVariables = Get-ObjectPropertyValue -Object $manifest -Name "variables"
  if ($null -ne $manifestVariables) {
    foreach ($variable in @($manifestVariables)) {
      Add-UniqueByName -List $variables -Item $variable -NameProperty "name"
    }
  }

  $manifestSecrets = Get-ObjectPropertyValue -Object $manifest -Name "secrets"
  if ($null -ne $manifestSecrets) {
    foreach ($secret in @($manifestSecrets)) {
      Add-UniqueByName -List $secrets -Item $secret -NameProperty "name"
    }
  }
}

$script:ResolvedGitHubToken = Get-GitHubTokenValue
$repo = Invoke-GitHubApi -Method GET -Path "/repos/$Repository"

$workflowResults = New-Object System.Collections.Generic.List[object]
foreach ($template in $templates) {
  $contentPath = ConvertTo-GitHubContentPath $template.relative_path
  $remote = Invoke-GitHubApi -Method GET -Path "/repos/$Repository/contents/$contentPath`?ref=$([System.Uri]::EscapeDataString($Ref))" -AllowNotFound

  $status = "missing"
  $remoteSha = $null
  $remoteContentSha256 = $null
  if ($null -ne $remote) {
    $remoteSha = [string]$remote.sha
    $remoteContent = [System.Text.Encoding]::UTF8.GetString([System.Convert]::FromBase64String(([string]$remote.content -replace '\s', '')))
    $remoteContentSha256 = Get-ContentSha256 $remoteContent
    if ($null -eq $template.expected_sha256) {
      $status = "present"
    } elseif ($remoteContentSha256 -eq $template.expected_sha256) {
      $status = "matched"
    } else {
      $status = "different"
    }
  }

  $workflowResults.Add([pscustomobject]@{
      file = $template.relative_path
      status = $status
      reason = $template.reason
      remote_sha = $remoteSha
      expected_sha256 = $template.expected_sha256
      remote_sha256 = $remoteContentSha256
    }) | Out-Null
}

$variableResults = New-Object System.Collections.Generic.List[object]
foreach ($variable in @($variables.ToArray())) {
  $name = [string](Get-ObjectPropertyValue -Object $variable -Name "name")
  $remoteVariable = Invoke-GitHubApi -Method GET -Path "/repos/$Repository/actions/variables/$([System.Uri]::EscapeDataString($name))" -AllowNotFound
  $variableResults.Add([pscustomobject]@{
      name = $name
      status = if ($null -eq $remoteVariable) { "missing" } else { "present" }
      purpose = [string](Get-ObjectPropertyValue -Object $variable -Name "purpose")
      scope = [string](Get-ObjectPropertyValue -Object $variable -Name "scope")
    }) | Out-Null
}

$secretResults = New-Object System.Collections.Generic.List[object]
foreach ($secret in @($secrets.ToArray())) {
  $name = [string](Get-ObjectPropertyValue -Object $secret -Name "name")
  $remoteSecret = Invoke-GitHubApi -Method GET -Path "/repos/$Repository/actions/secrets/$([System.Uri]::EscapeDataString($name))" -AllowNotFound
  $secretResults.Add([pscustomobject]@{
      name = $name
      status = if ($null -eq $remoteSecret) { "missing" } else { "present" }
      purpose = [string](Get-ObjectPropertyValue -Object $secret -Name "purpose")
      value_policy = [string](Get-ObjectPropertyValue -Object $secret -Name "value_policy")
    }) | Out-Null
}

$totals = [ordered]@{
  workflows_matched = @($workflowResults | Where-Object { $_.status -eq "matched" }).Count
  workflows_present = @($workflowResults | Where-Object { $_.status -in @("matched", "present", "different") }).Count
  workflows_different = @($workflowResults | Where-Object { $_.status -eq "different" }).Count
  workflows_missing = @($workflowResults | Where-Object { $_.status -eq "missing" }).Count
  variables_present = @($variableResults | Where-Object { $_.status -eq "present" }).Count
  variables_missing = @($variableResults | Where-Object { $_.status -eq "missing" }).Count
  secrets_present = @($secretResults | Where-Object { $_.status -eq "present" }).Count
  secrets_missing = @($secretResults | Where-Object { $_.status -eq "missing" }).Count
}

$ready = (
  $totals.workflows_missing -eq 0 -and
  $totals.workflows_different -eq 0 -and
  $totals.variables_missing -eq 0 -and
  $totals.secrets_missing -eq 0
)

$result = [pscustomobject]@{
  generated_at = (Get-Date).ToUniversalTime().ToString("o")
  status = if ($ready) { "ready" } else { "needs-action" }
  report_only = [bool]$ReportOnly
  repository = [string]$repo.full_name
  ref = $Ref
  source_type = $packInfo.source_type
  safety = @{
    reads_secret_values = $false
    prints_secret_values = $false
    mutates_repository = $false
    checks_secret_names_only = $true
  }
  totals = $totals
  workflows = @($workflowResults.ToArray())
  variables = @($variableResults.ToArray())
  secrets = @($secretResults.ToArray())
}

Write-Host ("Remote workflow readiness: status={0}; workflows_missing={1}; workflows_different={2}; variables_missing={3}; secrets_missing={4}" -f $result.status, $totals.workflows_missing, $totals.workflows_different, $totals.variables_missing, $totals.secrets_missing)

if (-not [string]::IsNullOrWhiteSpace($OutputPath)) {
  Write-JsonFile -Value $result -Path $OutputPath
  Write-Host "Wrote remote workflow readiness report: $OutputPath"
}

if ($result.status -ne "ready" -and -not $ReportOnly) {
  Fail-Readiness "Remote workflow installation is not ready. Re-run with -ReportOnly to collect a non-blocking report."
}

exit 0
