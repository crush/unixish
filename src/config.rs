use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	pub hotkey: Hotkey,
	pub layout: Layout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkey {
	pub almost: String,
	pub center: String,
	pub left: String,
	pub right: String,
	pub top: String,
	pub bottom: String,
	pub next: String,
	pub prev: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
	pub width: f64,
	pub height: f64,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			hotkey: Hotkey {
				almost: "ctrl+shift+c".into(),
				center: "ctrl+shift+x".into(),
				left: "shift+left".into(),
				right: "shift+right".into(),
				top: "shift+up".into(),
				bottom: "shift+down".into(),
				next: "ctrl+shift+right".into(),
				prev: "ctrl+shift+left".into(),
			},
			layout: Layout {
				width: 0.98,
				height: 0.98,
			},
		}
	}
}

pub fn path() -> Result<PathBuf> {
	let base = std::env::var("APPDATA").context("appdata")?;
	Ok(PathBuf::from(base).join("unixish").join("config.json"))
}

pub fn load() -> Result<Config> {
	let file = path()?;
	if let Some(dir) = file.parent() {
		fs::create_dir_all(dir)?;
	}
	if !file.exists() {
		let value = Config::default();
		save(&value)?;
		return Ok(value);
	}
	let text = fs::read_to_string(file)?;
	let mut value: Config = serde_json::from_str(&text).unwrap_or_default();
	value.layout.width = value.layout.width.clamp(0.2, 1.0);
	value.layout.height = value.layout.height.clamp(0.2, 1.0);
	Ok(value)
}

pub fn save(value: &Config) -> Result<()> {
	let file = path()?;
	if let Some(dir) = file.parent() {
		fs::create_dir_all(dir)?;
	}
	let text = serde_json::to_string_pretty(value)?;
	fs::write(file, text)?;
	Ok(())
}
