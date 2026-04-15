param(
    [string]$ServerUrl = "http://127.0.0.1:3000",
    [string]$ApiKey
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($ApiKey)) {
    $ApiKey = $env:GITGOV_API_KEY
}

if ([string]::IsNullOrWhiteSpace($ApiKey)) {
    Write-Host "FAIL [bootstrap] Missing API key. Use -ApiKey or set GITGOV_API_KEY."
    exit 1
}

$baseUrl = $ServerUrl.TrimEnd("/")
$keySource = if ($PSBoundParameters.ContainsKey("ApiKey")) { "-ApiKey" } else { "GITGOV_API_KEY" }

$authHeaders = @{
    "Authorization" = "Bearer $ApiKey"
}

$jsonHeaders = @{
    "Authorization" = "Bearer $ApiKey"
    "Content-Type"  = "application/json"
}

$passCount = 0
$failCount = 0

function Write-Pass {
    param(
        [string]$Name,
        [int]$StatusCode
    )
    Write-Host ("PASS [{0}] HTTP {1}" -f $Name, $StatusCode)
    $script:passCount++
}

function Write-Fail {
    param(
        [string]$Name,
        [int]$StatusCode,
        [string]$Reason
    )
    if ([string]::IsNullOrWhiteSpace($Reason)) {
        Write-Host ("FAIL [{0}] HTTP {1}" -f $Name, $StatusCode)
    } else {
        Write-Host ("FAIL [{0}] HTTP {1} - {2}" -f $Name, $StatusCode, $Reason)
    }
    $script:failCount++
}

function Invoke-HttpCheck {
    param(
        [ValidateSet("GET", "POST")]
        [string]$Method,
        [string]$Url,
        [hashtable]$Headers,
        [string]$Body
    )

    $statusCode = 0
    $content = ""
    $errorMessage = ""

    try {
        if ($Method -eq "POST") {
            $response = Invoke-WebRequest -Method $Method -Uri $Url -Headers $Headers -Body $Body -UseBasicParsing
        } else {
            $response = Invoke-WebRequest -Method $Method -Uri $Url -Headers $Headers -UseBasicParsing
        }

        $statusCode = [int]$response.StatusCode
        $content = [string]$response.Content
    } catch {
        $errorMessage = $_.Exception.Message
        $response = $_.Exception.Response

        if ($null -ne $response -and $null -ne $response.StatusCode) {
            $statusCode = [int]$response.StatusCode
        }

        if ($null -ne $response) {
            try {
                $stream = $response.GetResponseStream()
                if ($null -ne $stream) {
                    $reader = New-Object System.IO.StreamReader($stream)
                    $content = $reader.ReadToEnd()
                    $reader.Dispose()
                }
            } catch {
                # Keep the original error message if response content is not readable.
            }
        }
    }

    [pscustomobject]@{
        StatusCode   = $statusCode
        Content      = $content
        ErrorMessage = $errorMessage
    }
}

Write-Host "GitGov v19 smoke check"
Write-Host ("Server: {0}" -f $baseUrl)
Write-Host ("API key source: {0} (value hidden)" -f $keySource)
Write-Host ""

$healthResult = Invoke-HttpCheck -Method "GET" -Url "$baseUrl/health" -Headers @{} -Body $null
if ($healthResult.StatusCode -eq 200) {
    Write-Pass -Name "/health" -StatusCode $healthResult.StatusCode
} else {
    $healthReason = if ($healthResult.ErrorMessage) { $healthResult.ErrorMessage } else { $healthResult.Content }
    Write-Fail -Name "/health" -StatusCode $healthResult.StatusCode -Reason $healthReason
}

$eventUuid = [guid]::NewGuid().ToString().ToLowerInvariant()
$timestampMs = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
$eventsPayload = @{
    events = @(
        @{
            event_uuid = $eventUuid
            event_type = "commit"
            user_login = "v19_smoke"
            files = @()
            status = "success"
            timestamp = $timestampMs
        }
    )
    client_version = "v19-smoke-ps1"
} | ConvertTo-Json -Depth 6 -Compress

$eventsResult = Invoke-HttpCheck -Method "POST" -Url "$baseUrl/events" -Headers $jsonHeaders -Body $eventsPayload
if ($eventsResult.StatusCode -eq 200 -and $eventsResult.Content -match '"accepted"') {
    Write-Pass -Name "POST /events" -StatusCode $eventsResult.StatusCode
} else {
    $eventsReason = if ($eventsResult.ErrorMessage) { $eventsResult.ErrorMessage } else { $eventsResult.Content }
    Write-Fail -Name "POST /events" -StatusCode $eventsResult.StatusCode -Reason $eventsReason
}

$statsResult = Invoke-HttpCheck -Method "GET" -Url "$baseUrl/stats" -Headers $authHeaders -Body $null
if ($statsResult.StatusCode -eq 200) {
    Write-Pass -Name "/stats" -StatusCode $statsResult.StatusCode
} else {
    $statsReason = if ($statsResult.ErrorMessage) { $statsResult.ErrorMessage } else { $statsResult.Content }
    Write-Fail -Name "/stats" -StatusCode $statsResult.StatusCode -Reason $statsReason
}

$logsResult = Invoke-HttpCheck -Method "GET" -Url "$baseUrl/logs?limit=5&offset=0" -Headers $authHeaders -Body $null
if ($logsResult.StatusCode -eq 200) {
    Write-Pass -Name "/logs?limit=5&offset=0" -StatusCode $logsResult.StatusCode
} else {
    $logsReason = if ($logsResult.ErrorMessage) { $logsResult.ErrorMessage } else { $logsResult.Content }
    Write-Fail -Name "/logs?limit=5&offset=0" -StatusCode $logsResult.StatusCode -Reason $logsReason
}

Write-Host ""
Write-Host ("Summary: PASS={0} FAIL={1}" -f $passCount, $failCount)

if ($failCount -gt 0) {
    exit 1
}

exit 0
