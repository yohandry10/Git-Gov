param(
  [Parameter(Mandatory = $true)][string]$BaseUrl,
  [string]$ApiKey = "",
  [string]$ExpectedIp = "",
  [bool]$RequireHttps = $true,
  [int]$TimeoutSeconds = 15,
  [string]$OutputPath = "",
  [switch]$FailOnWarnings
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Add-CheckResult {
  param(
    [System.Collections.Generic.List[object]]$Bag,
    [string]$Check,
    [string]$Status,
    [string]$Details
  )
  $Bag.Add([pscustomobject]@{
    Check = $Check
    Status = $Status
    Details = $Details
  }) | Out-Null
}

function Invoke-StatusCode {
  param(
    [Parameter(Mandatory = $true)][string]$Uri,
    [ValidateSet("GET", "POST", "OPTIONS")][string]$Method = "GET",
    [hashtable]$Headers = @{},
    [string]$Body = ""
  )

  try {
    if ($Method -eq "POST") {
      $resp = Invoke-WebRequest -Uri $Uri -Method Post -Headers $Headers -Body $Body -TimeoutSec $TimeoutSeconds
    } elseif ($Method -eq "OPTIONS") {
      $resp = Invoke-WebRequest -Uri $Uri -Method Options -Headers $Headers -TimeoutSec $TimeoutSeconds
    } else {
      $resp = Invoke-WebRequest -Uri $Uri -Method Get -Headers $Headers -TimeoutSec $TimeoutSeconds
    }
    return @{
      code = [int]$resp.StatusCode
      body = $resp.Content
      error = $null
    }
  } catch {
    if ($_.Exception.Response) {
      $response = $_.Exception.Response
      $statusCode = [int]$response.StatusCode
      $reader = New-Object IO.StreamReader($response.GetResponseStream())
      $content = $reader.ReadToEnd()
      return @{
        code = $statusCode
        body = $content
        error = $_.Exception.Message
      }
    }
    return @{
      code = 0
      body = ""
      error = $_.Exception.Message
    }
  }
}

function Test-TlsCertificate {
  param(
    [Parameter(Mandatory = $true)][string]$HostName,
    [int]$Port = 443
  )

  $tcp = New-Object System.Net.Sockets.TcpClient
  try {
    $tcp.ReceiveTimeout = $TimeoutSeconds * 1000
    $tcp.SendTimeout = $TimeoutSeconds * 1000
    $tcp.Connect($HostName, $Port)
    $ssl = New-Object System.Net.Security.SslStream($tcp.GetStream(), $false, ({ $true }))
    $ssl.AuthenticateAsClient($HostName)
    $cert = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2($ssl.RemoteCertificate)
    return @{
      subject = $cert.Subject
      issuer = $cert.Issuer
      not_after = $cert.NotAfter
      thumbprint = $cert.Thumbprint
      days_remaining = [int]([math]::Floor(($cert.NotAfter.ToUniversalTime() - (Get-Date).ToUniversalTime()).TotalDays))
      error = $null
    }
  } catch {
    return @{
      subject = ""
      issuer = ""
      not_after = $null
      thumbprint = ""
      days_remaining = -1
      error = $_.Exception.Message
    }
  } finally {
    if ($null -ne $ssl) { $ssl.Dispose() }
    if ($null -ne $tcp) { $tcp.Close() }
  }
}

$results = New-Object System.Collections.Generic.List[object]
$warnings = New-Object System.Collections.Generic.List[string]
$failures = New-Object System.Collections.Generic.List[string]

$uri = $null
try {
  $uri = [Uri]$BaseUrl
} catch {
  Write-Error "Invalid -BaseUrl: $BaseUrl"
  exit 1
}

if ($RequireHttps -and $uri.Scheme -ne "https") {
  Add-CheckResult -Bag $results -Check "Scheme" -Status "FAIL" -Details "Expected HTTPS but got '$($uri.Scheme)'."
  $failures.Add("BaseUrl is not HTTPS") | Out-Null
} else {
  Add-CheckResult -Bag $results -Check "Scheme" -Status "PASS" -Details "Using scheme '$($uri.Scheme)'."
}

$resolvedIps = @()
try {
  $resolvedIps = @([System.Net.Dns]::GetHostAddresses($uri.Host) | ForEach-Object { $_.IPAddressToString })
  if ($resolvedIps.Count -eq 0) {
    Add-CheckResult -Bag $results -Check "DNS Resolution" -Status "FAIL" -Details "Host resolved with empty IP list."
    $failures.Add("DNS resolution empty") | Out-Null
  } else {
    Add-CheckResult -Bag $results -Check "DNS Resolution" -Status "PASS" -Details ("IPs: " + ($resolvedIps -join ", "))
  }
} catch {
  Add-CheckResult -Bag $results -Check "DNS Resolution" -Status "FAIL" -Details $_.Exception.Message
  $failures.Add("DNS resolution failed") | Out-Null
}

if (-not [string]::IsNullOrWhiteSpace($ExpectedIp)) {
  if ($resolvedIps -contains $ExpectedIp) {
    Add-CheckResult -Bag $results -Check "Expected IP" -Status "PASS" -Details "Expected IP '$ExpectedIp' found."
  } else {
    Add-CheckResult -Bag $results -Check "Expected IP" -Status "WARN" -Details "Expected IP '$ExpectedIp' not found in resolved set."
    $warnings.Add("Expected IP mismatch") | Out-Null
  }
}

if ($uri.Scheme -eq "https") {
  $tls = Test-TlsCertificate -HostName $uri.Host -Port $(if ($uri.Port -gt 0) { $uri.Port } else { 443 })
  if ($null -ne $tls.error) {
    Add-CheckResult -Bag $results -Check "TLS Certificate" -Status "FAIL" -Details $tls.error
    $failures.Add("TLS handshake/certificate check failed") | Out-Null
  } else {
    if ($tls.days_remaining -le 0) {
      Add-CheckResult -Bag $results -Check "TLS Certificate" -Status "FAIL" -Details "Certificate expired ($($tls.not_after))."
      $failures.Add("TLS certificate expired") | Out-Null
    } elseif ($tls.days_remaining -le 30) {
      Add-CheckResult -Bag $results -Check "TLS Certificate" -Status "WARN" -Details "Certificate expires in $($tls.days_remaining) days ($($tls.not_after))."
      $warnings.Add("TLS certificate expires within 30 days") | Out-Null
    } else {
      Add-CheckResult -Bag $results -Check "TLS Certificate" -Status "PASS" -Details "Subject: $($tls.subject); expires: $($tls.not_after) ($($tls.days_remaining) days)."
    }
  }
} else {
  Add-CheckResult -Bag $results -Check "TLS Certificate" -Status "WARN" -Details "Skipped (non-HTTPS base URL)."
  $warnings.Add("TLS check skipped due to non-HTTPS base URL") | Out-Null
}

$health = Invoke-StatusCode -Uri "$($BaseUrl.TrimEnd('/'))/health" -Method GET
if ($health.code -eq 200) {
  Add-CheckResult -Bag $results -Check "GET /health" -Status "PASS" -Details "HTTP 200."
} else {
  Add-CheckResult -Bag $results -Check "GET /health" -Status "FAIL" -Details "HTTP $($health.code). $($health.error)"
  $failures.Add("/health unavailable") | Out-Null
}

if (-not [string]::IsNullOrWhiteSpace($ApiKey)) {
  $stats = Invoke-StatusCode -Uri "$($BaseUrl.TrimEnd('/'))/stats" -Method GET -Headers @{ Authorization = "Bearer $ApiKey" }
  if ($stats.code -eq 200) {
    Add-CheckResult -Bag $results -Check "GET /stats (auth)" -Status "PASS" -Details "HTTP 200."
  } else {
    Add-CheckResult -Bag $results -Check "GET /stats (auth)" -Status "FAIL" -Details "HTTP $($stats.code). $($stats.error)"
    $failures.Add("/stats auth check failed") | Out-Null
  }
} else {
  Add-CheckResult -Bag $results -Check "GET /stats (auth)" -Status "WARN" -Details "Skipped (no -ApiKey provided)."
  $warnings.Add("Stats auth check skipped") | Out-Null
}

$webhookTargets = @(
  "/webhooks/github",
  "/integrations/jenkins",
  "/integrations/jira"
)

foreach ($path in $webhookTargets) {
  $probe = Invoke-StatusCode -Uri "$($BaseUrl.TrimEnd('/'))$path" -Method OPTIONS
  if ($probe.code -eq 404 -or $probe.code -eq 0) {
    Add-CheckResult -Bag $results -Check "Route probe $path" -Status "FAIL" -Details "HTTP $($probe.code). $($probe.error)"
    $failures.Add("Route $path unreachable") | Out-Null
  } elseif ($probe.code -ge 500) {
    Add-CheckResult -Bag $results -Check "Route probe $path" -Status "WARN" -Details "HTTP $($probe.code). Possible upstream/proxy issue."
    $warnings.Add("Route $path returned 5xx") | Out-Null
  } else {
    Add-CheckResult -Bag $results -Check "Route probe $path" -Status "PASS" -Details "HTTP $($probe.code). Endpoint is reachable."
  }
}

$summary = if ($failures.Count -gt 0) { "FAIL" } elseif ($warnings.Count -gt 0) { "WARN" } else { "PASS" }
if ($FailOnWarnings.IsPresent -and $summary -eq "WARN") {
  $summary = "FAIL"
  $failures.Add("FailOnWarnings enabled and at least one warning was found") | Out-Null
}

if ([string]::IsNullOrWhiteSpace($OutputPath)) {
  $stamp = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHHmmssZ")
  $OutputPath = "docs/reports/public-infra-validation-$stamp.md"
}

if (!(Test-Path (Split-Path -Parent $OutputPath))) {
  New-Item -ItemType Directory -Path (Split-Path -Parent $OutputPath) | Out-Null
}

$generatedUtc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-dd HH:mm:ss")
$rows = $results | ForEach-Object { "| $($_.Check) | $($_.Status) | $($_.Details -replace '\|','\\|') |" }
$warningText = if ($warnings.Count -eq 0) { "- none" } else { ($warnings | ForEach-Object { "- $_" }) -join "`n" }
$failureText = if ($failures.Count -eq 0) { "- none" } else { ($failures | ForEach-Object { "- $_" }) -join "`n" }

$report = @"
# Public Infra Validation Report

Generated (UTC): $generatedUtc
Summary: **$summary**
Base URL: $BaseUrl

## Checks

| Check | Status | Details |
|---|---|---|
$(($rows -join "`n"))

## Warnings

$warningText

## Failures

$failureText
"@

Set-Content -Path $OutputPath -Value $report -Encoding UTF8

Write-Host "${summary}: public infra validation completed"
Write-Host "  output: $OutputPath"

if ($summary -eq "FAIL") {
  exit 1
}
exit 0
