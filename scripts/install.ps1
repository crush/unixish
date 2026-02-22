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
$ver = "0.0.0"
if ($json -and $json.tag_name) {
	$ver = "$($json.tag_name)".TrimStart("v")
}

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

$uns = Join-Path $base "unixish-uninstall.ps1"
$unts = @'
param([switch]$silent)
$ErrorActionPreference = "SilentlyContinue"
$home = [Environment]::GetFolderPath("LocalApplicationData")
$base = Join-Path $home "unixish"
$exe = Join-Path $base "unixish.exe"
$run = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$app = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Unixish"
$start = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\Unixish.lnk"
try { Stop-Process -Name "unixish" -Force -ErrorAction SilentlyContinue } catch {}
try { Remove-ItemProperty -Path $run -Name "unixish" -ErrorAction SilentlyContinue } catch {}
try { Remove-Item -Force $start -ErrorAction SilentlyContinue } catch {}
try { Remove-Item -Recurse -Force $app -ErrorAction SilentlyContinue } catch {}
try { Remove-Item -Force $exe -ErrorAction SilentlyContinue } catch {}
try { Remove-Item -Force (Join-Path $base "README.md") -ErrorAction SilentlyContinue } catch {}
try { Remove-Item -Force (Join-Path $base "CONTRIBUTE.md") -ErrorAction SilentlyContinue } catch {}
try { Remove-Item -Force (Join-Path $base "unixish-uninstall.cmd") -ErrorAction SilentlyContinue } catch {}
try {
	$cfg = Join-Path $env:APPDATA "unixish\config.json"
	Remove-Item -Force $cfg -ErrorAction SilentlyContinue
} catch {}
try {
	$dir = Join-Path $env:APPDATA "unixish"
	if (Test-Path $dir) {
		$it = Get-ChildItem -Force $dir -ErrorAction SilentlyContinue
		if (-not $it) { Remove-Item -Force $dir -ErrorAction SilentlyContinue }
	}
} catch {}
try {
	$it = Get-ChildItem -Force $base -ErrorAction SilentlyContinue
	if (-not $it) { Remove-Item -Force $base -ErrorAction SilentlyContinue }
} catch {}
if (-not $silent) { Write-Output "ok" }
'@
Set-Content -Encoding Ascii -Path $uns -Value $unts
$unc = Join-Path $base "unixish-uninstall.cmd"
$uncc = '@echo off' + "`r`n" + 'powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0unixish-uninstall.ps1"'
Set-Content -Encoding Ascii -Path $unc -Value $uncc

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

$app = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Unixish"
New-Item -Path $app -Force | Out-Null
$size = [int][math]::Ceiling((Get-Item $exe).Length / 1KB)
$cmd = "powershell -NoProfile -ExecutionPolicy Bypass -File `"$uns`""
Set-ItemProperty -Path $app -Name "DisplayName" -Type String -Value "Unixish"
Set-ItemProperty -Path $app -Name "DisplayVersion" -Type String -Value $ver
Set-ItemProperty -Path $app -Name "Publisher" -Type String -Value "crush"
Set-ItemProperty -Path $app -Name "InstallLocation" -Type ExpandString -Value $base
Set-ItemProperty -Path $app -Name "DisplayIcon" -Type ExpandString -Value "$exe,0"
Set-ItemProperty -Path $app -Name "UninstallString" -Type ExpandString -Value $cmd
Set-ItemProperty -Path $app -Name "QuietUninstallString" -Type ExpandString -Value "$cmd -silent"
Set-ItemProperty -Path $app -Name "EstimatedSize" -Type DWord -Value $size
Set-ItemProperty -Path $app -Name "NoModify" -Type DWord -Value 1
Set-ItemProperty -Path $app -Name "NoRepair" -Type DWord -Value 1
Set-ItemProperty -Path $app -Name "URLInfoAbout" -Type String -Value "https://github.com/crush/unixish"

if (-not $silent) {
	Start-Process -FilePath $exe | Out-Null
}

if (Test-Path $temp) {
	Remove-Item -Recurse -Force $temp
}
Write-Output "ok"
