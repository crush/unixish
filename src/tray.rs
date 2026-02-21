use crate::boot;
use crate::config::{self, Config};
use crate::hotkey::{self, Bind};
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
const MENU_QUIT: usize = 1005;

struct State {
	config: Config,
	bind: Vec<Bind>,
	paused: bool,
}

pub fn run() -> Result<()> {
	unsafe {
		let class = wstr("unixish");
		let title = wstr("unixish");
		let icon = LoadIconW(None, IDI_APPLICATION)?;
		let cursor = LoadCursorW(None, IDC_ARROW)?;
		let wc = WNDCLASSW {
			hCursor: cursor,
			hIcon: icon,
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
		let bind = hotkey::bind(&config)?;
		hotkey::register(window, &bind)?;
		let state = Box::new(State {
			config,
			bind,
			paused: false,
		});
		SetWindowLongPtrW(window, GWLP_USERDATA, Box::into_raw(state) as isize);
		trayadd(window)?;
		looprun();
		traydel(window);
		let ptr = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut State;
		if !ptr.is_null() {
			hotkey::unregister(window, &(*ptr).bind);
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
				} else {
					let _ = hotkey::register(window, &state.bind);
				}
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
	let pausetext = if state.paused { "resume" } else { "pause" };
	let startuptext = if boot::enabled() { "startup off" } else { "startup on" };
	let _ = AppendMenuW(menu, MF_STRING, MENU_PAUSE, PCWSTR(wstr(pausetext).as_ptr()));
	let _ = AppendMenuW(menu, MF_STRING, MENU_CONFIG, PCWSTR(wstr("config").as_ptr()));
	let _ = AppendMenuW(menu, MF_STRING, MENU_RELOAD, PCWSTR(wstr("reload").as_ptr()));
	let _ = AppendMenuW(menu, MF_STRING, MENU_STARTUP, PCWSTR(wstr(startuptext).as_ptr()));
	let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
	let _ = AppendMenuW(menu, MF_STRING, MENU_QUIT, PCWSTR(wstr("quit").as_ptr()));
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

unsafe fn trayadd(window: HWND) -> Result<()> {
	let mut data = NOTIFYICONDATAW::default();
	data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
	data.hWnd = window;
	data.uID = 1;
	data.uFlags = NIF_MESSAGE | NIF_TIP | NIF_ICON;
	data.uCallbackMessage = WM_TRAY;
	data.hIcon = LoadIconW(None, IDI_APPLICATION)?;
	copytip(&mut data, "unixish");
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

fn wstr(text: &str) -> Vec<u16> {
	text.encode_utf16().chain(std::iter::once(0)).collect()
}
