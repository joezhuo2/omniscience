//! Hover detection by cursor polling (approach from islet). While collapsed the window ignores the
//! mouse, so clicks pass through its transparent margins; it therefore cannot see hover itself.

use std::sync::Arc;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

use crate::telemetry::Telemetry;

/// Pill geometry in logical px. Must match `--pill-h` and `--top-gap` in theme.css; the width is a setting.
pub const PILL_H: f64 = 34.0;
pub const TOP_GAP: f64 = 6.0;
/// Slack around the pill so the edge is not twitchy.
const SLACK: f64 = 4.0;
const POLL: Duration = Duration::from_millis(80);
const POLL_HIDDEN: Duration = Duration::from_millis(500);

pub const HOVER_EVENT: &str = "hover";
pub const DODGE_EVENT: &str = "hover-dodge";
const TAB_STRIP_H: f64 = 48.0;
const DODGE_RECHECK: Duration = Duration::from_millis(500);

/// Whether the cursor (physical px) is over the pill (collapsed) or the whole window (expanded).
pub fn inside(
    cursor: (f64, f64),
    window_pos: (f64, f64),
    window_size: (f64, f64),
    scale: f64,
    expanded: bool,
    pill_w: f64,
) -> bool {
    let (top, center) = (window_pos.1, window_pos.0 + window_size.0 / 2.0);
    let (half_w, bottom) = if expanded {
        (window_size.0 / 2.0, top + window_size.1)
    } else {
        (
            (pill_w / 2.0 + SLACK) * scale,
            top + (TOP_GAP + PILL_H + SLACK) * scale,
        )
    };
    (cursor.0 - center).abs() < half_w && cursor.1 >= top - 2.0 && cursor.1 < bottom
}

fn tabs_under_pill(
    browser: (f64, f64, f64),
    window_pos: (f64, f64),
    window_size: (f64, f64),
    scale: f64,
) -> bool {
    let (left, top, right) = browser;
    let pill_top = window_pos.1 + TOP_GAP * scale;
    let pill_bottom = pill_top + PILL_H * scale;
    let center = window_pos.0 + window_size.0 / 2.0;
    top < pill_bottom && top + TAB_STRIP_H * scale > pill_top && center >= left && center <= right
}

#[cfg(windows)]
const BROWSERS: &[&str] = &[
    "chrome",
    "msedge",
    "firefox",
    "brave",
    "opera",
    "vivaldi",
    "arc",
    "zen",
    "librewolf",
    "waterfox",
    "chromium",
    "thorium",
];

#[cfg(windows)]
fn foreground_browser() -> Option<(f64, f64, f64)> {
    use windows_sys::Win32::Foundation::{CloseHandle, RECT};
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId,
    };

    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut pid = 0u32;
    let mut buf = [0u16; 520];
    let mut len = buf.len() as u32;
    // SAFETY: plain Win32 queries with valid out pointers; the process handle is closed before returning.
    let ok = unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() || GetWindowRect(hwnd, &mut rect) == 0 {
            return None;
        }
        GetWindowThreadProcessId(hwnd, &mut pid);
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return None;
        }
        let ok = QueryFullProcessImageNameW(process, 0, buf.as_mut_ptr(), &mut len);
        CloseHandle(process);
        ok
    };
    if ok == 0 {
        return None;
    }
    let path = String::from_utf16_lossy(&buf[..len as usize]);
    let stem = std::path::Path::new(&path)
        .file_stem()?
        .to_string_lossy()
        .to_lowercase();
    BROWSERS.contains(&stem.as_str()).then_some((
        rect.left as f64,
        rect.top as f64,
        rect.right as f64,
    ))
}

#[cfg(not(windows))]
fn foreground_browser() -> Option<(f64, f64, f64)> {
    None
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
            let mut dodging = false;
            let mut checked = Instant::now();
            let mut handle = 0;
            loop {
                let control = app.state::<Arc<Telemetry>>().control();
                if !control.visible {
                    was_inside = false;
                    dodging = false;
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
                let now = inside(
                    p.cursor,
                    p.pos,
                    p.size,
                    p.scale,
                    control.expanded,
                    control.pill_w,
                );
                let dodge = if !(control.dodge_tabs && !control.expanded && now) {
                    false
                } else if !was_inside || checked.elapsed() >= DODGE_RECHECK {
                    checked = Instant::now();
                    foreground_browser().is_some_and(|b| tabs_under_pill(b, p.pos, p.size, p.scale))
                } else {
                    dodging
                };
                if dodge != dodging {
                    dodging = dodge;
                    let _ = win.emit_to("main", DODGE_EVENT, dodge);
                }
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
    const PILL_W: f64 = 260.0;
    const POS: (f64, f64) = (500.0, 0.0);
    const SIZE: (f64, f64) = (420.0, 48.0);

    #[test]
    fn collapsed_counts_only_the_pill() {
        assert!(inside((710.0, 20.0), POS, SIZE, 1.0, false, PILL_W));
        // Over the transparent margin left of the pill.
        assert!(!inside((520.0, 20.0), POS, SIZE, 1.0, false, PILL_W));
        // Below the pill but inside the window.
        assert!(!inside((710.0, 46.0), POS, SIZE, 1.0, false, PILL_W));
    }

    #[test]
    fn expanded_counts_the_whole_window() {
        let size = (420.0, 500.0);
        assert!(inside((520.0, 400.0), POS, size, 1.0, true, PILL_W));
        assert!(!inside((710.0, 520.0), POS, size, 1.0, true, PILL_W));
    }

    #[test]
    fn scales_pill_geometry() {
        // At 2x the pill is 520 physical px wide; 250 px right of center is still inside.
        let size = (840.0, 96.0);
        assert!(inside(
            (500.0 + 420.0 + 250.0, 40.0),
            POS,
            size,
            2.0,
            false,
            PILL_W
        ));
        assert!(!inside(
            (500.0 + 420.0 + 280.0, 40.0),
            POS,
            size,
            2.0,
            false,
            PILL_W
        ));
    }

    #[test]
    fn narrower_pill_shrinks_the_hit_area() {
        assert!(inside((710.0 + 100.0, 20.0), POS, SIZE, 1.0, false, PILL_W));
        assert!(!inside((710.0 + 100.0, 20.0), POS, SIZE, 1.0, false, 160.0));
    }

    #[test]
    fn tab_strip_must_reach_the_pill_horizontally_and_vertically() {
        let (pos, size) = ((500.0, 0.0), (420.0, 48.0));
        assert!(tabs_under_pill((0.0, 0.0, 1920.0), pos, size, 1.0));
        assert!(!tabs_under_pill((0.0, 0.0, 600.0), pos, size, 1.0));
        assert!(!tabs_under_pill((900.0, 0.0, 1920.0), pos, size, 1.0));
        assert!(!tabs_under_pill((0.0, 100.0, 1920.0), pos, size, 1.0));
        assert!(tabs_under_pill((0.0, -30.0, 1920.0), pos, size, 1.0));
        assert!(!tabs_under_pill((0.0, 100.0, 1920.0), pos, size, 1.0));
        assert!(tabs_under_pill(
            (0.0, 80.0, 1920.0),
            (500.0, 60.0),
            size,
            2.0
        ));
    }
}
