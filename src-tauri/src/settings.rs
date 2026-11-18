//! User settings, persisted as JSON in the app config directory (standalone) or the embedded data
//! directory.

use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::telemetry::Control;
use crate::telemetry::types::Thresholds;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct Settings {
    /// Percent above which a reading is "warning" (orange).
    pub warn_threshold: f32,
    /// Percent above which a reading is "critical" (red).
    pub crit_threshold: f32,
    /// CPU temperature in °C above which the temperature ring is "warning".
    pub cpu_temp_warn_c: f32,
    /// CPU temperature in °C above which the temperature ring is "critical".
    pub cpu_temp_crit_c: f32,
    /// Telemetry interval in milliseconds (500-5000).
    pub poll_interval_ms: u32,
    /// Global show/hide shortcut in Tauri accelerator syntax, e.g. `CommandOrControl+Shift+O`.
    pub hotkey: String,
    /// Mirrors the OS login items; the OS value wins at startup (see `Shell::init`).
    pub launch_at_startup: bool,
    pub auto_check_updates: bool,
    pub hide_over_fullscreen: bool,
    /// Host pinged for the latency readout while the dashboard is open; empty turns it off.
    pub ping_target: String,
    /// Look up the public IP (one DNS query to OpenDNS every ten minutes while open). Off by
    /// default because it contacts a third party.
    pub show_public_ip: bool,
    /// Shows the Config tab and allows GPU, power and display changes. Off by default.
    pub config_enabled: bool,
    /// Desktop notification when CPU, memory, disk or a GPU stays above 90% for 30 seconds.
    pub sustained_alerts: bool,
    /// Shows each reading's source, age and proxy status on the dashboard. Problems always show.
    pub sensor_details: bool,
    pub history_hours: u32,
    pub hover_open_delay_ms: u32,
    pub hover_close_delay_ms: u32,
    pub pill_rings: Vec<String>,
    pub pill_numbers: PillNumbers,
    pub dodge_browser_tabs: bool,
    pub energy_price_per_kwh: f32,
    pub energy_currency: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PillNumbers {
    #[default]
    Off,
    Percent,
    Temp,
}

pub const RING_KEYS: [&str; 6] = ["cpu", "ram", "igpu", "dgpu", "disk", "net"];
const MAX_HOVER_DELAY_MS: u32 = 2000;

impl Default for Settings {
    fn default() -> Self {
        Self {
            warn_threshold: 70.0,
            crit_threshold: 90.0,
            cpu_temp_warn_c: 80.0,
            cpu_temp_crit_c: 90.0,
            poll_interval_ms: 1000,
            hotkey: "CommandOrControl+Shift+O".into(),
            launch_at_startup: false,
            auto_check_updates: true,
            hide_over_fullscreen: true,
            ping_target: "1.1.1.1".into(),
            show_public_ip: false,
            config_enabled: false,
            sustained_alerts: true,
            sensor_details: false,
            history_hours: 24,
            hover_open_delay_ms: 150,
            hover_close_delay_ms: 250,
            pill_rings: RING_KEYS.iter().map(|k| k.to_string()).collect(),
            pill_numbers: PillNumbers::Off,
            dodge_browser_tabs: true,
            energy_price_per_kwh: 0.0,
            energy_currency: "$".into(),
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
        if !self.cpu_temp_warn_c.is_finite() {
            self.cpu_temp_warn_c = d.cpu_temp_warn_c;
        }
        if !self.cpu_temp_crit_c.is_finite() {
            self.cpu_temp_crit_c = d.cpu_temp_crit_c;
        }
        self.cpu_temp_warn_c = self.cpu_temp_warn_c.clamp(30.0, 109.0);
        self.cpu_temp_crit_c = self
            .cpu_temp_crit_c
            .clamp(self.cpu_temp_warn_c + 1.0, 110.0);
        self.poll_interval_ms = self.poll_interval_ms.clamp(500, 5000);
        self.history_hours = self.history_hours.clamp(1, 720);
        if self.hotkey.trim().is_empty() {
            self.hotkey = d.hotkey;
        }
        // 253 is the longest DNS name.
        self.ping_target = self.ping_target.trim().chars().take(253).collect();
        self.hover_open_delay_ms = self.hover_open_delay_ms.min(MAX_HOVER_DELAY_MS);
        self.hover_close_delay_ms = self.hover_close_delay_ms.min(MAX_HOVER_DELAY_MS);
        if !self.energy_price_per_kwh.is_finite() {
            self.energy_price_per_kwh = 0.0;
        }
        self.energy_price_per_kwh = self.energy_price_per_kwh.clamp(0.0, 1000.0);
        self.energy_currency = self.energy_currency.trim().chars().take(4).collect();
        let mut rings: Vec<String> = Vec::new();
        for key in &self.pill_rings {
            if RING_KEYS.contains(&key.as_str()) && !rings.contains(key) {
                rings.push(key.clone());
            }
        }
        self.pill_rings = if rings.is_empty() {
            d.pill_rings
        } else {
            rings
        };
        self
    }

    pub fn pill_width(&self) -> f64 {
        let n = self.pill_rings.len() as f64;
        let numbers = if self.pill_numbers == PillNumbers::Off {
            0.0
        } else {
            n * 24.0
        };
        88.0 + n * 22.0 + (n - 1.0) * 8.0 + numbers
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

    /// Copies the settings the collector uses into its control state.
    pub fn apply(&self, c: &mut Control) {
        c.thresholds = self.thresholds();
        c.interval = self.interval();
        c.ping_target = self.ping_target.as_str().into();
        c.public_ip = self.show_public_ip;
        c.sustained_alerts = self.sustained_alerts;
        c.history_hours = self.history_hours;
        c.pill_w = self.pill_width();
        c.dodge_tabs = self.dodge_browser_tabs;
    }
}

const FILE: &str = "settings.json";

/// Loads settings from `dir`, falling back to defaults for a missing or unreadable file.
pub fn load(dir: &Path) -> Settings {
    std::fs::read_to_string(dir.join(FILE))
        .ok()
        .and_then(|s| serde_json::from_str::<Settings>(&s).ok())
        .unwrap_or_default()
        .sanitized()
}

pub fn save(dir: &Path, settings: &Settings) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(FILE), json).map_err(|e| e.to_string())
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
        assert!(!s.sensor_details);
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
        assert_eq!(s.history_hours, 24);
        assert_eq!(s.hotkey, "CommandOrControl+Shift+O");
        let nan = Settings {
            warn_threshold: f32::NAN,
            ..Settings::default()
        }
        .sanitized();
        assert_eq!(nan.warn_threshold, 70.0);
    }

    #[test]
    fn sanitizes_cpu_temp_limits() {
        let s = Settings::default();
        assert_eq!((s.cpu_temp_warn_c, s.cpu_temp_crit_c), (80.0, 90.0));
        let swapped = Settings {
            cpu_temp_warn_c: 100.0,
            cpu_temp_crit_c: 50.0,
            ..Settings::default()
        }
        .sanitized();
        assert_eq!(swapped.cpu_temp_warn_c, 100.0);
        assert_eq!(swapped.cpu_temp_crit_c, 101.0);
        let low = Settings {
            cpu_temp_warn_c: 0.0,
            cpu_temp_crit_c: f32::NAN,
            ..Settings::default()
        }
        .sanitized();
        assert_eq!(low.cpu_temp_warn_c, 30.0);
        assert_eq!(low.cpu_temp_crit_c, 90.0);
        let top = Settings {
            cpu_temp_warn_c: 500.0,
            cpu_temp_crit_c: 500.0,
            ..Settings::default()
        }
        .sanitized();
        assert_eq!(top.cpu_temp_warn_c, 109.0);
        assert_eq!(top.cpu_temp_crit_c, 110.0);
    }

    #[test]
    fn clamps_history_retention() {
        let low = Settings {
            history_hours: 0,
            ..Settings::default()
        };
        assert_eq!(low.sanitized().history_hours, 1);
        let high = Settings {
            history_hours: 10_000,
            ..Settings::default()
        };
        assert_eq!(high.sanitized().history_hours, 720);
    }

    #[test]
    fn sanitizes_pill_rings_and_hover_delays() {
        let s = Settings {
            pill_rings: vec!["ram".into(), "bogus".into(), "cpu".into(), "ram".into()],
            hover_open_delay_ms: 99_999,
            ..Settings::default()
        }
        .sanitized();
        assert_eq!(s.pill_rings, ["ram", "cpu"]);
        assert_eq!(s.hover_open_delay_ms, 2000);
        let empty = Settings {
            pill_rings: vec![],
            ..Settings::default()
        }
        .sanitized();
        assert_eq!(empty.pill_rings.len(), 6);
    }

    #[test]
    fn pill_width_follows_rings_and_numbers() {
        let d = Settings::default();
        assert_eq!(d.pill_width(), 260.0);
        let numbers = Settings {
            pill_numbers: PillNumbers::Percent,
            ..Settings::default()
        };
        assert_eq!(numbers.pill_width(), 404.0);
        let two = Settings {
            pill_rings: vec!["cpu".into(), "ram".into()],
            ..Settings::default()
        };
        assert_eq!(two.pill_width(), 140.0);
    }

    #[test]
    fn trims_the_ping_target() {
        let s = Settings {
            ping_target: "  example.com ".into(),
            ..Settings::default()
        }
        .sanitized();
        assert_eq!(s.ping_target, "example.com");
    }
}
