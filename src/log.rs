use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub fn path() -> Result<PathBuf> {
	let base = std::env::var("APPDATA").context("appdata")?;
	Ok(PathBuf::from(base).join("unixish").join("unixish.log"))
}

pub fn write(text: &str) {
	let _ = append(text);
}

fn append(text: &str) -> Result<()> {
	let file = path()?;
	if let Some(dir) = file.parent() {
		fs::create_dir_all(dir)?;
	}
	let line = format!("{}\r\n", text);
	let mut value = if file.exists() {
		fs::read_to_string(&file).unwrap_or_default()
	} else {
		String::new()
	};
	value.push_str(&line);
	if value.len() > 100_000 {
		let keep = value.len() - 100_000;
		value = value[keep..].to_string();
	}
	fs::write(file, value)?;
	Ok(())
}
