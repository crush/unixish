param([switch]$silent)

$ErrorActionPreference = "Stop"
$owner = "crush"
$repo = "unixish"
$api = "https://api.github.com/repos/$owner/$repo/releases/latest"
$json = $null
$asset = $null
try {
	$json = Invoke-RestMethod -Uri $api
	$asset = $json.assets | Where-Object { $_.name -eq "unixish-windows-x64.zip" } | Select-Object -First 1
} catch {}

$home = [Environment]::GetFolderPath("LocalApplicationData")
$base = Join-Path $home "unixish"
$temp = Join-Path $base "temp"
New-Item -ItemType Directory -Force $base | Out-Null
New-Item -ItemType Directory -Force $temp | Out-Null

$exe = Join-Path $base "unixish.exe"
if ($asset) {
	$zip = Join-Path $temp "unixish-windows-x64.zip"
	Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $zip
	Expand-Archive -Path $zip -DestinationPath $temp -Force
	$one = Join-Path $temp "unixish.exe"
	$two = Join-Path $temp "target\release\unixish.exe"
	if (Test-Path $one) {
		Copy-Item -Force $one $exe
	} elseif (Test-Path $two) {
		Copy-Item -Force $two $exe
	} else {
		throw "exe"
	}
} else {
	cargo install --git "https://github.com/$owner/$repo" unixish
	$user = Join-Path $env:USERPROFILE ".cargo\bin\unixish.exe"
	$prog = Join-Path $home "Programs\Rust\bin\unixish.exe"
	if (Test-Path $user) {
		Copy-Item -Force $user $exe
	} elseif (Test-Path $prog) {
		Copy-Item -Force $prog $exe
	} else {
		throw "exe"
	}
}

$run = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$current = $null
try {
	$current = (Get-ItemProperty -Path $run -Name "unixish" -ErrorAction Stop).unixish
} catch {}
if ($current) {
	Set-ItemProperty -Path $run -Name "unixish" -Value "`"$exe`""
}

$start = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\Unixish.lnk"
$shell = New-Object -ComObject WScript.Shell
$link = $shell.CreateShortcut($start)
$link.TargetPath = $exe
$link.WorkingDirectory = $base
$link.IconLocation = "$exe,0"
$link.Save()

if (-not $silent) {
	Start-Process -FilePath $exe | Out-Null
}

if (Test-Path $temp) {
	Remove-Item -Recurse -Force $temp
}
Write-Output "ok"
