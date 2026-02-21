use crate::config;
use crate::hotkey;
use anyhow::Result;
use windows::Win32::UI::HiDpi::*;

pub fn run() -> Result<()> {
	let _ = unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
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
