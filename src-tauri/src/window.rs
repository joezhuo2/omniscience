//! Overlay window placement.

use tauri::{LogicalSize, PhysicalPosition, WebviewWindow};

/// Gap between the top of the work area and the overlay, in logical px.
pub const TOP_MARGIN: f64 = 8.0;
/// Collapsed overlay size in logical px. Must match `tauri.conf.json`.
pub const COLLAPSED_SIZE: (f64, f64) = (400.0, 42.0);
/// Tallest the expanded dashboard may grow (logical px). The frontend fits the window to its
/// content within this.
pub const EXPANDED_MAX_HEIGHT: f64 = 560.0;

/// Sets the window height (logical px), clamped to the collapsed and expanded bounds. Width and
/// the top-left corner never change, so the pill does not move while the window grows or shrinks.
pub fn set_height(window: &WebviewWindow, height: f64) -> tauri::Result<()> {
    window.set_size(LogicalSize::new(COLLAPSED_SIZE.0, clamp_height(height)))
}

pub fn clamp_height(height: f64) -> f64 {
    if height.is_finite() {
        height.clamp(COLLAPSED_SIZE.1, EXPANDED_MAX_HEIGHT)
    } else {
        COLLAPSED_SIZE.1
    }
}

/// Top-left position (physical px) that horizontally centers a window at the top of a monitor
/// work area, `margin` physical px below its top edge.
pub fn top_center(
    area_x: i32,
    area_y: i32,
    area_width: u32,
    window_width: u32,
    margin: i32,
) -> (i32, i32) {
    let offset = (i64::from(area_width) - i64::from(window_width)).div_euclid(2);
    (area_x + offset as i32, area_y + margin)
}

/// Moves `window` to the top-center of the primary monitor's work area.
/// Does nothing if no primary monitor is reported.
///
/// Two passes: the first move can cross onto a monitor with a different DPI, and the resulting
/// `WM_DPICHANGED` rescale changes the window's physical size. The size is then re-asserted in
/// logical px and the position recomputed from the final size. The window is still hidden, so
/// nothing visibly jumps.
pub fn place_top_center(window: &WebviewWindow) -> tauri::Result<()> {
    for _ in 0..2 {
        let Some(monitor) = window.primary_monitor()? else {
            return Ok(());
        };
        let area = monitor.work_area();
        let size = window.outer_size()?;
        let margin = (TOP_MARGIN * monitor.scale_factor()).round() as i32;
        let (x, y) = top_center(
            area.position.x,
            area.position.y,
            area.size.width,
            size.width,
            margin,
        );
        window.set_position(PhysicalPosition::new(x, y))?;
        window.set_size(LogicalSize::new(COLLAPSED_SIZE.0, COLLAPSED_SIZE.1))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_requested_heights() {
        assert_eq!(clamp_height(10.0), 42.0);
        assert_eq!(clamp_height(300.0), 300.0);
        assert_eq!(clamp_height(5000.0), 560.0);
        assert_eq!(clamp_height(f64::NAN), 42.0);
    }

    #[test]
    fn centers_on_monitor_at_origin() {
        assert_eq!(top_center(0, 0, 1920, 400, 8), (760, 8));
    }

    #[test]
    fn rounds_down_on_odd_remainder() {
        assert_eq!(top_center(0, 0, 1921, 400, 8), (760, 8));
    }

    #[test]
    fn handles_monitor_left_of_origin() {
        assert_eq!(top_center(-2560, 0, 2560, 400, 8), (-1480, 8));
    }

    #[test]
    fn respects_work_area_below_top_taskbar() {
        assert_eq!(top_center(0, 48, 1920, 400, 8), (760, 56));
    }

    #[test]
    fn uses_physical_sizes_at_150_percent_scale() {
        // 400 logical px window and 8 logical px margin at 1.5x on a 2880 px wide panel.
        assert_eq!(top_center(0, 0, 2880, 600, 12), (1140, 12));
    }

    #[test]
    fn window_wider_than_area_overhangs_evenly() {
        assert_eq!(top_center(0, 0, 300, 400, 0), (-50, 0));
    }
}
