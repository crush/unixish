#![allow(unsafe_op_in_unsafe_fn)]
mod app;
mod boot;
mod config;
mod hotkey;
mod icon;
mod key;
mod state;
mod tile;
mod tray;
mod update;
mod win;

fn main() -> anyhow::Result<()> {
	app::run()
}
