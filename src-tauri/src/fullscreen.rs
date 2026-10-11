//! Hides the overlay while a fullscreen app (game, video, presentation) covers its monitor.

/// A rectangle in physical px: left, top, right, bottom.
pub type Rect = (i32, i32, i32, i32);

/// True if `window` covers all of `monitor` (fullscreen or borderless-fullscreen).
pub fn covers(window: Rect, monitor: Rect) -> bool {
    window.0 <= monitor.0 && window.1 <= monitor.1 && window.2 >= monitor.2 && window.3 >= monitor.3
}

/// Shell windows that span the monitor without being a fullscreen app.
pub fn is_shell_class(class: &str) -> bool {
    matches!(
        class,
        "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd"
    )
}

/// Whether the foreground window is a fullscreen app on the same monitor as `overlay`.
#[cfg(windows)]
pub fn fullscreen_app_over(overlay: isize) -> bool {
    use windows_sys::Win32::Foundation::{HWND, RECT};
    use windows_sys::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetForegroundWindow, GetShellWindow, GetWindowRect,
    };
    // SAFETY: plain Win32 queries on window handles; buffers are correctly sized locals.
    unsafe {
        let fg: HWND = GetForegroundWindow();
        let overlay = overlay as HWND;
        if fg.is_null() || fg == overlay || fg == GetShellWindow() {
            return false;
        }
        let mut class = [0u16; 64];
        let len = GetClassNameW(fg, class.as_mut_ptr(), class.len() as i32);
        if len > 0 && is_shell_class(&String::from_utf16_lossy(&class[..len as usize])) {
            return false;
        }
        let fg_monitor = MonitorFromWindow(fg, MONITOR_DEFAULTTONEAREST);
        if fg_monitor != MonitorFromWindow(overlay, MONITOR_DEFAULTTONEAREST) {
            return false;
        }
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        let mut r: RECT = std::mem::zeroed();
        if GetMonitorInfoW(fg_monitor, &mut info) == 0 || GetWindowRect(fg, &mut r) == 0 {
            return false;
        }
        let m = info.rcMonitor;
        covers(
            (r.left, r.top, r.right, r.bottom),
            (m.left, m.top, m.right, m.bottom),
        )
    }
}

#[cfg(not(windows))]
pub fn fullscreen_app_over(_overlay: isize) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    const MON: Rect = (0, 0, 1920, 1080);

    #[test]
    fn detects_exact_and_oversized_fullscreen() {
        assert!(covers((0, 0, 1920, 1080), MON));
        // Maximized-with-borders windows overhang by a few px.
        assert!(covers((-8, -8, 1928, 1088), MON));
    }

    #[test]
    fn ignores_windows_that_leave_any_edge_uncovered() {
        assert!(!covers((0, 0, 1920, 1032), MON));
        assert!(!covers((100, 0, 1920, 1080), MON));
    }

    #[test]
    fn recognizes_desktop_and_taskbar_classes() {
        assert!(is_shell_class("WorkerW"));
        assert!(is_shell_class("Progman"));
        assert!(!is_shell_class("Chrome_WidgetWin_1"));
    }
}
