use crate::config::Config;
use crate::key;
use crate::win::Move;
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

#[derive(Clone)]
pub struct Bind {
	pub id: i32,
	pub moveid: Move,
	pub mods: HOT_KEY_MODIFIERS,
	pub key: u32,
}

pub fn bind(config: &Config) -> Result<Vec<Bind>> {
	load(config)
}

pub fn check(config: &Config) -> Result<()> {
	let _ = load(config)?;
	Ok(())
}

pub fn register(window: HWND, bind: &[Bind]) -> Result<()> {
	for item in bind {
		if unsafe { RegisterHotKey(Some(window), item.id, item.mods, item.key) }.is_err() {
			unregister(window, bind);
			return Err(anyhow!("hotkey"));
		}
	}
	Ok(())
}

pub fn unregister(window: HWND, bind: &[Bind]) {
	for item in bind {
		let _ = unsafe { UnregisterHotKey(Some(window), item.id) };
	}
}

pub fn action(bind: &[Bind], id: i32) -> Option<Move> {
	bind.iter().find(|item| item.id == id).map(|item| item.moveid)
}

fn load(config: &Config) -> Result<Vec<Bind>> {
	let list = [
		(Move::Almost, config.hotkey.almost.clone()),
		(Move::Center, config.hotkey.center.clone()),
		(Move::Left, config.hotkey.left.clone()),
		(Move::Right, config.hotkey.right.clone()),
		(Move::Top, config.hotkey.top.clone()),
		(Move::Bottom, config.hotkey.bottom.clone()),
		(Move::Next, config.hotkey.next.clone()),
		(Move::Prev, config.hotkey.prev.clone()),
	];
	let mut bind = Vec::with_capacity(list.len());
	let mut seen = HashSet::<(u32, u32)>::new();
	for (index, (moveid, text)) in list.into_iter().enumerate() {
		let chord = key::parse(&text)?;
		if !seen.insert((chord.mods.0, chord.key)) {
			return Err(anyhow!("hotkey"));
		}
		bind.push(Bind {
			id: index as i32 + 1,
			moveid,
			mods: chord.mods,
			key: chord.key,
		});
	}
	Ok(bind)
}
