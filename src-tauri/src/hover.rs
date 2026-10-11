//! Hover detection by cursor polling (approach from islet). While collapsed the window ignores the
//! mouse, so clicks pass through its transparent margins; it therefore cannot see hover itself.

use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

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

/// Polls the cursor and emits `hover` (bool) to the overlay when it enters or leaves.
pub fn spawn(app: AppHandle) {
    std::thread::Builder::new()
        .name("hover-poll".into())
        .spawn(move || {
            let mut was_inside = false;
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
                let (Ok(cursor), Ok(pos), Ok(size), Ok(scale)) = (
                    app.cursor_position(),
                    win.outer_position(),
                    win.outer_size(),
                    win.scale_factor(),
                ) else {
                    continue;
                };
                let now = inside(
                    (cursor.x, cursor.y),
                    (pos.x as f64, pos.y as f64),
                    (size.width as f64, size.height as f64),
                    scale,
                    control.expanded,
                );
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
