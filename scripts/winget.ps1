param(
	[string]$zip,
	[string]$out
)

$ErrorActionPreference = "Stop"
$pkg = "Crush.Unixish"
$cargo = Get-Content -Raw "Cargo.toml"
$version = ([regex]::Match($cargo, 'version\s*=\s*"([^"]+)"')).Groups[1].Value
if (-not $version) { throw "version" }
$sha = (Get-FileHash $zip -Algorithm SHA256).Hash.ToUpper()
$url = "https://github.com/crush/unixish/releases/download/v$version/unixish-windows-x64.zip"

New-Item -ItemType Directory -Force $out | Out-Null

$v = @"
PackageIdentifier: $pkg
PackageVersion: $version
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.9.0
"@

$l = @"
PackageIdentifier: $pkg
PackageVersion: $version
PackageLocale: en-US
Publisher: crush
PublisherUrl: https://github.com/crush
PackageName: unixish
PackageUrl: https://github.com/crush/unixish
License: MIT
ShortDescription: Keyboard-first Windows window manager with monitor-aware snapping and tray controls.
ManifestType: defaultLocale
ManifestVersion: 1.9.0
"@

$i = @"
PackageIdentifier: $pkg
PackageVersion: $version
InstallerType: zip
NestedInstallerType: portable
NestedInstallerFiles:
  - RelativeFilePath: unixish.exe
    PortableCommandAlias: unixish
Installers:
  - Architecture: x64
    InstallerUrl: $url
    InstallerSha256: $sha
ManifestType: installer
ManifestVersion: 1.9.0
"@

Set-Content -Encoding Utf8 -Path (Join-Path $out "crush.unixish.yaml") -Value $v
Set-Content -Encoding Utf8 -Path (Join-Path $out "crush.unixish.locale.en-US.yaml") -Value $l
Set-Content -Encoding Utf8 -Path (Join-Path $out "crush.unixish.installer.yaml") -Value $i
