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
    pub alerts: AlertSettings,
    pub profiles: Vec<Profile>,
    pub active_profile: String,
}

pub const POWER_MODES: [&str; 3] = ["efficiency", "balanced", "performance"];
pub const FAN_MODES: [&str; 3] = ["silent", "performance", "turbo"];
const MAX_PROFILES: usize = 12;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct Profile {
    pub id: String,
    pub name: String,
    #[ts(type = "'efficiency' | 'balanced' | 'performance' | null")]
    pub power_mode: Option<String>,
    pub spl_w: Option<u32>,
    pub sppt_w: Option<u32>,
    pub cpu_boost: Option<u32>,
    pub refresh_hz: Option<u32>,
    #[ts(type = "'silent' | 'performance' | 'turbo' | null")]
    pub fan_mode: Option<String>,
    pub hotkey: String,
    pub apps: Vec<String>,
}

impl Profile {
    fn preset(id: &str, name: &str, power: &str, boost: u32, fan: &str) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            power_mode: Some(power.into()),
            cpu_boost: Some(boost),
            fan_mode: Some(fan.into()),
            ..Self::default()
        }
    }

    fn sanitized(mut self) -> Self {
        self.id = self.id.trim().chars().take(32).collect();
        self.name = self.name.trim().chars().take(32).collect();
        self.power_mode = self
            .power_mode
            .filter(|m| POWER_MODES.contains(&m.as_str()));
        self.fan_mode = self.fan_mode.filter(|m| FAN_MODES.contains(&m.as_str()));
        self.hotkey = self.hotkey.trim().to_string();
        let mut apps: Vec<String> = Vec::new();
        for app in &self.apps {
            let key = app.trim().to_lowercase();
            let key = key.strip_suffix(".exe").unwrap_or(&key).to_string();
            if !key.is_empty() && !apps.contains(&key) && apps.len() < 32 {
                apps.push(key);
            }
        }
        self.apps = apps;
        self
    }
}

pub fn default_profiles() -> Vec<Profile> {
    vec![
        Profile::preset("gaming", "Gaming", "performance", 2, "turbo"),
        Profile::preset("quiet", "Quiet", "balanced", 0, "silent"),
        Profile::preset("battery", "Battery", "efficiency", 0, "silent"),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AlertRule {
    pub enabled: bool,
    pub threshold: f32,
    pub duration_s: u32,
}

impl AlertRule {
    const fn new(enabled: bool, threshold: f32, duration_s: u32) -> Self {
        Self {
            enabled,
            threshold,
            duration_s,
        }
    }

    fn sanitized(mut self, default: Self, min: f32, max: f32) -> Self {
        if !self.threshold.is_finite() {
            self.threshold = default.threshold;
        }
        self.threshold = self.threshold.clamp(min, max);
        self.duration_s = self.duration_s.min(3600);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct AlertSettings {
    pub cpu: AlertRule,
    pub memory: AlertRule,
    pub disk: AlertRule,
    pub igpu: AlertRule,
    pub dgpu: AlertRule,
    pub cpu_temp: AlertRule,
    pub gpu_temp: AlertRule,
    pub disk_space: AlertRule,
    pub network_down: AlertRule,
    pub ping_spike: AlertRule,
    pub battery_full: AlertRule,
    pub battery_low: AlertRule,
    pub quiet_hours: bool,
    pub quiet_start_min: u16,
    pub quiet_end_min: u16,
    #[ts(type = "number")]
    pub snooze_until_ms: u64,
}

impl Default for AlertSettings {
    fn default() -> Self {
        Self {
            cpu: AlertRule::new(true, 90.0, 30),
            memory: AlertRule::new(true, 90.0, 30),
            disk: AlertRule::new(true, 90.0, 30),
            igpu: AlertRule::new(true, 90.0, 30),
            dgpu: AlertRule::new(true, 90.0, 30),
            cpu_temp: AlertRule::new(false, 95.0, 30),
            gpu_temp: AlertRule::new(false, 90.0, 30),
            disk_space: AlertRule::new(false, 10.0, 0),
            network_down: AlertRule::new(false, 0.0, 30),
            ping_spike: AlertRule::new(false, 200.0, 30),
            battery_full: AlertRule::new(false, 80.0, 0),
            battery_low: AlertRule::new(false, 20.0, 0),
            quiet_hours: false,
            quiet_start_min: 22 * 60,
            quiet_end_min: 7 * 60,
            snooze_until_ms: 0,
        }
    }
}

impl AlertSettings {
    fn sanitized(mut self) -> Self {
        let d = Self::default();
        self.cpu = self.cpu.sanitized(d.cpu, 1.0, 100.0);
        self.memory = self.memory.sanitized(d.memory, 1.0, 100.0);
        self.disk = self.disk.sanitized(d.disk, 1.0, 100.0);
        self.igpu = self.igpu.sanitized(d.igpu, 1.0, 100.0);
        self.dgpu = self.dgpu.sanitized(d.dgpu, 1.0, 100.0);
        self.cpu_temp = self.cpu_temp.sanitized(d.cpu_temp, 30.0, 110.0);
        self.gpu_temp = self.gpu_temp.sanitized(d.gpu_temp, 30.0, 110.0);
        self.disk_space = self.disk_space.sanitized(d.disk_space, 1.0, 90.0);
        self.network_down = self.network_down.sanitized(d.network_down, 0.0, 0.0);
        self.ping_spike = self.ping_spike.sanitized(d.ping_spike, 10.0, 5000.0);
        self.battery_full = self.battery_full.sanitized(d.battery_full, 50.0, 100.0);
        self.battery_low = self.battery_low.sanitized(d.battery_low, 5.0, 50.0);
        self.quiet_start_min = self.quiet_start_min.min(1439);
        self.quiet_end_min = self.quiet_end_min.min(1439);
        self
    }
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
            alerts: AlertSettings::default(),
            profiles: default_profiles(),
            active_profile: String::new(),
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
        self.alerts = self.alerts.sanitized();
        let mut profiles: Vec<Profile> = Vec::new();
        for profile in std::mem::take(&mut self.profiles) {
            let profile = profile.sanitized();
            let unique = !profiles.iter().any(|p| p.id == profile.id);
            if !profile.id.is_empty() && unique && profiles.len() < MAX_PROFILES {
                profiles.push(profile);
            }
        }
        self.profiles = profiles;
        if !self.profiles.iter().any(|p| p.id == self.active_profile) {
            self.active_profile.clear();
        }
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
        c.alerts = self.alerts.clone();
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
    fn alert_defaults_match_the_old_fixed_alert() {
        let a = AlertSettings::default();
        for rule in [a.cpu, a.memory, a.disk, a.igpu, a.dgpu] {
            assert_eq!(rule, AlertRule::new(true, 90.0, 30));
        }
        assert!(!a.cpu_temp.enabled && !a.battery_full.enabled && !a.battery_low.enabled);
        let old: Settings = serde_json::from_str(r#"{"sustained_alerts": true}"#).unwrap();
        assert_eq!(old.alerts, a);
    }

    #[test]
    fn sanitizes_alert_rules() {
        let mut s = Settings::default();
        s.alerts.cpu.threshold = f32::NAN;
        s.alerts.ping_spike.threshold = 1.0;
        s.alerts.battery_low.duration_s = 999_999;
        s.alerts.quiet_start_min = 5000;
        let s = s.sanitized();
        assert_eq!(s.alerts.cpu.threshold, 90.0);
        assert_eq!(s.alerts.ping_spike.threshold, 10.0);
        assert_eq!(s.alerts.battery_low.duration_s, 3600);
        assert_eq!(s.alerts.quiet_start_min, 1439);
    }

    #[test]
    fn sanitizes_profiles() {
        let d = Settings::default();
        assert_eq!(
            d.profiles.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(),
            ["gaming", "quiet", "battery"]
        );
        let s = Settings {
            profiles: vec![
                Profile {
                    id: "a".into(),
                    power_mode: Some("warp".into()),
                    fan_mode: Some("turbo".into()),
                    apps: vec![" Game.EXE ".into(), "game".into(), "".into()],
                    ..Profile::default()
                },
                Profile {
                    id: "a".into(),
                    ..Profile::default()
                },
                Profile::default(),
            ],
            active_profile: "missing".into(),
            ..Settings::default()
        }
        .sanitized();
        assert_eq!(s.profiles.len(), 1);
        assert_eq!(s.profiles[0].power_mode, None);
        assert_eq!(s.profiles[0].fan_mode.as_deref(), Some("turbo"));
        assert_eq!(s.profiles[0].apps, ["game"]);
        assert_eq!(s.active_profile, "");
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
