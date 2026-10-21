<#
.SYNOPSIS
    Installs the Universal Autonomous AI Workspace 24/7 background service into Windows Service Control Manager (SCM).
.DESCRIPTION
    Requires Administrator privileges. Creates Windows service 'AAIRuntime' configured for Auto-Start.
#>
param (
    [string]$ServiceName = "AAIRuntime",
    [string]$DisplayName = "Universal Autonomous AI 24/7 Runtime Service",
    [string]$BinaryPath = "$PSScriptRoot\..\target\release\desktop.exe"
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Installing Universal AI 24/7 Windows Service             " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# Check Administrator privileges
$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Warning "Administrator elevation required to register Windows Services."
    Write-Host "Please re-run this script in an elevated PowerShell window." -ForegroundColor Yellow
    exit 1
}

$resolvedBin = (Resolve-Path $BinaryPath -ErrorAction SilentlyContinue)
if (-not $resolvedBin) {
    Write-Warning "Binary not found at: $BinaryPath"
    Write-Host "Building release binary..." -ForegroundColor Yellow
    & cargo build --release --bin desktop
    $resolvedBin = (Resolve-Path "$PSScriptRoot\..\target\release\desktop.exe")
}

$fullBinPath = "`"$resolvedBin`" --service"

Write-Host "Service Name : $ServiceName" -ForegroundColor White
Write-Host "Display Name : $DisplayName" -ForegroundColor White
Write-Host "Binary Path  : $fullBinPath" -ForegroundColor White

$existing = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if ($existing) {
    Write-Host "Service '$ServiceName' is already installed. Reconfiguring..." -ForegroundColor Yellow
    sc.exe config $ServiceName binPath= $fullBinPath start= auto
} else {
    sc.exe create $ServiceName binPath= $fullBinPath DisplayName= "`"$DisplayName`"" start= auto
}

# Set service recovery actions (restart on failure)
sc.exe failure $ServiceName reset= 86400 actions= restart/60000/restart/60000/restart/60000

Write-Host "`n[SUCCESS] Windows Service '$ServiceName' registered successfully." -ForegroundColor Green
Write-Host "Run 'scripts/start-service.ps1' or 'sc.exe start $ServiceName' to begin 24/7 execution." -ForegroundColor Cyan
