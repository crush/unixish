use crate::config;
use crate::hotkey;
use anyhow::Result;

pub fn run() -> Result<()> {
	let arg = std::env::args().nth(1);
	match arg.as_deref() {
		Some("path") => {
			let path = config::path()?;
			println!("{}", path.display());
			Ok(())
		}
		Some("check") => {
			let config = config::load()?;
			hotkey::check(&config)?;
			println!("ok");
			Ok(())
		}
		_ => {
			let config = config::load()?;
			hotkey::run(&config)
		}
	}
}
