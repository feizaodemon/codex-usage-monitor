//! Native tooltips for freshness, update failures and unavailable quota windows.
use std::sync::Mutex;

use windows::core::{w, PWSTR};
use windows::Win32::Foundation::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::WindowsAndMessaging::*;

#[derive(Default)]
struct State {
    window: isize,
    owner: isize,
    key: String,
    texts: Vec<Vec<u16>>,
}

static STATE: Mutex<State> = Mutex::new(State {
    window: 0,
    owner: 0,
    key: String::new(),
    texts: Vec::new(),
});

// The app has no comctl32 v6 manifest. Exclude the v6-only reserved field so
// the standard tooltip control accepts these TOOLINFO structures.
const TOOL_INFO_SIZE: u32 = std::mem::offset_of!(TTTOOLINFOW, lpReserved) as u32;

pub fn message(chinese: bool, weekly: bool) -> &'static str {
    match (chinese, weekly) {
        (true, false) => "账号未提供 5 小时限额；-- 表示无该窗口的数据，并非剩余 100%。",
        (true, true) => "账号未提供每周限额；-- 表示无该窗口的数据，并非剩余 100%。",
        (false, false) => {
            "The account does not report a 5-hour quota. -- means unavailable, not 100% remaining."
        }
        (false, true) => {
            "The account does not report a weekly quota. -- means unavailable, not 100% remaining."
        }
    }
}

/// Called on the UI thread as displayed data and freshness change. Rects
/// are widget-client coordinates, so the tips also work when it is dragged.
pub fn sync(owner: HWND, regions: Vec<(RECT, String)>) {
    let mut state = STATE.lock().unwrap_or_else(|e| e.into_inner());
    let key = regions
        .iter()
        .map(|(r, t)| format!("{},{},{},{}:{t}", r.left, r.top, r.right, r.bottom))
        .collect::<Vec<_>>()
        .join("|");
    if state.owner == owner.0 as isize && state.key == key {
        return;
    }
    unsafe {
        let old = HWND(state.window as *mut _);
        if state.window != 0 {
            for index in 0..state.texts.len() {
                let info = TTTOOLINFOW {
                    cbSize: TOOL_INFO_SIZE,
                    hwnd: HWND(state.owner as *mut _),
                    uId: index + 1,
                    ..Default::default()
                };
                SendMessageW(
                    old,
                    TTM_DELTOOLW,
                    WPARAM(0),
                    LPARAM(&info as *const _ as isize),
                );
            }
            if state.owner != owner.0 as isize {
                let _ = DestroyWindow(old);
                state.window = 0;
            }
        }
        state.owner = owner.0 as isize;
        state.key = key;
        state.texts.clear();
        if regions.is_empty() {
            return;
        }
        if state.window == 0 {
            let controls = INITCOMMONCONTROLSEX {
                dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
                dwICC: ICC_WIN95_CLASSES,
            };
            let _ = InitCommonControlsEx(&controls);
            let Ok(window) = CreateWindowExW(
                WS_EX_TOPMOST,
                w!("tooltips_class32"),
                w!(""),
                WS_POPUP | WINDOW_STYLE(TTS_ALWAYSTIP | TTS_NOPREFIX),
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                owner,
                None,
                None,
                None,
            ) else {
                state.key.clear();
                return;
            };
            state.window = window.0 as isize;
            SendMessageW(window, TTM_SETMAXTIPWIDTH, WPARAM(0), LPARAM(420));
        }
        for (index, (rect, text)) in regions.into_iter().enumerate() {
            let mut wide: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
            let info = TTTOOLINFOW {
                cbSize: TOOL_INFO_SIZE,
                uFlags: TTF_SUBCLASS,
                hwnd: owner,
                uId: index + 1,
                rect,
                lpszText: PWSTR(wide.as_mut_ptr()),
                ..Default::default()
            };
            SendMessageW(
                HWND(state.window as *mut _),
                TTM_ADDTOOLW,
                WPARAM(0),
                LPARAM(&info as *const _ as isize),
            );
            state.texts.push(wide);
        }
    }
}

pub fn clear() {
    let mut state = STATE.lock().unwrap_or_else(|e| e.into_inner());
    if state.window != 0 {
        unsafe {
            let _ = DestroyWindow(HWND(state.window as *mut _));
        }
    }
    *state = State::default();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_tooltips_follow_available_regions() {
        unsafe {
            let owner = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("STATIC"),
                w!("quota-tip-test"),
                WS_POPUP,
                0,
                0,
                600,
                60,
                None,
                None,
                None,
                None,
            )
            .unwrap();
            let rect = RECT {
                left: 200,
                top: 0,
                right: 400,
                bottom: 20,
            };
            sync(
                owner,
                vec![
                    (rect, message(true, false).into()),
                    (rect, message(true, true).into()),
                ],
            );
            let tooltip = HWND(STATE.lock().unwrap().window as *mut _);
            assert_eq!(
                SendMessageW(tooltip, TTM_GETTOOLCOUNT, WPARAM(0), LPARAM(0)).0,
                2
            );
            sync(owner, vec![(rect, message(false, false).into())]);
            assert_eq!(
                SendMessageW(tooltip, TTM_GETTOOLCOUNT, WPARAM(0), LPARAM(0)).0,
                1
            );
            sync(owner, Vec::new());
            assert_eq!(
                SendMessageW(tooltip, TTM_GETTOOLCOUNT, WPARAM(0), LPARAM(0)).0,
                0
            );
            clear();
            let _ = DestroyWindow(owner);
        }
    }
}
