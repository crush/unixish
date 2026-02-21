use anyhow::{anyhow, Result};
use std::process::Command;

const REPO: &str = "https://github.com/crush/unixish";
const SCRIPT: &str = "https://raw.githubusercontent.com/crush/unixish/main/scripts/install.ps1";

pub fn run() -> Result<()> {
	let cmd = format!("irm {} | iex", SCRIPT);
	let status = Command::new("powershell")
		.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &cmd])
		.status()?;
	if status.success() {
		Ok(())
	} else {
		Err(anyhow!("update"))
	}
}

pub fn available() -> bool {
	if let Some(remote) = remote() {
		return newer(&remote, env!("CARGO_PKG_VERSION"));
	}
	false
}

fn remote() -> Option<String> {
	let cmd = format!(
		"$ProgressPreference='SilentlyContinue'; try {{ (Invoke-RestMethod -TimeoutSec 3 '{}/releases/latest').tag_name }} catch {{ '' }}",
		REPO.replace("github.com", "api.github.com/repos")
	);
	let out = Command::new("powershell")
		.args(["-NoProfile", "-Command", &cmd])
		.output()
		.ok()?;
	if !out.status.success() {
		return None;
	}
	let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
	if text.is_empty() {
		return None;
	}
	Some(text.trim_start_matches('v').to_string())
}

fn newer(remote: &str, local: &str) -> bool {
	let r = parse(remote);
	let l = parse(local);
	r > l
}

fn parse(text: &str) -> (u32, u32, u32) {
	let mut list = text.split('.').map(|item| item.parse::<u32>().unwrap_or(0));
	(
		list.next().unwrap_or(0),
		list.next().unwrap_or(0),
		list.next().unwrap_or(0),
	)
}
