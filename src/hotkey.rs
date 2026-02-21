use crate::config::Config;
use crate::key;
use crate::win::{self, Move};
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

#[derive(Clone)]
struct Bind {
	id: i32,
	moveid: Move,
	mods: HOT_KEY_MODIFIERS,
	key: u32,
}

pub fn run(config: &Config) -> Result<()> {
	let bind = load(config)?;
	for item in &bind {
		if unsafe { RegisterHotKey(None, item.id, item.mods, item.key) }.is_err() {
			for value in &bind {
				let _ = unsafe { UnregisterHotKey(None, value.id) };
			}
			return Err(anyhow!("hotkey"));
		}
	}
	looprun(config, &bind)?;
	for item in &bind {
		let _ = unsafe { UnregisterHotKey(None, item.id) };
	}
	Ok(())
}

pub fn check(config: &Config) -> Result<()> {
	let _ = load(config)?;
	Ok(())
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

fn looprun(config: &Config, bind: &[Bind]) -> Result<()> {
	let mut msg = MSG::default();
	loop {
		let state = unsafe { GetMessageW(&mut msg, None, 0, 0) };
		if state.0 <= 0 {
			break;
		}
		if msg.message == WM_HOTKEY {
			let id = msg.wParam.0 as i32;
			if let Some(item) = bind.iter().find(|value| value.id == id) {
				let _ = win::apply(item.moveid, config.layout.clone());
			}
		}
	}
	Ok(())
}
