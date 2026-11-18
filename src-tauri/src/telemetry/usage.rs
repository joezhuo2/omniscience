use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::types::CoreMetrics;

const DAY_MS: i64 = 86_400_000;
const SAVE_INTERVAL_MS: u64 = 60_000;
const MAX_GAP_MS: u64 = 10_000;
const KEEP_DAYS: i64 = 62;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct UsageTotals {
    pub data_today_bytes: u64,
    pub data_month_bytes: u64,
    pub energy_today_wh: Option<f32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
struct Day {
    #[serde(default)]
    bytes: f64,
    #[serde(default)]
    wh: Option<f64>,
}

#[derive(Debug)]
pub struct Usage {
    path: Option<PathBuf>,
    days: BTreeMap<String, Day>,
    offset_ms: i64,
    last_ms: Option<u64>,
    last_save_ms: Option<u64>,
}

#[cfg(windows)]
pub(super) fn local_offset_ms() -> i64 {
    use windows_sys::Win32::System::Time::{GetTimeZoneInformation, TIME_ZONE_INFORMATION};

    // SAFETY: TIME_ZONE_INFORMATION is plain data and the pointer is valid for the call.
    let (id, info) = unsafe {
        let mut info: TIME_ZONE_INFORMATION = std::mem::zeroed();
        (GetTimeZoneInformation(&mut info), info)
    };
    let bias = info.Bias
        + match id {
            1 => info.StandardBias,
            2 => info.DaylightBias,
            _ => 0,
        };
    -(bias as i64) * 60_000
}

#[cfg(not(windows))]
pub(super) fn local_offset_ms() -> i64 {
    0
}

pub(super) fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

fn day_key(days: i64) -> String {
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

impl Usage {
    pub fn open(dir: Option<&Path>) -> Self {
        let path = dir.map(|d| d.join("usage.json"));
        let days = path
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Self {
            path,
            days,
            offset_ms: local_offset_ms(),
            last_ms: None,
            last_save_ms: None,
        }
    }

    fn local_day(&self, now_ms: u64) -> i64 {
        (now_ms as i64 + self.offset_ms).div_euclid(DAY_MS)
    }

    pub fn record(&mut self, now_ms: u64, core: &CoreMetrics) {
        let dt = self
            .last_ms
            .replace(now_ms)
            .map(|l| now_ms.saturating_sub(l));
        if let Some(dt @ 1..=MAX_GAP_MS) = dt {
            let day = self
                .days
                .entry(day_key(self.local_day(now_ms)))
                .or_default();
            day.bytes += (core.net.down_bps + core.net.up_bps) * dt as f64 / 1000.0;
            if let Some(w) = core.package_watts {
                *day.wh.get_or_insert(0.0) += f64::from(w) * dt as f64 / 3_600_000.0;
            }
        }
        let last_save = *self.last_save_ms.get_or_insert(now_ms);
        if now_ms.saturating_sub(last_save) >= SAVE_INTERVAL_MS {
            self.last_save_ms = Some(now_ms);
            self.save(now_ms);
        }
    }

    fn save(&mut self, now_ms: u64) {
        let cutoff = day_key(self.local_day(now_ms) - KEEP_DAYS);
        self.days.retain(|k, _| *k >= cutoff);
        let Some(path) = &self.path else { return };
        let Ok(json) = serde_json::to_string(&self.days) else {
            return;
        };
        let tmp = path.with_extension("json.tmp");
        if std::fs::create_dir_all(path.parent().unwrap_or(Path::new(".")))
            .and_then(|_| std::fs::write(&tmp, json))
            .and_then(|_| std::fs::rename(&tmp, path))
            .is_err()
        {
            let _ = std::fs::remove_file(&tmp);
        }
    }

    pub fn totals(&self, now_ms: u64) -> UsageTotals {
        let today = day_key(self.local_day(now_ms));
        let month = &today[..7];
        let day = self.days.get(&today);
        UsageTotals {
            data_today_bytes: day.map_or(0, |d| d.bytes as u64),
            data_month_bytes: self
                .days
                .iter()
                .filter(|(k, _)| k.starts_with(month))
                .map(|(_, d)| d.bytes)
                .sum::<f64>() as u64,
            energy_today_wh: day.and_then(|d| d.wh).map(|w| w as f32),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::types::{Level, MemoryMetric, Metric, NetMetric, Thresholds};

    fn core(bps: f64, watts: Option<f32>) -> CoreMetrics {
        let t = Thresholds::default();
        CoreMetrics {
            cpu: Metric::new(0.0, t),
            cpu_mhz: None,
            cpu_temp_c: None,
            igpu_temp_c: None,
            dgpu_temp_c: None,
            ram: MemoryMetric {
                used_bytes: 1,
                total_bytes: 2,
                percent: 50.0,
                level: Level::Normal,
            },
            disk: None,
            net: NetMetric {
                down_bps: bps,
                up_bps: bps,
                down_pct: 0.0,
                up_pct: 0.0,
                level: Level::Normal,
            },
            igpu: None,
            dgpu: None,
            package_watts: watts,
        }
    }

    fn usage(dir: Option<&Path>) -> Usage {
        let mut u = Usage::open(dir);
        u.offset_ms = 0;
        u
    }

    const D1: u64 = 20_000 * 86_400_000;

    #[test]
    fn civil_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
        assert_eq!(day_key(20_000), "2024-10-04");
    }

    #[test]
    fn accumulates_and_skips_first_call_and_gaps() {
        let mut u = usage(None);
        u.record(D1, &core(500.0, Some(36.0)));
        assert_eq!(u.totals(D1), UsageTotals::default());
        u.record(D1 + 2000, &core(500.0, Some(36.0)));
        let t = u.totals(D1 + 2000);
        assert_eq!(t.data_today_bytes, 2000);
        assert!((t.energy_today_wh.unwrap() - 0.02).abs() < 1e-6);
        u.record(D1 + 20_000, &core(500.0, Some(36.0)));
        assert_eq!(u.totals(D1).data_today_bytes, 2000);
        u.record(D1 + 21_000, &core(500.0, None));
        assert_eq!(u.totals(D1).data_today_bytes, 3000);
    }

    #[test]
    fn no_power_means_no_energy() {
        let mut u = usage(None);
        u.record(D1, &core(1.0, None));
        u.record(D1 + 1000, &core(1.0, None));
        assert_eq!(u.totals(D1 + 1000).energy_today_wh, None);
    }

    #[test]
    fn rollover_and_month_totals() {
        let mut u = usage(None);
        let next = D1 + 86_400_000;
        for t in [next - 2000, next - 1000, next, next + 1000] {
            u.record(t, &core(100.0, None));
        }
        assert_eq!(u.totals(next - 1).data_today_bytes, 200);
        assert_eq!(u.totals(next + 1000).data_today_bytes, 400);
        assert_eq!(u.totals(next + 1000).data_month_bytes, 600);
        let other_month = D1 + 40 * 86_400_000;
        assert_eq!(u.totals(other_month).data_month_bytes, 0);
    }

    #[test]
    fn local_offset_shifts_day() {
        let mut u = usage(None);
        u.offset_ms = 3_600_000;
        let t = D1 - 1_800_000;
        u.record(t, &core(100.0, None));
        u.record(t + 1000, &core(100.0, None));
        assert_eq!(u.totals(t + 1000).data_today_bytes, 200);
        assert_eq!(u.local_day(t), 20_000);
    }

    #[test]
    fn save_load_and_prune() {
        let dir = std::env::temp_dir().join(format!("omni-usage-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut u = usage(Some(&dir));
        u.days.insert(
            day_key(20_000 - 100),
            Day {
                bytes: 5.0,
                wh: None,
            },
        );
        u.record(D1, &core(100.0, Some(10.0)));
        u.record(D1 + 1000, &core(100.0, Some(10.0)));
        assert!(!dir.join("usage.json").exists());
        u.record(D1 + 61_000, &core(100.0, Some(10.0)));
        assert!(dir.join("usage.json").exists());
        let loaded = usage(Some(&dir));
        assert_eq!(loaded.days, u.days);
        assert_eq!(loaded.days.len(), 1);
        assert_eq!(loaded.totals(D1).data_today_bytes, 200);
        std::fs::write(dir.join("usage.json"), "garbage").unwrap();
        assert!(usage(Some(&dir)).days.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
