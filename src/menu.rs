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

struct State {
    owner: HWND,
    list: Vec<Item>,
    hover: i32,
}

const ROW: i32 = 33;
const SEP: i32 = 12;
const PAD: i32 = 6;
const WIDTH: i32 = 184;

pub unsafe fn show(owner: HWND, list: Vec<Item>) {
    let class = wstr("UnixishMenu");
    let cursor = LoadCursorW(None, IDC_ARROW).ok();
    let wc = WNDCLASSW {
        hCursor: cursor.unwrap_or_default(),
        lpszClassName: PCWSTR(class.as_ptr()),
        lpfnWndProc: Some(proc),
        ..Default::default()
    };
    let _ = RegisterClassW(&wc);
    let height = size(&list);
    let point = spot(height);
    let data = Box::new(State {
        owner,
        list,
        hover: -1,
    });
    if let Ok(window) = CreateWindowExW(
        WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
        PCWSTR(class.as_ptr()),
        PCWSTR(wstr("Unixish").as_ptr()),
        WS_POPUP | WS_BORDER,
        point.x,
        point.y,
        WIDTH,
        height,
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
            if !ptr.is_null() {
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
            if !ptr.is_null() {
                let y = ((l.0 >> 16) & 0xffff) as i16 as i32;
                let hit = pick(&(*ptr).list, y);
                if hit >= 0 {
                    let item = &(*ptr).list[hit as usize];
                    if !item.sep {
                        let _ = PostMessageW(
                            Some((*ptr).owner),
                            WM_COMMAND,
                            WPARAM(item.id),
                            LPARAM(0),
                        );
                    }
                }
            }
            let _ = DestroyWindow(window);
            LRESULT(0)
        }
        WM_KILLFOCUS => {
            let _ = DestroyWindow(window);
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
                let _ = Box::from_raw(ptr);
                let _ = SetWindowLongPtrW(window, GWLP_USERDATA, 0);
            }
            DefWindowProcW(window, msg, w, l)
        }
        _ => DefWindowProcW(window, msg, w, l),
    }
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
    let bg = CreateSolidBrush(rgb(9, 11, 18));
    let _ = FillRect(hdc, &rc, bg);
    let _ = DeleteObject(bg.into());
    let font = GetStockObject(DEFAULT_GUI_FONT);
    let old = SelectObject(hdc, font);
    let _ = SetBkMode(hdc, TRANSPARENT);
    let mut top = PAD;
    for (index, item) in (*ptr).list.iter().enumerate() {
        if item.sep {
            let line = RECT {
                left: PAD,
                top: top + SEP / 2,
                right: rc.right - PAD,
                bottom: top + SEP / 2 + 1,
            };
            let brush = CreateSolidBrush(rgb(48, 53, 71));
            let _ = FillRect(hdc, &line, brush);
            let _ = DeleteObject(brush.into());
            top += SEP;
            continue;
        }
        let row = RECT {
            left: PAD - 1,
            top,
            right: rc.right - PAD + 1,
            bottom: top + ROW,
        };
        if (*ptr).hover == index as i32 {
            let hover = CreateSolidBrush(rgb(36, 39, 49));
            let _ = FillRect(hdc, &row, hover);
            let edge = CreateSolidBrush(rgb(52, 56, 72));
            let _ = FrameRect(hdc, &row, edge);
            let _ = DeleteObject(edge.into());
            let _ = DeleteObject(hover.into());
        }
        let _ = SetTextColor(hdc, rgb(246, 247, 251));
        let mut text = RECT {
            left: row.left + 11,
            top: row.top,
            right: row.right,
            bottom: row.bottom,
        };
        let mut wide = wstr(&item.text);
        let _ = DrawTextW(
            hdc,
            &mut wide,
            &mut text,
            DT_SINGLELINE | DT_VCENTER | DT_LEFT,
        );
        top += ROW;
    }
    let _ = SelectObject(hdc, old);
    let _ = EndPaint(window, &ps);
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
        let height = if item.sep { SEP } else { ROW };
        if y >= top && y < top + height {
            return if item.sep { -1 } else { index as i32 };
        }
        top += height;
    }
    -1
}

unsafe fn spot(height: i32) -> POINT {
    let mut point = POINT::default();
    let _ = GetCursorPos(&mut point);
    let monitor = MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST);
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let _ = GetMonitorInfoW(monitor, &mut info as *mut MONITORINFO as *mut _);
    let work = info.rcWork;
    let mut x = point.x - WIDTH + 10;
    let mut y = point.y - height + 6;
    if x < work.left {
        x = work.left;
    }
    if y < work.top {
        y = work.top;
    }
    if x + WIDTH > work.right {
        x = work.right - WIDTH;
    }
    if y + height > work.bottom {
        y = work.bottom - height;
    }
    POINT { x, y }
}

fn rgb(red: u8, green: u8, blue: u8) -> COLORREF {
    COLORREF((red as u32) | ((green as u32) << 8) | ((blue as u32) << 16))
}

fn wstr(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
