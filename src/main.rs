#![allow(unsafe_op_in_unsafe_fn)]
mod app;
mod boot;
mod config;
mod hotkey;
mod icon;
mod key;
mod lock;
mod menu;
mod panel;
mod state;
mod tile;
mod tray;
mod update;
mod win;

fn main() -> anyhow::Result<()> {
    let tray = std::env::args().nth(1).is_none();
    if !tray {
        return app::run();
    }
    let key = "UNIXISH_RESTART";
    let once = std::env::var_os(key).is_some();
    match std::panic::catch_unwind(app::run) {
        Ok(result) => {
            if result.is_err() && !once {
                let _ = restart(key);
            }
            result
        }
        Err(_) => {
            if !once {
                let _ = restart(key);
            }
            Ok(())
        }
    }
}

fn restart(key: &str) -> anyhow::Result<()> {
    let exe = std::env::current_exe()?;
    std::process::Command::new(exe).env(key, "1").spawn()?;
    Ok(())
}
