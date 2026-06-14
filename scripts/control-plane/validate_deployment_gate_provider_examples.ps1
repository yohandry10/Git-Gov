param(
  [string]$ExamplesDir = "docs/examples/deployment-gates",
  [string]$OutputPath = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")

function Resolve-RepoPath {
  param([Parameter(Mandatory = $true)][string]$Path)

  if ([System.IO.Path]::IsPathRooted($Path)) {
    return $Path
  }
  return Join-Path $repoRoot $Path
}

function Add-Finding {
  param(
    [System.Collections.Generic.List[object]]$Findings,
    [Parameter(Mandatory = $true)][string]$File,
    [Parameter(Mandatory = $true)][string]$Code,
    [Parameter(Mandatory = $true)][string]$Message
  )

  $Findings.Add([pscustomobject]@{
    file = $File
    code = $Code
    message = $Message
  }) | Out-Null
}

$resolvedExamplesDir = Resolve-RepoPath $ExamplesDir
$findings = New-Object System.Collections.Generic.List[object]
$expectedFiles = @(
  "README.md",
  "github-actions-deployment-gate.yml",
  "Jenkinsfile.deployment-gate",
  "gitlab-ci-deployment-gate.yml"
)

foreach ($file in $expectedFiles) {
  $path = Join-Path $resolvedExamplesDir $file
  if (-not (Test-Path -LiteralPath $path)) {
    Add-Finding -Findings $findings -File $file -Code "missing_file" -Message "Expected provider example file is missing."
  }
}

$providerFiles = @(
  "github-actions-deployment-gate.yml",
  "Jenkinsfile.deployment-gate",
  "gitlab-ci-deployment-gate.yml"
)
$requiredTokens = @(
  "/deployment-gates/authorize",
  "release_id",
  "repository_full_name",
  "branch",
  "target_sha",
  "environment",
  "deployer",
  "evidence_packet_hash",
  "requested_by",
  "deployment_run_id",
  "metadata"
)

foreach ($file in $providerFiles) {
  $path = Join-Path $resolvedExamplesDir $file
  if (-not (Test-Path -LiteralPath $path)) {
    continue
  }

  $content = Get-Content -LiteralPath $path -Raw
  foreach ($token in $requiredTokens) {
    if ($content -notmatch [regex]::Escape($token)) {
      Add-Finding -Findings $findings -File $file -Code "missing_required_token" -Message "Missing required deployment authorization token: $token"
    }
  }

  if ($content -match "/enterprise/release-governance/evaluate") {
    Add-Finding -Findings $findings -File $file -Code "uses_lower_level_evaluator" -Message "Provider example must call Deployment Gates authorization, not the lower-level evaluator."
  }

  if ($content -match "Bearer\s+[A-Za-z0-9_\-\.]{12,}") {
    Add-Finding -Findings $findings -File $file -Code "hardcoded_bearer_token" -Message "Example appears to hardcode a bearer token."
  }

  if ($content -match "(?m)GITGOV_API_KEY\s*[:=]\s*['""][^`${%][^'""]+['""]") {
    Add-Finding -Findings $findings -File $file -Code "hardcoded_api_key" -Message "Example appears to assign GITGOV_API_KEY directly instead of reading provider secrets."
  }

  if ($file -like "*.yml") {
    if ($content -notmatch "(?m)^\s{2,}artifacts:|actions/upload-artifact") {
      Add-Finding -Findings $findings -File $file -Code "missing_evidence_artifact" -Message "Provider example should preserve GitGov authorization evidence as an artifact."
    }
  }

  if ($content -notmatch "blocking" -or $content -notmatch "would_block") {
    Add-Finding -Findings $findings -File $file -Code "missing_decision_handling" -Message "Provider example should handle blocking and advisory would_block decisions."
  }
}

$readmePath = Join-Path $resolvedExamplesDir "README.md"
if (Test-Path -LiteralPath $readmePath) {
  $readme = Get-Content -LiteralPath $readmePath -Raw
  foreach ($providerName in @("GitHub Actions", "Jenkins", "GitLab CI")) {
    if ($readme -notmatch [regex]::Escape($providerName)) {
      Add-Finding -Findings $findings -File "README.md" -Code "missing_provider_readme" -Message "README does not describe $providerName."
    }
  }
}

$result = [ordered]@{
  generated_at = (Get-Date).ToUniversalTime().ToString("o")
  status = if ($findings.Count -eq 0) { "passed" } else { "failed" }
  examples_dir = $ExamplesDir
  checked_files = @($expectedFiles)
  required_tokens = @($requiredTokens)
  findings = @($findings.ToArray())
  safety = [ordered]@{
    prints_secret_values = $false
    validates_no_hardcoded_bearer_token = $true
    validates_no_lower_level_evaluator = $true
  }
}

$json = $result | ConvertTo-Json -Depth 8
if (-not [string]::IsNullOrWhiteSpace($OutputPath)) {
  $parent = Split-Path -Parent $OutputPath
  if (-not [string]::IsNullOrWhiteSpace($parent)) {
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
  }
  Set-Content -LiteralPath $OutputPath -Value $json -Encoding UTF8
}

Write-Output $json

if ($findings.Count -gt 0) {
  exit 1
}
