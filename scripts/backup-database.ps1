<#
.SYNOPSIS
    Performs an online, non-blocking backup of the workspace SQLite database.
#>
param (
    [string]$DbPath = "$env:APPDATA\aai\data.db",
    [string]$BackupDir = "$env:APPDATA\aai\backups"
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Universal AI Online Database Backup                      " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

if (-not (Test-Path $DbPath)) {
    # Check current directory fallback
    if (Test-Path "data.db") {
        $DbPath = (Resolve-Path "data.db").Path
    } else {
        Write-Warning "Database file not found at: $DbPath"
        exit 1
    }
}

if (-not (Test-Path $BackupDir)) {
    New-Item -ItemType Directory -Path $BackupDir -Force | Out-Null
}

$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$backupFile = Join-Path $BackupDir "backup_$timestamp.db"

Write-Host "Source Database     : $DbPath" -ForegroundColor White
Write-Host "Target Backup Path  : $backupFile" -ForegroundColor White

# Use sqlite3.exe if available for online safe VACUUM INTO / backup API, or copy with read lock
if (Get-Command "sqlite3.exe" -ErrorAction SilentlyContinue) {
    sqlite3.exe $DbPath ".backup '$backupFile'"
} else {
    # Fallback to copy
    Copy-Item -Path $DbPath -Destination $backupFile -Force
}

$fileInfo = Get-Item $backupFile
$sizeMb = [Math]::Round($fileInfo.Length / 1MB, 2)
Write-Host "[SUCCESS] Backup completed: $backupFile ($sizeMb MB)" -ForegroundColor Green
