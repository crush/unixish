use crate::boot;
use crate::config::{self, Config};
use crate::hotkey::{self, Bind};
use crate::icon;
use crate::state;
use crate::update;
use crate::win;
use anyhow::Result;
use std::mem::size_of;
use std::ptr::null_mut;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

const WM_TRAY: u32 = WM_APP + 1;
const MENU_PAUSE: usize = 1001;
const MENU_CONFIG: usize = 1002;
const MENU_RELOAD: usize = 1003;
const MENU_STARTUP: usize = 1004;
const MENU_RESET: usize = 1005;
const MENU_UPDATE: usize = 1006;
const MENU_QUIT: usize = 1007;

struct State {
	config: Config,
	bind: Vec<Bind>,
	paused: bool,
	update: bool,
	iconbig: HICON,
	iconsmall: HICON,
}

pub fn run() -> Result<()> {
	unsafe {
		let class = wstr("Unixish");
		let title = wstr("Unixish");
		let iconbig = icon::load(32).unwrap_or(LoadIconW(None, IDI_APPLICATION)?);
		let iconsmall = icon::load(16).unwrap_or(iconbig);
		let cursor = LoadCursorW(None, IDC_ARROW)?;
		let wc = WNDCLASSW {
			hCursor: cursor,
			hIcon: iconbig,
			lpszClassName: PCWSTR(class.as_ptr()),
			lpfnWndProc: Some(proc),
			..Default::default()
		};
		RegisterClassW(&wc);
		let window = CreateWindowExW(
			Default::default(),
			PCWSTR(class.as_ptr()),
			PCWSTR(title.as_ptr()),
			WS_OVERLAPPEDWINDOW,
			0,
			0,
			0,
			0,
			None,
			None,
			None,
			None,
		)?;
		if window.0.is_null() {
			return Ok(());
		}
		let config = config::load()?;
		let bind = hotkey::bind(&config).unwrap_or_default();
		let mut paused = false;
		if !bind.is_empty() && hotkey::register(window, &bind).is_err() {
			paused = true;
			alert(window, "Conflict");
		}
		if bind.is_empty() {
			paused = true;
			alert(window, "Conflict");
		}
		let state = Box::new(State {
			config,
			bind,
			paused,
			update: update::available(),
			iconbig,
			iconsmall,
		});
		SetWindowLongPtrW(window, GWLP_USERDATA, Box::into_raw(state) as isize);
		trayadd(window, iconsmall)?;
		if state::first() {
			notify(window, "Ready");
		}
		looprun();
		traydel(window);
		let ptr = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut State;
		if !ptr.is_null() {
			hotkey::unregister(window, &(*ptr).bind);
			let _ = DestroyIcon((*ptr).iconsmall);
			if (*ptr).iconbig.0 != (*ptr).iconsmall.0 {
				let _ = DestroyIcon((*ptr).iconbig);
			}
			let _ = Box::from_raw(ptr);
		}
	}
	Ok(())
}

unsafe fn looprun() {
	let mut msg = MSG::default();
	loop {
		let state = GetMessageW(&mut msg, None, 0, 0);
		if state.0 <= 0 {
			break;
		}
		let _ = TranslateMessage(&msg);
		DispatchMessageW(&msg);
	}
}

unsafe extern "system" fn proc(window: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
	match msg {
		WM_HOTKEY => {
			let ptr = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut State;
			if ptr.is_null() || (*ptr).paused {
				return LRESULT(0);
			}
			if let Some(moveid) = hotkey::action(&(*ptr).bind, w.0 as i32) {
				let _ = win::apply(moveid, (*ptr).config.layout.clone());
			}
			LRESULT(0)
		}
		WM_COMMAND => {
			handle(window, w.0 & 0xffff);
			LRESULT(0)
		}
		WM_TRAY => {
			if l.0 as u32 == WM_RBUTTONUP || l.0 as u32 == WM_CONTEXTMENU || l.0 as u32 == WM_LBUTTONUP {
				menu(window);
			}
			LRESULT(0)
		}
		WM_DESTROY => {
			PostQuitMessage(0);
			LRESULT(0)
		}
		_ => DefWindowProcW(window, msg, w, l),
	}
}

unsafe fn handle(window: HWND, cmd: usize) {
	let ptr = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut State;
	if ptr.is_null() {
		return;
	}
	let state = &mut *ptr;
	if cmd == MENU_PAUSE {
		state.paused = !state.paused;
		return;
	}
	if cmd == MENU_CONFIG {
		if let Ok(path) = config::path() {
			let _ = std::process::Command::new("notepad").arg(path).spawn();
		}
		return;
	}
	if cmd == MENU_RELOAD {
		if let Ok(config) = config::load() {
			if let Ok(bind) = hotkey::bind(&config) {
				hotkey::unregister(window, &state.bind);
				if hotkey::register(window, &bind).is_ok() {
					state.config = config;
					state.bind = bind;
					state.paused = false;
				} else {
					let _ = hotkey::register(window, &state.bind);
					alert(window, "Conflict");
				}
			} else {
				alert(window, "Conflict");
			}
		}
		return;
	}
	if cmd == MENU_STARTUP {
		if boot::enabled() {
			let _ = boot::off();
		} else {
			let _ = boot::on();
		}
		return;
	}
	if cmd == MENU_RESET {
		if let Ok(config) = config::reset() {
			if let Ok(bind) = hotkey::bind(&config) {
				hotkey::unregister(window, &state.bind);
				if hotkey::register(window, &bind).is_ok() {
					state.config = config;
					state.bind = bind;
					state.paused = false;
				} else {
					alert(window, "Conflict");
				}
			} else {
				alert(window, "Conflict");
			}
		}
		return;
	}
	if cmd == MENU_UPDATE {
		if update::run().is_ok() {
			state.update = update::available();
			notify(window, "Updated");
		} else {
			alert(window, "Update");
		}
		return;
	}
	if cmd == MENU_QUIT {
		let _ = DestroyWindow(window);
	}
}

unsafe fn menu(window: HWND) {
	let ptr = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut State;
	if ptr.is_null() {
		return;
	}
	let state = &*ptr;
	let menu = CreatePopupMenu().unwrap_or(HMENU(null_mut()));
	if menu.0.is_null() {
		return;
	}
	let pausetext = if state.paused { "Resume" } else { "Pause" };
	let startuptext = if boot::enabled() { "Startup Off" } else { "Startup On" };
	let _ = AppendMenuW(menu, MF_STRING, MENU_PAUSE, PCWSTR(wstr(pausetext).as_ptr()));
	let _ = AppendMenuW(menu, MF_STRING, MENU_CONFIG, PCWSTR(wstr("Config").as_ptr()));
	let _ = AppendMenuW(menu, MF_STRING, MENU_RELOAD, PCWSTR(wstr("Reload").as_ptr()));
	let _ = AppendMenuW(menu, MF_STRING, MENU_STARTUP, PCWSTR(wstr(startuptext).as_ptr()));
	let _ = AppendMenuW(menu, MF_STRING, MENU_RESET, PCWSTR(wstr("Reset").as_ptr()));
	let updatetext = if state.update { "Update Now" } else { "Update" };
	let _ = AppendMenuW(menu, MF_STRING, MENU_UPDATE, PCWSTR(wstr(updatetext).as_ptr()));
	let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
	let _ = AppendMenuW(menu, MF_STRING, MENU_QUIT, PCWSTR(wstr("Quit").as_ptr()));
	let mut point = POINT::default();
	let _ = GetCursorPos(&mut point);
	let _ = SetForegroundWindow(window);
	let _ = TrackPopupMenu(
		menu,
		TPM_LEFTALIGN | TPM_RIGHTBUTTON,
		point.x,
		point.y,
		Some(0),
		window,
		None,
	);
	let _ = DestroyMenu(menu);
}

unsafe fn trayadd(window: HWND, icon: HICON) -> Result<()> {
	let mut data = NOTIFYICONDATAW::default();
	data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
	data.hWnd = window;
	data.uID = 1;
	data.uFlags = NIF_MESSAGE | NIF_TIP | NIF_ICON;
	data.uCallbackMessage = WM_TRAY;
	data.hIcon = icon;
	copytip(&mut data, "Unixish");
	let ok = Shell_NotifyIconW(NIM_ADD, &data).as_bool();
	if !ok {
		return Err(anyhow::anyhow!("tray"));
	}
	Ok(())
}

unsafe fn traydel(window: HWND) {
	let mut data = NOTIFYICONDATAW::default();
	data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
	data.hWnd = window;
	data.uID = 1;
	let _ = Shell_NotifyIconW(NIM_DELETE, &data);
}

fn copytip(data: &mut NOTIFYICONDATAW, text: &str) {
	let wide: Vec<u16> = text.encode_utf16().collect();
	let max = data.szTip.len().saturating_sub(1).min(wide.len());
	data.szTip[..max].copy_from_slice(&wide[..max]);
	data.szTip[max] = 0;
}

unsafe fn notify(window: HWND, text: &str) {
	let mut data = NOTIFYICONDATAW::default();
	data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
	data.hWnd = window;
	data.uID = 1;
	data.uFlags = NIF_INFO;
	data.dwInfoFlags = NIIF_INFO;
	copytext(&mut data.szInfoTitle, "Unixish");
	copytext(&mut data.szInfo, text);
	let _ = Shell_NotifyIconW(NIM_MODIFY, &data);
}

unsafe fn alert(window: HWND, text: &str) {
	let _ = MessageBoxW(
		Some(window),
		PCWSTR(wstr(text).as_ptr()),
		PCWSTR(wstr("Unixish").as_ptr()),
		MB_OK | MB_ICONWARNING,
	);
}

fn copytext<const N: usize>(value: &mut [u16; N], text: &str) {
	let wide: Vec<u16> = text.encode_utf16().collect();
	let max = value.len().saturating_sub(1).min(wide.len());
	value[..max].copy_from_slice(&wide[..max]);
	value[max] = 0;
}

fn wstr(text: &str) -> Vec<u16> {
	text.encode_utf16().chain(std::iter::once(0)).collect()
}
