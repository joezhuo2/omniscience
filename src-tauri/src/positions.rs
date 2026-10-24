//! Remembers where the user dragged the overlay, per monitor layout.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Monitor, PhysicalPosition, WebviewWindow};

/// A monitor rectangle in physical px plus its scale, as used for the layout key and validation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Screen {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale: f64,
    /// Work area (monitor minus taskbars) as left, top, right, bottom edges.
    pub work: (i32, i32, i32, i32),
}

impl From<&Monitor> for Screen {
    fn from(m: &Monitor) -> Self {
        let area = m.work_area();
        Self {
            x: m.position().x,
            y: m.position().y,
            width: m.size().width,
            height: m.size().height,
            scale: m.scale_factor(),
            work: (
                area.position.x,
                area.position.y,
                area.position.x + area.size.width as i32,
                area.position.y + area.size.height as i32,
            ),
        }
    }
}

/// Identifies a monitor arrangement: the same monitors in the same places at the same scales.
pub fn layout_key(screens: &[Screen]) -> String {
    let mut parts: Vec<String> = screens
        .iter()
        .map(|s| format!("{},{},{}x{}@{}", s.x, s.y, s.width, s.height, s.scale))
        .collect();
    parts.sort();
    parts.join("|")
}

/// Identifies a monitor by its size and scale, wherever it sits in the layout. Two identical
/// monitors share a key, and so share a remembered spot.
pub fn monitor_key(s: &Screen) -> String {
    format!("{}x{}@{}", s.width, s.height, s.scale)
}

/// True if a window of `width` x `height` at `pos` keeps its top strip (where the pill is) on
/// some screen, so a saved position can never restore off-screen.
pub fn is_visible(pos: (i32, i32), width: u32, height: u32, screens: &[Screen]) -> bool {
    let (x, y) = pos;
    let (w, h) = (width as i32, height.min(42) as i32);
    screens.iter().any(|s| {
        let (sx, sy) = (s.x, s.y);
        let (ex, ey) = (s.x + s.width as i32, s.y + s.height as i32);
        x >= sx && y >= sy && x + w <= ex && y + h <= ey
    })
}

/// Moves a `width` x `height` window at `pos` the least distance that puts it fully inside `area`
/// (left, top, right, bottom). A window bigger than the area keeps its top-left corner inside.
pub fn clamp_into(
    pos: (i32, i32),
    width: u32,
    height: u32,
    area: (i32, i32, i32, i32),
) -> (i32, i32) {
    let (left, top, right, bottom) = area;
    (
        pos.0.min(right - width as i32).max(left),
        pos.1.min(bottom - height as i32).max(top),
    )
}

/// The screen a window at `pos` belongs to: the one it overlaps most, or else the one whose work
/// area is nearest its center.
pub fn screen_for(pos: (i32, i32), width: u32, height: u32, screens: &[Screen]) -> Option<&Screen> {
    let (x, y, w, h) = (pos.0 as i64, pos.1 as i64, width as i64, height as i64);
    let (cx, cy) = (x + w / 2, y + h / 2);
    let overlap = |s: &Screen| {
        let (l, t, r, b) = (
            s.x as i64,
            s.y as i64,
            s.x as i64 + s.width as i64,
            s.y as i64 + s.height as i64,
        );
        (r.min(x + w) - l.max(x)).max(0) * (b.min(y + h) - t.max(y)).max(0)
    };
    let distance = |s: &Screen| {
        let (l, t, r, b) = s.work;
        let dx = (cx - cx.clamp(l as i64, r as i64)).abs();
        let dy = (cy - cy.clamp(t as i64, b as i64)).abs();
        dx * dx + dy * dy
    };
    screens
        .iter()
        .max_by_key(|s| overlap(s))
        .filter(|s| overlap(s) > 0)
        .or_else(|| screens.iter().min_by_key(|s| distance(s)))
}

/// How close (logical px) the window must be to an edge, or to the top-center, to snap to it.
pub const SNAP_DISTANCE: f64 = 24.0;

/// Snaps each axis of a window at `pos` to the nearest work-area edge (or the horizontal center)
/// within `distance` physical px. Axes with nothing close enough are left alone.
pub fn snap(
    pos: (i32, i32),
    width: u32,
    height: u32,
    area: (i32, i32, i32, i32),
    distance: i32,
) -> (i32, i32) {
    let (left, top, right, bottom) = area;
    let (w, h) = (width as i32, height as i32);
    let nearest = |v: i32, targets: &[i32]| {
        targets
            .iter()
            .copied()
            .filter(|t| (v - t).abs() <= distance)
            .min_by_key(|t| (v - t).abs())
            .unwrap_or(v)
    };
    (
        nearest(pos.0, &[left, left + (right - left - w) / 2, right - w]),
        nearest(pos.1, &[top, bottom - h]),
    )
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct Saved {
    /// Absolute position per monitor layout.
    layouts: HashMap<String, (i32, i32)>,
    /// Position relative to the monitor's top-left corner, per monitor.
    monitors: HashMap<String, (i32, i32)>,
    /// Key of the monitor the overlay was last moved onto.
    last_monitor: Option<String>,
}

impl Saved {
    /// Reads `positions.json`, including the 0.1.0 format (a bare layout map).
    fn parse(json: &str) -> Self {
        serde_json::from_str::<HashMap<String, (i32, i32)>>(json)
            .map(|layouts| Self {
                layouts,
                ..Self::default()
            })
            .or_else(|_| serde_json::from_str(json))
            .unwrap_or_default()
    }

    /// The spot for `screens`: this exact layout's, else the last monitor's relative spot if
    /// that monitor is still connected.
    fn lookup(&self, screens: &[Screen]) -> Option<(i32, i32)> {
        if let Some(&pos) = self.layouts.get(&layout_key(screens)) {
            return Some(pos);
        }
        let key = self.last_monitor.as_ref()?;
        let (dx, dy) = *self.monitors.get(key)?;
        let s = screens.iter().find(|s| &monitor_key(s) == key)?;
        Some((s.x + dx, s.y + dy))
    }
}

pub struct Positions {
    path: Option<PathBuf>,
    saved: Mutex<Saved>,
    /// Bumped on every move; a pending write only lands if no newer move followed it.
    generation: AtomicU64,
    /// Moves are recorded only after startup placement, so programmatic placement is not saved.
    tracking: AtomicBool,
}

impl Positions {
    pub fn load(app: &AppHandle) -> Self {
        let path = app
            .path()
            .app_config_dir()
            .ok()
            .map(|d| d.join("positions.json"));
        let saved = path
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map(|s| Saved::parse(&s))
            .unwrap_or_default();
        Self {
            path,
            saved: Mutex::new(saved),
            generation: AtomicU64::new(0),
            tracking: AtomicBool::new(false),
        }
    }

    fn screens(window: &WebviewWindow) -> Vec<Screen> {
        window
            .available_monitors()
            .unwrap_or_default()
            .iter()
            .map(Screen::from)
            .collect()
    }

    /// The saved position for the current monitor layout, if it is still on-screen.
    pub fn restore(&self, window: &WebviewWindow) -> Option<PhysicalPosition<i32>> {
        let screens = Self::screens(window);
        let size = window.outer_size().ok()?;
        let saved = self.saved.lock().ok()?;
        let pos = saved.lookup(&screens)?;
        is_visible(pos, size.width, size.height, &screens)
            .then(|| PhysicalPosition::new(pos.0, pos.1))
    }

    /// Starts or stops recording moves (stopped around programmatic placement).
    pub fn set_tracking(&self, on: bool) {
        self.tracking.store(on, Ordering::SeqCst);
    }

    /// Records a move and writes it to disk once moves stop for half a second.
    pub fn record(self: &std::sync::Arc<Self>, window: &WebviewWindow, pos: PhysicalPosition<i32>) {
        if !self.tracking.load(Ordering::SeqCst) {
            return;
        }
        let screens = Self::screens(window);
        let size = window.outer_size().unwrap_or_default();
        if let Ok(mut saved) = self.saved.lock() {
            saved.layouts.insert(layout_key(&screens), (pos.x, pos.y));
            if let Some(s) = screen_for((pos.x, pos.y), size.width, size.height, &screens) {
                let key = monitor_key(s);
                saved
                    .monitors
                    .insert(key.clone(), (pos.x - s.x, pos.y - s.y));
                saved.last_monitor = Some(key);
            }
        }
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let this = std::sync::Arc::clone(self);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(500));
            if this.generation.load(Ordering::SeqCst) == generation {
                this.write();
            }
        });
    }

    /// Forgets the position for the current layout (used by "Reset position").
    pub fn forget(&self, window: &WebviewWindow) {
        let key = layout_key(&Self::screens(window));
        if let Ok(mut saved) = self.saved.lock() {
            saved.layouts.remove(&key);
        }
        self.write();
    }

    fn write(&self) {
        let (Some(path), Ok(saved)) = (&self.path, self.saved.lock()) else {
            return;
        };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(&*saved) {
            let _ = std::fs::write(path, json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: Screen = Screen {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        scale: 1.0,
        work: (0, 0, 1920, 1040),
    };
    const B: Screen = Screen {
        x: -2560,
        y: 0,
        width: 2560,
        height: 1440,
        scale: 1.5,
        work: (-2560, 0, 0, 1380),
    };

    #[test]
    fn monitor_key_ignores_where_the_monitor_sits() {
        assert_eq!(monitor_key(&B), monitor_key(&Screen { x: 1920, ..B }));
        assert_ne!(monitor_key(&A), monitor_key(&B));
    }

    #[test]
    fn layout_key_ignores_monitor_order_but_not_scale() {
        assert_eq!(layout_key(&[A, B]), layout_key(&[B, A]));
        let b2 = Screen { scale: 1.25, ..B };
        assert_ne!(layout_key(&[A, B]), layout_key(&[A, b2]));
    }

    #[test]
    fn accepts_positions_on_any_screen() {
        assert!(is_visible((760, 8), 400, 42, &[A, B]));
        assert!(is_visible((-1480, 8), 400, 560, &[A, B]));
    }

    #[test]
    fn rejects_positions_off_every_screen() {
        assert!(!is_visible((760, 8), 400, 42, &[B]));
        assert!(!is_visible((1700, 8), 400, 42, &[A]));
        assert!(!is_visible((760, -30), 400, 42, &[A]));
    }

    #[test]
    fn only_the_pill_strip_needs_to_fit() {
        // An expanded-height window near the bottom edge is fine as long as the pill row fits.
        assert!(is_visible((760, 1030), 400, 560, &[A]));
    }

    #[test]
    fn clamp_pulls_a_window_back_inside() {
        assert_eq!(clamp_into((1700, -20), 420, 48, A.work), (1500, 0));
        assert_eq!(clamp_into((-100, 1030), 420, 48, A.work), (0, 992));
    }

    #[test]
    fn clamp_leaves_an_inside_window_alone() {
        assert_eq!(clamp_into((760, 8), 420, 48, A.work), (760, 8));
    }

    #[test]
    fn clamp_keeps_an_oversized_window_at_the_top_left() {
        assert_eq!(clamp_into((50, 50), 3000, 2000, A.work), (0, 0));
    }

    #[test]
    fn snaps_to_nearby_edges() {
        assert_eq!(snap((15, 1000), 420, 48, A.work, 24), (0, 992));
        assert_eq!(snap((1490, 20), 420, 48, A.work, 24), (1500, 0));
    }

    #[test]
    fn snaps_to_the_horizontal_center() {
        assert_eq!(snap((770, 300), 420, 48, A.work, 24), (750, 300));
    }

    #[test]
    fn leaves_far_positions_alone() {
        assert_eq!(snap((400, 300), 420, 48, A.work, 24), (400, 300));
    }

    #[test]
    fn snaps_on_a_screen_left_of_the_origin() {
        assert_eq!(snap((-2550, 10), 420, 48, B.work, 24), (-2560, 0));
    }

    #[test]
    fn picks_the_screen_with_most_overlap() {
        assert_eq!(screen_for((-200, 8), 420, 48, &[A, B]), Some(&A));
        assert_eq!(screen_for((-300, 8), 420, 48, &[A, B]), Some(&B));
    }

    #[test]
    fn picks_the_nearest_screen_when_off_all_of_them() {
        assert_eq!(screen_for((5000, 8), 420, 48, &[A, B]), Some(&A));
        assert_eq!(screen_for((-9000, 8), 420, 48, &[A, B]), Some(&B));
        assert_eq!(screen_for((0, 0), 420, 48, &[]), None);
    }

    #[test]
    fn reads_the_old_bare_layout_map() {
        let saved = Saved::parse(r#"{"0,0,1920x1080@1": [760, 8]}"#);
        assert_eq!(saved.layouts.get("0,0,1920x1080@1"), Some(&(760, 8)));
    }

    #[test]
    fn round_trips_the_current_format() {
        let mut saved = Saved::default();
        saved.layouts.insert("k".into(), (1, 2));
        let json = serde_json::to_string(&saved).unwrap();
        assert_eq!(Saved::parse(&json).layouts.get("k"), Some(&(1, 2)));
    }

    #[test]
    fn garbage_parses_to_empty() {
        assert!(Saved::parse("not json").layouts.is_empty());
    }

    #[test]
    fn lookup_prefers_the_exact_layout() {
        let mut saved = Saved::default();
        saved.layouts.insert(layout_key(&[A, B]), (100, 8));
        saved.monitors.insert(monitor_key(&A), (500, 8));
        saved.last_monitor = Some(monitor_key(&A));
        assert_eq!(saved.lookup(&[A, B]), Some((100, 8)));
    }

    #[test]
    fn lookup_falls_back_to_the_last_monitor_wherever_it_moved() {
        let mut saved = Saved::default();
        saved.monitors.insert(monitor_key(&B), (300, 20));
        saved.last_monitor = Some(monitor_key(&B));
        let moved = Screen { x: 1920, ..B };
        assert_eq!(saved.lookup(&[A, moved]), Some((2220, 20)));
        assert_eq!(saved.lookup(&[A]), None);
    }
}
