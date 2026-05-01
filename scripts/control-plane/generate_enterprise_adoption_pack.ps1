param(
  [string]$ProfilePath,
  [string]$OutputDir = "out/enterprise-adoption-pack",
  [string]$CustomerName,
  [string]$RepositoryFullName,
  [string]$DefaultBranch = "main",
  [ValidateSet("audit-only", "moderate", "strict")]
  [string]$PolicyPreset = "moderate",
  [string]$JiraProjectKey,
  [string[]]$Providers,
  [string[]]$Modules
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Fail-Pack {
  param([Parameter(Mandatory = $true)][string]$Message)
  Write-Host "[FAIL] $Message"
  exit 1
}

function Normalize-List {
  param([string[]]$Values)

  if ($null -eq $Values) {
    return @()
  }

  return @($Values | ForEach-Object { [string]$_ } | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | ForEach-Object { $_.Trim().ToLowerInvariant() } | Sort-Object -Unique)
}

function Add-Unique {
  param(
    [System.Collections.Generic.List[object]]$List,
    [Parameter(Mandatory = $true)]$Item,
    [Parameter(Mandatory = $true)][string]$Key
  )

  $existing = @($List | Where-Object { $_.$Key -eq $Item.$Key })
  if ($existing.Count -eq 0) {
    $List.Add($Item) | Out-Null
  }
}

function Escape-MarkdownCell {
  param([string]$Value)
  return ($Value -replace '\|', '\|')
}

function Get-ProfileProperty {
  param(
    [Parameter(Mandatory = $true)]$Profile,
    [Parameter(Mandatory = $true)][string]$Name
  )

  $property = $Profile.PSObject.Properties[$Name]
  if ($null -eq $property) {
    return $null
  }
  return $property.Value
}

function New-ReleaseGovernancePolicy {
  param(
    [ValidateSet("record-only", "advisory", "approval-required", "quorum-required")]
    [string]$Mode = "record-only",
    [string]$Environment = "production"
  )

  $normalizedEnvironment = if ([string]::IsNullOrWhiteSpace($Environment)) { "production" } else { $Environment.Trim() }
  if ($Mode -eq "advisory") {
    return [pscustomobject]@{
      mode = "advisory"
      environment = $normalizedEnvironment
      approval_required = $false
      enforcement = "advisory"
      quorum = [pscustomobject]@{ enabled = $false; rules = @() }
    }
  }
  if ($Mode -eq "approval-required") {
    return [pscustomobject]@{
      mode = "approval-required"
      environment = $normalizedEnvironment
      approval_required = $true
      enforcement = "blocking"
      quorum = [pscustomobject]@{ enabled = $false; rules = @() }
    }
  }
  if ($Mode -eq "quorum-required") {
    return [pscustomobject]@{
      mode = "quorum-required"
      environment = $normalizedEnvironment
      approval_required = $true
      enforcement = "blocking"
      quorum = [pscustomobject]@{
        enabled = $true
        rules = @(
          [pscustomobject]@{ role = "engineering"; required = 1 },
          [pscustomobject]@{ role = "security"; required = 1 }
        )
      }
    }
  }
  return [pscustomobject]@{
    mode = "record-only"
    environment = $normalizedEnvironment
    approval_required = $false
    enforcement = "disabled"
    quorum = [pscustomobject]@{ enabled = $false; rules = @() }
  }
}

function Normalize-ReleaseGovernancePolicy {
  param($Policy)

  if ($null -eq $Policy) {
    return New-ReleaseGovernancePolicy
  }

  $mode = "record-only"
  $modeProperty = $Policy.PSObject.Properties["mode"]
  if ($modeProperty -and $modeProperty.Value -in @("record-only", "advisory", "approval-required", "quorum-required")) {
    $mode = [string]$modeProperty.Value
  }

  $environment = "production"
  $environmentProperty = $Policy.PSObject.Properties["environment"]
  if ($environmentProperty -and -not [string]::IsNullOrWhiteSpace([string]$environmentProperty.Value)) {
    $environment = [string]$environmentProperty.Value
  }

  $normalized = New-ReleaseGovernancePolicy -Mode $mode -Environment $environment
  if ($mode -ne "quorum-required") {
    return $normalized
  }

  $quorumProperty = $Policy.PSObject.Properties["quorum"]
  $rulesProperty = if ($quorumProperty -and $null -ne $quorumProperty.Value) { $quorumProperty.Value.PSObject.Properties["rules"] } else { $null }
  $rules = New-Object System.Collections.Generic.List[object]
  if ($rulesProperty -and $rulesProperty.Value) {
    foreach ($rule in @($rulesProperty.Value)) {
      $roleProperty = $rule.PSObject.Properties["role"]
      $requiredProperty = $rule.PSObject.Properties["required"]
      $role = if ($roleProperty) { ([string]$roleProperty.Value).Trim().ToLowerInvariant() } else { "" }
      $required = if ($requiredProperty) { [int]$requiredProperty.Value } else { 1 }
      if (-not [string]::IsNullOrWhiteSpace($role)) {
        $rules.Add([pscustomobject]@{ role = $role; required = [Math]::Max(1, [Math]::Min(20, $required)) }) | Out-Null
      }
    }
  }
  if ($rules.Count -gt 0) {
    $normalized.quorum = [pscustomobject]@{ enabled = $true; rules = @($rules.ToArray()) }
  }
  return $normalized
}

$profile = $null
$profileReleaseGovernance = $null
if (-not [string]::IsNullOrWhiteSpace($ProfilePath)) {
  if (-not (Test-Path -LiteralPath $ProfilePath)) {
    Fail-Pack "Profile file not found: $ProfilePath"
  }
  $profile = Get-Content -Raw -LiteralPath $ProfilePath | ConvertFrom-Json
}

if ($null -ne $profile) {
  $profileCustomerName = Get-ProfileProperty -Profile $profile -Name "customer_name"
  $profileRepositoryFullName = Get-ProfileProperty -Profile $profile -Name "repository_full_name"
  $profileDefaultBranch = Get-ProfileProperty -Profile $profile -Name "default_branch"
  $profilePolicyPreset = Get-ProfileProperty -Profile $profile -Name "policy_preset"
  $profileJiraProjectKey = Get-ProfileProperty -Profile $profile -Name "jira_project_key"
  $profileProviders = Get-ProfileProperty -Profile $profile -Name "providers"
  $profileModules = Get-ProfileProperty -Profile $profile -Name "modules"
  $profileReleaseGovernance = Get-ProfileProperty -Profile $profile -Name "release_governance"

  if ([string]::IsNullOrWhiteSpace($CustomerName) -and $profileCustomerName) {
    $CustomerName = [string]$profileCustomerName
  }
  if ([string]::IsNullOrWhiteSpace($RepositoryFullName) -and $profileRepositoryFullName) {
    $RepositoryFullName = [string]$profileRepositoryFullName
  }
  if ([string]::IsNullOrWhiteSpace($DefaultBranch) -and $profileDefaultBranch) {
    $DefaultBranch = [string]$profileDefaultBranch
  }
  if ($profilePolicyPreset) {
    $PolicyPreset = [string]$profilePolicyPreset
  }
  if ([string]::IsNullOrWhiteSpace($JiraProjectKey) -and $profileJiraProjectKey) {
    $JiraProjectKey = [string]$profileJiraProjectKey
  }
  if (($null -eq $Providers -or $Providers.Count -eq 0) -and $profileProviders) {
    $Providers = @($profileProviders)
  }
  if (($null -eq $Modules -or $Modules.Count -eq 0) -and $profileModules) {
    $Modules = @($profileModules)
  }
}

if ([string]::IsNullOrWhiteSpace($CustomerName)) {
  Fail-Pack "Missing -CustomerName or profile customer_name."
}

if ([string]::IsNullOrWhiteSpace($RepositoryFullName)) {
  Fail-Pack "Missing -RepositoryFullName or profile repository_full_name."
}

if ($RepositoryFullName -notmatch '^[^/\s]+/[^/\s]+$') {
  Fail-Pack "-RepositoryFullName must look like owner/repo."
}

if ([string]::IsNullOrWhiteSpace($DefaultBranch)) {
  $DefaultBranch = "main"
}

$knownProviders = @("github", "jira", "jenkins", "sonarqube", "render", "vercel")
$knownModules = @(
  "traceability",
  "github-evidence",
  "release-readiness",
  "quality-gates",
  "evidence-packets",
  "vulnerability-review",
  "artifact-monitoring",
  "trend-enforcement",
  "formal-approval"
)

if ($null -eq $Providers -or $Providers.Count -eq 0) {
  $Providers = @("github", "jira", "jenkins", "sonarqube")
}

if ($null -eq $Modules -or $Modules.Count -eq 0) {
  $Modules = @("traceability", "github-evidence", "release-readiness", "quality-gates", "evidence-packets", "vulnerability-review", "artifact-monitoring", "trend-enforcement")
}

$providersNormalized = Normalize-List $Providers
$modulesNormalized = Normalize-List $Modules
$releaseGovernance = Normalize-ReleaseGovernancePolicy $profileReleaseGovernance

$unknownProviders = @($providersNormalized | Where-Object { $_ -notin $knownProviders })
if ($unknownProviders.Count -gt 0) {
  Fail-Pack "Unknown provider(s): $($unknownProviders -join ', '). Known providers: $($knownProviders -join ', ')."
}

$unknownModules = @($modulesNormalized | Where-Object { $_ -notin $knownModules })
if ($unknownModules.Count -gt 0) {
  Fail-Pack "Unknown module(s): $($unknownModules -join ', '). Known modules: $($knownModules -join ', ')."
}

if ($releaseGovernance.mode -ne "record-only" -and $modulesNormalized -notcontains "formal-approval") {
  Fail-Pack "Release governance mode '$($releaseGovernance.mode)' requires the formal-approval module. Use record-only for non-blocking defaults."
}

$readinessTarget = switch ($PolicyPreset) {
  "audit-only" { 0 }
  "moderate" { 75 }
  "strict" { 85 }
}

$criticalHighPolicy = switch ($PolicyPreset) {
  "audit-only" { "report-only" }
  "moderate" { "block reachable critical/high vulnerabilities" }
  "strict" { "block reachable critical/high vulnerabilities and require documented medium-risk acceptance" }
}

$prReviewRequired = $PolicyPreset -eq "strict"
$freshArtifactRequired = $PolicyPreset -ne "audit-only"
$trendEnforcementRequired = $PolicyPreset -eq "strict" -or ($modulesNormalized -contains "trend-enforcement")

$workflowPlan = New-Object System.Collections.Generic.List[object]
$variablePlan = New-Object System.Collections.Generic.List[object]
$secretPlan = New-Object System.Collections.Generic.List[object]
$manualSteps = New-Object System.Collections.Generic.List[object]
$openProductGaps = New-Object System.Collections.Generic.List[object]

Add-Unique $workflowPlan ([pscustomobject]@{ file = ".github/workflows/ci.yml"; reason = "core build, lint, typecheck, and tests" }) "file"
Add-Unique $workflowPlan ([pscustomobject]@{ file = ".github/workflows/secret-scan.yml"; reason = "publication guard, secret policy, and traceability hygiene" }) "file"

if ($modulesNormalized -contains "traceability") {
  Add-Unique $workflowPlan ([pscustomobject]@{ file = ".github/workflows/public-naming-guard.yml"; reason = "public naming and repository hygiene" }) "file"
  Add-Unique $manualSteps ([pscustomobject]@{ step = "Set Jira-style ticket ID policy"; detail = "Require branch names, PR titles, and commit messages to include ticket IDs such as $JiraProjectKey-123." }) "step"
}

if ($modulesNormalized -contains "github-evidence") {
  Add-Unique $workflowPlan ([pscustomobject]@{ file = ".github/workflows/github-evidence-report.yml"; reason = "GitHub evidence executive report" }) "file"
  Add-Unique $workflowPlan ([pscustomobject]@{ file = ".github/workflows/github-evidence-artifact-monitor.yml"; reason = "GitHub evidence artifact freshness" }) "file"
  Add-Unique $workflowPlan ([pscustomobject]@{ file = ".github/workflows/github-evidence-trend-report.yml"; reason = "GitHub evidence trend history" }) "file"
}

if ($modulesNormalized -contains "release-readiness") {
  Add-Unique $workflowPlan ([pscustomobject]@{ file = ".github/workflows/release-readiness-gate.yml"; reason = "release readiness score and evidence artifact" }) "file"
}

if ($modulesNormalized -contains "quality-gates") {
  Add-Unique $workflowPlan ([pscustomobject]@{ file = ".github/workflows/quality-gate-policy-matrix.yml"; reason = "quality gate warn/block matrix validation" }) "file"
  if ($providersNormalized -contains "sonarqube") {
    Add-Unique $workflowPlan ([pscustomobject]@{ file = ".github/workflows/sonar-governance.yml"; reason = "SonarQube governance telemetry when reachable" }) "file"
  }
}

if ($modulesNormalized -contains "vulnerability-review") {
  Add-Unique $workflowPlan ([pscustomobject]@{ file = ".github/workflows/product-vulnerability-review.yml"; reason = "product vulnerability review evidence" }) "file"
}

if ($modulesNormalized -contains "artifact-monitoring") {
  Add-Unique $workflowPlan ([pscustomobject]@{ file = ".github/workflows/product-vulnerability-review-artifact-monitor.yml"; reason = "product vulnerability review artifact freshness" }) "file"
}

if ($modulesNormalized -contains "vulnerability-review") {
  Add-Unique $workflowPlan ([pscustomobject]@{ file = ".github/workflows/product-vulnerability-review-trend-report.yml"; reason = "product vulnerability review trend report" }) "file"
}

if ($trendEnforcementRequired) {
  Add-Unique $workflowPlan ([pscustomobject]@{ file = ".github/workflows/product-vulnerability-review-trend-enforcement.yml"; reason = "block regressions in vulnerability review trend" }) "file"
}

if ($modulesNormalized -contains "formal-approval") {
  $releasePolicyDetail = if ($releaseGovernance.mode -eq "record-only") {
    "Default record-only mode stores release approval evidence and does not block customer releases."
  } else {
    "Customer selected $($releaseGovernance.mode) for $($releaseGovernance.environment); review this explicit opt-in policy before installing any blocking workflow."
  }
  Add-Unique $manualSteps ([pscustomobject]@{ step = "Review release approval policy"; detail = $releasePolicyDetail }) "step"
}

if ($providersNormalized -contains "github") {
  Add-Unique $variablePlan ([pscustomobject]@{ name = "GITGOV_URL"; scope = "GitHub Actions variable"; purpose = "GitGov API base URL"; example = "https://gitgov-api.example.com" }) "name"
  Add-Unique $secretPlan ([pscustomobject]@{ name = "GITGOV_API_KEY"; scope = "GitHub Actions secret"; purpose = "GitGov API authentication for workflow telemetry"; value_policy = "secret value only, never committed" }) "name"
  Add-Unique $manualSteps ([pscustomobject]@{ step = "Install GitHub webhook"; detail = "Configure signed GitHub webhook events for push, pull_request, pull_request_review, comments, checks, and status." }) "step"
}

if ($providersNormalized -contains "jira") {
  Add-Unique $manualSteps ([pscustomobject]@{ step = "Connect Jira project"; detail = "Set Jira project key, enable signed Jira webhook, and verify ticket ingestion." }) "step"
}

if ($providersNormalized -contains "jenkins") {
  Add-Unique $manualSteps ([pscustomobject]@{ step = "Connect Jenkins"; detail = "Configure authenticated Jenkins API access and GitGov telemetry publishing from pipeline jobs." }) "step"
}

if ($providersNormalized -contains "sonarqube") {
  Add-Unique $variablePlan ([pscustomobject]@{ name = "SONAR_HOST_URL"; scope = "GitHub Actions variable"; purpose = "SonarQube endpoint when reachable by runner"; example = "https://sonarqube.example.com" }) "name"
  Add-Unique $variablePlan ([pscustomobject]@{ name = "SONAR_PROJECT_KEY"; scope = "GitHub Actions variable"; purpose = "SonarQube project key"; example = "example_org_example_repo" }) "name"
  Add-Unique $secretPlan ([pscustomobject]@{ name = "SONAR_TOKEN"; scope = "GitHub Actions secret"; purpose = "Optional SonarQube API token when runner can reach SonarQube"; value_policy = "secret value only, never committed" }) "name"
  Add-Unique $manualSteps ([pscustomobject]@{ step = "Validate Sonar runtime"; detail = "Use reachable SonarQube for customer environments; skip GitHub-hosted scans when Sonar is private/local." }) "step"
}

if ($providersNormalized -contains "render") {
  Add-Unique $manualSteps ([pscustomobject]@{ step = "Connect deployment provider"; detail = "Record deployment health and service metadata without storing provider tokens in the repository." }) "step"
}

if ($providersNormalized -contains "vercel") {
  Add-Unique $manualSteps ([pscustomobject]@{ step = "Connect Vercel deployment evidence"; detail = "Use deployment status and preview evidence as governance context when the customer deploys on Vercel." }) "step"
}

$policyRules = @(
  [pscustomobject]@{ rule = "Ticket traceability"; setting = if ($modulesNormalized -contains "traceability") { "required" } else { "optional" } },
  [pscustomobject]@{ rule = "Release readiness target"; setting = [string]$readinessTarget },
  [pscustomobject]@{ rule = "Release approval governance"; setting = $releaseGovernance.mode },
  [pscustomobject]@{ rule = "Release approval enforcement"; setting = $releaseGovernance.enforcement },
  [pscustomobject]@{ rule = "Release approval quorum"; setting = if ($releaseGovernance.quorum.enabled) { (($releaseGovernance.quorum.rules | ForEach-Object { "$($_.role):$($_.required)" }) -join ", ") } else { "disabled" } },
  [pscustomobject]@{ rule = "Critical/high vulnerability policy"; setting = $criticalHighPolicy },
  [pscustomobject]@{ rule = "PR review evidence"; setting = if ($prReviewRequired) { "required" } else { "recommended" } },
  [pscustomobject]@{ rule = "Fresh evidence artifacts"; setting = if ($freshArtifactRequired) { "required" } else { "report-only" } },
  [pscustomobject]@{ rule = "Vulnerability trend enforcement"; setting = if ($trendEnforcementRequired) { "enabled" } else { "informational" } }
)

$generatedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
$output = [pscustomobject]@{
  generated_at = $generatedAtUtc
  customer_name = $CustomerName
  repository_full_name = $RepositoryFullName
  default_branch = $DefaultBranch
  jira_project_key = $JiraProjectKey
  policy_preset = $PolicyPreset
  release_governance = $releaseGovernance
  providers = @($providersNormalized)
  modules = @($modulesNormalized)
  workflow_plan = @($workflowPlan.ToArray())
  variables = @($variablePlan.ToArray())
  secrets = @($secretPlan.ToArray())
  policy_rules = @($policyRules)
  manual_steps = @($manualSteps.ToArray())
  open_product_gaps = @($openProductGaps.ToArray())
}

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$jsonPath = Join-Path $OutputDir "enterprise-adoption-pack.json"
$markdownPath = Join-Path $OutputDir "enterprise-adoption-pack.md"

$output | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $jsonPath -Encoding UTF8

$markdown = New-Object System.Collections.Generic.List[string]
$markdown.Add("# GitGov Enterprise Adoption Pack") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add(('Generated: `{0}`' -f $generatedAtUtc)) | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add(('Customer: `{0}`' -f $CustomerName)) | Out-Null
$markdown.Add(('Repository: `{0}`' -f $RepositoryFullName)) | Out-Null
$markdown.Add(('Default branch: `{0}`' -f $DefaultBranch)) | Out-Null
$markdown.Add(('Policy preset: `{0}`' -f $PolicyPreset)) | Out-Null
$markdown.Add(('Release governance: `{0}`' -f $releaseGovernance.mode)) | Out-Null
$markdown.Add(('Release enforcement: `{0}`' -f $releaseGovernance.enforcement)) | Out-Null
if (-not [string]::IsNullOrWhiteSpace($JiraProjectKey)) {
  $markdown.Add(('Jira project key: `{0}`' -f $JiraProjectKey)) | Out-Null
}
$markdown.Add("") | Out-Null
$markdown.Add("## Modules") | Out-Null
$markdown.Add("") | Out-Null
foreach ($module in $modulesNormalized) {
  $markdown.Add(('- `{0}`' -f $module)) | Out-Null
}
$markdown.Add("") | Out-Null
$markdown.Add("## Providers") | Out-Null
$markdown.Add("") | Out-Null
foreach ($provider in $providersNormalized) {
  $markdown.Add(('- `{0}`' -f $provider)) | Out-Null
}
$markdown.Add("") | Out-Null
$markdown.Add("## Release Governance") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add(('- Mode: `{0}`' -f $releaseGovernance.mode)) | Out-Null
$markdown.Add(('- Environment: `{0}`' -f $releaseGovernance.environment)) | Out-Null
$markdown.Add(('- Enforcement: `{0}`' -f $releaseGovernance.enforcement)) | Out-Null
if ($releaseGovernance.quorum.enabled) {
  foreach ($rule in $releaseGovernance.quorum.rules) {
    $markdown.Add(('- Quorum `{0}`: `{1}` required' -f $rule.role, $rule.required)) | Out-Null
  }
} else {
  $markdown.Add('- Quorum: `disabled`') | Out-Null
}
$markdown.Add("") | Out-Null
$markdown.Add("## Workflow Plan") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("| Workflow | Why |") | Out-Null
$markdown.Add("|---|---|") | Out-Null
foreach ($workflow in $workflowPlan) {
  $markdown.Add(('| `{0}` | {1} |' -f (Escape-MarkdownCell $workflow.file), (Escape-MarkdownCell $workflow.reason))) | Out-Null
}
$markdown.Add("") | Out-Null
$markdown.Add("## Policy Rules") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("| Rule | Setting |") | Out-Null
$markdown.Add("|---|---|") | Out-Null
foreach ($rule in $policyRules) {
  $markdown.Add(('| {0} | `{1}` |' -f (Escape-MarkdownCell $rule.rule), (Escape-MarkdownCell $rule.setting))) | Out-Null
}
$markdown.Add("") | Out-Null
$markdown.Add("## Required Variables") | Out-Null
$markdown.Add("") | Out-Null
if ($variablePlan.Count -eq 0) {
  $markdown.Add("- None selected.") | Out-Null
} else {
  $markdown.Add("| Name | Scope | Purpose | Example |") | Out-Null
  $markdown.Add("|---|---|---|---|") | Out-Null
  foreach ($variable in $variablePlan) {
    $markdown.Add(('| `{0}` | {1} | {2} | `{3}` |' -f $variable.name, (Escape-MarkdownCell $variable.scope), (Escape-MarkdownCell $variable.purpose), (Escape-MarkdownCell $variable.example))) | Out-Null
  }
}
$markdown.Add("") | Out-Null
$markdown.Add("## Required Secrets") | Out-Null
$markdown.Add("") | Out-Null
if ($secretPlan.Count -eq 0) {
  $markdown.Add("- None selected.") | Out-Null
} else {
  $markdown.Add("| Name | Scope | Purpose | Value Policy |") | Out-Null
  $markdown.Add("|---|---|---|---|") | Out-Null
  foreach ($secret in $secretPlan) {
    $markdown.Add(('| `{0}` | {1} | {2} | {3} |' -f $secret.name, (Escape-MarkdownCell $secret.scope), (Escape-MarkdownCell $secret.purpose), (Escape-MarkdownCell $secret.value_policy))) | Out-Null
  }
}
$markdown.Add("") | Out-Null
$markdown.Add("## Manual Setup Checklist") | Out-Null
$markdown.Add("") | Out-Null
foreach ($step in $manualSteps) {
  $markdown.Add(('- **{0}:** {1}' -f $step.step, $step.detail)) | Out-Null
}
if ($openProductGaps.Count -gt 0) {
  $markdown.Add("") | Out-Null
  $markdown.Add("## Open Product Gaps") | Out-Null
  $markdown.Add("") | Out-Null
  foreach ($gap in $openProductGaps) {
    $markdown.Add(('- **{0}:** {1}' -f $gap.gap, $gap.detail)) | Out-Null
  }
}
$markdown.Add("") | Out-Null
$markdown.Add("## Safety Notes") | Out-Null
$markdown.Add("") | Out-Null
$markdown.Add("- This pack uses secret and variable names only. It does not contain secret values.") | Out-Null
$markdown.Add("- Provider tokens must stay in the customer's secret manager or GitHub Actions secrets.") | Out-Null
$markdown.Add("- SonarCloud is not assumed. Use the customer's selected SonarQube runtime when applicable.") | Out-Null

Set-Content -LiteralPath $markdownPath -Value $markdown -Encoding UTF8

Write-Host "Wrote enterprise adoption pack Markdown: $markdownPath"
Write-Host "Wrote enterprise adoption pack JSON: $jsonPath"
