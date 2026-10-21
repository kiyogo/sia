<#
.SYNOPSIS
    Development runner for Universal Autonomous AI Workspace.
.DESCRIPTION
    Starts the Vite dev server on port 1420 and launches the desktop app.
#>

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Universal Autonomous AI Workspace — Development Mode      " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Check if Vite dev server is running on port 1420, if not start it
$vitePort = 1420
$conn = Get-NetTCPConnection -LocalPort $vitePort -ErrorAction SilentlyContinue

$viteProcess = $null
if (-not $conn) {
    Write-Host "`n[1/2] Starting Vite dev server on http://localhost:$vitePort..." -ForegroundColor Yellow
    Push-Location "apps/desktop"
    try {
        $viteProcess = Start-Process -FilePath "pnpm" -ArgumentList "run", "dev" -PassThru -WindowStyle Hidden
    } finally {
        Pop-Location
    }

    # Wait up to 10 seconds for Vite to bind to port
    $started = $false
    for ($i = 0; $i -lt 20; $i++) {
        Start-Sleep -Milliseconds 500
        if (Get-NetTCPConnection -LocalPort $vitePort -ErrorAction SilentlyContinue) {
            $started = $true
            break
        }
    }
    if ($started) {
        Write-Host "Vite dev server is ready on http://localhost:$vitePort" -ForegroundColor Green
    } else {
        Write-Host "Warning: Vite dev server did not bind to port $vitePort within 10s." -ForegroundColor DarkYellow
    }
} else {
    Write-Host "`n[1/2] Vite dev server already listening on port $vitePort." -ForegroundColor Green
}

# 2. Run Desktop App
Write-Host "`n[2/2] Launching Tauri Desktop App in dev mode..." -ForegroundColor Yellow
try {
    cargo run -p desktop
} finally {
    if ($viteProcess -and (-not $viteProcess.HasExited)) {
        Write-Host "Stopping background Vite process..." -ForegroundColor DarkGray
        Stop-Process -Id $viteProcess.Id -Force -ErrorAction SilentlyContinue
    }
}
