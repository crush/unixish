use anyhow::{anyhow, Result};
use windows::Win32::UI::WindowsAndMessaging::*;

const ICON: &[u8] = include_bytes!("../assets/unixish.ico");

pub fn load() -> Result<HICON> {
	if ICON.len() < 22 {
		return Err(anyhow!("icon"));
	}
	let length = u32::from_le_bytes([ICON[14], ICON[15], ICON[16], ICON[17]]) as usize;
	let offset = u32::from_le_bytes([ICON[18], ICON[19], ICON[20], ICON[21]]) as usize;
	if offset.checked_add(length).is_none() || offset + length > ICON.len() {
		return Err(anyhow!("icon"));
	}
	let data = &ICON[offset..offset + length];
	unsafe { Ok(CreateIconFromResourceEx(data, true, 0x0003_0000, 0, 0, LR_DEFAULTCOLOR)?) }
}
