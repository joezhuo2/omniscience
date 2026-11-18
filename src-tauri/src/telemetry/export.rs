use std::fmt::Write;

use super::history::HistorySample;
use super::types::HistoryEvent;
use super::usage::{civil_from_days, local_offset_ms};

const DAY_MS: i64 = 86_400_000;
const HEADER: &str = "timestamp,ts_ms,cpu_pct,cpu_temp_c,ram_pct,igpu_pct,dgpu_pct,igpu_temp_c,dgpu_temp_c,disk_pct,net_down_bps,net_up_bps,battery_pct,package_watts";

fn stamp(ms: i64, sep: &str, date_dash: &str, time_colon: &str) -> String {
    let (y, mo, d) = civil_from_days(ms.div_euclid(DAY_MS));
    let s = ms.rem_euclid(DAY_MS) / 1000;
    let (h, mi, se) = (s / 3600, s / 60 % 60, s % 60);
    format!(
        "{y:04}{date_dash}{mo:02}{date_dash}{d:02}{sep}{h:02}{time_colon}{mi:02}{time_colon}{se:02}"
    )
}

fn iso_utc(ts_ms: u64) -> String {
    format!("{}Z", stamp(ts_ms as i64, "T", "-", ":"))
}

pub fn file_name(kind: &str, now_ms: u64, ext: &str) -> String {
    let local = now_ms as i64 + local_offset_ms();
    format!("omniscience-{kind}-{}.{ext}", stamp(local, "-", "", ""))
}

fn cell<T: std::fmt::Display>(v: Option<T>) -> String {
    v.map(|v| v.to_string()).unwrap_or_default()
}

pub fn csv(samples: &[HistorySample]) -> String {
    let mut out = format!("{HEADER}\n");
    for s in samples {
        let _ = writeln!(
            out,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            iso_utc(s.ts_ms),
            s.ts_ms,
            s.cpu_pct,
            cell(s.cpu_temp_c),
            s.ram_pct,
            cell(s.igpu_pct),
            cell(s.dgpu_pct),
            cell(s.igpu_temp_c),
            cell(s.dgpu_temp_c),
            cell(s.disk_pct),
            s.net_down_bps,
            s.net_up_bps,
            cell(s.battery_pct),
            cell(s.package_watts),
        );
    }
    out
}

pub fn json(samples: &[HistorySample], events: &[HistoryEvent]) -> Result<String, String> {
    serde_json::to_string(&serde_json::json!({ "samples": samples, "events": events }))
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::types::HistoryEventKind;

    fn sample(ts_ms: u64) -> HistorySample {
        HistorySample {
            ts_ms,
            cpu_pct: 12.5,
            cpu_temp_c: None,
            ram_pct: 50.0,
            igpu_pct: Some(3.0),
            dgpu_pct: None,
            igpu_temp_c: None,
            dgpu_temp_c: Some(60.0),
            disk_pct: None,
            net_down_bps: 100.0,
            net_up_bps: 0.0,
            battery_pct: None,
            package_watts: Some(9.5),
        }
    }

    #[test]
    fn csv_has_a_header_and_one_row_per_sample() {
        let out = csv(&[sample(1_700_000_040_000)]);
        let mut lines = out.lines();
        assert_eq!(lines.next(), Some(HEADER));
        assert_eq!(
            lines.next(),
            Some("2023-11-14T22:14:00Z,1700000040000,12.5,,50,3,,,60,,100,0,,9.5")
        );
        assert_eq!(lines.next(), None);
        assert_eq!(
            HEADER.split(',').count(),
            out.lines().nth(1).unwrap().split(',').count()
        );
    }

    #[test]
    fn json_wraps_samples_and_events() {
        let event = HistoryEvent {
            ts_ms: 5,
            kind: HistoryEventKind::PowerSource,
            label: "Plugged in".into(),
        };
        let v: serde_json::Value =
            serde_json::from_str(&json(&[sample(1)], &[event]).unwrap()).unwrap();
        assert_eq!(v["samples"][0]["ts_ms"], 1);
        assert_eq!(v["samples"][0]["cpu_temp_c"], serde_json::Value::Null);
        assert_eq!(v["events"][0]["kind"], "power_source");
        assert_eq!(v["events"][0]["label"], "Plugged in");
    }

    #[test]
    fn file_names_use_compact_local_time() {
        let name = file_name("history", 1_700_000_040_000, "csv");
        let stem = name
            .strip_prefix("omniscience-history-")
            .and_then(|n| n.strip_suffix(".csv"))
            .unwrap();
        assert_eq!(stem.len(), 15);
        assert_eq!(stem.as_bytes()[8], b'-');
    }
}
