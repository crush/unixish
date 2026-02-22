$ErrorActionPreference = "Stop"
$owner = "crush"
$repo = "unixish"
$api = "https://api.github.com/repos/$owner/$repo/releases/latest"
$asset = $null
try {
	$json = Invoke-RestMethod -Uri $api
	$asset = $json.assets | Where-Object { $_.name -eq "unixish-windows-x64.zip" } | Select-Object -First 1
} catch {}

$home = [Environment]::GetFolderPath("LocalApplicationData")
$base = Join-Path $home "unixish"
New-Item -ItemType Directory -Force $base | Out-Null

if ($asset) {
	$zip = Join-Path $base "unixish-windows-x64.zip"
	Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $zip
	Expand-Archive -Path $zip -DestinationPath $base -Force
	$exe = Join-Path $base "target\release\unixish.exe"
	if (-not (Test-Path $exe)) { $exe = Join-Path $base "unixish.exe" }
} else {
	cargo install --git "https://github.com/$owner/$repo" unixish
	$exe = Join-Path $home "Programs\Rust\bin\unixish.exe"
	if (-not (Test-Path $exe)) { $exe = Join-Path $env:USERPROFILE ".cargo\bin\unixish.exe" }
}
if (-not (Test-Path $exe)) { throw "exe" }

$run = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
Set-ItemProperty -Path $run -Name "unixish" -Value "`"$exe`""
Write-Output "ok"