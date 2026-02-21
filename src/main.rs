mod app;
mod config;
mod hotkey;
mod key;
mod tile;
mod win;

fn main() -> anyhow::Result<()> {
	app::run()
}
