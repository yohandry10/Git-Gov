param(
  [string]$ReadinessPath = "out/enterprise-onboarding-readiness/enterprise-onboarding-readiness.json",
  [string]$AdoptionPackPath = "",
  [string]$OutputDir = "out/enterprise-onboarding-remediation-plan",
  [switch]$FailOnBlocked
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")

function Fail-RemediationPlan {
  param([Parameter(Mandatory = $true)][string]$Message)
  Write-Host "[FAIL] $Message"
  exit 1
}

function Resolve-RepoPath {
  param([Parameter(Mandatory = $true)][string]$Path)

  if ([System.IO.Path]::IsPathRooted($Path)) {
    return $Path
  }
  return Join-Path $repoRoot $Path
}

function Read-JsonFile {
  param([Parameter(Mandatory = $true)][string]$Path)

  $resolved = Resolve-RepoPath $Path
  if (-not (Test-Path -LiteralPath $resolved)) {
    Fail-RemediationPlan "JSON file not found: $Path"
  }
  return Get-Content -Raw -LiteralPath $resolved | ConvertFrom-Json
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

function Get-ObjectPropertyValue {
  param(
    [Parameter(Mandatory = $true)]$Object,
    [Parameter(Mandatory = $true)][string]$Name
  )

  if ($null -eq $Object) {
    return $null
  }
  $property = $Object.PSObject.Properties[$Name]
  if ($null -eq $property) {
    return $null
  }
  return $property.Value
}

function ConvertTo-Array {
  param($Value)

  if ($null -eq $Value) {
    return @()
  }
  return @($Value)
}

function Escape-MarkdownCell {
  param([string]$Value)
  if ([string]::IsNullOrEmpty($Value)) {
    return ""
  }
  return ($Value -replace '\|', '\|')
}

function Get-ActionPriority {
  param(
    [Parameter(Mandatory = $true)][string]$StageId,
    [Parameter(Mandatory = $true)][string]$Status
  )

  if ($Status -eq "blocked") {
    return 0
  }

  switch ($StageId) {
    "profile" { return 1 }
    "providers" { return 2 }
    "workflow-pack" { return 3 }
    "remote-workflows" { return 4 }
    "actions-config" { return 5 }
    "release-governance" { return 6 }
    default { return 50 }
  }
}

function Get-ActionOwner {
  param([Parameter(Mandatory = $true)][string]$StageId)

  switch ($StageId) {
    "profile" { return "GitGov admin" }
    "providers" { return "Platform owner" }
    "workflow-pack" { return "DevOps owner" }
    "remote-workflows" { return "Repository admin" }
    "actions-config" { return "Repository admin" }
    "release-governance" { return "Release governance owner" }
    default { return "GitGov operator" }
  }
}

function Get-ValidationEvidence {
  param([Parameter(Mandatory = $true)][string]$StageId)

  switch ($StageId) {
    "profile" { return "Regenerate onboarding readiness and confirm the profile stage is ready." }
    "providers" { return "Attach a sanitized provider connection report with ready provider checks." }
    "workflow-pack" { return "Regenerate the workflow template pack and review the manifest." }
    "remote-workflows" { return "Run remote workflow readiness validation after install or remote PR merge." }
    "actions-config" { return "Re-run workflow readiness and confirm required variable and secret names are present." }
    "release-governance" { return "Run the release governance evaluator or confirm record-only policy remains intentional." }
    default { return "Regenerate onboarding readiness and confirm the stage is ready." }
  }
}

function New-ActionItem {
  param(
    [Parameter(Mandatory = $true)]$Stage
  )

  $stageId = [string](Get-ObjectPropertyValue $Stage "id")
  $status = [string](Get-ObjectPropertyValue $Stage "status")
  $label = [string](Get-ObjectPropertyValue $Stage "label")
  $summary = [string](Get-ObjectPropertyValue $Stage "summary")
  $nextAction = [string](Get-ObjectPropertyValue $Stage "next_action")

  [pscustomobject]@{
    priority = Get-ActionPriority -StageId $stageId -Status $status
    stage_id = $stageId
    stage = $label
    status = $status
    owner = Get-ActionOwner -StageId $stageId
    action = $nextAction
    reason = $summary
    validation = Get-ValidationEvidence -StageId $stageId
  }
}

function New-ConfigCommand {
  param(
    [Parameter(Mandatory = $true)][string]$Kind,
    [Parameter(Mandatory = $true)][string]$Name,
    [Parameter(Mandatory = $true)][string]$RepositoryFullName
  )

  $command = if ($Kind -eq "variable") {
    'gh variable set {0} --repo {1} --body "<value>"' -f $Name, $RepositoryFullName
  } else {
    'gh secret set {0} --repo {1}' -f $Name, $RepositoryFullName
  }

  [pscustomobject]@{
    kind = $Kind
    name = $Name
    command = $command
    contains_secret_value = $false
  }
}

if ([string]::IsNullOrWhiteSpace($ReadinessPath)) {
  Fail-RemediationPlan "Missing -ReadinessPath."
}

$readiness = Read-JsonFile $ReadinessPath
$sourceFiles = Get-ObjectPropertyValue $readiness "source_files"
$packPathFromReadiness = [string](Get-ObjectPropertyValue $sourceFiles "adoption_pack")
if ([string]::IsNullOrWhiteSpace($AdoptionPackPath) -and -not [string]::IsNullOrWhiteSpace($packPathFromReadiness)) {
  $AdoptionPackPath = $packPathFromReadiness
}

$pack = $null
if (-not [string]::IsNullOrWhiteSpace($AdoptionPackPath)) {
  $resolvedPackPath = Resolve-RepoPath $AdoptionPackPath
  if (Test-Path -LiteralPath $resolvedPackPath) {
    $pack = Get-Content -Raw -LiteralPath $resolvedPackPath | ConvertFrom-Json
  }
}

$customerName = [string](Get-ObjectPropertyValue $readiness "customer_name")
$repositoryFullName = [string](Get-ObjectPropertyValue $readiness "repository_full_name")
$defaultBranch = [string](Get-ObjectPropertyValue $readiness "default_branch")
$status = [string](Get-ObjectPropertyValue $readiness "status")
$readinessScore = Get-ObjectPropertyValue $readiness "readiness_score"
$policyPreset = [string](Get-ObjectPropertyValue $readiness "policy_preset")
$stages = @(ConvertTo-Array (Get-ObjectPropertyValue $readiness "stages"))

if ([string]::IsNullOrWhiteSpace($repositoryFullName)) {
  Fail-RemediationPlan "Readiness JSON is missing repository_full_name."
}

$actionItems = New-Object System.Collections.Generic.List[object]
foreach ($stage in $stages) {
  $stageStatus = [string](Get-ObjectPropertyValue $stage "status")
  if ($stageStatus -ne "ready") {
    $actionItems.Add((New-ActionItem -Stage $stage)) | Out-Null
  }
}
$orderedActionItems = @($actionItems.ToArray() | Sort-Object -Property priority, stage_id)

$variables = @()
$secrets = @()
if ($null -ne $pack) {
  $variables = @(ConvertTo-Array (Get-ObjectPropertyValue $pack "variables"))
  $secrets = @(ConvertTo-Array (Get-ObjectPropertyValue $pack "secrets"))
}

$configurationCommands = New-Object System.Collections.Generic.List[object]
foreach ($variable in $variables) {
  $name = [string](Get-ObjectPropertyValue $variable "name")
  if (-not [string]::IsNullOrWhiteSpace($name)) {
    $configurationCommands.Add((New-ConfigCommand -Kind "variable" -Name $name -RepositoryFullName $repositoryFullName)) | Out-Null
  }
}
foreach ($secret in $secrets) {
  $name = [string](Get-ObjectPropertyValue $secret "name")
  if (-not [string]::IsNullOrWhiteSpace($name)) {
    $configurationCommands.Add((New-ConfigCommand -Kind "secret" -Name $name -RepositoryFullName $repositoryFullName)) | Out-Null
  }
}

$generatedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
$planStatus = if ($status -eq "ready") {
  "ready"
} elseif (@($orderedActionItems | Where-Object { $_.status -eq "blocked" }).Count -gt 0) {
  "blocked"
} else {
  "needs-action"
}

$plan = [ordered]@{
  generated_at = $generatedAtUtc
  customer_name = $customerName
  repository_full_name = $repositoryFullName
  default_branch = $defaultBranch
  policy_preset = $policyPreset
  readiness_status = $status
  readiness_score = $readinessScore
  remediation_status = $planStatus
  action_count = $orderedActionItems.Count
  actions = @($orderedActionItems)
  github_actions_configuration = [ordered]@{
    source = if ($null -ne $pack) { $AdoptionPackPath } else { $null }
    variables_count = $variables.Count
    secrets_count = $secrets.Count
    commands_are_placeholders = $true
    commands = @($configurationCommands.ToArray())
  }
  validation = [ordered]@{
    regenerate_readiness = "Run scripts/control-plane/generate_enterprise_onboarding_readiness_report.ps1 after completing actions."
    rerun_provider_checks = "Run scripts/control-plane/validate_enterprise_provider_connections.ps1 only with customer-approved credentials."
    rerun_workflow_readiness = "Run scripts/control-plane/validate_enterprise_workflow_installation_readiness.ps1 after workflow installation or remote PR merge."
  }
  safety = [ordered]@{
    contains_secret_values = $false
    reads_secret_values = $false
    mutates_customer_repository = $false
    mutates_provider_state = $false
    creates_github_actions_variables = $false
    creates_github_actions_secrets = $false
    release_blocking_default = $false
  }
}

$resolvedOutputDir = Resolve-RepoPath $OutputDir
New-Item -ItemType Directory -Force -Path $resolvedOutputDir | Out-Null
$jsonPath = Join-Path $resolvedOutputDir "enterprise-onboarding-remediation-plan.json"
$markdownPath = Join-Path $resolvedOutputDir "enterprise-onboarding-remediation-plan.md"
Write-JsonFile -Value $plan -Path $jsonPath

$markdown = New-Object System.Collections.Generic.List[string]
$markdown.Add("# GitGov Enterprise Onboarding Remediation Plan") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add(('Generated: `{0}`' -f $generatedAtUtc)) | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add(('Customer: `{0}`' -f $customerName)) | Out-Null
$markdown.Add(('Repository: `{0}`' -f $repositoryFullName)) | Out-Null
$markdown.Add(('Default branch: `{0}`' -f $defaultBranch)) | Out-Null
$markdown.Add(('Policy preset: `{0}`' -f $policyPreset)) | Out-Null
$markdown.Add(('Readiness status: `{0}`' -f $status)) | Out-Null
$markdown.Add(('Readiness score: `{0}`' -f $readinessScore)) | Out-Null
$markdown.Add(('Remediation status: `{0}`' -f $planStatus)) | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("## Priority Actions") | Out-Null
$markdown.Add("") | Out-Null
if ($orderedActionItems.Count -eq 0) {
  $markdown.Add("- None. Onboarding readiness is already `ready`.") | Out-Null
} else {
  $markdown.Add("| Priority | Stage | Status | Owner | Action | Validation |") | Out-Null
  $markdown.Add("|---:|---|---|---|---|---|") | Out-Null
  foreach ($item in $orderedActionItems) {
    $markdown.Add(('| {0} | {1} | `{2}` | {3} | {4} | {5} |' -f $item.priority, (Escape-MarkdownCell $item.stage), $item.status, (Escape-MarkdownCell $item.owner), (Escape-MarkdownCell $item.action), (Escape-MarkdownCell $item.validation))) | Out-Null
  }
}

$markdown.Add("") | Out-Null
$markdown.Add("## GitHub Actions Configuration") | Out-Null
$markdown.Add("") | Out-Null
if ($configurationCommands.Count -eq 0) {
  $markdown.Add("- No GitHub Actions variable or secret names were available from an adoption pack.") | Out-Null
} else {
  $markdown.Add("These commands are placeholders. Operators must provide real values outside GitGov.") | Out-Null
  $markdown.Add("") | Out-Null
  $markdown.Add("| Kind | Name | Command |") | Out-Null
  $markdown.Add("|---|---|---|") | Out-Null
  foreach ($command in $configurationCommands) {
    $markdown.Add(('| `{0}` | `{1}` | `{2}` |' -f $command.kind, (Escape-MarkdownCell $command.name), (Escape-MarkdownCell $command.command))) | Out-Null
  }
}

$markdown.Add("") | Out-Null
$markdown.Add("## Validation Loop") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("- Re-run provider validation only with customer-approved credentials.") | Out-Null
$markdown.Add("- Re-run workflow readiness after workflow installation or remote PR merge.") | Out-Null
$markdown.Add("- Regenerate onboarding readiness and confirm the readiness score improves.") | Out-Null
$markdown.Add("- Re-run trend and monitor workflows after recurring readiness evidence exists.") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("## Safety") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("- This plan contains names, statuses, placeholder commands, and next actions only.") | Out-Null
$markdown.Add("- It does not read or print secret values.") | Out-Null
$markdown.Add("- It does not mutate customer repositories, providers, branch protection, workflows, variables, or secrets.") | Out-Null
$markdown.Add("- Release blocking remains customer opt-in only; record-only is the safe default.") | Out-Null

Set-Content -LiteralPath $markdownPath -Value $markdown -Encoding UTF8

Write-Host ("Enterprise onboarding remediation plan: status={0}; actions={1}; variables={2}; secrets={3}" -f $planStatus, $orderedActionItems.Count, $variables.Count, $secrets.Count)
Write-Host "Wrote remediation plan Markdown: $markdownPath"
Write-Host "Wrote remediation plan JSON: $jsonPath"

if ($FailOnBlocked -and $planStatus -eq "blocked") {
  Fail-RemediationPlan "Enterprise onboarding remediation plan contains blocked actions."
}
