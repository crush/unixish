use crate::boot;
use crate::config;
use crate::hotkey;
use crate::tray;
use crate::update;
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
            println!(
                "{}",
                if hotkey::check(&config).is_ok() {
                    "ok"
                } else {
                    "conflict"
                }
            );
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
        Some("update") => {
            update::run()?;
            println!("ok");
            Ok(())
        }
        _ => {
            if boot::place()? {
                return Ok(());
            }
            let _ = boot::ensure();
            tray::run()
        }
    }
}
