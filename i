$ErrorActionPreference = "Stop"
$owner = "crush"
$repo = "unixish"
$sha = (Invoke-RestMethod -Uri "https://api.github.com/repos/$owner/$repo/commits/main").sha
$url = "https://raw.githubusercontent.com/$owner/$repo/$sha/scripts/install.ps1"
$script = Invoke-WebRequest -UseBasicParsing -Uri $url | Select-Object -ExpandProperty Content
Invoke-Expression $script
