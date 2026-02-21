use crate::config::Layout;
use crate::tile::{self, Rect};
use anyhow::{anyhow, Result};
use windows::core::BOOL;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

#[derive(Debug, Clone, Copy)]
pub enum Move {
	Almost,
	Center,
	Left,
	Right,
	Top,
	Bottom,
	Next,
	Prev,
}

#[derive(Clone, Copy)]
struct Screen {
	handle: HMONITOR,
	work: Rect,
}

pub fn apply(action: Move, layout: Layout) -> Result<()> {
	let window = unsafe { GetForegroundWindow() };
	if window.0.is_null() {
		return Ok(());
	}
	unsafe {
		let _ = ShowWindow(window, SW_RESTORE);
	}
	let current = windowrect(window)?;
	let screen = monitorwork(unsafe { MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST) })?;
	let target = match action {
		Move::Almost => tile::almost(screen, layout.width, layout.height),
		Move::Center => tile::center(screen, current),
		Move::Left => tile::left(screen),
		Move::Right => tile::right(screen),
		Move::Top => tile::top(screen),
		Move::Bottom => tile::bottom(screen),
		Move::Next => moveother(window, current, 1)?,
		Move::Prev => moveother(window, current, -1)?,
	};
	setwindow(window, target)?;
	Ok(())
}

fn moveother(window: HWND, current: Rect, step: i32) -> Result<Rect> {
	let list = screens()?;
	if list.is_empty() {
		return Err(anyhow!("screen"));
	}
	let now = unsafe { MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST) };
	let index = list.iter().position(|item| item.handle == now).unwrap_or(0) as i32;
	let next = (index + step).rem_euclid(list.len() as i32) as usize;
	let target = list[next].work;
	let width = current.width.min(target.width);
	let height = current.height.min(target.height);
	let x = target.x + (target.width - width) / 2;
	let y = target.y + (target.height - height) / 2;
	Ok(Rect { x, y, width, height })
}

fn setwindow(window: HWND, rect: Rect) -> Result<()> {
	unsafe {
		SetWindowPos(
			window,
			None,
			rect.x,
			rect.y,
			rect.width,
			rect.height,
			SWP_NOACTIVATE | SWP_NOZORDER,
		)?;
	}
	Ok(())
}

fn windowrect(window: HWND) -> Result<Rect> {
	let mut value = RECT::default();
	unsafe {
		GetWindowRect(window, &mut value)?;
	}
	Ok(Rect {
		x: value.left,
		y: value.top,
		width: value.right - value.left,
		height: value.bottom - value.top,
	})
}

fn monitorwork(handle: HMONITOR) -> Result<Rect> {
	let mut info = MONITORINFO {
		cbSize: std::mem::size_of::<MONITORINFO>() as u32,
		..Default::default()
	};
	let ok = unsafe { GetMonitorInfoW(handle, &mut info as *mut MONITORINFO as *mut _) };
	if !ok.as_bool() {
		return Err(anyhow!("monitor"));
	}
	let value = info.rcWork;
	Ok(Rect {
		x: value.left,
		y: value.top,
		width: value.right - value.left,
		height: value.bottom - value.top,
	})
}

fn screens() -> Result<Vec<Screen>> {
	unsafe extern "system" fn call(
		handle: HMONITOR,
		_: HDC,
		_: *mut RECT,
		data: LPARAM,
	) -> BOOL {
		let list = unsafe { &mut *(data.0 as *mut Vec<Screen>) };
		let mut info = MONITORINFO {
			cbSize: std::mem::size_of::<MONITORINFO>() as u32,
			..Default::default()
		};
		if unsafe { GetMonitorInfoW(handle, &mut info as *mut MONITORINFO as *mut _) }.as_bool() {
			let work = info.rcWork;
			list.push(Screen {
				handle,
				work: Rect {
					x: work.left,
					y: work.top,
					width: work.right - work.left,
					height: work.bottom - work.top,
				},
			});
		}
		true.into()
	}
	let mut list = Vec::<Screen>::new();
	let ok = unsafe { EnumDisplayMonitors(None, None, Some(call), LPARAM(&mut list as *mut _ as isize)) };
	if !ok.as_bool() {
		return Err(anyhow!("enum"));
	}
	Ok(list)
}
