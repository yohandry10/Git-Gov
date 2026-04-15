param(
    [string]$ServerUrl = "http://127.0.0.1:3000",
    [string]$AdminApiKey = $env:ADMIN_API_KEY,
    [string]$DeveloperApiKeyA = $env:DEV_API_KEY_A,
    [string]$DeveloperApiKeyB = $env:DEV_API_KEY_B,
    [string]$PolicyRepo = "acme/repo",
    [switch]$ValidateOnly
)

$ErrorActionPreference = "Stop"

function Fail($message) {
    Write-Host "FAIL: $message"
    exit 1
}

function QuoteForShell($value) {
    $escaped = $value -replace "'", "'\\''"
    return "'$escaped'"
}

$gitBash = Join-Path ${env:ProgramFiles} 'Git\bin\bash.exe'
if (-not (Test-Path $gitBash)) {
    Fail "Git Bash not found at $gitBash."
}

$smokeScript = Join-Path (Split-Path -Parent $MyInvocation.MyCommand.Definition) 'policy_requests_multisession_smoke.sh'
if (-not (Test-Path $smokeScript)) {
    Fail "Smoke script not found at $smokeScript."
}

$keys = @{
    "ADMIN_API_KEY" = $AdminApiKey
    "DEV_API_KEY_A" = $DeveloperApiKeyA
    "DEV_API_KEY_B" = $DeveloperApiKeyB
}

foreach ($keyName in $keys.Keys) {
    if ([string]::IsNullOrWhiteSpace($keys[$keyName])) {
        Fail "Environment variable $keyName is required."
    }
}

if ($ValidateOnly) {
    Write-Host "Validating smoke script syntax..."
    & $gitBash -lc ("bash -n " + (QuoteForShell $smokeScript))
    Write-Host "Validation OK"
    exit 0
}

$envCommands = @(
    "export SERVER_URL=$(QuoteForShell $ServerUrl)",
    "export ADMIN_API_KEY=$(QuoteForShell $AdminApiKey)",
    "export DEV_API_KEY_A=$(QuoteForShell $DeveloperApiKeyA)",
    "export DEV_API_KEY_B=$(QuoteForShell $DeveloperApiKeyB)",
    "export POLICY_REPO=$(QuoteForShell $PolicyRepo)"
) -join '; '

Write-Host "Running governance multi-session smoke via Git Bash"
Write-Host "Server: $ServerUrl"
Write-Host "Repo:   $PolicyRepo"
Write-Host "API keys: Admin + 2 developers (value hidden)"

$command = "$envCommands; bash $(QuoteForShell $smokeScript)"
& $gitBash -lc $command
$exitCode = $LASTEXITCODE

if ($exitCode -ne 0) {
    Fail "governance-smoke failed (exit code $exitCode)."
}

Write-Host "governance-smoke completed (exit code 0)."
