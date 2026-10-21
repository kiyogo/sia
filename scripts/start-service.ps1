<#
.SYNOPSIS
    Starts the Universal Autonomous AI 24/7 background service.
#>
param (
    [string]$ServiceName = "AAIRuntime"
)

$ErrorActionPreference = "Stop"

$svc = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if (-not $svc) {
    Write-Error "Service '$ServiceName' is not installed. Run 'scripts/install-service.ps1' first."
}

if ($svc.Status -eq "Running") {
    Write-Host "Service '$ServiceName' is already running." -ForegroundColor Green
    exit 0
}

Write-Host "Starting '$ServiceName'..." -ForegroundColor Cyan
Start-Service -Name $ServiceName
Start-Sleep -Seconds 2

$svc.Refresh()
Write-Host "Status: $($svc.Status)" -ForegroundColor Green
