use crate::config::{self, Config, Hotkey, Layout};
use crate::hotkey;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::PCWSTR;

const RELOAD: usize = 1003;
const APPLY: usize = 3101;
const RESET: usize = 3102;
const CLOSE: usize = 3103;

struct Seed {
    owner: HWND,
}

struct State {
    owner: HWND,
    edit: [HWND; 10],
    brush: HBRUSH,
}

pub unsafe fn open(owner: HWND) {
    let class = wstr("UnixishConfig");
    if let Ok(found) = FindWindowW(PCWSTR(class.as_ptr()), PCWSTR::null()) {
        if !found.0.is_null() {
            let _ = ShowWindow(found, SW_SHOWNORMAL);
            let _ = SetForegroundWindow(found);
            return;
        }
    }
    let cursor = LoadCursorW(None, IDC_ARROW).ok();
    let wc = WNDCLASSW {
        hCursor: cursor.unwrap_or_default(),
        lpszClassName: PCWSTR(class.as_ptr()),
        lpfnWndProc: Some(proc),
        hbrBackground: HBRUSH(std::ptr::null_mut()),
        ..Default::default()
    };
    let _ = RegisterClassW(&wc);
    let seed = Box::new(Seed { owner });
    if let Ok(window) = CreateWindowExW(
        WS_EX_TOOLWINDOW,
        PCWSTR(class.as_ptr()),
        PCWSTR(wstr("Unixish").as_ptr()),
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        420,
        500,
        Some(owner),
        None,
        None,
        Some(Box::into_raw(seed) as *const _),
    ) {
        let _ = ShowWindow(window, SW_SHOWNORMAL);
        let _ = UpdateWindow(window);
        let _ = SetForegroundWindow(window);
    }
}

unsafe extern "system" fn proc(window: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            create(window, l);
            LRESULT(0)
        }
        WM_COMMAND => {
            act(window, w.0 & 0xffff);
            LRESULT(0)
        }
        WM_CTLCOLORSTATIC | WM_CTLCOLOREDIT => style(window, w),
        WM_DESTROY => {
            dropstate(window);
            LRESULT(0)
        }
        _ => DefWindowProcW(window, msg, w, l),
    }
}

unsafe fn create(window: HWND, l: LPARAM) {
    let make = l.0 as *const CREATESTRUCTW;
    if make.is_null() {
        return;
    }
    let seed = Box::from_raw((*make).lpCreateParams as *mut Seed);
    let brush = CreateSolidBrush(rgb(19, 19, 22));
    let edit = build(window);
    let state = Box::new(State {
        owner: seed.owner,
        edit,
        brush,
    });
    let _ = SetWindowLongPtrW(window, GWLP_USERDATA, Box::into_raw(state) as isize);
    load(window);
}

unsafe fn build(window: HWND) -> [HWND; 10] {
    let font = GetStockObject(DEFAULT_GUI_FONT);
    let name = [
        "Almost", "Center", "Left", "Right", "Top", "Bottom", "Next", "Prev", "Width", "Height",
    ];
    let mut list = [HWND::default(); 10];
    for (index, text) in name.iter().enumerate() {
        let y = 16 + index as i32 * 36;
        let label = CreateWindowExW(
            Default::default(),
            PCWSTR(wstr("STATIC").as_ptr()),
            PCWSTR(wstr(text).as_ptr()),
            WS_CHILD | WS_VISIBLE,
            14,
            y + 4,
            100,
            22,
            Some(window),
            Some(mid(2000 + index as i32)),
            None,
            None,
        )
        .unwrap_or_default();
        let value = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            PCWSTR(wstr("EDIT").as_ptr()),
            PCWSTR(wstr("").as_ptr()),
            WINDOW_STYLE((WS_CHILD | WS_VISIBLE | WS_TABSTOP).0 | ES_AUTOHSCROLL as u32),
            120,
            y,
            280,
            26,
            Some(window),
            Some(mid(3000 + index as i32)),
            None,
            None,
        )
        .unwrap_or_default();
        setfont(label, font);
        setfont(value, font);
        list[index] = value;
    }
    let a = button(window, "Apply", APPLY as i32, 120, 405, font);
    let r = button(window, "Reset", RESET as i32, 215, 405, font);
    let c = button(window, "Close", CLOSE as i32, 310, 405, font);
    let _ = (a, r, c);
    list
}

unsafe fn button(window: HWND, text: &str, id: i32, x: i32, y: i32, font: HGDIOBJ) -> HWND {
    let value = CreateWindowExW(
        Default::default(),
        PCWSTR(wstr("BUTTON").as_ptr()),
        PCWSTR(wstr(text).as_ptr()),
        WINDOW_STYLE((WS_CHILD | WS_VISIBLE | WS_TABSTOP).0 | BS_PUSHBUTTON as u32),
        x,
        y,
        90,
        30,
        Some(window),
        Some(mid(id)),
        None,
        None,
    )
    .unwrap_or_default();
    setfont(value, font);
    value
}

unsafe fn setfont(window: HWND, font: HGDIOBJ) {
    let _ = SendMessageW(
        window,
        WM_SETFONT,
        Some(WPARAM(font.0 as usize)),
        Some(LPARAM(1)),
    );
}

unsafe fn act(window: HWND, cmd: usize) {
    if cmd == APPLY {
        apply(window);
    } else if cmd == RESET {
        if config::reset().is_ok() {
            let _ = PostMessageW(Some(owner(window)), WM_COMMAND, WPARAM(RELOAD), LPARAM(0));
            load(window);
        }
    } else if cmd == CLOSE {
        let _ = DestroyWindow(window);
    }
}

unsafe fn apply(window: HWND) {
    if let Some(value) = pull(window) {
        if hotkey::check(&value).is_err() {
            alert(window, "Conflict");
            return;
        }
        if config::save(&value).is_ok() {
            let _ = PostMessageW(Some(owner(window)), WM_COMMAND, WPARAM(RELOAD), LPARAM(0));
        } else {
            alert(window, "Error");
        }
    } else {
        alert(window, "Invalid");
    }
}

unsafe fn pull(window: HWND) -> Option<Config> {
    let list = edit(window);
    let width = text(list[8]).parse::<f64>().ok()?.clamp(0.2, 1.0);
    let height = text(list[9]).parse::<f64>().ok()?.clamp(0.2, 1.0);
    Some(Config {
        hotkey: Hotkey {
            almost: text(list[0]),
            center: text(list[1]),
            left: text(list[2]),
            right: text(list[3]),
            top: text(list[4]),
            bottom: text(list[5]),
            next: text(list[6]),
            prev: text(list[7]),
        },
        layout: Layout { width, height },
    })
}

unsafe fn load(window: HWND) {
    if let Ok(value) = config::load() {
        let list = edit(window);
        set(list[0], &value.hotkey.almost);
        set(list[1], &value.hotkey.center);
        set(list[2], &value.hotkey.left);
        set(list[3], &value.hotkey.right);
        set(list[4], &value.hotkey.top);
        set(list[5], &value.hotkey.bottom);
        set(list[6], &value.hotkey.next);
        set(list[7], &value.hotkey.prev);
        set(list[8], &format!("{:.2}", value.layout.width));
        set(list[9], &format!("{:.2}", value.layout.height));
    }
}

unsafe fn style(window: HWND, w: WPARAM) -> LRESULT {
    let hdc = HDC(w.0 as *mut _);
    let _ = SetTextColor(hdc, rgb(240, 240, 244));
    let _ = SetBkColor(hdc, rgb(19, 19, 22));
    let ptr = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut State;
    if ptr.is_null() {
        return LRESULT(0);
    }
    LRESULT((*ptr).brush.0 as isize)
}

unsafe fn dropstate(window: HWND) {
    let ptr = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut State;
    if ptr.is_null() {
        return;
    }
    let state = Box::from_raw(ptr);
    let _ = DeleteObject(state.brush.into());
    let _ = SetWindowLongPtrW(window, GWLP_USERDATA, 0);
}

unsafe fn owner(window: HWND) -> HWND {
    let ptr = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut State;
    if ptr.is_null() {
        HWND::default()
    } else {
        (*ptr).owner
    }
}

unsafe fn edit(window: HWND) -> [HWND; 10] {
    let ptr = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut State;
    if ptr.is_null() {
        [HWND::default(); 10]
    } else {
        (*ptr).edit
    }
}

unsafe fn text(window: HWND) -> String {
    let size = GetWindowTextLengthW(window);
    let mut data = vec![0u16; size as usize + 1];
    let _ = GetWindowTextW(window, &mut data);
    String::from_utf16_lossy(&data)
        .trim_end_matches('\0')
        .trim()
        .to_string()
}

unsafe fn set(window: HWND, value: &str) {
    let _ = SetWindowTextW(window, PCWSTR(wstr(value).as_ptr()));
}

unsafe fn alert(window: HWND, text: &str) {
    let _ = MessageBoxW(
        Some(window),
        PCWSTR(wstr(text).as_ptr()),
        PCWSTR(wstr("Unixish").as_ptr()),
        MB_OK | MB_ICONWARNING,
    );
}

fn rgb(red: u8, green: u8, blue: u8) -> COLORREF {
    COLORREF((red as u32) | ((green as u32) << 8) | ((blue as u32) << 16))
}

fn wstr(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn mid(id: i32) -> HMENU {
    HMENU(id as isize as *mut _)
}
