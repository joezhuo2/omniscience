//! Platform-specific window tweaks.

/// `WS_EX_TOOLWINDOW`: no taskbar button and no Alt+Tab entry.
pub const WS_EX_TOOLWINDOW: isize = 0x0000_0080;
/// `WS_EX_APPWINDOW`: forces a taskbar button. It must be cleared or it wins over TOOLWINDOW.
pub const WS_EX_APPWINDOW: isize = 0x0004_0000;

/// Extended window style for the overlay.
pub fn tool_window_ex_style(ex_style: isize) -> isize {
    (ex_style | WS_EX_TOOLWINDOW) & !WS_EX_APPWINDOW
}

/// Subclass id for [`enforce_tool_window`] (ASCII "OMNI").
#[cfg(windows)]
const SUBCLASS_ID: usize = 0x4F4D_4E49;

/// Hides the overlay from the taskbar and Alt+Tab. Call before the window is first shown.
///
/// tao rebuilds `GWL_EXSTYLE` from its own flags (always including `WS_EX_APPWINDOW`) whenever a
/// flag changes, e.g. on every show/hide, so setting the style once does not stick. A subclass
/// rewrites every extended-style change on its way in instead.
#[cfg(windows)]
pub fn prepare_overlay(window: &tauri::WebviewWindow) {
    use windows_sys::Win32::UI::Shell::SetWindowSubclass;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetWindowLongPtrW, SetWindowLongPtrW,
    };
    let Ok(hwnd) = window.hwnd() else { return };
    let hwnd = hwnd.0 as windows_sys::Win32::Foundation::HWND;
    // SAFETY: hwnd is a live window owned by this process, used on the thread that created it.
    // The subclass procedure only touches the STYLESTRUCT Windows passes to it.
    unsafe {
        SetWindowSubclass(hwnd, Some(enforce_tool_window), SUBCLASS_ID, 0);
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, tool_window_ex_style(style));
    }
}

#[cfg(windows)]
unsafe extern "system" fn enforce_tool_window(
    hwnd: windows_sys::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows_sys::Win32::Foundation::WPARAM,
    lparam: windows_sys::Win32::Foundation::LPARAM,
    _id: usize,
    _data: usize,
) -> windows_sys::Win32::Foundation::LRESULT {
    use windows_sys::Win32::UI::Shell::DefSubclassProc;
    use windows_sys::Win32::UI::WindowsAndMessaging::{GWL_EXSTYLE, STYLESTRUCT, WM_STYLECHANGING};
    if msg == WM_STYLECHANGING && wparam as i32 == GWL_EXSTYLE {
        // SAFETY: for WM_STYLECHANGING, lparam points to a STYLESTRUCT owned by the caller.
        let change = unsafe { &mut *(lparam as *mut STYLESTRUCT) };
        change.styleNew = tool_window_ex_style(change.styleNew as isize) as u32;
    }
    // SAFETY: forwarding the unmodified message arguments to the next procedure in the chain.
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

#[cfg(not(windows))]
pub fn prepare_overlay(_window: &tauri::WebviewWindow) {}

#[cfg(test)]
mod tests {
    use super::*;

    const WS_EX_TOPMOST: isize = 0x0000_0008;
    const WS_EX_LAYERED: isize = 0x0008_0000;

    #[test]
    fn adds_toolwindow() {
        assert_eq!(tool_window_ex_style(0) & WS_EX_TOOLWINDOW, WS_EX_TOOLWINDOW);
    }

    #[test]
    fn clears_appwindow() {
        assert_eq!(tool_window_ex_style(WS_EX_APPWINDOW) & WS_EX_APPWINDOW, 0);
    }

    #[test]
    fn keeps_other_bits() {
        let style = WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_APPWINDOW;
        assert_eq!(
            tool_window_ex_style(style),
            WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TOOLWINDOW
        );
    }

    #[test]
    fn is_idempotent() {
        let once = tool_window_ex_style(WS_EX_APPWINDOW);
        assert_eq!(tool_window_ex_style(once), once);
    }
}
