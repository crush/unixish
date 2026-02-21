use anyhow::{anyhow, Result};
use windows::Win32::UI::WindowsAndMessaging::*;

const ICON: &[u8] = include_bytes!("../assets/unixish.ico");

pub fn load(size: i32) -> Result<HICON> {
	if ICON.len() < 6 {
		return Err(anyhow!("icon"));
	}
	let count = u16::from_le_bytes([ICON[4], ICON[5]]) as usize;
	if count == 0 || ICON.len() < 6 + count * 16 {
		return Err(anyhow!("icon"));
	}
	let target = if size <= 0 { 16_u32 } else { size as u32 };
	let mut best = 0usize;
	let mut score = u32::MAX;
	for index in 0..count {
		let base = 6 + index * 16;
		let width = if ICON[base] == 0 { 256_u32 } else { ICON[base] as u32 };
		let height = if ICON[base + 1] == 0 { 256_u32 } else { ICON[base + 1] as u32 };
		let delta = width.abs_diff(target) + height.abs_diff(target);
		if delta < score {
			score = delta;
			best = base;
		}
	}
	let length =
		u32::from_le_bytes([ICON[best + 8], ICON[best + 9], ICON[best + 10], ICON[best + 11]]) as usize;
	let offset = u32::from_le_bytes([ICON[best + 12], ICON[best + 13], ICON[best + 14], ICON[best + 15]]) as usize;
	if offset.checked_add(length).is_none() || offset + length > ICON.len() {
		return Err(anyhow!("icon"));
	}
	let data = &ICON[offset..offset + length];
	unsafe {
		Ok(CreateIconFromResourceEx(
			data,
			true,
			0x0003_0000,
			size,
			size,
			LR_DEFAULTCOLOR,
		)?)
	}
}
