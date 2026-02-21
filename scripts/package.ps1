param(
	[string]$out = "dist"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location (Join-Path $root "..")

cargo build --release

New-Item -ItemType Directory -Force $out | Out-Null
$zip = Join-Path $out "unixish-windows-x64.zip"
$exe = "target\release\unixish.exe"
$readme = "README.md"
$config = "contribute\readme.md"

if (Test-Path $zip) { Remove-Item $zip -Force }
Compress-Archive -Path $exe, $readme, $config -DestinationPath $zip

$sha = (Get-FileHash $zip -Algorithm SHA256).Hash.ToLower()
"$sha  unixish-windows-x64.zip" | Set-Content -Encoding Ascii (Join-Path $out "SHA256SUMS.txt")
