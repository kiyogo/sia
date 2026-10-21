<#
.SYNOPSIS
    Restores the workspace SQLite database from a selected backup file.
.DESCRIPTION
    Requires stopping any active runtime service before overwriting data.db.
#>
param (
    [Parameter(Mandatory=$true)]
    [string]$BackupFile,
    [string]$DbPath = "$env:APPDATA\aai\data.db"
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Universal AI Database Restoration                        " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

if (-not (Test-Path $BackupFile)) {
    Write-Error "Backup file not found at: $BackupFile"
}

# Check if service is running
$svc = Get-Service -Name "AAIRuntime" -ErrorAction SilentlyContinue
if ($svc -and $svc.Status -eq "Running") {
    Write-Warning "AAIRuntime service is running. Stopping service before restore..."
    Stop-Service -Name "AAIRuntime" -Force
    Start-Sleep -Seconds 2
}

# Backup existing database before overwrite
if (Test-Path $DbPath) {
    $preRestoreBackup = "$DbPath.pre_restore_$(Get-Date -Format 'yyyyMMdd_HHmmss').bak"
    Write-Host "Creating safety snapshot of current database: $preRestoreBackup" -ForegroundColor Gray
    Copy-Item -Path $DbPath -Destination $preRestoreBackup -Force
}

Write-Host "Restoring database from: $BackupFile -> $DbPath" -ForegroundColor Cyan
Copy-Item -Path $BackupFile -Destination $DbPath -Force

Write-Host "[SUCCESS] Database restored successfully." -ForegroundColor Green
if ($svc) {
    Write-Host "You may now restart the service with: 'scripts/start-service.ps1'" -ForegroundColor Cyan
}
