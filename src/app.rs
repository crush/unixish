use crate::config;
use crate::hotkey;
use anyhow::Result;

pub fn run() -> Result<()> {
	let config = config::load()?;
	hotkey::run(&config)
}
