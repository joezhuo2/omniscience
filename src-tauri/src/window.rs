//! Overlay window placement.

use tauri::{PhysicalPosition, WebviewWindow};

/// Gap between the top of the work area and the overlay, in logical px.
pub const TOP_MARGIN: f64 = 8.0;

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
pub fn place_top_center(window: &WebviewWindow) -> tauri::Result<()> {
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
    window.set_position(PhysicalPosition::new(x, y))
}

#[cfg(test)]
mod tests {
    use super::*;

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
