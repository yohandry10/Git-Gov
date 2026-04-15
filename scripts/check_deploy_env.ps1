param(
    [string]$EnvFile = "gitgov/gitgov-server/.env",
    [switch]$NoEnvFile
)

$ErrorActionPreference = "Stop"

function Read-EnvFile {
    param([string]$Path)
    $data = @{}
    if (-not (Test-Path $Path)) {
        return $data
    }
    Get-Content $Path | ForEach-Object {
        $line = $_.Trim()
        if ($line -eq "" -or $line.StartsWith("#")) {
            return
        }
        $parts = $line -split "=", 2
        if ($parts.Count -ne 2) {
            return
        }
        $key = $parts[0].Trim()
        $value = $parts[1].Trim()
        if ($key -eq "") {
            return
        }
        $data[$key] = $value
    }
    return $data
}

$envValues = @{}
if (-not $NoEnvFile) {
    $envValues = Read-EnvFile -Path $EnvFile
}

function HasValue {
    param([string]$Name)
    if (${env:$Name}) {
        return @{ Exists = $true; Source = "env" }
    }
    if ($envValues.ContainsKey($Name)) {
        return @{ Exists = $true; Source = "envfile" }
    }
    return @{ Exists = $false }
}

$checks = @(
    "GITGOV_SERVER_ADDR",
    "GITGOV_API_KEY",
    "GITHUB_WEBHOOK_SECRET",
    "SUPABASE_URL"
)

$awsPairs = @(
    @("AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY"),
    @("Accesskey", "Secretaccesskey")
)

$missing = @()
Write-Host "Checking deployment prerequisites (values hidden)"
Write-Host ""

foreach ($name in $checks) {
    $result = HasValue -Name $name
    if ($result.Exists) {
        Write-Host ("PASS [{0}] source={1}" -f $name, $result.Source)
    } else {
        Write-Host ("FAIL [{0}] (missing)" -f $name)
        $missing += $name
    }
}

$awsFlavorOk = $false
foreach ($pair in $awsPairs) {
    $firstValue = HasValue -Name $pair[0]
    $secondValue = HasValue -Name $pair[1]
    if ($firstValue.Exists -and $secondValue.Exists) {
        Write-Host ("PASS [{0}/{1}] AWS credentials present" -f $pair[0], $pair[1])
        $awsFlavorOk = $true
        break
    }
}

if (-not $awsFlavorOk) {
    Write-Host "FAIL [AWS keys] missing cualquiera de los pares requeridos"
    $missing += "AWS_ACCESS_KEY_ID/AWS_SECRET_ACCESS_KEY or Accesskey/Secretaccesskey"
}

Write-Host ""
if ($missing.Count -eq 0) {
    Write-Host "Result: PASS (all required vars present)"
    exit 0
}

Write-Host ("Result: FAIL ({0} missing)" -f $missing.Count)
exit 1
