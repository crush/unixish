use anyhow::Result;
use std::process::Command;

const KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const NAME: &str = "unixish";

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
