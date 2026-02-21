use crate::boot;
use crate::config;
use crate::hotkey;
use crate::log;
use crate::tray;
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
		Some("startup") => {
			let value = std::env::args().nth(2).unwrap_or_else(|| "status".into());
			match value.as_str() {
				"on" => {
					boot::on()?;
					println!("on");
				}
				"off" => {
					boot::off()?;
					println!("off");
				}
				_ => {
					println!("{}", if boot::enabled() { "on" } else { "off" });
				}
			}
			Ok(())
		}
		Some("reset") => {
			let _ = config::reset()?;
			println!("ok");
			Ok(())
		}
		Some("log") => {
			let path = log::path()?;
			println!("{}", path.display());
			Ok(())
		}
		_ => tray::run(),
	}
}
