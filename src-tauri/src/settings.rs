//! User settings, persisted as JSON in the app config directory.

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use ts_rs::TS;

use crate::telemetry::types::Thresholds;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct Settings {
    /// Percent above which a reading is "warning" (orange).
    pub warn_threshold: f32,
    /// Percent above which a reading is "critical" (red).
    pub crit_threshold: f32,
    /// Telemetry interval in milliseconds (500-5000).
    pub poll_interval_ms: u32,
    /// Global show/hide shortcut in Tauri accelerator syntax, e.g. `CommandOrControl+Shift+O`.
    pub hotkey: String,
    /// Mirrors the OS login items; the OS value wins at startup (see `Shell::init`).
    pub launch_at_startup: bool,
    pub auto_check_updates: bool,
    pub hide_over_fullscreen: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            warn_threshold: 70.0,
            crit_threshold: 90.0,
            poll_interval_ms: 1000,
            hotkey: "CommandOrControl+Shift+O".into(),
            launch_at_startup: false,
            auto_check_updates: true,
            hide_over_fullscreen: true,
        }
    }
}

impl Settings {
    /// Clamps values into their valid ranges so a hand-edited file cannot break the app.
    pub fn sanitized(mut self) -> Self {
        let d = Settings::default();
        if !self.warn_threshold.is_finite() {
            self.warn_threshold = d.warn_threshold;
        }
        if !self.crit_threshold.is_finite() {
            self.crit_threshold = d.crit_threshold;
        }
        self.warn_threshold = self.warn_threshold.clamp(1.0, 98.0);
        self.crit_threshold = self.crit_threshold.clamp(self.warn_threshold + 1.0, 99.0);
        self.poll_interval_ms = self.poll_interval_ms.clamp(500, 5000);
        if self.hotkey.trim().is_empty() {
            self.hotkey = d.hotkey;
        }
        self
    }

    pub fn thresholds(&self) -> Thresholds {
        Thresholds {
            warn: self.warn_threshold,
            crit: self.crit_threshold,
        }
    }

    pub fn interval(&self) -> Duration {
        Duration::from_millis(self.poll_interval_ms as u64)
    }
}

fn path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|d| d.join("settings.json"))
}

/// Loads settings, falling back to defaults for a missing or unreadable file.
pub fn load(app: &AppHandle) -> Settings {
    path(app)
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str::<Settings>(&s).ok())
        .unwrap_or_default()
        .sanitized()
}

pub fn save(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let p = path(app).ok_or("no config directory")?;
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(p, json).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_file_fills_in_defaults() {
        let s: Settings = serde_json::from_str(r#"{"warn_threshold": 60}"#).unwrap();
        assert_eq!(s.warn_threshold, 60.0);
        assert_eq!(s.crit_threshold, 90.0);
        assert!(s.hide_over_fullscreen);
    }

    #[test]
    fn sanitizes_out_of_range_values() {
        let s = Settings {
            warn_threshold: 95.0,
            crit_threshold: 50.0,
            poll_interval_ms: 10,
            hotkey: "  ".into(),
            ..Settings::default()
        }
        .sanitized();
        assert_eq!(s.warn_threshold, 95.0);
        assert_eq!(s.crit_threshold, 96.0);
        assert_eq!(s.poll_interval_ms, 500);
        assert_eq!(s.hotkey, "CommandOrControl+Shift+O");
        let nan = Settings {
            warn_threshold: f32::NAN,
            ..Settings::default()
        }
        .sanitized();
        assert_eq!(nan.warn_threshold, 70.0);
    }
}
