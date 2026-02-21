use crate::config::{self, Config, Hotkey, Layout};
use crate::hotkey;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::PCWSTR;

pub struct Item {
    pub id: usize,
    pub text: String,
    pub sep: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Menu,
    Config,
}

struct State {
    owner: HWND,
    list: Vec<Item>,
    hover: i32,
    mode: Mode,
    lab: [HWND; 11],
    edit: [HWND; 10],
    back: HWND,
    apply: HWND,
    reset: HWND,
    close: HWND,
    brush: HBRUSH,
}

const ROW: i32 = 33;
const SEP: i32 = 12;
const PAD: i32 = 6;
const WIDE: i32 = 184;
const CFG: usize = 1002;
const RLD: usize = 1003;
const BID: i32 = 7101;
const AID: i32 = 7102;
const RID: i32 = 7103;
const CID: i32 = 7104;

pub unsafe fn show(owner: HWND, list: Vec<Item>) {
    let class = wstr("UnixishMenu");
    let cursor = LoadCursorW(None, IDC_ARROW).ok();
    let wc = WNDCLASSW {
        style: CS_DROPSHADOW,
        hCursor: cursor.unwrap_or_default(),
        lpszClassName: PCWSTR(class.as_ptr()),
        lpfnWndProc: Some(proc),
        ..Default::default()
    };
    let _ = RegisterClassW(&wc);
    let high = size(&list);
    let point = spot(high, WIDE);
    let data = Box::new(State {
        owner,
        list,
        hover: -1,
        mode: Mode::Menu,
        lab: [HWND::default(); 11],
        edit: [HWND::default(); 10],
        back: HWND::default(),
        apply: HWND::default(),
        reset: HWND::default(),
        close: HWND::default(),
        brush: CreateSolidBrush(rgb(18, 18, 20)),
    });
    if let Ok(window) = CreateWindowExW(
        WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
        PCWSTR(class.as_ptr()),
        PCWSTR(wstr("Unixish").as_ptr()),
        WS_POPUP,
        point.x,
        point.y,
        WIDE,
        high,
        Some(owner),
        None,
        None,
        Some(Box::into_raw(data) as *const _),
    ) {
        let _ = ShowWindow(window, SW_SHOWNOACTIVATE);
        let _ = UpdateWindow(window);
        let _ = SetForegroundWindow(window);
    }
}

unsafe extern "system" fn proc(window: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            let make = l.0 as *const CREATESTRUCTW;
            if !make.is_null() {
                let ptr = (*make).lpCreateParams as *mut State;
                let _ = SetWindowLongPtrW(window, GWLP_USERDATA, ptr as isize);
            }
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            let ptr = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut State;
            if !ptr.is_null() && (*ptr).mode == Mode::Menu {
                let y = ((l.0 >> 16) & 0xffff) as i16 as i32;
                let hit = pick(&(*ptr).list, y);
                if hit != (*ptr).hover {
                    (*ptr).hover = hit;
                    let _ = InvalidateRect(Some(window), None, true);
                }
            }
            LRESULT(0)
        }
        WM_LBUTTONUP | WM_RBUTTONUP => {
            let ptr = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut State;
            if !ptr.is_null() && (*ptr).mode == Mode::Menu {
                let y = ((l.0 >> 16) & 0xffff) as i16 as i32;
                let hit = pick(&(*ptr).list, y);
                if hit >= 0 {
                    let item = &(*ptr).list[hit as usize];
                    if !item.sep {
                        if item.id == CFG {
                            configmode(window, ptr);
                        } else {
                            let _ = PostMessageW(
                                Some((*ptr).owner),
                                WM_COMMAND,
                                WPARAM(item.id),
                                LPARAM(0),
                            );
                            let _ = DestroyWindow(window);
                        }
                    }
                } else {
                    let _ = DestroyWindow(window);
                }
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            action(window, w.0 & 0xffff);
            LRESULT(0)
        }
        WM_CTLCOLORSTATIC | WM_CTLCOLOREDIT | WM_CTLCOLORBTN => color(window, w),
        WM_KILLFOCUS => {
            let next = HWND(w.0 as *mut _);
            if next.0.is_null() || !IsChild(window, next).as_bool() {
                let _ = DestroyWindow(window);
            }
            LRESULT(0)
        }
        WM_KEYDOWN => {
            if w.0 as u32 == VK_ESCAPE.0 as u32 {
                let _ = DestroyWindow(window);
            }
            LRESULT(0)
        }
        WM_PAINT => {
            paint(window);
            LRESULT(0)
        }
        WM_NCDESTROY => {
            let ptr = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut State;
            if !ptr.is_null() {
                let state = Box::from_raw(ptr);
                let _ = DeleteObject(state.brush.into());
                let _ = SetWindowLongPtrW(window, GWLP_USERDATA, 0);
            }
            DefWindowProcW(window, msg, w, l)
        }
        _ => DefWindowProcW(window, msg, w, l),
    }
}

unsafe fn configmode(window: HWND, state: *mut State) {
    (*state).mode = Mode::Config;
    let high = 520;
    let wide = 520;
    let point = expand(window, high, wide);
    let _ = SetWindowPos(
        window,
        None,
        point.x,
        point.y,
        wide,
        high,
        SWP_NOACTIVATE | SWP_NOZORDER,
    );
    controls(window, state);
    load(state);
    let _ = InvalidateRect(Some(window), None, true);
}

unsafe fn menumode(window: HWND, state: *mut State) {
    (*state).mode = Mode::Menu;
    hide(state);
    let high = size(&(*state).list);
    let point = collapse(window, high, WIDE);
    let _ = SetWindowPos(
        window,
        None,
        point.x,
        point.y,
        WIDE,
        high,
        SWP_NOACTIVATE | SWP_NOZORDER,
    );
    let _ = InvalidateRect(Some(window), None, true);
}

unsafe fn action(window: HWND, cmd: usize) {
    let ptr = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut State;
    if ptr.is_null() || (*ptr).mode != Mode::Config {
        return;
    }
    if cmd == BID as usize {
        menumode(window, ptr);
    } else if cmd == AID as usize {
        if let Some(value) = pull(ptr) {
            if hotkey::check(&value).is_err() {
                alert(window, "Conflict");
            } else if config::save(&value).is_ok() {
                let _ = PostMessageW(Some((*ptr).owner), WM_COMMAND, WPARAM(RLD), LPARAM(0));
            } else {
                alert(window, "Error");
            }
        } else {
            alert(window, "Invalid");
        }
    } else if cmd == RID as usize {
        if config::reset().is_ok() {
            let _ = PostMessageW(Some((*ptr).owner), WM_COMMAND, WPARAM(RLD), LPARAM(0));
            load(ptr);
        }
    } else if cmd == CID as usize {
        let _ = DestroyWindow(window);
    }
}

unsafe fn controls(window: HWND, state: *mut State) {
    if !(*state).back.0.is_null() {
        showcontrols(state);
        return;
    }
    let font = GetStockObject(DEFAULT_GUI_FONT);
    let title = CreateWindowExW(
        Default::default(),
        PCWSTR(wstr("STATIC").as_ptr()),
        PCWSTR(wstr("Config").as_ptr()),
        WS_CHILD | WS_VISIBLE,
        18,
        14,
        200,
        24,
        Some(window),
        Some(mid(7010)),
        None,
        None,
    )
    .unwrap_or_default();
    setfont(title, font);
    (*state).lab[0] = title;
    let keys = [
        "Almost", "Center", "Left", "Right", "Top", "Bottom", "Next", "Prev", "Width", "Height",
    ];
    for (index, key) in keys.iter().enumerate() {
        let y = 52 + index as i32 * 40;
        let label = CreateWindowExW(
            Default::default(),
            PCWSTR(wstr("STATIC").as_ptr()),
            PCWSTR(wstr(key).as_ptr()),
            WS_CHILD | WS_VISIBLE,
            18,
            y + 6,
            110,
            24,
            Some(window),
            Some(mid(7200 + index as i32)),
            None,
            None,
        )
        .unwrap_or_default();
        let input = CreateWindowExW(
            Default::default(),
            PCWSTR(wstr("EDIT").as_ptr()),
            PCWSTR(wstr("").as_ptr()),
            WINDOW_STYLE(
                (WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_BORDER).0 | ES_AUTOHSCROLL as u32,
            ),
            130,
            y,
            370,
            30,
            Some(window),
            Some(mid(7300 + index as i32)),
            None,
            None,
        )
        .unwrap_or_default();
        setfont(label, font);
        setfont(input, font);
        (*state).lab[index + 1] = label;
        (*state).edit[index] = input;
    }
    (*state).back = button(window, "Back", BID, 18, 460, font);
    (*state).apply = button(window, "Apply", AID, 260, 460, font);
    (*state).reset = button(window, "Reset", RID, 346, 460, font);
    (*state).close = button(window, "Close", CID, 432, 460, font);
    showcontrols(state);
}

unsafe fn showcontrols(state: *mut State) {
    let list = [
        (*state).lab[0],
        (*state).lab[1],
        (*state).lab[2],
        (*state).lab[3],
        (*state).lab[4],
        (*state).lab[5],
        (*state).lab[6],
        (*state).lab[7],
        (*state).lab[8],
        (*state).lab[9],
        (*state).lab[10],
        (*state).back,
        (*state).apply,
        (*state).reset,
        (*state).close,
        (*state).edit[0],
        (*state).edit[1],
        (*state).edit[2],
        (*state).edit[3],
        (*state).edit[4],
        (*state).edit[5],
        (*state).edit[6],
        (*state).edit[7],
        (*state).edit[8],
        (*state).edit[9],
    ];
    for item in list {
        let _ = ShowWindow(item, SW_SHOW);
    }
}

unsafe fn hide(state: *mut State) {
    let list = [
        (*state).lab[0],
        (*state).lab[1],
        (*state).lab[2],
        (*state).lab[3],
        (*state).lab[4],
        (*state).lab[5],
        (*state).lab[6],
        (*state).lab[7],
        (*state).lab[8],
        (*state).lab[9],
        (*state).lab[10],
        (*state).back,
        (*state).apply,
        (*state).reset,
        (*state).close,
        (*state).edit[0],
        (*state).edit[1],
        (*state).edit[2],
        (*state).edit[3],
        (*state).edit[4],
        (*state).edit[5],
        (*state).edit[6],
        (*state).edit[7],
        (*state).edit[8],
        (*state).edit[9],
    ];
    for item in list {
        if !item.0.is_null() {
            let _ = ShowWindow(item, SW_HIDE);
        }
    }
}

unsafe fn button(window: HWND, text: &str, id: i32, x: i32, y: i32, font: HGDIOBJ) -> HWND {
    let value = CreateWindowExW(
        Default::default(),
        PCWSTR(wstr("BUTTON").as_ptr()),
        PCWSTR(wstr(text).as_ptr()),
        WINDOW_STYLE(
            (WS_CHILD | WS_VISIBLE | WS_TABSTOP).0 | BS_PUSHBUTTON as u32 | BS_FLAT as u32,
        ),
        x,
        y,
        78,
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

unsafe fn load(state: *mut State) {
    if let Ok(value) = config::load() {
        set((*state).edit[0], &value.hotkey.almost);
        set((*state).edit[1], &value.hotkey.center);
        set((*state).edit[2], &value.hotkey.left);
        set((*state).edit[3], &value.hotkey.right);
        set((*state).edit[4], &value.hotkey.top);
        set((*state).edit[5], &value.hotkey.bottom);
        set((*state).edit[6], &value.hotkey.next);
        set((*state).edit[7], &value.hotkey.prev);
        set((*state).edit[8], &format!("{:.2}", value.layout.width));
        set((*state).edit[9], &format!("{:.2}", value.layout.height));
    }
}

unsafe fn pull(state: *mut State) -> Option<Config> {
    Some(Config {
        hotkey: Hotkey {
            almost: text((*state).edit[0]),
            center: text((*state).edit[1]),
            left: text((*state).edit[2]),
            right: text((*state).edit[3]),
            top: text((*state).edit[4]),
            bottom: text((*state).edit[5]),
            next: text((*state).edit[6]),
            prev: text((*state).edit[7]),
        },
        layout: Layout {
            width: text((*state).edit[8]).parse::<f64>().ok()?.clamp(0.2, 1.0),
            height: text((*state).edit[9]).parse::<f64>().ok()?.clamp(0.2, 1.0),
        },
    })
}

unsafe fn color(window: HWND, w: WPARAM) -> LRESULT {
    let ptr = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut State;
    if ptr.is_null() {
        return LRESULT(0);
    }
    let hdc = HDC(w.0 as *mut _);
    let _ = SetTextColor(hdc, rgb(242, 242, 244));
    let _ = SetBkColor(hdc, rgb(18, 18, 20));
    LRESULT((*ptr).brush.0 as isize)
}

unsafe fn paint(window: HWND) {
    let ptr = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut State;
    if ptr.is_null() {
        return;
    }
    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(window, &mut ps);
    let mut rc = RECT::default();
    let _ = GetClientRect(window, &mut rc);
    let _ = FillRect(hdc, &rc, (*ptr).brush);
    let frame = RECT {
        left: 0,
        top: 0,
        right: rc.right - 1,
        bottom: rc.bottom - 1,
    };
    fillround(hdc, frame, rgb(18, 18, 20), rgb(74, 74, 78), 8);
    if (*ptr).mode == Mode::Menu {
        paintmenu(ptr, hdc, rc);
    }
    let _ = EndPaint(window, &ps);
}

unsafe fn paintmenu(state: *mut State, hdc: HDC, rc: RECT) {
    let font = GetStockObject(DEFAULT_GUI_FONT);
    let old = SelectObject(hdc, font);
    let _ = SetBkMode(hdc, TRANSPARENT);
    let mut top = PAD;
    for (index, item) in (*state).list.iter().enumerate() {
        if item.sep {
            let line = RECT {
                left: PAD,
                top: top + SEP / 2,
                right: rc.right - PAD,
                bottom: top + SEP / 2 + 1,
            };
            let brush = CreateSolidBrush(rgb(64, 64, 68));
            let _ = FillRect(hdc, &line, brush);
            let _ = DeleteObject(brush.into());
            top += SEP;
            continue;
        }
        let row = RECT {
            left: PAD + 2,
            top,
            right: rc.right - PAD - 2,
            bottom: top + ROW,
        };
        let (fill, edge) = if (*state).hover == index as i32 {
            (rgb(50, 50, 54), rgb(84, 84, 88))
        } else {
            (rgb(24, 24, 26), rgb(52, 52, 56))
        };
        fillround(hdc, row, fill, edge, 8);
        let _ = SetTextColor(hdc, rgb(242, 242, 244));
        let mut textrect = RECT {
            left: row.left + 11,
            top: row.top,
            right: row.right,
            bottom: row.bottom,
        };
        let mut wide = wstr(&item.text);
        let _ = DrawTextW(
            hdc,
            &mut wide,
            &mut textrect,
            DT_SINGLELINE | DT_VCENTER | DT_LEFT,
        );
        top += ROW;
    }
    let _ = SelectObject(hdc, old);
}

fn size(list: &[Item]) -> i32 {
    PAD * 2
        + list
            .iter()
            .map(|item| if item.sep { SEP } else { ROW })
            .sum::<i32>()
}

fn pick(list: &[Item], y: i32) -> i32 {
    let mut top = PAD;
    for (index, item) in list.iter().enumerate() {
        let high = if item.sep { SEP } else { ROW };
        if y >= top && y < top + high {
            return if item.sep { -1 } else { index as i32 };
        }
        top += high;
    }
    -1
}

unsafe fn spot(high: i32, wide: i32) -> POINT {
    let mut point = POINT::default();
    let _ = GetCursorPos(&mut point);
    let monitor = MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST);
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let _ = GetMonitorInfoW(monitor, &mut info as *mut MONITORINFO as *mut _);
    let work = info.rcWork;
    let mut x = point.x - wide + 10;
    let mut y = point.y - high + 6;
    if x < work.left {
        x = work.left;
    }
    if y < work.top {
        y = work.top;
    }
    if x + wide > work.right {
        x = work.right - wide;
    }
    if y + high > work.bottom {
        y = work.bottom - high;
    }
    POINT { x, y }
}

unsafe fn expand(window: HWND, high: i32, wide: i32) -> POINT {
    let mut rect = RECT::default();
    let _ = GetWindowRect(window, &mut rect);
    let monitor = MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST);
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let _ = GetMonitorInfoW(monitor, &mut info as *mut MONITORINFO as *mut _);
    let work = info.rcWork;
    let mut x = rect.right - wide;
    let mut y = rect.bottom - high;
    if x < work.left {
        x = work.left;
    }
    if y < work.top {
        y = work.top;
    }
    if x + wide > work.right {
        x = work.right - wide;
    }
    if y + high > work.bottom {
        y = work.bottom - high;
    }
    POINT { x, y }
}

unsafe fn collapse(window: HWND, high: i32, wide: i32) -> POINT {
    let mut rect = RECT::default();
    let _ = GetWindowRect(window, &mut rect);
    let monitor = MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST);
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let _ = GetMonitorInfoW(monitor, &mut info as *mut MONITORINFO as *mut _);
    let work = info.rcWork;
    let mut x = rect.right - wide;
    let mut y = rect.top;
    if x < work.left {
        x = work.left;
    }
    if y < work.top {
        y = work.top;
    }
    if x + wide > work.right {
        x = work.right - wide;
    }
    if y + high > work.bottom {
        y = work.bottom - high;
    }
    POINT { x, y }
}

unsafe fn set(window: HWND, value: &str) {
    let _ = SetWindowTextW(window, PCWSTR(wstr(value).as_ptr()));
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

unsafe fn fillround(hdc: HDC, rect: RECT, fill: COLORREF, edge: COLORREF, radius: i32) {
    let brush = CreateSolidBrush(fill);
    let pen = CreatePen(PS_SOLID, 1, edge);
    let oldbrush = SelectObject(hdc, brush.into());
    let oldpen = SelectObject(hdc, pen.into());
    let _ = RoundRect(
        hdc,
        rect.left,
        rect.top,
        rect.right,
        rect.bottom,
        radius,
        radius,
    );
    let _ = SelectObject(hdc, oldbrush);
    let _ = SelectObject(hdc, oldpen);
    let _ = DeleteObject(brush.into());
    let _ = DeleteObject(pen.into());
}
