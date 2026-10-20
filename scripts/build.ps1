<#
.SYNOPSIS
    Production build script for Universal Autonomous AI Workspace.
.DESCRIPTION
    Builds the production web bundle and compiles the release Windows binary.
#>

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Universal Autonomous AI Workspace - Production Release   " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Build frontend bundle
Write-Host "`n[1/2] Compiling Frontend Assets..." -ForegroundColor Yellow
Push-Location "apps/desktop"
try {
    pnpm run build
} finally {
    Pop-Location
}

# 2. Compile release binary
Write-Host "`n[2/2] Compiling Windows Desktop Executable (Release profile)..." -ForegroundColor Yellow
cargo build --release -p desktop

$exePath = "target\release\desktop.exe"
if (Test-Path $exePath) {
    $fileItem = Get-Item $exePath
    $sizeMb = [math]::Round($fileItem.Length / 1MB, 2)
    Write-Host "`n==========================================================" -ForegroundColor Green
    Write-Host " Build SUCCESSFUL!" -ForegroundColor Green
    Write-Host " Executable: $exePath - $sizeMb MB" -ForegroundColor Green
    Write-Host "==========================================================" -ForegroundColor Green
} else {
    Write-Error "Release build failed to produce $exePath"
}
