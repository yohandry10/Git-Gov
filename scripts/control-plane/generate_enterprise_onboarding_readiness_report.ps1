param(
  [string]$ProfilePath = "docs/examples/enterprise-adoption-profile.example.json",
  [string]$AdoptionPackPath = "",
  [string]$ProviderConnectionsPath = "",
  [string]$WorkflowReadinessPath = "",
  [string]$OutputDir = "out/enterprise-onboarding-readiness",
  [switch]$ReportOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")

function Fail-OnboardingReadiness {
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

function Get-Number {
  param($Value)

  if ($null -eq $Value) {
    return 0
  }
  return [Math]::Max(0, [int]$Value)
}

function ConvertTo-Array {
  param($Value)

  if ($null -eq $Value) {
    return @()
  }
  return @($Value)
}

function New-Stage {
  param(
    [Parameter(Mandatory = $true)][string]$Id,
    [Parameter(Mandatory = $true)][string]$Label,
    [ValidateSet("ready", "needs-action", "blocked")]
    [Parameter(Mandatory = $true)][string]$Status,
    [Parameter(Mandatory = $true)][string]$Summary,
    [Parameter(Mandatory = $true)][string]$NextAction
  )

  [pscustomobject]@{
    id = $Id
    label = $Label
    status = $Status
    summary = $Summary
    next_action = $NextAction
  }
}

function Get-StageWeight {
  param([Parameter(Mandatory = $true)][string]$Status)

  if ($Status -eq "ready") {
    return 1.0
  }
  if ($Status -eq "needs-action") {
    return 0.5
  }
  return 0.0
}

function Escape-MarkdownCell {
  param([string]$Value)
  if ([string]::IsNullOrEmpty($Value)) {
    return ""
  }
  return ($Value -replace '\|', '\|')
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

function Read-JsonFile {
  param([Parameter(Mandatory = $true)][string]$Path)

  $resolved = Resolve-RepoPath $Path
  if (-not (Test-Path -LiteralPath $resolved)) {
    Fail-OnboardingReadiness "JSON file not found: $Path"
  }
  return Get-Content -Raw -LiteralPath $resolved | ConvertFrom-Json
}

$resolvedOutputDir = Resolve-RepoPath $OutputDir
New-Item -ItemType Directory -Force -Path $resolvedOutputDir | Out-Null

if ([string]::IsNullOrWhiteSpace($AdoptionPackPath)) {
  $packOutputDir = Join-Path $resolvedOutputDir "_adoption-pack"
  $packScript = Join-Path $repoRoot "scripts\control-plane\generate_enterprise_adoption_pack.ps1"
  $resolvedProfilePath = Resolve-RepoPath $ProfilePath
  & $packScript -ProfilePath $resolvedProfilePath -OutputDir $packOutputDir | Out-Host
  $lastExitCodeVariable = Get-Variable -Name LASTEXITCODE -ErrorAction SilentlyContinue
  if ($null -ne $lastExitCodeVariable -and $null -ne $lastExitCodeVariable.Value -and [int]$lastExitCodeVariable.Value -ne 0) {
    Fail-OnboardingReadiness "Adoption pack generation failed."
  }
  $AdoptionPackPath = Join-Path $packOutputDir "enterprise-adoption-pack.json"
}

$pack = Read-JsonFile $AdoptionPackPath
$providerReport = if ([string]::IsNullOrWhiteSpace($ProviderConnectionsPath)) { $null } else { Read-JsonFile $ProviderConnectionsPath }
$workflowReadiness = if ([string]::IsNullOrWhiteSpace($WorkflowReadinessPath)) { $null } else { Read-JsonFile $WorkflowReadinessPath }

$customerName = [string](Get-ObjectPropertyValue $pack "customer_name")
$repositoryFullName = [string](Get-ObjectPropertyValue $pack "repository_full_name")
$defaultBranch = [string](Get-ObjectPropertyValue $pack "default_branch")
$jiraProjectKey = [string](Get-ObjectPropertyValue $pack "jira_project_key")
$policyPreset = [string](Get-ObjectPropertyValue $pack "policy_preset")
$providers = @(ConvertTo-Array (Get-ObjectPropertyValue $pack "providers") | ForEach-Object { [string]$_ })
$modules = @(ConvertTo-Array (Get-ObjectPropertyValue $pack "modules") | ForEach-Object { [string]$_ })
$workflowPlan = @(ConvertTo-Array (Get-ObjectPropertyValue $pack "workflow_plan"))
$variables = @(ConvertTo-Array (Get-ObjectPropertyValue $pack "variables"))
$secrets = @(ConvertTo-Array (Get-ObjectPropertyValue $pack "secrets"))
$releaseGovernance = Get-ObjectPropertyValue $pack "release_governance"
$releaseGovernanceMode = [string](Get-ObjectPropertyValue $releaseGovernance "mode")
$releaseGovernanceEnvironment = [string](Get-ObjectPropertyValue $releaseGovernance "environment")
$releaseGovernanceEnforcement = [string](Get-ObjectPropertyValue $releaseGovernance "enforcement")

$profileErrors = New-Object System.Collections.Generic.List[string]
if ([string]::IsNullOrWhiteSpace($customerName)) {
  $profileErrors.Add("Missing customer_name.") | Out-Null
}
if ($repositoryFullName -notmatch '^[^/\s]+/[^/\s]+$') {
  $profileErrors.Add("Repository must look like owner/repo.") | Out-Null
}
if ([string]::IsNullOrWhiteSpace($defaultBranch)) {
  $profileErrors.Add("Missing default_branch.") | Out-Null
}
if (-not [string]::IsNullOrWhiteSpace($jiraProjectKey) -and $jiraProjectKey -notmatch '^[A-Z0-9]+$') {
  $profileErrors.Add("Jira project key should be uppercase letters/numbers.") | Out-Null
}
if ($releaseGovernanceMode -ne "record-only" -and $modules -notcontains "formal-approval") {
  $profileErrors.Add("Non-record-only release governance requires formal-approval module.") | Out-Null
}

$providerStatus = "not-run"
$providerReady = 0
$providerMissing = 0
$providerFailed = 0
if ($null -ne $providerReport) {
  $providerStatus = [string](Get-ObjectPropertyValue $providerReport "status")
  $providerTotals = Get-ObjectPropertyValue $providerReport "totals"
  $providerReady = Get-Number (Get-ObjectPropertyValue $providerTotals "ready")
  $providerMissing = Get-Number (Get-ObjectPropertyValue $providerTotals "missing_config")
  $providerFailed = Get-Number (Get-ObjectPropertyValue $providerTotals "failed")
}

$workflowStatus = "not-run"
$workflowsMissing = 0
$workflowsDifferent = 0
$variablesMissing = 0
$secretsMissing = 0
if ($null -ne $workflowReadiness) {
  $workflowStatus = [string](Get-ObjectPropertyValue $workflowReadiness "status")
  $workflowTotals = Get-ObjectPropertyValue $workflowReadiness "totals"
  $workflowsMissing = Get-Number (Get-ObjectPropertyValue $workflowTotals "workflows_missing")
  $workflowsDifferent = Get-Number (Get-ObjectPropertyValue $workflowTotals "workflows_different")
  $variablesMissing = Get-Number (Get-ObjectPropertyValue $workflowTotals "variables_missing")
  $secretsMissing = Get-Number (Get-ObjectPropertyValue $workflowTotals "secrets_missing")
}

$stages = New-Object System.Collections.Generic.List[object]
$stages.Add((New-Stage `
      -Id "profile" `
      -Label "Adoption profile" `
      -Status $(if ($profileErrors.Count -eq 0) { "ready" } else { "blocked" }) `
      -Summary $(if ($profileErrors.Count -eq 0) { "$customerName profile targets ${repositoryFullName}:$defaultBranch" } else { "$($profileErrors.Count) profile validation issue(s)" }) `
      -NextAction $(if ($profileErrors.Count -eq 0) { "Keep the profile saved before generating customer artifacts." } else { ($profileErrors -join " ") }))) | Out-Null

$stages.Add((New-Stage `
      -Id "providers" `
      -Label "Provider connections" `
      -Status $(if ($providerReport -and $providerStatus -eq "ready") { "ready" } else { "needs-action" }) `
      -Summary $(if ($providerReport) { "$providerReady ready, $providerMissing missing-config, $providerFailed failed provider check(s)" } else { "No provider connection report attached" }) `
      -NextAction $(if ($providerReport -and $providerStatus -eq "ready") { "Keep the sanitized provider connection report with onboarding evidence." } else { "Run validate_enterprise_provider_connections.ps1 with customer-approved credentials." }))) | Out-Null

$stages.Add((New-Stage `
      -Id "workflow-pack" `
      -Label "Workflow template pack" `
      -Status $(if ($workflowPlan.Count -gt 0) { "ready" } else { "needs-action" }) `
      -Summary "$($workflowPlan.Count) workflow template(s), $($variables.Count) variable name(s), $($secrets.Count) secret name(s)" `
      -NextAction $(if ($workflowPlan.Count -gt 0) { "Review the generated workflow pack before local install or remote PR creation." } else { "Enable at least one governance module that generates workflow evidence." }))) | Out-Null

$stages.Add((New-Stage `
      -Id "remote-workflows" `
      -Label "Remote workflow readiness" `
      -Status $(if ($workflowReadiness -and $workflowStatus -eq "ready") { "ready" } else { "needs-action" }) `
      -Summary $(if ($workflowReadiness) { "$workflowsMissing missing, $workflowsDifferent different workflow file(s)" } else { "No remote workflow readiness report attached" }) `
      -NextAction $(if ($workflowReadiness -and $workflowStatus -eq "ready") { "Keep the workflow readiness report with customer onboarding evidence." } else { "Run validate_enterprise_workflow_installation_readiness.ps1 after install or remote PR merge." }))) | Out-Null

$stages.Add((New-Stage `
      -Id "actions-config" `
      -Label "GitHub Actions configuration" `
      -Status $(if ($workflowReadiness) { if (($variablesMissing + $secretsMissing) -eq 0) { "ready" } else { "needs-action" } } elseif (($variables.Count + $secrets.Count) -eq 0) { "ready" } else { "needs-action" }) `
      -Summary $(if ($workflowReadiness) { "$variablesMissing missing variable name(s), $secretsMissing missing secret name(s)" } else { "$($variables.Count) variable name(s), $($secrets.Count) secret name(s) required by the pack" }) `
      -NextAction $(if ($workflowReadiness -and (($variablesMissing + $secretsMissing) -eq 0)) { "Required GitHub Actions configuration names are present." } else { "Create required variables/secrets outside GitGov and re-run readiness validation." }))) | Out-Null

$stages.Add((New-Stage `
      -Id "release-governance" `
      -Label "Release governance policy" `
      -Status $(if ($profileErrors.Count -eq 0) { "ready" } else { "blocked" }) `
      -Summary "$releaseGovernanceMode for $releaseGovernanceEnvironment, enforcement $releaseGovernanceEnforcement" `
      -NextAction $(if ($releaseGovernanceEnforcement -eq "disabled") { "Record-only remains the safe default and does not block releases." } else { "Confirm the customer explicitly selected this policy before treating it as release blocking." }))) | Out-Null

$stageArray = @($stages.ToArray())
$readyCount = @($stageArray | Where-Object { $_.status -eq "ready" }).Count
$needsActionCount = @($stageArray | Where-Object { $_.status -eq "needs-action" }).Count
$blockedCount = @($stageArray | Where-Object { $_.status -eq "blocked" }).Count
$weighted = 0.0
foreach ($stage in $stageArray) {
  $weighted += Get-StageWeight $stage.status
}
$readinessScore = [Math]::Round(($weighted / [Math]::Max(1, $stageArray.Count)) * 100)
$overallStatus = if ($blockedCount -gt 0) {
  "blocked"
} elseif ($needsActionCount -gt 0) {
  "needs-action"
} else {
  "ready"
}

$generatedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
$report = [ordered]@{
  generated_at = $generatedAtUtc
  customer_name = $customerName
  repository_full_name = $repositoryFullName
  default_branch = $defaultBranch
  jira_project_key = $jiraProjectKey
  policy_preset = $policyPreset
  status = $overallStatus
  readiness_score = $readinessScore
  stage_counts = [ordered]@{
    ready = $readyCount
    "needs-action" = $needsActionCount
    blocked = $blockedCount
  }
  release_governance = $releaseGovernance
  providers = @($providers)
  modules = @($modules)
  stages = $stageArray
  next_actions = @($stageArray | Where-Object { $_.status -ne "ready" } | ForEach-Object { "$($_.label): $($_.next_action)" })
  source_files = [ordered]@{
    adoption_pack = $AdoptionPackPath
    provider_connections = if ([string]::IsNullOrWhiteSpace($ProviderConnectionsPath)) { $null } else { $ProviderConnectionsPath }
    workflow_readiness = if ([string]::IsNullOrWhiteSpace($WorkflowReadinessPath)) { $null } else { $WorkflowReadinessPath }
  }
  safety = [ordered]@{
    contains_secret_values = $false
    reads_secret_values = $false
    mutates_customer_repository = $false
    mutates_provider_state = $false
    release_blocking_default = $false
  }
}

$jsonPath = Join-Path $resolvedOutputDir "enterprise-onboarding-readiness.json"
$markdownPath = Join-Path $resolvedOutputDir "enterprise-onboarding-readiness.md"
Write-JsonFile -Value $report -Path $jsonPath

$markdown = New-Object System.Collections.Generic.List[string]
$markdown.Add("# GitGov Enterprise Onboarding Readiness") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add(('Generated: `{0}`' -f $generatedAtUtc)) | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add(('Customer: `{0}`' -f $customerName)) | Out-Null
$markdown.Add(('Repository: `{0}`' -f $repositoryFullName)) | Out-Null
$markdown.Add(('Default branch: `{0}`' -f $defaultBranch)) | Out-Null
$markdown.Add(('Policy preset: `{0}`' -f $policyPreset)) | Out-Null
$markdown.Add(('Status: `{0}`' -f $overallStatus)) | Out-Null
$markdown.Add(('Readiness score: `{0}`' -f $readinessScore)) | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("## Stages") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("| Stage | Status | Summary | Next action |") | Out-Null
$markdown.Add("|---|---|---|---|") | Out-Null
foreach ($stage in $stageArray) {
  $markdown.Add(('| {0} | `{1}` | {2} | {3} |' -f (Escape-MarkdownCell $stage.label), $stage.status, (Escape-MarkdownCell $stage.summary), (Escape-MarkdownCell $stage.next_action))) | Out-Null
}
$markdown.Add("") | Out-Null
$markdown.Add("## Next Actions") | Out-Null
$markdown.Add("") | Out-Null
if ($report.next_actions.Count -eq 0) {
  $markdown.Add("- None. Onboarding evidence is ready for customer review.") | Out-Null
} else {
  foreach ($action in $report.next_actions) {
    $markdown.Add(("- {0}" -f $action)) | Out-Null
  }
}
$markdown.Add("") | Out-Null
$markdown.Add("## Release Governance") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add(('- Mode: `{0}`' -f $releaseGovernanceMode)) | Out-Null
$markdown.Add(('- Environment: `{0}`' -f $releaseGovernanceEnvironment)) | Out-Null
$markdown.Add(('- Enforcement: `{0}`' -f $releaseGovernanceEnforcement)) | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("## Safety") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("- This report contains names, statuses, counters, and next actions only.") | Out-Null
$markdown.Add("- It does not read or print secret values.") | Out-Null
$markdown.Add("- It does not mutate customer repositories, providers, branch protection, workflows, variables, or secrets.") | Out-Null
$markdown.Add("- Release blocking remains customer opt-in only; record-only is the safe default.") | Out-Null

Set-Content -LiteralPath $markdownPath -Value $markdown -Encoding UTF8

Write-Host ("Enterprise onboarding readiness: status={0}; score={1}; ready={2}; needs_action={3}; blocked={4}" -f $overallStatus, $readinessScore, $readyCount, $needsActionCount, $blockedCount)
Write-Host "Wrote onboarding readiness Markdown: $markdownPath"
Write-Host "Wrote onboarding readiness JSON: $jsonPath"

if (-not $ReportOnly -and $overallStatus -ne "ready") {
  Fail-OnboardingReadiness "Enterprise onboarding is not fully ready. Re-run with -ReportOnly to collect a non-blocking report."
}
