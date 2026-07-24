<#
.SYNOPSIS
    Remove the bwsshd scheduled task and installed files on Windows.

.PARAMETER TaskName
    Scheduled Task name (default "bwsshd").

.PARAMETER KeepLogs
    Keep the log directory (%LOCALAPPDATA%\bwsshd). Logs are kept by default.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File .\uninstall.ps1
#>
[CmdletBinding()]
param(
    [string]$TaskName = "bwsshd",
    [switch]$KeepLogs = $true
)

$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\bwsshd"

# Stop and remove the task if present (ignore "not found").
try { Stop-ScheduledTask -TaskName $TaskName -ErrorAction Stop } catch {}
try {
    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction Stop
    Write-Host "removed scheduled task: $TaskName"
} catch {
    Write-Host "no scheduled task named $TaskName"
}

# The daemon may still be running from this session; stop it so the exe isn't locked.
Get-CimInstance Win32_Process -Filter "Name='bwsshd.exe'" -ErrorAction SilentlyContinue |
    Where-Object { $_.ExecutablePath -and $_.ExecutablePath.StartsWith($InstallDir) } |
    ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }

if (Test-Path $InstallDir) {
    Remove-Item -Recurse -Force $InstallDir -ErrorAction SilentlyContinue
    Write-Host "removed $InstallDir"
}

if (-not $KeepLogs) {
    $logDir = Join-Path $env:LOCALAPPDATA "bwsshd"
    Remove-Item -Recurse -Force $logDir -ErrorAction SilentlyContinue
    Write-Host "removed logs $logDir"
} else {
    Write-Host "logs kept at $env:LOCALAPPDATA\bwsshd"
}
