<#
.SYNOPSIS
    Uninstalls the Universal Autonomous AI Workspace 24/7 background service from Windows SCM.
.DESCRIPTION
    Requires Administrator privileges. Stops the service if running and unregisters it.
#>
param (
    [string]$ServiceName = "AAIRuntime"
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Uninstalling Universal AI 24/7 Windows Service           " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Warning "Administrator elevation required to uninstall Windows Services."
    exit 1
}

$existing = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if (-not $existing) {
    Write-Host "Service '$ServiceName' is not installed." -ForegroundColor Yellow
    exit 0
}

if ($existing.Status -eq "Running") {
    Write-Host "Stopping service '$ServiceName'..." -ForegroundColor Yellow
    Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
}

sc.exe delete $ServiceName
Write-Host "[SUCCESS] Windows Service '$ServiceName' uninstalled successfully." -ForegroundColor Green
