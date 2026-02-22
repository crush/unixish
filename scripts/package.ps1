param(
	[string]$out = "dist"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location (Join-Path $root "..")

cargo build --release

New-Item -ItemType Directory -Force $out | Out-Null
$stage = Join-Path $out "stage"
if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
New-Item -ItemType Directory -Force $stage | Out-Null

Copy-Item -Force "target\release\unixish.exe" (Join-Path $stage "unixish.exe")
Copy-Item -Force "target\release\unixish.exe" (Join-Path $out "unixish.exe")
Copy-Item -Force "README.md" (Join-Path $stage "README.md")
Copy-Item -Force "contribute\readme.md" (Join-Path $stage "CONTRIBUTE.md")
Copy-Item -Force "scripts\install.ps1" (Join-Path $out "unixish-setup.ps1")

$cmd = '@echo off' + "`r`n" + 'powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0unixish-setup.ps1"'
Set-Content -Encoding Ascii -Path (Join-Path $out "unixish-setup.cmd") -Value $cmd

$zip = Join-Path $out "unixish-windows-x64.zip"
if (Test-Path $zip) { Remove-Item $zip -Force }
Compress-Archive -Path (Join-Path $stage "*") -DestinationPath $zip

$hashzip = (Get-FileHash $zip -Algorithm SHA256).Hash.ToLower()
$hashexe = (Get-FileHash (Join-Path $out "unixish.exe") -Algorithm SHA256).Hash.ToLower()
"$hashzip  unixish-windows-x64.zip`n$hashexe  unixish.exe" | Set-Content -Encoding Ascii (Join-Path $out "SHA256SUMS.txt")

& ".\scripts\winget.ps1" -zip $zip -out (Join-Path $out "winget")
