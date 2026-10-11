//! Hover detection by cursor polling (approach from islet). While collapsed the window ignores the
//! mouse, so clicks pass through its transparent margins; it therefore cannot see hover itself.

use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

use crate::telemetry::Telemetry;

/// Pill geometry in logical px. Must match `--pill-w`, `--pill-h` and `--top-gap` in theme.css.
pub const PILL_W: f64 = 260.0;
pub const PILL_H: f64 = 34.0;
pub const TOP_GAP: f64 = 6.0;
/// Slack around the pill so the edge is not twitchy.
const SLACK: f64 = 4.0;
const POLL: Duration = Duration::from_millis(80);
const POLL_HIDDEN: Duration = Duration::from_millis(500);

pub const HOVER_EVENT: &str = "hover";

/// Whether the cursor (physical px) is over the pill (collapsed) or the whole window (expanded).
pub fn inside(
    cursor: (f64, f64),
    window_pos: (f64, f64),
    window_size: (f64, f64),
    scale: f64,
    expanded: bool,
) -> bool {
    let (top, center) = (window_pos.1, window_pos.0 + window_size.0 / 2.0);
    let (half_w, bottom) = if expanded {
        (window_size.0 / 2.0, top + window_size.1)
    } else {
        (
            (PILL_W / 2.0 + SLACK) * scale,
            top + (TOP_GAP + PILL_H + SLACK) * scale,
        )
    };
    (cursor.0 - center).abs() < half_w && cursor.1 >= top - 2.0 && cursor.1 < bottom
}

/// Cursor position, window position and size (physical px) and scale factor.
struct Probe {
    cursor: (f64, f64),
    pos: (f64, f64),
    size: (f64, f64),
    scale: f64,
}

/// Reads the cursor and window geometry straight from Win32. The Tauri getters (including
/// `hwnd()`, so it is cached in `handle`) each round-trip through the event loop, which woke the
/// main thread about 50 times a second at this poll rate.
#[cfg(windows)]
fn probe(win: &WebviewWindow, handle: &mut isize) -> Option<Probe> {
    use windows_sys::Win32::Foundation::{HWND, POINT, RECT};
    use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetWindowRect};

    if *handle == 0 {
        *handle = win.hwnd().ok()?.0 as isize;
    }
    let hwnd = *handle as HWND;
    let mut pt = POINT { x: 0, y: 0 };
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    // SAFETY: plain Win32 queries on our own window handle with valid out pointers.
    let (cursor_ok, rect_ok, dpi) = unsafe {
        (
            GetCursorPos(&mut pt),
            GetWindowRect(hwnd, &mut rect),
            GetDpiForWindow(hwnd),
        )
    };
    if cursor_ok == 0 || rect_ok == 0 || dpi == 0 {
        return None;
    }
    Some(Probe {
        cursor: (pt.x as f64, pt.y as f64),
        pos: (rect.left as f64, rect.top as f64),
        size: (
            (rect.right - rect.left) as f64,
            (rect.bottom - rect.top) as f64,
        ),
        scale: dpi as f64 / 96.0,
    })
}

#[cfg(not(windows))]
fn probe(win: &WebviewWindow, _handle: &mut isize) -> Option<Probe> {
    let (Ok(cursor), Ok(pos), Ok(size), Ok(scale)) = (
        win.cursor_position(),
        win.outer_position(),
        win.outer_size(),
        win.scale_factor(),
    ) else {
        return None;
    };
    Some(Probe {
        cursor: (cursor.x, cursor.y),
        pos: (pos.x as f64, pos.y as f64),
        size: (size.width as f64, size.height as f64),
        scale,
    })
}

/// Polls the cursor and emits `hover` (bool) to the overlay when it enters or leaves.
pub fn spawn(app: AppHandle) {
    std::thread::Builder::new()
        .name("hover-poll".into())
        .spawn(move || {
            let mut was_inside = false;
            let mut handle = 0;
            loop {
                let control = app.state::<Arc<Telemetry>>().control();
                if !control.visible {
                    was_inside = false;
                    std::thread::sleep(POLL_HIDDEN);
                    continue;
                }
                std::thread::sleep(POLL);
                let Some(win) = app.get_webview_window("main") else {
                    continue;
                };
                let Some(p) = probe(&win, &mut handle) else {
                    continue;
                };
                let now = inside(p.cursor, p.pos, p.size, p.scale, control.expanded);
                if now != was_inside {
                    was_inside = now;
                    let _ = win.emit_to("main", HOVER_EVENT, now);
                }
            }
        })
        .expect("spawn hover poll");
}

#[cfg(test)]
mod tests {
    use super::*;

    // A 420 x 48 window at (500, 0), scale 1.
    const POS: (f64, f64) = (500.0, 0.0);
    const SIZE: (f64, f64) = (420.0, 48.0);

    #[test]
    fn collapsed_counts_only_the_pill() {
        assert!(inside((710.0, 20.0), POS, SIZE, 1.0, false));
        // Over the transparent margin left of the pill.
        assert!(!inside((520.0, 20.0), POS, SIZE, 1.0, false));
        // Below the pill but inside the window.
        assert!(!inside((710.0, 46.0), POS, SIZE, 1.0, false));
    }

    #[test]
    fn expanded_counts_the_whole_window() {
        let size = (420.0, 500.0);
        assert!(inside((520.0, 400.0), POS, size, 1.0, true));
        assert!(!inside((710.0, 520.0), POS, size, 1.0, true));
    }

    #[test]
    fn scales_pill_geometry() {
        // At 2x the pill is 520 physical px wide; 250 px right of center is still inside.
        let size = (840.0, 96.0);
        assert!(inside((500.0 + 420.0 + 250.0, 40.0), POS, size, 2.0, false));
        assert!(!inside(
            (500.0 + 420.0 + 280.0, 40.0),
            POS,
            size,
            2.0,
            false
        ));
    }
}
