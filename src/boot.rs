use anyhow::Result;
use std::fs;
use std::process::Command;
use winreg::RegKey;
use winreg::enums::*;

const KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const NAME: &str = "unixish";
const APP: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Unixish";

pub fn enabled() -> bool {
    let out = Command::new("reg")
        .args(["query", KEY, "/v", NAME])
        .output();
    matches!(out, Ok(value) if value.status.success())
}

pub fn on() -> Result<()> {
    let exe = std::env::current_exe()?;
    let data = format!("\"{}\"", exe.display());
    Command::new("reg")
        .args(["add", KEY, "/v", NAME, "/t", "REG_SZ", "/d", &data, "/f"])
        .status()?;
    Ok(())
}

pub fn off() -> Result<()> {
    Command::new("reg")
        .args(["delete", KEY, "/v", NAME, "/f"])
        .status()?;
    Ok(())
}

pub fn ensure() -> Result<()> {
    let root = RegKey::predef(HKEY_CURRENT_USER);
    if root.open_subkey(APP).is_ok() {
        return Ok(());
    }
    let exe = std::env::current_exe()?;
    let base = exe.parent().ok_or_else(|| anyhow::anyhow!("path"))?;
    let uns = base.join("unixish-uninstall.ps1");
    let unv = base.join("unixish-uninstall.vbs");
    let psh = format!(
        "param([switch]$silent)\n$ErrorActionPreference = \"SilentlyContinue\"\n$exe = \"{}\"\n$base = \"{}\"\n$run = \"HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run\"\n$app = \"HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Unixish\"\n$start = Join-Path $env:APPDATA \"Microsoft\\Windows\\Start Menu\\Programs\\Unixish.lnk\"\ntry {{ Stop-Process -Name \"unixish\" -Force -ErrorAction SilentlyContinue }} catch {{}}\ntry {{ Remove-ItemProperty -Path $run -Name \"unixish\" -ErrorAction SilentlyContinue }} catch {{}}\ntry {{ Remove-Item -Force $start -ErrorAction SilentlyContinue }} catch {{}}\ntry {{ Remove-Item -Recurse -Force $app -ErrorAction SilentlyContinue }} catch {{}}\ntry {{ Remove-Item -Force $exe -ErrorAction SilentlyContinue }} catch {{}}\ntry {{ Remove-Item -Force (Join-Path $base \"unixish-uninstall.ps1\") -ErrorAction SilentlyContinue }} catch {{}}\ntry {{ Remove-Item -Force (Join-Path $base \"unixish-uninstall.vbs\") -ErrorAction SilentlyContinue }} catch {{}}\nif (-not $silent) {{ Write-Output \"ok\" }}\n",
        exe.display(),
        base.display()
    );
    let vbs = format!(
        "CreateObject(\"WScript.Shell\").Run \"powershell -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File \"\"{}\"\" -silent\",0,False",
        uns.display()
    );
    fs::write(&uns, psh)?;
    fs::write(&unv, vbs)?;
    let cmd = format!("wscript.exe \"{}\"", unv.display());
    let size = fs::metadata(&exe)?.len().div_ceil(1024) as u32;
    let key = root.create_subkey(APP)?.0;
    key.set_value("DisplayName", &"Unixish")?;
    key.set_value("DisplayVersion", &env!("CARGO_PKG_VERSION"))?;
    key.set_value("Publisher", &"Crush")?;
    key.set_value("InstallLocation", &base.display().to_string())?;
    key.set_value("DisplayIcon", &format!("{},0", exe.display()))?;
    key.set_value("UninstallString", &cmd)?;
    key.set_value("QuietUninstallString", &cmd)?;
    key.set_value("EstimatedSize", &size)?;
    key.set_value("NoModify", &1u32)?;
    key.set_value("NoRepair", &1u32)?;
    key.set_value("URLInfoAbout", &"https://github.com/crush/unixish")?;
    Ok(())
}
