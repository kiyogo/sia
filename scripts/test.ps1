<#
.SYNOPSIS
    Test script for Universal Autonomous AI Workspace.
.DESCRIPTION
    Executes unit tests, integration tests, and acceptance test scenarios.
#>

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Universal Autonomous AI Workspace - Test Suite Runner    " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Run unit tests across all workspace crates
Write-Host "`n[1/3] Running Rust Workspace Unit Tests..." -ForegroundColor Yellow
cargo test --workspace --lib

# 2. Run Acceptance Test Suite across all phases (Phase 1, 2, 3, 4, 5)
Write-Host "`n[2/3] Running Acceptance Scenarios (Phase 1, 2, 3, 4, 5)..." -ForegroundColor Yellow
cargo test --workspace --tests

# 3. Test Frontend TypeScript compilation & Vite build
Write-Host "`n[3/3] Verifying Desktop Frontend Build..." -ForegroundColor Yellow
Push-Location "apps/desktop"
try {
    pnpm run build
} finally {
    Pop-Location
}

Write-Host "`n==========================================================" -ForegroundColor Green
Write-Host " All Acceptance and Unit Tests PASSED!                     " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
