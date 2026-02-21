use crate::config::Layout;
use crate::tile::{self, Rect};
use anyhow::{anyhow, Result};
use windows::core::BOOL;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::Shell::*;
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
	full: Rect,
	autohide: bool,
}

impl Screen {
	fn area(&self) -> Rect {
		if self.autohide {
			self.full
		} else {
			self.work
		}
	}
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
	let screen = monitorarea(monitorfromrect(current))?;
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
	let now = monitorfromrect(current);
	let index = list.iter().position(|item| item.handle == now).unwrap_or_else(|| {
		let near = unsafe { MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST) };
		list.iter().position(|item| item.handle == near).unwrap_or(0)
	}) as i32;
	let next = (index + step).rem_euclid(list.len() as i32) as usize;
	let source = list[index as usize].area();
	let target = list[next].area();
	let sourcew = source.width.max(1) as f64;
	let sourceh = source.height.max(1) as f64;
	let relx = (current.x - source.x) as f64 / sourcew;
	let rely = (current.y - source.y) as f64 / sourceh;
	let relw = current.width as f64 / sourcew;
	let relh = current.height as f64 / sourceh;
	let mut width = ((target.width as f64) * relw).round() as i32;
	let mut height = ((target.height as f64) * relh).round() as i32;
	width = width.clamp(80, target.width.max(80));
	height = height.clamp(60, target.height.max(60));
	let mut x = target.x + ((target.width as f64) * relx).round() as i32;
	let mut y = target.y + ((target.height as f64) * rely).round() as i32;
	if x + width > target.x + target.width {
		x = target.x + target.width - width;
	}
	if y + height > target.y + target.height {
		y = target.y + target.height - height;
	}
	if x < target.x {
		x = target.x;
	}
	if y < target.y {
		y = target.y;
	}
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

fn monitorfull(handle: HMONITOR) -> Result<Rect> {
	let mut info = MONITORINFO {
		cbSize: std::mem::size_of::<MONITORINFO>() as u32,
		..Default::default()
	};
	let ok = unsafe { GetMonitorInfoW(handle, &mut info as *mut MONITORINFO as *mut _) };
	if !ok.as_bool() {
		return Err(anyhow!("monitor"));
	}
	let value = info.rcMonitor;
	Ok(Rect {
		x: value.left,
		y: value.top,
		width: value.right - value.left,
		height: value.bottom - value.top,
	})
}

fn monitorarea(handle: HMONITOR) -> Result<Rect> {
	let work = monitorwork(handle)?;
	let full = monitorfull(handle)?;
	if hasautohide(full) {
		Ok(full)
	} else {
		Ok(work)
	}
}

fn hasautohide(full: Rect) -> bool {
	let mut data = APPBARDATA {
		cbSize: std::mem::size_of::<APPBARDATA>() as u32,
		rc: RECT {
			left: full.x,
			top: full.y,
			right: full.x + full.width,
			bottom: full.y + full.height,
		},
		..Default::default()
	};
	for edge in [ABE_LEFT, ABE_TOP, ABE_RIGHT, ABE_BOTTOM] {
		data.uEdge = edge;
		let hit = unsafe { SHAppBarMessage(ABM_GETAUTOHIDEBAREX, &mut data) };
		if hit != 0 {
			return true;
		}
	}
	false
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
			let full = info.rcMonitor;
			let fullrect = Rect {
				x: full.left,
				y: full.top,
				width: full.right - full.left,
				height: full.bottom - full.top,
			};
			list.push(Screen {
				handle,
				work: Rect {
					x: work.left,
					y: work.top,
					width: work.right - work.left,
					height: work.bottom - work.top,
				},
				full: fullrect,
				autohide: hasautohide(fullrect),
			});
		}
		true.into()
	}
	let mut list = Vec::<Screen>::new();
	let ok = unsafe { EnumDisplayMonitors(None, None, Some(call), LPARAM(&mut list as *mut _ as isize)) };
	if !ok.as_bool() {
		return Err(anyhow!("enum"));
	}
	list.sort_by_key(|item| (item.work.x, item.work.y));
	Ok(list)
}

fn monitorfromrect(rect: Rect) -> HMONITOR {
	let point = POINT {
		x: rect.x + rect.width / 2,
		y: rect.y + rect.height / 2,
	};
	unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) }
}
