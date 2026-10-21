<#
.SYNOPSIS
    Restarts the Universal Autonomous AI 24/7 background service.
#>
param (
    [string]$ServiceName = "AAIRuntime"
)

$ErrorActionPreference = "Stop"

Write-Host "Restarting '$ServiceName'..." -ForegroundColor Cyan
Restart-Service -Name $ServiceName -Force
Start-Sleep -Seconds 2

$svc = Get-Service -Name $ServiceName
Write-Host "Status: $($svc.Status)" -ForegroundColor Green
