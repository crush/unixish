$ErrorActionPreference = "Stop"
$run = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
Remove-ItemProperty -Path $run -Name "unixish" -ErrorAction SilentlyContinue
$home = [Environment]::GetFolderPath("LocalApplicationData")
$base = Join-Path $home "unixish"
if (Test-Path $base) { Remove-Item -Recurse -Force $base }
Write-Output "ok"
