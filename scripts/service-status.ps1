<#
.SYNOPSIS
    Queries and displays the real-time operational status of the Universal Autonomous AI Windows Service.
#>
param (
    [string]$ServiceName = "AAIRuntime"
)

$ErrorActionPreference = "Continue"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Universal AI 24/7 Service Status                         " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

$svc = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if (-not $svc) {
    Write-Host "Service State       : NOT INSTALLED" -ForegroundColor Red
    Write-Host "Installed Services  : None matching '$ServiceName'" -ForegroundColor Gray
    exit 0
}

Write-Host "Service Name        : $($svc.Name)" -ForegroundColor White
Write-Host "Display Name        : $($svc.DisplayName)" -ForegroundColor White
Write-Host "Status              : $($svc.Status)" -ForegroundColor $(if ($svc.Status -eq 'Running') { 'Green' } else { 'Yellow' })
Write-Host "Startup Type        : $($svc.StartType)" -ForegroundColor White

if ($svc.Status -eq "Running") {
    $proc = Get-CimInstance Win32_Service -Filter "Name='$ServiceName'"
    if ($proc.ProcessId -gt 0) {
        $sysProc = Get-Process -Id $proc.ProcessId -ErrorAction SilentlyContinue
        Write-Host "Process ID (PID)    : $($proc.ProcessId)" -ForegroundColor Cyan
        if ($sysProc) {
            $workingSetMb = [Math]::Round($sysProc.WorkingSet64 / 1MB, 2)
            $cpuSecs = [Math]::Round($sysProc.CPU, 2)
            Write-Host "Memory Usage        : $workingSetMb MB" -ForegroundColor White
            Write-Host "Total CPU Time      : $cpuSecs seconds" -ForegroundColor White
            Write-Host "Start Time          : $($sysProc.StartTime)" -ForegroundColor White
        }
    }
}
