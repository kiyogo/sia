<#
.SYNOPSIS
    Development runner for Universal Autonomous AI Workspace.
.DESCRIPTION
    Builds the frontend and launches the Tauri desktop application.
#>

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Universal Autonomous AI Workspace — Development Mode      " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Build frontend assets
Write-Host "`n[1/2] Building frontend UI assets..." -ForegroundColor Yellow
Push-Location "apps/desktop"
try {
    pnpm run build
} finally {
    Pop-Location
}

# 2. Run Desktop App
Write-Host "`n[2/2] Launching Tauri Desktop App in dev mode..." -ForegroundColor Yellow
cargo run -p desktop
