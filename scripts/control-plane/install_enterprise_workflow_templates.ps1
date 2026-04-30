param(
  [Parameter(Mandatory = $true, ParameterSetName = "PackPath")]
  [string]$PackPath,

  [Parameter(Mandatory = $true, ParameterSetName = "PackDir")]
  [string]$PackDir,

  [Parameter(Mandatory = $true)]
  [string]$TargetRepoPath,

  [string]$OutputPlanPath,

  [switch]$Apply,
  [switch]$Overwrite
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Fail-Install {
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
    Fail-Install "Directory not found: $Path"
  }

  $item = Get-Item -LiteralPath $Path
  if (-not $item.PSIsContainer) {
    Fail-Install "Path is not a directory: $Path"
  }

  return $item.FullName
}

function ConvertTo-SafeWorkflowPath {
  param([Parameter(Mandatory = $true)][string]$Path)

  if ([string]::IsNullOrWhiteSpace($Path)) {
    Fail-Install "Workflow file path is empty."
  }
  if ($Path.IndexOf([char]0) -ge 0) {
    Fail-Install "Workflow file path contains a null byte."
  }

  $normalized = $Path.Trim().Replace("\", "/")
  while ($normalized.StartsWith("./")) {
    $normalized = $normalized.Substring(2)
  }

  if ([System.IO.Path]::IsPathRooted($Path) -or $normalized -match '^[A-Za-z]:') {
    Fail-Install "Workflow file path must be relative: $Path"
  }
  if ($normalized -match '(^|/)\.\.($|/)') {
    Fail-Install "Workflow file path must not contain parent directory segments: $Path"
  }
  if ($normalized -notmatch '^\.github/workflows/[A-Za-z0-9._-]+\.ya?ml$') {
    Fail-Install "Workflow file path must be a .yml or .yaml file directly under .github/workflows: $Path"
  }

  return $normalized
}

function Join-WorkflowTargetPath {
  param(
    [Parameter(Mandatory = $true)][string]$TargetRoot,
    [Parameter(Mandatory = $true)][string]$RelativePath
  )

  $combined = $TargetRoot
  foreach ($segment in ($RelativePath -split "/")) {
    $combined = Join-Path $combined $segment
  }

  $targetFullPath = [System.IO.Path]::GetFullPath($combined)
  $workflowRoot = [System.IO.Path]::GetFullPath((Join-Path $TargetRoot ".github\workflows"))
  $workflowPrefix = $workflowRoot.TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar) + [System.IO.Path]::DirectorySeparatorChar
  if (-not $targetFullPath.StartsWith($workflowPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
    Fail-Install "Resolved workflow target escapes .github/workflows: $RelativePath"
  }

  return $targetFullPath
}

function Assert-PackSafety {
  param($Safety)

  if ($null -eq $Safety) {
    return
  }

  $containsSecretValues = Get-ObjectPropertyValue -Object $Safety -Name "contains_secret_values"
  if ($containsSecretValues -eq $true) {
    Fail-Install "Pack declares that it contains secret values. Refusing installation."
  }

  $mutatesRepository = Get-ObjectPropertyValue -Object $Safety -Name "mutates_customer_repository"
  if ($mutatesRepository -eq $true) {
    Fail-Install "Pack declares repository mutation behavior. This installer only accepts static workflow templates."
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
    Fail-Install "Workflow content contains a null byte: $RelativePath"
  }
  if ([string]::IsNullOrWhiteSpace($Content)) {
    Fail-Install "Workflow content is empty: $RelativePath"
  }

  return [pscustomobject]@{
    relative_path = ConvertTo-SafeWorkflowPath $RelativePath
    reason = $Reason
    source = $Source
    content = $Content
  }
}

function Get-TemplatesFromPackPath {
  param([Parameter(Mandatory = $true)][string]$Path)

  if (-not (Test-Path -LiteralPath $Path)) {
    Fail-Install "Pack JSON not found: $Path"
  }

  $pack = Get-Content -Raw -LiteralPath $Path | ConvertFrom-Json
  $manifest = Get-ObjectPropertyValue -Object $pack -Name "manifest"
  if ($null -ne $manifest) {
    Assert-PackSafety (Get-ObjectPropertyValue -Object $manifest -Name "safety")
  }

  $files = Get-ObjectPropertyValue -Object $pack -Name "files"
  if ($null -eq $files) {
    Fail-Install "Pack JSON must contain a files array."
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
      Fail-Install "Every pack file must contain file or path."
    }
    if ($null -eq $content) {
      Fail-Install "Every pack file must contain content: $filePath"
    }

    $templates.Add((New-WorkflowTemplateFile -RelativePath ([string]$filePath) -Content ([string]$content) -Reason ([string]$reason) -Source $Path)) | Out-Null
  }

  if ($templates.Count -eq 0) {
    Fail-Install "Pack JSON contains no workflow files."
  }

  return @($templates.ToArray())
}

function Get-TemplatesFromPackDir {
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
    Fail-Install "Pack directory must contain .github/workflows."
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
    Fail-Install "Pack directory contains no .yml or .yaml workflow files."
  }

  return @($templates.ToArray())
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
  $Value | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $Path -Encoding UTF8
}

$targetRoot = Resolve-ExistingDirectory $TargetRepoPath
$gitMarker = Join-Path $targetRoot ".git"
if (-not (Test-Path -LiteralPath $gitMarker)) {
  Fail-Install "Target path must be a git repository checkout with a .git marker: $TargetRepoPath"
}

$sourceType = $PSCmdlet.ParameterSetName
$templates = if ($sourceType -eq "PackPath") {
  Get-TemplatesFromPackPath -Path $PackPath
} else {
  Get-TemplatesFromPackDir -Path $PackDir
}

$seen = @{}
$planFiles = New-Object System.Collections.Generic.List[object]
foreach ($template in $templates) {
  $relativePath = $template.relative_path
  if ($seen.ContainsKey($relativePath)) {
    Fail-Install "Duplicate workflow template path: $relativePath"
  }
  $seen[$relativePath] = $true

  $targetPath = Join-WorkflowTargetPath -TargetRoot $targetRoot -RelativePath $relativePath
  $exists = Test-Path -LiteralPath $targetPath
  $status = "create"
  if ($exists) {
    $currentContent = Get-Content -Raw -LiteralPath $targetPath
    if ($currentContent -eq $template.content) {
      $status = "skip"
    } elseif ($Overwrite) {
      $status = "update"
    } else {
      $status = "blocked"
    }
  }

  $planFiles.Add([pscustomobject]@{
      file = $relativePath
      target_path = $targetPath
      status = $status
      reason = $template.reason
      source = $template.source
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
  source_type = $sourceType
  target_repo_path = $targetRoot
  overwrite = [bool]$Overwrite
  safety = @{
    writes_only_under_github_workflows = $true
    reads_secret_values = $false
    prints_secret_values = $false
    requires_apply_for_writes = $true
    requires_overwrite_for_replacements = $true
  }
  totals = $totals
  files = @($planFiles.ToArray())
}

if (-not [string]::IsNullOrWhiteSpace($OutputPlanPath)) {
  Write-JsonFile -Value $plan -Path $OutputPlanPath
  Write-Host "Wrote workflow install plan: $OutputPlanPath"
}

Write-Host ("Workflow install plan: create={0}; update={1}; skip={2}; blocked={3}" -f $totals.create, $totals.update, $totals.skip, $totals.blocked)
Write-Host ("Mode: {0}" -f $plan.mode)

if ($totals.blocked -gt 0) {
  Write-Warning "One or more target workflow files already exist and differ. Re-run with -Overwrite only after review."
  if ($Apply) {
    Fail-Install "Apply refused because blocked files are present."
  }
}

if ($Apply) {
  foreach ($template in $templates) {
    $relativePath = $template.relative_path
    $entry = @($planFiles | Where-Object { $_.file -eq $relativePath } | Select-Object -First 1)
    if ($entry.Count -eq 0 -or $entry[0].status -notin @("create", "update")) {
      continue
    }

    $targetPath = $entry[0].target_path
    $parent = Split-Path -Parent $targetPath
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
    Set-Content -LiteralPath $targetPath -Value $template.content -Encoding UTF8
  }
  Write-Host "Applied workflow templates to target repository."
} else {
  Write-Host "Dry run only. Re-run with -Apply to write workflow files."
}
