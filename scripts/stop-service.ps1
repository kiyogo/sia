<#
.SYNOPSIS
    Stops the Universal Autonomous AI 24/7 background service gracefully.
#>
param (
    [string]$ServiceName = "AAIRuntime"
)

$ErrorActionPreference = "Stop"

$svc = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if (-not $svc) {
    Write-Host "Service '$ServiceName' is not installed." -ForegroundColor Yellow
    exit 0
}

if ($svc.Status -eq "Stopped") {
    Write-Host "Service '$ServiceName' is already stopped." -ForegroundColor Yellow
    exit 0
}

Write-Host "Stopping '$ServiceName' gracefully..." -ForegroundColor Cyan
Stop-Service -Name $ServiceName -Force
Start-Sleep -Seconds 2

$svc.Refresh()
Write-Host "Status: $($svc.Status)" -ForegroundColor Yellow
