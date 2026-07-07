# pomo-tree installer for Windows (x64)
#
#   powershell -ExecutionPolicy Bypass -c "irm https://raw.githubusercontent.com/TakashiAihara/pomo-tree/main/scripts/install.ps1 | iex"
#
# Downloads the latest release NSIS installer and runs it silently.

$ErrorActionPreference = "Stop"

$repo = "TakashiAihara/pomo-tree"
$asset = "pomo-tree_windows_x64-setup.exe"
$url = "https://github.com/$repo/releases/latest/download/$asset"
$dest = Join-Path $env:TEMP $asset

Write-Host "Downloading $asset ..."
Invoke-WebRequest -Uri $url -OutFile $dest

Write-Host "Installing (silent) ..."
Start-Process -FilePath $dest -ArgumentList "/S" -Wait

Remove-Item $dest -Force

Write-Host ""
Write-Host "Installed. pomo-tree lives in the system tray (colored dot icon)."
