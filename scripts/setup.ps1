<#
.SYNOPSIS
    Setup script for Universal Autonomous AI Workspace (Phase 1).
.DESCRIPTION
    Verifies system prerequisites (Rust, MSVC, Node, pnpm, Python 3.14+, uv, Git),
    installs frontend node_modules, and builds workspace dependencies.
#>

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Universal Autonomous AI Workspace — Environment Setup    " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Check Rust & Cargo
Write-Host "`n[1/6] Checking Rust toolchain..." -ForegroundColor Yellow
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    $cargoVer = cargo --version
    Write-Host "  ✓ Found: $cargoVer" -ForegroundColor Green
} else {
    Write-Error "Rust and Cargo are required. Install via https://rustup.rs"
}

# 2. Check Node & pnpm
Write-Host "`n[2/6] Checking Node.js and pnpm..." -ForegroundColor Yellow
if (Get-Command node -ErrorAction SilentlyContinue) {
    $nodeVer = node --version
    Write-Host "  ✓ Node: $nodeVer" -ForegroundColor Green
} else {
    Write-Error "Node.js is required. Install via https://nodejs.org"
}

if (Get-Command pnpm -ErrorAction SilentlyContinue) {
    $pnpmVer = pnpm --version
    Write-Host "  ✓ pnpm: $pnpmVer" -ForegroundColor Green
} else {
    Write-Host "  Installing pnpm via npm..." -ForegroundColor DarkYellow
    npm install -g pnpm
}

# 3. Check Python 3.14+ (for Windows-MCP)
Write-Host "`n[3/6] Checking Python 3.14+ (Required for Windows-MCP)..." -ForegroundColor Yellow
$pyFound = $false
try {
    $pyVer = python -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}')" 2>$null
    if ($pyVer -and ([version]$pyVer -ge [version]"3.14.0")) {
        Write-Host "  ✓ Python detected: $pyVer (>=3.14)" -ForegroundColor Green
        $pyFound = $true
    }
} catch {}

if (-not $pyFound) {
    Write-Host "  ⚠ Python 3.14+ not default in PATH. Checking uv python list..." -ForegroundColor DarkYellow
    if (Get-Command uv -ErrorAction SilentlyContinue) {
        $uvPys = uv python list 2>$null
        if ($uvPys -match "3\.14") {
            Write-Host "  ✓ Python 3.14 managed by uv is available." -ForegroundColor Green
            $pyFound = $true
        }
    }
}

if (-not $pyFound) {
    Write-Host "  ℹ Notice: Windows-MCP requires Python 3.14+. You can install it using: 'uv python install 3.14'" -ForegroundColor Yellow
}

# 4. Check uv
Write-Host "`n[4/6] Checking uv package manager..." -ForegroundColor Yellow
if (Get-Command uv -ErrorAction SilentlyContinue) {
    $uvVer = uv --version
    Write-Host "  ✓ uv: $uvVer" -ForegroundColor Green
} else {
    Write-Host "  ℹ uv is recommended for Windows-MCP. Install via: irm https://astral.sh/uv/install.ps1 | iex" -ForegroundColor Yellow
}

# 5. Install frontend packages
Write-Host "`n[5/6] Installing desktop frontend dependencies..." -ForegroundColor Yellow
Push-Location "apps/desktop"
try {
    pnpm install
    pnpm approve-builds --all
    Write-Host "  ✓ Frontend packages installed successfully." -ForegroundColor Green
} finally {
    Pop-Location
}

# 6. Verify Cargo workspace
Write-Host "`n[6/6] Verifying Rust workspace compile check..." -ForegroundColor Yellow
cargo check --workspace
Write-Host "  ✓ Rust workspace verified successfully." -ForegroundColor Green

Write-Host "`n==========================================================" -ForegroundColor Green
Write-Host " Setup complete! Run .\scripts\dev.ps1 to start in dev mode" -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
