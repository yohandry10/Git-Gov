param(
  [string]$ProfilePath,
  [string]$OutputDir = "out/enterprise-workflow-templates",
  [string]$CustomerName,
  [string]$RepositoryFullName,
  [string]$DefaultBranch = "main",
  [ValidateSet("audit-only", "moderate", "strict")]
  [string]$PolicyPreset = "moderate",
  [string]$JiraProjectKey,
  [string[]]$Providers,
  [string[]]$Modules,
  [switch]$Force
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Fail-Templates {
  param([Parameter(Mandatory = $true)][string]$Message)
  Write-Host "[FAIL] $Message"
  exit 1
}

function Normalize-List {
  param([string[]]$Values)

  if ($null -eq $Values) {
    return @()
  }

  return @(
    $Values |
      ForEach-Object { [string]$_ } |
      Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
      ForEach-Object { $_.Trim().ToLowerInvariant() } |
      Sort-Object -Unique
  )
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
      environment_overrides = @()
    }
  }
  if ($Mode -eq "approval-required") {
    return [pscustomobject]@{
      mode = "approval-required"
      environment = $normalizedEnvironment
      approval_required = $true
      enforcement = "blocking"
      quorum = [pscustomobject]@{ enabled = $false; rules = @() }
      environment_overrides = @()
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
      environment_overrides = @()
    }
  }
  return [pscustomobject]@{
    mode = "record-only"
    environment = $normalizedEnvironment
    approval_required = $false
    enforcement = "disabled"
    quorum = [pscustomobject]@{ enabled = $false; rules = @() }
    environment_overrides = @()
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
  if ($mode -eq "quorum-required") {
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
  }

  $overrides = New-Object System.Collections.Generic.List[object]
  $seenOverrideEnvironments = New-Object 'System.Collections.Generic.HashSet[string]'
  $overridesProperty = $Policy.PSObject.Properties["environment_overrides"]
  if ($overridesProperty -and $overridesProperty.Value) {
    foreach ($override in @($overridesProperty.Value)) {
      $overridePolicy = Normalize-ReleaseGovernancePolicy $override
      $overridePolicy.environment_overrides = @()
      $environmentKey = ([string]$overridePolicy.environment).Trim().ToLowerInvariant()
      if (-not [string]::IsNullOrWhiteSpace($environmentKey) -and $seenOverrideEnvironments.Add($environmentKey)) {
        $overrides.Add($overridePolicy) | Out-Null
      }
    }
  }
  $normalized.environment_overrides = @($overrides.ToArray())
  return $normalized
}

function Get-ReleaseGovernancePolicies {
  param($Policy)
  $policies = New-Object System.Collections.Generic.List[object]
  $policies.Add($Policy) | Out-Null
  $overridesProperty = $Policy.PSObject.Properties["environment_overrides"]
  if ($overridesProperty -and $overridesProperty.Value) {
    foreach ($override in @($overridesProperty.Value)) {
      $policies.Add($override) | Out-Null
    }
  }
  return @($policies.ToArray())
}

function Test-ReleaseGovernanceRequiresFormalApproval {
  param($Policy)
  return @(Get-ReleaseGovernancePolicies $Policy | Where-Object { $_.mode -ne "record-only" }).Count -gt 0
}

function Get-ReleaseGovernanceGatePolicy {
  param($Policy)
  $blocking = @(Get-ReleaseGovernancePolicies $Policy | Where-Object { $_.mode -in @("approval-required", "quorum-required") } | Select-Object -First 1)
  if ($blocking.Count -gt 0) { return $blocking[0] }
  $nonRecord = @(Get-ReleaseGovernancePolicies $Policy | Where-Object { $_.mode -ne "record-only" } | Select-Object -First 1)
  if ($nonRecord.Count -gt 0) { return $nonRecord[0] }
  return $Policy
}

function Get-ReleaseGovernanceOverrideSummary {
  param($Policy)
  $overridesProperty = $Policy.PSObject.Properties["environment_overrides"]
  if (-not $overridesProperty -or -not $overridesProperty.Value -or @($overridesProperty.Value).Count -eq 0) {
    return "none"
  }
  return ((@($overridesProperty.Value) | ForEach-Object { "$($_.environment):$($_.mode)" }) -join ", ")
}

function Add-UniqueObject {
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

function Set-TextFile {
  param(
    [Parameter(Mandatory = $true)][string]$Path,
    [Parameter(Mandatory = $true)][string]$Value
  )

  $parent = Split-Path -Parent $Path
  if (-not [string]::IsNullOrWhiteSpace($parent)) {
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
  }

  Set-Content -LiteralPath $Path -Value $Value -Encoding UTF8
}

function Escape-MarkdownCell {
  param([string]$Value)
  return ($Value -replace '\|', '\|')
}

function Join-TemplatePath {
  param(
    [Parameter(Mandatory = $true)][string]$Root,
    [Parameter(Mandatory = $true)][string]$RelativePath
  )

  $segments = $RelativePath -split '/'
  $path = $Root
  foreach ($segment in $segments) {
    $path = Join-Path $path $segment
  }
  return $path
}

function Resolve-Template {
  param(
    [Parameter(Mandatory = $true)][string]$Template,
    [Parameter(Mandatory = $true)][hashtable]$Tokens
  )

  $result = $Template
  foreach ($key in $Tokens.Keys) {
    $result = $result.Replace(("__{0}__" -f $key), [string]$Tokens[$key])
  }
  return $result
}

function Get-CiWorkflow {
  return @'
# Generated by GitGov workflow template generation.
# Review and customize project commands before installing in a customer repository.
name: GitGov Customer CI

on:
  pull_request:
  push:
    branches: ["__DEFAULT_BRANCH__"]
  workflow_dispatch:

permissions:
  contents: read

jobs:
  ci:
    name: Build and test
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - name: Checkout
        uses: actions/checkout@v6

      - name: Setup Node.js
        uses: actions/setup-node@v6
        with:
          node-version: 20

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Run detected checks
        shell: pwsh
        run: |
          $ErrorActionPreference = "Stop"
          $ran = $false

          if (Test-Path "package.json") {
            $ran = $true
            if (Test-Path "package-lock.json") {
              npm ci
            } else {
              npm install
            }
            npm run lint --if-present
            npm run typecheck --if-present
            npm test --if-present
            npm run build --if-present
          }

          if (Test-Path "Cargo.toml") {
            $ran = $true
            cargo check
            cargo test
          }

          if (-not $ran) {
            Write-Warning "No package.json or Cargo.toml was found. Customize this template for the customer stack."
          }
'@
}

function Get-SecretScanWorkflow {
  return @'
# Generated by GitGov workflow template generation.
# This baseline blocks committed env files. Add the customer's preferred scanner as needed.
name: GitGov Secret And Publication Guard

on:
  pull_request:
  push:
    branches: ["__DEFAULT_BRANCH__"]
  workflow_dispatch:

permissions:
  contents: read

jobs:
  secret-publication-guard:
    name: Secret publication guard
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v6
        with:
          fetch-depth: 0

      - name: Block committed secret files
        shell: pwsh
        run: |
          $ErrorActionPreference = "Stop"
          $blocked = @(
            git ls-files |
              Where-Object {
                ($_ -match '(^|/)\.env($|\.|/)' -and $_ -notmatch '\.env\.example$') -or
                ($_ -match '(^|/)secrets/')
              }
          )

          if ($blocked.Count -gt 0) {
            Write-Host "Blocked files:"
            $blocked | ForEach-Object { Write-Host "  - $_" }
            throw "Secret-like files must not be committed. Keep values in GitHub Actions secrets or the customer secret manager."
          }

          Write-Host "PASS: no blocked secret file paths are tracked."
'@
}

function Get-PublicNamingGuardWorkflow {
  return @'
# Generated by GitGov workflow template generation.
# Enforces ticket IDs in branch names, PR titles, and commit messages.
name: GitGov Traceability Guard

on:
  pull_request:
  push:
    branches: ["__DEFAULT_BRANCH__"]
  workflow_dispatch:

permissions:
  contents: read
  pull-requests: read

env:
  JIRA_PROJECT_KEY: "__JIRA_PROJECT_KEY__"

jobs:
  traceability:
    name: Validate ticket traceability
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v6
        with:
          fetch-depth: 0

      - name: Check branch, PR title, and latest commit
        shell: pwsh
        env:
          EVENT_NAME_VALUE: ${{ github.event_name }}
          HEAD_REF_VALUE: ${{ github.head_ref }}
          REF_NAME_VALUE: ${{ github.ref_name }}
          PR_TITLE_VALUE: ${{ github.event.pull_request.title }}
        run: |
          $ErrorActionPreference = "Stop"
          if ([string]::IsNullOrWhiteSpace($env:JIRA_PROJECT_KEY)) {
            throw "JIRA_PROJECT_KEY is required for traceability."
          }

          $pattern = "\b$([regex]::Escape($env:JIRA_PROJECT_KEY))-\d+\b"
          $branch = $env:HEAD_REF_VALUE
          if ([string]::IsNullOrWhiteSpace($branch)) {
            $branch = $env:REF_NAME_VALUE
          }
          $title = $env:PR_TITLE_VALUE
          $commitSubject = (git log -1 --pretty=%s)

          $failures = New-Object System.Collections.Generic.List[string]
          if ($branch -notmatch $pattern) {
            $failures.Add("branch name")
          }
          if ($env:EVENT_NAME_VALUE -eq "pull_request" -and $title -notmatch $pattern) {
            $failures.Add("PR title")
          }
          if ($commitSubject -notmatch $pattern) {
            $failures.Add("latest commit subject")
          }

          if ($failures.Count -gt 0) {
            throw "Missing ticket ID pattern $env:JIRA_PROJECT_KEY-123 in: $($failures -join ', ')."
          }

          Write-Host "PASS: traceability pattern found."
'@
}

function Get-GitGovStatsWorkflow {
  return @'
# Generated by GitGov workflow template generation.
# Produces a sanitized GitGov stats artifact for onboarding evidence.
name: GitGov Evidence Report

on:
  workflow_dispatch:
  schedule:
    - cron: "23 13 * * 1"

permissions:
  contents: read

jobs:
  github-evidence-report:
    name: Generate evidence report
    runs-on: ubuntu-latest
    steps:
      - name: Fetch GitGov stats
        id: fetch
        shell: pwsh
        env:
          GITGOV_URL: ${{ vars.GITGOV_URL }}
          GITGOV_API_KEY: ${{ secrets.GITGOV_API_KEY }}
          GITHUB_REPOSITORY_NAME: ${{ github.repository }}
          RUN_ID_VALUE: ${{ github.run_id }}
        run: |
          $ErrorActionPreference = "Stop"
          New-Item -ItemType Directory -Force -Path "gitgov-evidence" | Out-Null
          $outputPath = "gitgov-evidence/github-evidence-report-$env:RUN_ID_VALUE.json"

          if ([string]::IsNullOrWhiteSpace($env:GITGOV_URL) -or [string]::IsNullOrWhiteSpace($env:GITGOV_API_KEY)) {
            $result = [ordered]@{
              status = "skipped"
              reason = "missing_gitgov_url_or_api_key"
              repository = $env:GITHUB_REPOSITORY_NAME
              generated_at = [DateTimeOffset]::UtcNow.ToString("o")
            }
            $result | ConvertTo-Json -Depth 8 | Out-File -FilePath $outputPath -Encoding UTF8
            Write-Warning "Skipping GitGov stats fetch because configuration is missing."
            exit 0
          }

          $baseUrl = $env:GITGOV_URL.TrimEnd("/")
          $headers = @{ Authorization = "Bearer $env:GITGOV_API_KEY" }
          $stats = Invoke-RestMethod -Method GET -Uri "$baseUrl/stats" -Headers $headers
          $result = [ordered]@{
            status = "ok"
            repository = $env:GITHUB_REPOSITORY_NAME
            generated_at = [DateTimeOffset]::UtcNow.ToString("o")
            stats = $stats
          }
          $result | ConvertTo-Json -Depth 12 | Out-File -FilePath $outputPath -Encoding UTF8
          Write-Host "Wrote sanitized evidence report: $outputPath"

      - name: Upload evidence artifact
        uses: actions/upload-artifact@v7
        with:
          name: github-evidence-report-${{ github.run_id }}
          path: gitgov-evidence
          if-no-files-found: error
          retention-days: 30
'@
}

function Get-ArtifactMonitorWorkflow {
  param(
    [Parameter(Mandatory = $true)][string]$WorkflowName,
    [Parameter(Mandatory = $true)][string]$ArtifactPrefix,
    [Parameter(Mandatory = $true)][string]$Cron,
    [Parameter(Mandatory = $true)][string]$OutputFilePrefix,
    [int]$MaxAgeHours = 192,
    [bool]$IncludeSchedule = $true
  )

  $scheduleBlock = if ($IncludeSchedule) {
    "  schedule:`n    - cron: `"__CRON__`""
  } else {
    ""
  }

  $template = @'
# Generated by GitGov workflow template generation.
# Validates that a recent evidence artifact exists and is not expired.
name: __WORKFLOW_NAME__

on:
  workflow_dispatch:
__SCHEDULE_BLOCK__

permissions:
  actions: read
  contents: read

jobs:
  artifact-monitor:
    name: Monitor evidence artifact freshness
    runs-on: ubuntu-latest
    steps:
      - name: Check artifact freshness
        shell: pwsh
        env:
          GH_TOKEN: ${{ github.token }}
          REPOSITORY_NAME: ${{ github.repository }}
          RUN_ID_VALUE: ${{ github.run_id }}
          ARTIFACT_PREFIX: "__ARTIFACT_PREFIX__"
          OUTPUT_PREFIX: "__OUTPUT_FILE_PREFIX__"
          MAX_ARTIFACT_AGE_HOURS: "__MAX_AGE_HOURS__"
        run: |
          $ErrorActionPreference = "Stop"
          New-Item -ItemType Directory -Force -Path "gitgov-evidence" | Out-Null
          $headers = @{
            Authorization = "Bearer $env:GH_TOKEN"
            Accept = "application/vnd.github+json"
            "X-GitHub-Api-Version" = "2022-11-28"
          }
          $uri = "https://api.github.com/repos/$env:REPOSITORY_NAME/actions/artifacts?per_page=100"
          $response = Invoke-RestMethod -Method GET -Uri $uri -Headers $headers
          $artifacts = @($response.artifacts | Where-Object { $_.name -like "$env:ARTIFACT_PREFIX*" } | Sort-Object created_at -Descending)
          $latest = $artifacts | Select-Object -First 1
          $status = "fail"
          $reason = "missing_artifact"
          $ageHours = $null

          if ($null -ne $latest) {
            $createdAt = [DateTimeOffset]::Parse([string]$latest.created_at)
            $ageHours = [Math]::Round(([DateTimeOffset]::UtcNow - $createdAt).TotalHours, 2)
            if ($latest.expired -eq $true) {
              $reason = "artifact_expired"
            } elseif ($ageHours -gt [double]$env:MAX_ARTIFACT_AGE_HOURS) {
              $reason = "artifact_too_old"
            } else {
              $status = "pass"
              $reason = "fresh_artifact_found"
            }
          }

          $result = [ordered]@{
            status = $status
            reason = $reason
            repository = $env:REPOSITORY_NAME
            artifact_prefix = $env:ARTIFACT_PREFIX
            latest_artifact_name = if ($null -eq $latest) { $null } else { $latest.name }
            latest_artifact_id = if ($null -eq $latest) { $null } else { $latest.id }
            latest_artifact_age_hours = $ageHours
            generated_at = [DateTimeOffset]::UtcNow.ToString("o")
          }
          $outputPath = "gitgov-evidence/$env:OUTPUT_PREFIX-$env:RUN_ID_VALUE.json"
          $result | ConvertTo-Json -Depth 8 | Out-File -FilePath $outputPath -Encoding UTF8
          if ($status -ne "pass") {
            throw "Artifact freshness check failed: $reason"
          }

      - name: Upload monitor artifact
        uses: actions/upload-artifact@v7
        with:
          name: __OUTPUT_FILE_PREFIX__
          path: gitgov-evidence
          if-no-files-found: error
          retention-days: 30
'@

  return $template.
    Replace("__WORKFLOW_NAME__", $WorkflowName).
    Replace("__ARTIFACT_PREFIX__", $ArtifactPrefix).
    Replace("__SCHEDULE_BLOCK__", $scheduleBlock).
    Replace("__CRON__", $Cron).
    Replace("__MAX_AGE_HOURS__", [string]$MaxAgeHours).
    Replace("__OUTPUT_FILE_PREFIX__", $OutputFilePrefix)
}

function Get-ArtifactTrendWorkflow {
  param(
    [Parameter(Mandatory = $true)][string]$WorkflowName,
    [Parameter(Mandatory = $true)][string]$ArtifactPrefix,
    [Parameter(Mandatory = $true)][string]$Cron,
    [Parameter(Mandatory = $true)][string]$OutputFilePrefix
  )

  $template = @'
# Generated by GitGov workflow template generation.
# Builds a lightweight trend inventory from recent evidence artifacts.
name: __WORKFLOW_NAME__

on:
  workflow_dispatch:
  schedule:
    - cron: "__CRON__"

permissions:
  actions: read
  contents: read

jobs:
  trend-report:
    name: Build evidence trend inventory
    runs-on: ubuntu-latest
    steps:
      - name: Build trend report
        shell: pwsh
        env:
          GH_TOKEN: ${{ github.token }}
          REPOSITORY_NAME: ${{ github.repository }}
          RUN_ID_VALUE: ${{ github.run_id }}
          ARTIFACT_PREFIX: "__ARTIFACT_PREFIX__"
          OUTPUT_PREFIX: "__OUTPUT_FILE_PREFIX__"
        run: |
          $ErrorActionPreference = "Stop"
          New-Item -ItemType Directory -Force -Path "gitgov-evidence" | Out-Null
          $headers = @{
            Authorization = "Bearer $env:GH_TOKEN"
            Accept = "application/vnd.github+json"
            "X-GitHub-Api-Version" = "2022-11-28"
          }
          $uri = "https://api.github.com/repos/$env:REPOSITORY_NAME/actions/artifacts?per_page=100"
          $response = Invoke-RestMethod -Method GET -Uri $uri -Headers $headers
          $artifacts = @(
            $response.artifacts |
              Where-Object { $_.name -like "$env:ARTIFACT_PREFIX*" } |
              Sort-Object created_at -Descending |
              Select-Object -First 10
          )
          $trend = @(
            $artifacts | ForEach-Object {
              [ordered]@{
                name = $_.name
                id = $_.id
                created_at = $_.created_at
                expired = $_.expired
              }
            }
          )
          $result = [ordered]@{
            status = if ($trend.Count -gt 0) { "pass" } else { "missing" }
            repository = $env:REPOSITORY_NAME
            artifact_prefix = $env:ARTIFACT_PREFIX
            artifact_count = $trend.Count
            trend = $trend
            generated_at = [DateTimeOffset]::UtcNow.ToString("o")
          }
          $outputPath = "gitgov-evidence/$env:OUTPUT_PREFIX-$env:RUN_ID_VALUE.json"
          $result | ConvertTo-Json -Depth 8 | Out-File -FilePath $outputPath -Encoding UTF8
          if ($trend.Count -eq 0) {
            throw "No artifacts found for prefix $env:ARTIFACT_PREFIX."
          }

      - name: Upload trend artifact
        uses: actions/upload-artifact@v7
        with:
          name: __OUTPUT_FILE_PREFIX__
          path: gitgov-evidence
          if-no-files-found: error
          retention-days: 30
'@

  return $template.
    Replace("__WORKFLOW_NAME__", $WorkflowName).
    Replace("__ARTIFACT_PREFIX__", $ArtifactPrefix).
    Replace("__CRON__", $Cron).
    Replace("__OUTPUT_FILE_PREFIX__", $OutputFilePrefix)
}

function Get-ReleaseReadinessWorkflow {
  return @'
# Generated by GitGov workflow template generation.
# Computes a compact readiness score from GitGov Jira and Jenkins evidence.
name: GitGov Release Readiness Gate

on:
  push:
    branches: ["__DEFAULT_BRANCH__"]
  workflow_dispatch:
    inputs:
      enforce_gate:
        description: "Fail when readiness is below target"
        required: false
        default: __ENFORCE_GATE__
        type: boolean

permissions:
  contents: read

jobs:
  readiness:
    name: Validate release readiness
    runs-on: ubuntu-latest
    steps:
      - name: Compute readiness
        shell: pwsh
        env:
          GITGOV_URL: ${{ vars.GITGOV_URL }}
          GITGOV_API_KEY: ${{ secrets.GITGOV_API_KEY }}
          REPOSITORY_NAME: ${{ github.repository }}
          REF_NAME_VALUE: ${{ github.ref_name }}
          RUN_ID_VALUE: ${{ github.run_id }}
          INPUT_ENFORCE_GATE: ${{ inputs.enforce_gate }}
          TARGET_READINESS: "__READINESS_TARGET__"
        run: |
          $ErrorActionPreference = "Stop"
          New-Item -ItemType Directory -Force -Path "gitgov-evidence" | Out-Null
          $outputPath = "gitgov-evidence/release-readiness-gate-$env:RUN_ID_VALUE.json"

          if ([string]::IsNullOrWhiteSpace($env:GITGOV_URL) -or [string]::IsNullOrWhiteSpace($env:GITGOV_API_KEY)) {
            $result = [ordered]@{
              status = "skipped"
              reason = "missing_gitgov_url_or_api_key"
              generated_at = [DateTimeOffset]::UtcNow.ToString("o")
            }
            $result | ConvertTo-Json -Depth 8 | Out-File -FilePath $outputPath -Encoding UTF8
            Write-Warning "Skipping readiness because GitGov configuration is missing."
            exit 0
          }

          $baseUrl = $env:GITGOV_URL.TrimEnd("/")
          $headers = @{ Authorization = "Bearer $env:GITGOV_API_KEY" }
          $repo = [Uri]::EscapeDataString($env:REPOSITORY_NAME)
          $branch = [Uri]::EscapeDataString($env:REF_NAME_VALUE)
          $ticketCoverage = Invoke-RestMethod -Method GET -Uri "$baseUrl/integrations/jira/ticket-coverage?repo_full_name=$repo&branch=$branch&hours=720" -Headers $headers
          $correlations = Invoke-RestMethod -Method GET -Uri "$baseUrl/integrations/jenkins/correlations?repo_full_name=$repo&branch=$branch&limit=500&offset=0" -Headers $headers

          $pipelineRuns = @($correlations.correlations | Where-Object { $null -ne $_.pipeline })
          $pipelineTotal = $pipelineRuns.Count
          $pipelineSuccess = @($pipelineRuns | Where-Object { ([string]$_.pipeline.status).ToLowerInvariant() -eq "success" }).Count
          $pipelineRate = if ($pipelineTotal -gt 0) { [Math]::Round((100.0 * $pipelineSuccess) / $pipelineTotal, 2) } else { 0 }
          $jiraCoverage = if ($null -ne $ticketCoverage.coverage_percentage) { [double]$ticketCoverage.coverage_percentage } else { 0 }
          $available = 0
          $scoreTotal = 0.0
          if ($pipelineTotal -gt 0) { $available += 1; $scoreTotal += $pipelineRate }
          if ($ticketCoverage.total_commits -gt 0) { $available += 1; $scoreTotal += $jiraCoverage }
          $score = if ($available -gt 0) { [int][Math]::Round($scoreTotal / $available) } else { 0 }
          $target = [int]$env:TARGET_READINESS
          $passed = $available -gt 0 -and $score -ge $target

          $result = [ordered]@{
            status = if ($passed) { "pass" } else { "fail" }
            repository = $env:REPOSITORY_NAME
            branch = $env:REF_NAME_VALUE
            target_readiness = $target
            readiness_score = $score
            signal_coverage = "$available/2"
            metrics = @{
              pipeline_total = $pipelineTotal
              pipeline_success_rate = $pipelineRate
              jira_ticket_coverage = $jiraCoverage
            }
            generated_at = [DateTimeOffset]::UtcNow.ToString("o")
          }
          $result | ConvertTo-Json -Depth 8 | Out-File -FilePath $outputPath -Encoding UTF8
          if (-not $passed -and $env:INPUT_ENFORCE_GATE -eq "true") {
            throw "Release readiness below target."
          }

      - name: Upload readiness artifact
        uses: actions/upload-artifact@v7
        with:
          name: release-readiness-gate-${{ github.run_id }}
          path: gitgov-evidence
          if-no-files-found: error
          retention-days: 30
'@
}

function Get-QualityGatePolicyMatrixWorkflow {
  return @'
# Generated by GitGov workflow template generation.
# Documents the selected quality-gate policy preset as executable CI evidence.
name: GitGov Quality Gate Policy Matrix

on:
  pull_request:
  push:
    branches: ["__DEFAULT_BRANCH__"]
  workflow_dispatch:

permissions:
  contents: read

jobs:
  quality-gate-policy:
    name: Validate selected quality policy
    runs-on: ubuntu-latest
    steps:
      - name: Emit policy evidence
        shell: pwsh
        env:
          POLICY_PRESET: "__POLICY_PRESET__"
          READINESS_TARGET: "__READINESS_TARGET__"
        run: |
          $ErrorActionPreference = "Stop"
          $known = @("audit-only", "moderate", "strict")
          if ($env:POLICY_PRESET -notin $known) {
            throw "Unsupported policy preset: $env:POLICY_PRESET"
          }
          New-Item -ItemType Directory -Force -Path "gitgov-evidence" | Out-Null
          $rules = [ordered]@{
            policy_preset = $env:POLICY_PRESET
            readiness_target = [int]$env:READINESS_TARGET
            ticket_traceability = if ($env:POLICY_PRESET -eq "audit-only") { "recommended" } else { "required" }
            critical_high_vulnerabilities = if ($env:POLICY_PRESET -eq "audit-only") { "report-only" } else { "block-reachable" }
            generated_at = [DateTimeOffset]::UtcNow.ToString("o")
          }
          $rules | ConvertTo-Json -Depth 6 | Out-File -FilePath "gitgov-evidence/quality-gate-policy.json" -Encoding UTF8
          Write-Host "PASS: quality gate policy preset is valid."

      - name: Upload policy artifact
        uses: actions/upload-artifact@v7
        with:
          name: quality-gate-policy-${{ github.run_id }}
          path: gitgov-evidence
          if-no-files-found: error
          retention-days: 30
'@
}

function Get-SonarGovernanceWorkflow {
  return @'
# Generated by GitGov workflow template generation.
# SonarQube governance evidence when the selected runner can reach the service.
name: GitGov SonarQube Governance

on:
  workflow_dispatch:
  schedule:
    - cron: "37 13 * * 2"

permissions:
  contents: read

jobs:
  sonarqube-governance:
    name: Query SonarQube quality gate
    runs-on: ubuntu-latest
    steps:
      - name: Check SonarQube status
        shell: pwsh
        env:
          SONAR_HOST_URL: ${{ vars.SONAR_HOST_URL }}
          SONAR_PROJECT_KEY: ${{ vars.SONAR_PROJECT_KEY }}
          SONAR_TOKEN: ${{ secrets.SONAR_TOKEN }}
          RUN_ID_VALUE: ${{ github.run_id }}
        run: |
          $ErrorActionPreference = "Stop"
          New-Item -ItemType Directory -Force -Path "gitgov-evidence" | Out-Null
          $outputPath = "gitgov-evidence/sonarqube-governance-$env:RUN_ID_VALUE.json"
          $status = "skipped"
          $reason = "missing_sonar_configuration"
          $qualityGate = $null

          if (-not [string]::IsNullOrWhiteSpace($env:SONAR_HOST_URL) -and -not [string]::IsNullOrWhiteSpace($env:SONAR_PROJECT_KEY)) {
            $hostUrl = $env:SONAR_HOST_URL.TrimEnd("/")
            if ($hostUrl -match '^https?://(localhost|127\.0\.0\.1)(:\d+)?$') {
              $reason = "sonarqube_is_local_to_runner"
            } else {
              $headers = @{}
              if (-not [string]::IsNullOrWhiteSpace($env:SONAR_TOKEN)) {
                $headers.Authorization = "Bearer $env:SONAR_TOKEN"
              }
              $qualityGate = Invoke-RestMethod -Method GET -Uri "$hostUrl/api/qualitygates/project_status?projectKey=$([Uri]::EscapeDataString($env:SONAR_PROJECT_KEY))" -Headers $headers
              $status = "ok"
              $reason = "quality_gate_queried"
            }
          }

          $result = [ordered]@{
            status = $status
            reason = $reason
            project_key = $env:SONAR_PROJECT_KEY
            quality_gate = $qualityGate
            generated_at = [DateTimeOffset]::UtcNow.ToString("o")
          }
          $result | ConvertTo-Json -Depth 10 | Out-File -FilePath $outputPath -Encoding UTF8
          Write-Host "Wrote SonarQube governance evidence: $outputPath"

      - name: Upload SonarQube evidence
        uses: actions/upload-artifact@v7
        with:
          name: sonarqube-governance-${{ github.run_id }}
          path: gitgov-evidence
          if-no-files-found: error
          retention-days: 30
'@
}

function Get-ProductVulnerabilityReviewWorkflow {
  return @'
# Generated by GitGov workflow template generation.
# Runs a portable dependency review baseline. Extend with product-specific reachability triage before blocking releases.
name: GitGov Product Vulnerability Review

on:
  workflow_dispatch:
    inputs:
      mode:
        description: "Review mode"
        required: false
        default: "__REVIEW_MODE__"
        type: choice
        options:
          - DependenciesOnly
          - StaticOnly
  schedule:
    - cron: "41 12 * * 4"

permissions:
  contents: read

jobs:
  product-vulnerability-review:
    name: Run vulnerability review
    runs-on: ubuntu-latest
    timeout-minutes: 60
    steps:
      - name: Checkout
        uses: actions/checkout@v6

      - name: Setup Node.js
        uses: actions/setup-node@v6
        with:
          node-version: 20

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Run baseline review
        shell: pwsh
        env:
          RUN_ID_VALUE: ${{ github.run_id }}
          REVIEW_MODE: ${{ inputs.mode }}
        run: |
          $ErrorActionPreference = "Continue"
          New-Item -ItemType Directory -Force -Path "gitgov-evidence/product-vulnerability-review" | Out-Null
          $findings = New-Object System.Collections.Generic.List[object]
          $checks = New-Object System.Collections.Generic.List[object]

          if (Test-Path "package-lock.json") {
            npm audit --json | Out-File -FilePath "gitgov-evidence/product-vulnerability-review/npm-audit.json" -Encoding UTF8
            $checks.Add([ordered]@{ name = "npm_audit"; status = if ($LASTEXITCODE -eq 0) { "pass" } else { "finding" } }) | Out-Null
          }

          if (Test-Path "pnpm-lock.yaml") {
            corepack enable
            pnpm audit --json | Out-File -FilePath "gitgov-evidence/product-vulnerability-review/pnpm-audit.json" -Encoding UTF8
            $checks.Add([ordered]@{ name = "pnpm_audit"; status = if ($LASTEXITCODE -eq 0) { "pass" } else { "finding" } }) | Out-Null
          }

          if (Test-Path "Cargo.lock") {
            cargo install cargo-audit --locked
            cargo audit --json | Out-File -FilePath "gitgov-evidence/product-vulnerability-review/cargo-audit.json" -Encoding UTF8
            $checks.Add([ordered]@{ name = "cargo_audit"; status = if ($LASTEXITCODE -eq 0) { "pass" } else { "finding" } }) | Out-Null
          }

          if ($checks.Count -eq 0) {
            $checks.Add([ordered]@{ name = "baseline"; status = "skipped"; reason = "no_supported_lockfiles_found" }) | Out-Null
          }

          $findingCount = @($checks | Where-Object { $_.status -eq "finding" }).Count
          $summary = [ordered]@{
            status = if ($findingCount -eq 0) { "pass" } else { "findings" }
            findings = $findingCount
            checks = @($checks)
            generated_at = [DateTimeOffset]::UtcNow.ToString("o")
          }
          $summary | ConvertTo-Json -Depth 8 | Out-File -FilePath "gitgov-evidence/product-vulnerability-review/summary.json" -Encoding UTF8
          if ($findingCount -gt 0) {
            Write-Warning "Dependency review reported findings. Triage reachability before release enforcement."
          }

      - name: Upload review evidence
        uses: actions/upload-artifact@v7
        with:
          name: product-vulnerability-review-${{ github.run_id }}
          path: gitgov-evidence/product-vulnerability-review
          if-no-files-found: error
          retention-days: 30
'@
}

function Get-ReleaseGovernanceGateWorkflow {
  return @'
# Generated by GitGov workflow template generation.
# Evaluates release approval policy through GitGov. It is manual and opt-in.
name: GitGov Release Governance Gate

on:
  workflow_dispatch:
    inputs:
      org_name:
        description: "GitGov organization scope"
        required: false
        default: ""
        type: string
      release_id:
        description: "Release identifier to evaluate"
        required: false
        default: ""
        type: string
      environment:
        description: "Release environment"
        required: false
        default: "__RELEASE_GOVERNANCE_ENVIRONMENT__"
        type: string
      evidence_packet_hash:
        description: "Optional SHA-256 evidence packet hash"
        required: false
        default: ""
        type: string
      enforce_gate:
        description: "Fail only when explicitly blocking policy is not satisfied"
        required: false
        default: __RELEASE_GOVERNANCE_ENFORCE_GATE__
        type: boolean
      fail_on_would_block:
        description: "Also fail on advisory/would-block results"
        required: false
        default: __RELEASE_GOVERNANCE_FAIL_ON_WOULD_BLOCK__
        type: boolean

permissions:
  contents: read

jobs:
  release-governance:
    name: Evaluate release governance
    runs-on: ubuntu-latest
    steps:
      - name: Evaluate policy
        shell: pwsh
        env:
          GITGOV_URL: ${{ vars.GITGOV_URL }}
          GITGOV_API_KEY: ${{ secrets.GITGOV_API_KEY }}
          REPOSITORY_NAME: ${{ github.repository }}
          REF_NAME_VALUE: ${{ github.ref_name }}
          SHA_VALUE: ${{ github.sha }}
          RUN_ID_VALUE: ${{ github.run_id }}
          ACTOR_VALUE: ${{ github.actor }}
          INPUT_ORG_NAME: ${{ inputs.org_name }}
          INPUT_RELEASE_ID: ${{ inputs.release_id }}
          INPUT_ENVIRONMENT: ${{ inputs.environment }}
          INPUT_EVIDENCE_PACKET_HASH: ${{ inputs.evidence_packet_hash }}
          INPUT_ENFORCE_GATE: ${{ inputs.enforce_gate }}
          INPUT_FAIL_ON_WOULD_BLOCK: ${{ inputs.fail_on_would_block }}
        run: |
          $ErrorActionPreference = "Stop"
          New-Item -ItemType Directory -Force -Path "gitgov-evidence" | Out-Null
          $outputPath = "gitgov-evidence/release-governance-gate-$env:RUN_ID_VALUE.json"

          if ([string]::IsNullOrWhiteSpace($env:GITGOV_URL) -or [string]::IsNullOrWhiteSpace($env:GITGOV_API_KEY)) {
            $result = [ordered]@{
              status = "skipped"
              reason = "missing_gitgov_url_or_api_key"
              repository = $env:REPOSITORY_NAME
              generated_at = [DateTimeOffset]::UtcNow.ToString("o")
            }
            $result | ConvertTo-Json -Depth 8 | Out-File -FilePath $outputPath -Encoding UTF8
            if ($env:INPUT_ENFORCE_GATE -eq "true" -or $env:INPUT_FAIL_ON_WOULD_BLOCK -eq "true") {
              throw "Missing GITGOV_URL or GITGOV_API_KEY for strict release governance gate."
            }
            Write-Warning "Skipping release governance gate because configuration is missing."
            exit 0
          }

          $releaseId = $env:INPUT_RELEASE_ID
          if ([string]::IsNullOrWhiteSpace($releaseId)) { $releaseId = $env:REF_NAME_VALUE }
          if ([string]::IsNullOrWhiteSpace($releaseId)) { $releaseId = "manual-$env:RUN_ID_VALUE" }

          $environment = $env:INPUT_ENVIRONMENT
          if ([string]::IsNullOrWhiteSpace($environment)) { $environment = "__RELEASE_GOVERNANCE_ENVIRONMENT__" }
          $targetSha = $env:SHA_VALUE

          if (-not [string]::IsNullOrWhiteSpace($env:INPUT_EVIDENCE_PACKET_HASH)) {
            $payload = [ordered]@{
              org_name = if ([string]::IsNullOrWhiteSpace($env:INPUT_ORG_NAME)) { $null } else { $env:INPUT_ORG_NAME }
              release_id = $releaseId
              repository_full_name = $env:REPOSITORY_NAME
              branch = $env:REF_NAME_VALUE
              target_sha = $targetSha
              environment = $environment
              deployer = if ([string]::IsNullOrWhiteSpace($env:ACTOR_VALUE)) { "github-actions" } else { $env:ACTOR_VALUE }
              ticket_id = $releaseId
              evidence_packet_hash = $env:INPUT_EVIDENCE_PACKET_HASH
              requested_by = "github-actions"
              deployment_run_id = $env:RUN_ID_VALUE
              metadata = @{ source = "gitgov-generated-release-governance-gate"; workflow = "GitGov Release Governance Gate" }
            }
          } else {
            $result = [ordered]@{
              status = "skipped"
              reason = "missing_release_bound_evidence_packet_hash"
              repository = $env:REPOSITORY_NAME
              release_id = $releaseId
              environment = $environment
              generated_at = [DateTimeOffset]::UtcNow.ToString("o")
            }
            $result | ConvertTo-Json -Depth 8 | Out-File -FilePath $outputPath -Encoding UTF8
            if ($env:INPUT_ENFORCE_GATE -eq "true" -or $env:INPUT_FAIL_ON_WOULD_BLOCK -eq "true") {
              throw "Missing release-bound GitGov evidence packet hash for strict release governance gate."
            }
            Write-Warning "Skipping release governance gate because release-bound evidence is missing."
            exit 0
          }

          $baseUrl = $env:GITGOV_URL.TrimEnd("/")
          $headers = @{ Authorization = "Bearer $env:GITGOV_API_KEY"; Accept = "application/json"; "Content-Type" = "application/json" }
          $authorization = Invoke-RestMethod -Method POST -Uri "$baseUrl/deployment-gates/authorize" -Headers $headers -Body ($payload | ConvertTo-Json -Depth 8)
          $result = [ordered]@{
            status = "authorized"
            repository = $env:REPOSITORY_NAME
            release_id = $releaseId
            environment = $environment
            enforce = ($env:INPUT_ENFORCE_GATE -eq "true")
            fail_on_would_block = ($env:INPUT_FAIL_ON_WOULD_BLOCK -eq "true")
            authorization = [ordered]@{
              authorization_id = $authorization.authorization_id
              decision = $authorization.decision
              approved = $authorization.approved
              blocking = $authorization.blocking
              would_block = $authorization.would_block
              reason = $authorization.reason
              warnings = $authorization.warnings
              blocked_by = $authorization.blocked_by
              policy_checksum = $authorization.policy_checksum
              break_glass_eligible = $authorization.break_glass_eligible
            }
            evaluation = [ordered]@{
              status = $authorization.evaluation.status
              policy_mode = $authorization.evaluation.policy.mode
              policy_enforcement = $authorization.evaluation.policy.enforcement
              policy_satisfied = $authorization.evaluation.policy_satisfied
              blocking = $authorization.evaluation.blocking
              would_block = $authorization.evaluation.would_block
              valid_approval_count = $authorization.evaluation.valid_approval_count
              required_approval_count = $authorization.evaluation.required_approval_count
              issues = $authorization.evaluation.issues
              next_steps = $authorization.evaluation.next_steps
            }
            generated_at = [DateTimeOffset]::UtcNow.ToString("o")
            safety = @{
              contains_secret_values = $false
              prints_authorization_header = $false
            }
          }
          $result | ConvertTo-Json -Depth 10 | Out-File -FilePath $outputPath -Encoding UTF8

          if ($env:INPUT_ENFORCE_GATE -eq "true" -and $authorization.blocking -eq $true) {
            throw "Release governance blocking policy is not satisfied."
          }
          if ($env:INPUT_FAIL_ON_WOULD_BLOCK -eq "true" -and $authorization.would_block -eq $true) {
            throw "Release governance would block."
          }

      - name: Upload release governance artifact
        uses: actions/upload-artifact@v7
        with:
          name: release-governance-gate-${{ github.run_id }}
          path: gitgov-evidence
          if-no-files-found: error
          retention-days: 30
'@
}

function Get-VulnerabilityTrendEnforcementWorkflow {
  return @'
# Generated by GitGov workflow template generation.
# Enforces artifact presence and accepted finding baseline for the latest review summary.
name: GitGov Vulnerability Trend Enforcement

on:
  workflow_dispatch:
  schedule:
    - cron: "13 13 * * 5"

permissions:
  actions: read
  contents: read

jobs:
  trend-enforcement:
    name: Enforce vulnerability trend baseline
    runs-on: ubuntu-latest
    steps:
      - name: Enforce latest review artifact presence
        shell: pwsh
        env:
          GH_TOKEN: ${{ github.token }}
          REPOSITORY_NAME: ${{ github.repository }}
          RUN_ID_VALUE: ${{ github.run_id }}
          ARTIFACT_PREFIX: "product-vulnerability-review-"
          ACCEPTED_FINDING_BASELINE: "__ACCEPTED_FINDING_BASELINE__"
        run: |
          $ErrorActionPreference = "Stop"
          New-Item -ItemType Directory -Force -Path "gitgov-evidence" | Out-Null
          $headers = @{
            Authorization = "Bearer $env:GH_TOKEN"
            Accept = "application/vnd.github+json"
            "X-GitHub-Api-Version" = "2022-11-28"
          }
          $uri = "https://api.github.com/repos/$env:REPOSITORY_NAME/actions/artifacts?per_page=100"
          $response = Invoke-RestMethod -Method GET -Uri $uri -Headers $headers
          $latest = @($response.artifacts | Where-Object { $_.name -like "$env:ARTIFACT_PREFIX*" -and $_.expired -ne $true } | Sort-Object created_at -Descending | Select-Object -First 1)
          $status = if ($latest.Count -gt 0) { "pass" } else { "fail" }
          $result = [ordered]@{
            status = $status
            accepted_finding_baseline = [int]$env:ACCEPTED_FINDING_BASELINE
            latest_artifact_name = if ($latest.Count -eq 0) { $null } else { $latest[0].name }
            latest_artifact_id = if ($latest.Count -eq 0) { $null } else { $latest[0].id }
            generated_at = [DateTimeOffset]::UtcNow.ToString("o")
          }
          $result | ConvertTo-Json -Depth 8 | Out-File -FilePath "gitgov-evidence/product-vulnerability-review-trend-enforcement-$env:RUN_ID_VALUE.json" -Encoding UTF8
          if ($status -ne "pass") {
            throw "No fresh product vulnerability review artifact was found."
          }

      - name: Upload enforcement artifact
        uses: actions/upload-artifact@v7
        with:
          name: product-vulnerability-review-trend-enforcement
          path: gitgov-evidence
          if-no-files-found: error
          retention-days: 30
'@
}

$profile = $null
$profileReleaseGovernance = $null
if (-not [string]::IsNullOrWhiteSpace($ProfilePath)) {
  if (-not (Test-Path -LiteralPath $ProfilePath)) {
    Fail-Templates "Profile file not found: $ProfilePath"
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
  Fail-Templates "Missing -CustomerName or profile customer_name."
}
if ([string]::IsNullOrWhiteSpace($RepositoryFullName)) {
  Fail-Templates "Missing -RepositoryFullName or profile repository_full_name."
}
if ($RepositoryFullName -notmatch '^[^/\s]+/[^/\s]+$') {
  Fail-Templates "-RepositoryFullName must look like owner/repo."
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
$releaseGovernanceRequiresFormalApproval = Test-ReleaseGovernanceRequiresFormalApproval $releaseGovernance
$releaseGovernanceGatePolicy = Get-ReleaseGovernanceGatePolicy $releaseGovernance

$unknownProviders = @($providersNormalized | Where-Object { $_ -notin $knownProviders })
if ($unknownProviders.Count -gt 0) {
  Fail-Templates "Unknown provider(s): $($unknownProviders -join ', '). Known providers: $($knownProviders -join ', ')."
}

$unknownModules = @($modulesNormalized | Where-Object { $_ -notin $knownModules })
if ($unknownModules.Count -gt 0) {
  Fail-Templates "Unknown module(s): $($unknownModules -join ', '). Known modules: $($knownModules -join ', ')."
}

if ($releaseGovernanceRequiresFormalApproval -and $modulesNormalized -notcontains "formal-approval") {
  Fail-Templates "Non-record-only release governance requires the formal-approval module. Use record-only for non-blocking defaults."
}

if (($modulesNormalized -contains "traceability") -and [string]::IsNullOrWhiteSpace($JiraProjectKey)) {
  Fail-Templates "Traceability module requires -JiraProjectKey or profile jira_project_key."
}

if (-not [string]::IsNullOrWhiteSpace($JiraProjectKey) -and $JiraProjectKey -notmatch '^[A-Z][A-Z0-9]+$') {
  Fail-Templates "-JiraProjectKey must use uppercase letters/numbers, for example EX."
}

if ((Test-Path -LiteralPath $OutputDir) -and -not $Force) {
  $existingFiles = @(Get-ChildItem -LiteralPath $OutputDir -File -Recurse -ErrorAction SilentlyContinue)
  if ($existingFiles.Count -gt 0) {
    Fail-Templates "Output directory already contains files. Use -Force or choose another -OutputDir."
  }
}

$readinessTarget = switch ($PolicyPreset) {
  "audit-only" { 0 }
  "moderate" { 75 }
  "strict" { 85 }
}
$enforceGate = if ($PolicyPreset -eq "audit-only") { "false" } else { "true" }
$reviewMode = if ($PolicyPreset -eq "strict") { "StaticOnly" } else { "DependenciesOnly" }
$blockReachableFindings = "false"
$acceptedFindingBaseline = if ($PolicyPreset -eq "audit-only") { "999" } else { "1" }
$trendEnforcementRequired = $PolicyPreset -eq "strict" -or ($modulesNormalized -contains "trend-enforcement")
$releaseGovernanceGateRequired = $releaseGovernanceRequiresFormalApproval -and ($modulesNormalized -contains "formal-approval")
$releaseGovernanceEnforceGate = if ($releaseGovernanceGatePolicy.mode -in @("approval-required", "quorum-required")) { "true" } else { "false" }
$releaseGovernanceFailOnWouldBlock = "false"

$tokens = @{
  DEFAULT_BRANCH = $DefaultBranch
  JIRA_PROJECT_KEY = $JiraProjectKey
  POLICY_PRESET = $PolicyPreset
  READINESS_TARGET = [string]$readinessTarget
  ENFORCE_GATE = $enforceGate
  REVIEW_MODE = $reviewMode
  BLOCK_REACHABLE_FINDINGS = $blockReachableFindings
  ACCEPTED_FINDING_BASELINE = $acceptedFindingBaseline
  RELEASE_GOVERNANCE_ENVIRONMENT = $releaseGovernanceGatePolicy.environment
  RELEASE_GOVERNANCE_ENFORCE_GATE = $releaseGovernanceEnforceGate
  RELEASE_GOVERNANCE_FAIL_ON_WOULD_BLOCK = $releaseGovernanceFailOnWouldBlock
}

$templatePlan = New-Object System.Collections.Generic.List[object]
$variablePlan = New-Object System.Collections.Generic.List[object]
$secretPlan = New-Object System.Collections.Generic.List[object]
$manualSteps = New-Object System.Collections.Generic.List[object]
$openProductGaps = New-Object System.Collections.Generic.List[object]

function Add-Template {
  param(
    [Parameter(Mandatory = $true)][string]$RelativePath,
    [Parameter(Mandatory = $true)][string]$Reason,
    [Parameter(Mandatory = $true)][string]$Content,
    [string[]]$Modules = @(),
    [string[]]$Providers = @()
  )

  $outputPath = Join-TemplatePath -Root $OutputDir -RelativePath $RelativePath
  Set-TextFile -Path $outputPath -Value (Resolve-Template -Template $Content -Tokens $tokens)
  Add-UniqueObject $templatePlan ([pscustomobject]@{
      file = $RelativePath
      reason = $Reason
      modules = @($Modules)
      providers = @($Providers)
      requires_review_before_install = $true
    }) "file"
}

function Add-Variable {
  param(
    [Parameter(Mandatory = $true)][string]$Name,
    [Parameter(Mandatory = $true)][string]$Purpose,
    [string]$Example = ""
  )

  Add-UniqueObject $variablePlan ([pscustomobject]@{
      name = $Name
      scope = "GitHub Actions variable"
      purpose = $Purpose
      example = $Example
    }) "name"
}

function Add-Secret {
  param(
    [Parameter(Mandatory = $true)][string]$Name,
    [Parameter(Mandatory = $true)][string]$Purpose
  )

  Add-UniqueObject $secretPlan ([pscustomobject]@{
      name = $Name
      scope = "GitHub Actions secret"
      purpose = $Purpose
      value_policy = "secret value only, never committed or printed"
    }) "name"
}

Add-Template ".github/workflows/ci.yml" "core build, lint, typecheck, and tests" (Get-CiWorkflow)
Add-Template ".github/workflows/secret-scan.yml" "baseline publication and secret-file guard" (Get-SecretScanWorkflow)

if ($modulesNormalized -contains "traceability") {
  Add-Template ".github/workflows/public-naming-guard.yml" "Jira-style branch, PR title, and commit traceability" (Get-PublicNamingGuardWorkflow) @("traceability") @("github", "jira")
  Add-UniqueObject $manualSteps ([pscustomobject]@{ step = "Review traceability pattern"; detail = "Confirm $JiraProjectKey-123 is the customer's accepted branch, PR title, and commit subject format." }) "step"
}

if ($modulesNormalized -contains "github-evidence") {
  Add-Variable "GITGOV_URL" "GitGov API base URL" "https://gitgov-api.example.com"
  Add-Secret "GITGOV_API_KEY" "GitGov API authentication for workflow evidence"
  Add-Template ".github/workflows/github-evidence-report.yml" "GitGov stats evidence artifact" (Get-GitGovStatsWorkflow) @("github-evidence") @("github")
  if ($modulesNormalized -contains "artifact-monitoring") {
    Add-Template ".github/workflows/github-evidence-artifact-monitor.yml" "freshness monitor for GitHub evidence artifacts" (Get-ArtifactMonitorWorkflow "GitGov Evidence Artifact Monitor" "github-evidence-report-" "07 14 * * 2" "github-evidence-artifact-monitor") @("github-evidence", "artifact-monitoring") @("github")
  }
  Add-Template ".github/workflows/github-evidence-trend-report.yml" "recent GitHub evidence artifact trend inventory" (Get-ArtifactTrendWorkflow "GitGov Evidence Trend Report" "github-evidence-report-" "17 14 * * 2" "github-evidence-trend-report") @("github-evidence") @("github")
}

if ($modulesNormalized -contains "release-readiness") {
  Add-Variable "GITGOV_URL" "GitGov API base URL" "https://gitgov-api.example.com"
  Add-Secret "GITGOV_API_KEY" "GitGov API authentication for readiness evidence"
  Add-Template ".github/workflows/release-readiness-gate.yml" "release readiness score and evidence artifact" (Get-ReleaseReadinessWorkflow) @("release-readiness") @("github", "jira", "jenkins")
}

if ($releaseGovernanceGateRequired) {
  Add-Variable "GITGOV_URL" "GitGov API base URL" "https://gitgov-api.example.com"
  Add-Secret "GITGOV_API_KEY" "GitGov API authentication for release governance evaluation"
  Add-Template ".github/workflows/release-governance-gate.yml" "optional release governance policy evaluator; blocking only when customer-selected policy requires it" (Get-ReleaseGovernanceGateWorkflow) @("formal-approval") @("github")
  if ($modulesNormalized -contains "artifact-monitoring") {
    Add-Template ".github/workflows/release-governance-gate-artifact-monitor.yml" "freshness monitor for release governance gate artifacts; generated only after explicit release governance opt-in" (Get-ArtifactMonitorWorkflow "GitGov Release Governance Gate Artifact Monitor" "release-governance-gate-" "29 15 * * 1" "release-governance-gate-artifact-monitor" 720 $false) @("formal-approval", "artifact-monitoring") @("github")
  }
}

if ($modulesNormalized -contains "quality-gates") {
  Add-Template ".github/workflows/quality-gate-policy-matrix.yml" "selected quality-gate policy evidence" (Get-QualityGatePolicyMatrixWorkflow) @("quality-gates") @("github")
  if ($providersNormalized -contains "sonarqube") {
    Add-Variable "SONAR_HOST_URL" "SonarQube endpoint reachable by the selected runner" "https://sonarqube.example.com"
    Add-Variable "SONAR_PROJECT_KEY" "SonarQube project key" "example_org_example_repo"
    Add-Secret "SONAR_TOKEN" "Optional SonarQube API token for governance evidence"
    Add-Template ".github/workflows/sonar-governance.yml" "SonarQube quality gate evidence when reachable" (Get-SonarGovernanceWorkflow) @("quality-gates") @("sonarqube")
  }
}

if ($modulesNormalized -contains "vulnerability-review") {
  Add-Template ".github/workflows/product-vulnerability-review.yml" "baseline product vulnerability review evidence" (Get-ProductVulnerabilityReviewWorkflow) @("vulnerability-review") @("github")
  if ($modulesNormalized -contains "artifact-monitoring") {
    Add-Template ".github/workflows/product-vulnerability-review-artifact-monitor.yml" "freshness monitor for vulnerability review artifacts" (Get-ArtifactMonitorWorkflow "GitGov Product Vulnerability Review Artifact Monitor" "product-vulnerability-review-" "53 12 * * 5" "product-vulnerability-review-artifact-monitor") @("vulnerability-review", "artifact-monitoring") @("github")
  }
  Add-Template ".github/workflows/product-vulnerability-review-trend-report.yml" "recent vulnerability review artifact trend inventory" (Get-ArtifactTrendWorkflow "GitGov Product Vulnerability Review Trend Report" "product-vulnerability-review-" "03 13 * * 5" "product-vulnerability-review-trend-report") @("vulnerability-review") @("github")
}

if ($trendEnforcementRequired) {
  Add-Template ".github/workflows/product-vulnerability-review-trend-enforcement.yml" "enforcement gate for vulnerability review artifact presence" (Get-VulnerabilityTrendEnforcementWorkflow) @("trend-enforcement", "vulnerability-review") @("github")
}

if ($modulesNormalized -contains "evidence-packets") {
  Add-UniqueObject $manualSteps ([pscustomobject]@{ step = "Validate Evidence Packets"; detail = "Confirm ticket-scoped Evidence Packets return expected commits, PRs, pipeline evidence, and quality evidence before customer release reviews rely on them." }) "step"
}

if ($modulesNormalized -contains "formal-approval") {
  $releasePolicyDetail = if ($releaseGovernance.mode -eq "record-only") {
    "Default record-only mode stores release approval evidence and does not block customer releases. Environment overrides: $(Get-ReleaseGovernanceOverrideSummary $releaseGovernance)."
  } else {
    "Customer selected $($releaseGovernance.mode) for $($releaseGovernance.environment); review this explicit opt-in policy before installing any blocking workflow."
  }
  Add-UniqueObject $manualSteps ([pscustomobject]@{ step = "Review release approval policy"; detail = $releasePolicyDetail }) "step"
  if ($releaseGovernanceGateRequired) {
    Add-UniqueObject $manualSteps ([pscustomobject]@{ step = "Validate release governance gate manually"; detail = "Run release-governance-gate.yml with workflow_dispatch before using it as a blocking release check. Default enforcement follows the selected release_governance mode or matching environment override." }) "step"
    if ($modulesNormalized -contains "artifact-monitoring") {
      Add-UniqueObject $manualSteps ([pscustomobject]@{ step = "Validate release governance gate artifact"; detail = "Run release-governance-gate-artifact-monitor.yml after at least one successful gate run to confirm the release governance evidence artifact exists and is still fresh." }) "step"
    }
  }
}

if ($providersNormalized -contains "jira") {
  Add-UniqueObject $manualSteps ([pscustomobject]@{ step = "Connect Jira"; detail = "Configure the Jira project key and signed Jira webhook before enforcing traceability." }) "step"
}
if ($providersNormalized -contains "jenkins") {
  Add-UniqueObject $manualSteps ([pscustomobject]@{ step = "Connect Jenkins"; detail = "Publish Jenkins pipeline telemetry to GitGov before enforcing release readiness." }) "step"
}
if ($providersNormalized -contains "sonarqube") {
  Add-UniqueObject $manualSteps ([pscustomobject]@{ step = "Connect SonarQube"; detail = "Use a SonarQube URL reachable by the selected runner; local-only SonarQube should use a self-hosted runner." }) "step"
}
if ($providersNormalized -contains "render") {
  Add-UniqueObject $manualSteps ([pscustomobject]@{ step = "Connect Render deployment evidence"; detail = "Record service health and deployment evidence without storing provider tokens in workflow files." }) "step"
}
if ($providersNormalized -contains "vercel") {
  Add-UniqueObject $manualSteps ([pscustomobject]@{ step = "Connect Vercel deployment evidence"; detail = "Record deployment and preview evidence through approved provider integration paths." }) "step"
}
Add-UniqueObject $manualSteps ([pscustomobject]@{ step = "Review generated YAML"; detail = "Install templates only after customer owners review commands, schedules, permissions, and branch names." }) "step"
Add-UniqueObject $manualSteps ([pscustomobject]@{ step = "Create GitHub Actions variables and secrets"; detail = "Create variable and secret names listed in the manifest. Never paste values into workflow YAML." }) "step"
Add-UniqueObject $manualSteps ([pscustomobject]@{ step = "Run workflow_dispatch first"; detail = "Validate each workflow manually before relying on schedules or blocking behavior." }) "step"

$generatedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
$manifest = [pscustomobject]@{
  generated_at = $generatedAtUtc
  customer_name = $CustomerName
  repository_full_name = $RepositoryFullName
  default_branch = $DefaultBranch
  jira_project_key = $JiraProjectKey
  policy_preset = $PolicyPreset
  release_governance = $releaseGovernance
  providers = @($providersNormalized)
  modules = @($modulesNormalized)
  workflow_templates = @($templatePlan.ToArray())
  variables = @($variablePlan.ToArray())
  secrets = @($secretPlan.ToArray())
  manual_steps = @($manualSteps.ToArray())
  open_product_gaps = @($openProductGaps.ToArray())
  safety = @{
    contains_secret_values = $false
    mutates_customer_repository = $false
    requires_manual_install_review = $true
  }
}

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$manifestPath = Join-Path $OutputDir "workflow-template-manifest.json"
$readmePath = Join-Path $OutputDir "README.md"
$manifest | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $manifestPath -Encoding UTF8

$readme = New-Object System.Collections.Generic.List[string]
$readme.Add("# GitGov Workflow Template Pack") | Out-Null
$readme.Add("") | Out-Null
$readme.Add(('Generated: `{0}`' -f $generatedAtUtc)) | Out-Null
$readme.Add("") | Out-Null
$readme.Add(('Customer: `{0}`' -f $CustomerName)) | Out-Null
$readme.Add(('Repository: `{0}`' -f $RepositoryFullName)) | Out-Null
$readme.Add(('Default branch: `{0}`' -f $DefaultBranch)) | Out-Null
$readme.Add(('Policy preset: `{0}`' -f $PolicyPreset)) | Out-Null
$readme.Add(('Release governance: `{0}`' -f $releaseGovernance.mode)) | Out-Null
$readme.Add(('Release enforcement: `{0}`' -f $releaseGovernance.enforcement)) | Out-Null
$readme.Add(('Release environment overrides: `{0}`' -f (Get-ReleaseGovernanceOverrideSummary $releaseGovernance))) | Out-Null
if (-not [string]::IsNullOrWhiteSpace($JiraProjectKey)) {
  $readme.Add(('Jira project key: `{0}`' -f $JiraProjectKey)) | Out-Null
}
$readme.Add("") | Out-Null
$readme.Add("## Release Governance") | Out-Null
$readme.Add("") | Out-Null
$readme.Add(('- Mode: `{0}`' -f $releaseGovernance.mode)) | Out-Null
$readme.Add(('- Environment: `{0}`' -f $releaseGovernance.environment)) | Out-Null
$readme.Add(('- Enforcement: `{0}`' -f $releaseGovernance.enforcement)) | Out-Null
$readme.Add(('- Environment overrides: `{0}`' -f (Get-ReleaseGovernanceOverrideSummary $releaseGovernance))) | Out-Null
if ($releaseGovernance.quorum.enabled) {
  foreach ($rule in $releaseGovernance.quorum.rules) {
    $readme.Add(('- Quorum `{0}`: `{1}` required' -f $rule.role, $rule.required)) | Out-Null
  }
} else {
  $readme.Add('- Quorum: `disabled`') | Out-Null
}
$readme.Add("") | Out-Null
$readme.Add("## Generated Templates") | Out-Null
$readme.Add("") | Out-Null
$readme.Add("| Workflow | Why |") | Out-Null
$readme.Add("|---|---|") | Out-Null
foreach ($template in $templatePlan) {
  $readme.Add(('| `{0}` | {1} |' -f (Escape-MarkdownCell $template.file), (Escape-MarkdownCell $template.reason))) | Out-Null
}
$readme.Add("") | Out-Null
$readme.Add("## Required Variables") | Out-Null
$readme.Add("") | Out-Null
if ($variablePlan.Count -eq 0) {
  $readme.Add("- None.") | Out-Null
} else {
  $readme.Add("| Name | Purpose | Example |") | Out-Null
  $readme.Add("|---|---|---|") | Out-Null
  foreach ($variable in $variablePlan) {
    $readme.Add(('| `{0}` | {1} | `{2}` |' -f $variable.name, (Escape-MarkdownCell $variable.purpose), (Escape-MarkdownCell $variable.example))) | Out-Null
  }
}
$readme.Add("") | Out-Null
$readme.Add("## Required Secrets") | Out-Null
$readme.Add("") | Out-Null
if ($secretPlan.Count -eq 0) {
  $readme.Add("- None.") | Out-Null
} else {
  $readme.Add("| Name | Purpose | Value Policy |") | Out-Null
  $readme.Add("|---|---|---|") | Out-Null
  foreach ($secret in $secretPlan) {
    $readme.Add(('| `{0}` | {1} | {2} |' -f $secret.name, (Escape-MarkdownCell $secret.purpose), (Escape-MarkdownCell $secret.value_policy))) | Out-Null
  }
}
$readme.Add("") | Out-Null
$readme.Add("## Manual Install Checklist") | Out-Null
$readme.Add("") | Out-Null
foreach ($step in $manualSteps) {
  $readme.Add(('- **{0}:** {1}' -f $step.step, $step.detail)) | Out-Null
}
if ($openProductGaps.Count -gt 0) {
  $readme.Add("") | Out-Null
  $readme.Add("## Open Product Gaps") | Out-Null
  $readme.Add("") | Out-Null
  foreach ($gap in $openProductGaps) {
    $readme.Add(('- **{0}:** {1}' -f $gap.gap, $gap.detail)) | Out-Null
  }
}
$readme.Add("") | Out-Null
$readme.Add("## Safety Notes") | Out-Null
$readme.Add("") | Out-Null
$readme.Add("- This pack contains workflow templates, variable names, and secret names only.") | Out-Null
$readme.Add("- It does not contain secret values.") | Out-Null
$readme.Add("- It does not mutate the customer repository automatically.") | Out-Null
$readme.Add("- Review generated commands and permissions before copying templates into `.github/workflows`.") | Out-Null
$readme.Add("- SonarQube is supported when reachable by the selected runner.") | Out-Null

Set-Content -LiteralPath $readmePath -Value $readme -Encoding UTF8

Write-Host "Wrote workflow templates: $OutputDir"
Write-Host "Wrote workflow template manifest: $manifestPath"
Write-Host "Wrote workflow template README: $readmePath"
