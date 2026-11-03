use std::collections::VecDeque;
use std::fmt::Display;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Manager, State, command};

use crate::charge_limit::ChargeLimit;
use crate::display::DisplayConfig;
use crate::gpu_mode::GpuConfig;
use crate::power_limits::PowerLimits;
use crate::power_mode::PowerConfig;
use crate::telemetry::Telemetry;
use crate::telemetry::types::Snapshot;
use crate::{battery_saver, config_gate, cpu_boost};

const MAX_ERRORS: usize = 20;
const MAX_TEXT_CHARS: usize = 300;

static ERRORS: ErrorLog = ErrorLog::new();

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ErrorEntry {
    timestamp_ms: u64,
    message: String,
}

struct ErrorLog {
    entries: Mutex<VecDeque<ErrorEntry>>,
}

impl ErrorLog {
    const fn new() -> Self {
        Self {
            entries: Mutex::new(VecDeque::new()),
        }
    }

    fn record(&self, timestamp_ms: u64, message: &str) {
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        if entries.len() == MAX_ERRORS {
            entries.pop_front();
        }
        entries.push_back(ErrorEntry {
            timestamp_ms,
            message: truncated(message),
        });
    }

    fn recent(&self) -> Vec<ErrorEntry> {
        self.entries
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .cloned()
            .collect()
    }
}

fn truncated(text: &str) -> String {
    text.chars().take(MAX_TEXT_CHARS).collect()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_default()
}

pub fn log_error(message: impl Display) {
    eprintln!("{message}");
    ERRORS.record(now_ms(), &message.to_string());
}

#[derive(Debug, PartialEq, Serialize)]
struct SensorStatus {
    name: &'static str,
    available: bool,
}

#[derive(Debug, PartialEq, Serialize)]
struct ControlStatus {
    name: &'static str,
    supported: bool,
    error: Option<String>,
}

#[derive(Serialize)]
struct AppInfo {
    name: String,
    version: String,
}

#[derive(Serialize)]
struct Platform {
    os: &'static str,
    arch: &'static str,
}

#[derive(Serialize)]
struct Meta {
    app: AppInfo,
    platform: Platform,
    generated_ms: u64,
    system_config_enabled: bool,
}

#[derive(Serialize)]
struct Report {
    #[serde(flatten)]
    meta: Meta,
    detail_sampled: bool,
    sensors: Vec<SensorStatus>,
    controls: Vec<ControlStatus>,
    recent_errors: Vec<ErrorEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    telemetry: Option<Snapshot>,
}

fn sensor(name: &'static str, available: bool) -> SensorStatus {
    SensorStatus { name, available }
}

fn sensors(snapshot: &Snapshot) -> Vec<SensorStatus> {
    let core = &snapshot.core;
    let mut list = vec![
        sensor("cpu_clock", core.cpu_mhz.is_some()),
        sensor("cpu_temperature", core.cpu_temp_c.is_some()),
        sensor("igpu_usage", core.igpu.is_some()),
        sensor("igpu_temperature", core.igpu_temp_c.is_some()),
        sensor("dgpu_usage", core.dgpu.is_some()),
        sensor("dgpu_temperature", core.dgpu_temp_c.is_some()),
        sensor("disk_activity", core.disk.is_some()),
        sensor("package_watts", core.package_watts.is_some()),
    ];
    if let Some(detail) = &snapshot.detail {
        list.extend([
            sensor("gpu_adapters", !detail.gpus.is_empty()),
            sensor("thermal_zones", !detail.temps.is_empty()),
            sensor(
                "per_core_watts",
                detail.cores.iter().any(|c| c.watts.is_some()),
            ),
            sensor("power_breakdown", detail.power.is_some()),
            sensor("cpu_performance_limit", detail.cpu_limit_pct.is_some()),
            sensor("battery", detail.battery.is_some()),
            sensor("fans", detail.fans.is_some()),
        ]);
    }
    list
}

fn redacted(mut snapshot: Snapshot) -> Snapshot {
    if let Some(detail) = &mut snapshot.detail {
        detail.processes.clear();
        detail.public_ip = None;
        for adapter in &mut detail.net_adapters {
            adapter.ipv4 = None;
        }
    }
    snapshot
}

fn control(name: &'static str, status: Result<impl Serialize, String>) -> ControlStatus {
    match status {
        Ok(status) => {
            let value = serde_json::to_value(status).unwrap_or_default();
            ControlStatus {
                name,
                supported: value["supported"].as_bool().unwrap_or(false),
                error: value["error"].as_str().map(truncated),
            }
        }
        Err(error) => ControlStatus {
            name,
            supported: false,
            error: Some(truncated(&error)),
        },
    }
}

fn controls(app: &AppHandle) -> Vec<ControlStatus> {
    vec![
        control("gpu_mode", app.state::<Arc<GpuConfig>>().status()),
        control("power_limits", app.state::<Arc<PowerLimits>>().status()),
        control("cpu_boost", Ok(cpu_boost::read())),
        control("power_mode", app.state::<Arc<PowerConfig>>().status()),
        control("display", app.state::<Arc<DisplayConfig>>().status()),
        control("battery_saver", Ok(battery_saver::read())),
        control("charge_limit", Ok(app.state::<Arc<ChargeLimit>>().status())),
    ]
}

fn build(
    meta: Meta,
    snapshot: Option<Snapshot>,
    include_telemetry: bool,
    controls: Vec<ControlStatus>,
    recent_errors: Vec<ErrorEntry>,
) -> Report {
    Report {
        meta,
        detail_sampled: snapshot.as_ref().is_some_and(|s| s.detail.is_some()),
        sensors: snapshot.as_ref().map(sensors).unwrap_or_default(),
        controls,
        recent_errors,
        telemetry: snapshot.filter(|_| include_telemetry).map(redacted),
    }
}

#[command]
pub async fn get_diagnostic_report(
    include_telemetry: bool,
    app: AppHandle,
    telemetry: State<'_, Arc<Telemetry>>,
) -> Result<String, String> {
    let info = app.package_info();
    let meta = Meta {
        app: AppInfo {
            name: info.name.clone(),
            version: info.version.to_string(),
        },
        platform: Platform {
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
        },
        generated_ms: now_ms(),
        system_config_enabled: config_gate::enabled(),
    };
    let snapshot = telemetry.latest();
    tauri::async_runtime::spawn_blocking(move || {
        let report = build(
            meta,
            snapshot,
            include_telemetry,
            controls(&app),
            ERRORS.recent(),
        );
        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::types::{
        CoreMetrics, DetailMetrics, FanSpeeds, Level, MemoryMetric, Metric, NetAdapter, NetMetric,
        PowerBreakdown, ProcessGroup, Thresholds,
    };

    fn core() -> CoreMetrics {
        let t = Thresholds::default();
        CoreMetrics {
            cpu: Metric::new(10.0, t),
            cpu_mhz: Some(3200),
            cpu_temp_c: None,
            igpu_temp_c: None,
            dgpu_temp_c: None,
            ram: MemoryMetric {
                used_bytes: 1,
                total_bytes: 2,
                percent: 50.0,
                level: Level::Normal,
            },
            disk: Some(Metric::new(1.0, t)),
            net: NetMetric {
                down_bps: 0.0,
                up_bps: 0.0,
                down_pct: 0.0,
                up_pct: 0.0,
                level: Level::Normal,
            },
            igpu: None,
            dgpu: None,
            package_watts: None,
        }
    }

    fn detail() -> DetailMetrics {
        DetailMetrics {
            cores: vec![],
            temps: vec![],
            gpus: vec![],
            power: Some(PowerBreakdown::default()),
            cpu_limit_pct: None,
            net_adapters: vec![NetAdapter {
                name: "Wi-Fi".into(),
                down_bps: 1.0,
                up_bps: 1.0,
                ipv4: Some("192.168.1.20".into()),
            }],
            ping_ms: Some(12.0),
            public_ip: Some("203.0.113.9".into()),
            battery: None,
            fans: Some(FanSpeeds {
                cpu_rpm: Some(2000),
                gpu_rpm: None,
            }),
            processes: vec![ProcessGroup {
                key: "secret".into(),
                name: "secret.exe".into(),
                pids: vec![7],
                cpu_pct: 1.0,
                mem_bytes: 1,
                gpu_pct: 0.0,
                gpu_mem_bytes: 0,
                disk_bps: None,
                net_bps: None,
            }],
        }
    }

    fn snapshot(detail: Option<DetailMetrics>) -> Snapshot {
        Snapshot {
            timestamp_ms: 5,
            core: core(),
            detail,
        }
    }

    fn meta() -> Meta {
        Meta {
            app: AppInfo {
                name: "omniscience".into(),
                version: "1.2.3".into(),
            },
            platform: Platform {
                os: "windows",
                arch: "x86_64",
            },
            generated_ms: 9,
            system_config_enabled: true,
        }
    }

    fn available(list: &[SensorStatus], name: &str) -> Option<bool> {
        list.iter().find(|s| s.name == name).map(|s| s.available)
    }

    #[test]
    fn error_log_keeps_only_the_most_recent_entries() {
        let log = ErrorLog::new();
        for i in 0..MAX_ERRORS + 5 {
            log.record(i as u64, &format!("error {i}"));
        }
        let recent = log.recent();
        assert_eq!(recent.len(), MAX_ERRORS);
        assert_eq!(recent[0].message, "error 5");
        assert_eq!(recent.last().unwrap().timestamp_ms, (MAX_ERRORS + 4) as u64);
    }

    #[test]
    fn error_log_truncates_long_messages() {
        let log = ErrorLog::new();
        log.record(1, &"Ã©".repeat(MAX_TEXT_CHARS + 50));
        assert_eq!(log.recent()[0].message.chars().count(), MAX_TEXT_CHARS);
    }

    #[test]
    fn log_error_is_readable_from_the_report_buffer() {
        log_error("diagnostics test marker");
        assert!(
            ERRORS
                .recent()
                .iter()
                .any(|e| e.message == "diagnostics test marker")
        );
    }

    #[test]
    fn sensors_reflect_what_the_snapshot_reports() {
        let list = sensors(&snapshot(None));
        assert_eq!(available(&list, "cpu_clock"), Some(true));
        assert_eq!(available(&list, "disk_activity"), Some(true));
        assert_eq!(available(&list, "cpu_temperature"), Some(false));
        assert_eq!(available(&list, "dgpu_usage"), Some(false));
        assert_eq!(available(&list, "fans"), None);

        let list = sensors(&snapshot(Some(detail())));
        assert_eq!(available(&list, "fans"), Some(true));
        assert_eq!(available(&list, "power_breakdown"), Some(true));
        assert_eq!(available(&list, "battery"), Some(false));
        assert_eq!(available(&list, "gpu_adapters"), Some(false));
        assert_eq!(available(&list, "per_core_watts"), Some(false));
    }

    #[test]
    fn control_reads_support_and_error_from_a_status() {
        #[derive(Serialize)]
        struct Status {
            supported: bool,
            error: Option<String>,
            other: u32,
        }
        let ok = control(
            "a",
            Ok(Status {
                supported: true,
                error: None,
                other: 1,
            }),
        );
        assert_eq!(
            ok,
            ControlStatus {
                name: "a",
                supported: true,
                error: None
            }
        );
        let unsupported = control(
            "b",
            Ok(Status {
                supported: false,
                error: Some("needs ASUS".into()),
                other: 2,
            }),
        );
        assert!(!unsupported.supported);
        assert_eq!(unsupported.error.as_deref(), Some("needs ASUS"));
        let failed = control("c", Err::<Status, _>("lock poisoned".into()));
        assert!(!failed.supported);
        assert_eq!(failed.error.as_deref(), Some("lock poisoned"));
        let shapeless = control("d", Ok(1));
        assert!(!shapeless.supported);
        assert_eq!(shapeless.error, None);
    }

    #[test]
    fn telemetry_is_omitted_unless_requested() {
        let report = build(meta(), Some(snapshot(None)), false, vec![], vec![]);
        let json = serde_json::to_value(&report).unwrap();
        assert!(json.get("telemetry").is_none());
        assert_eq!(json["app"]["version"], "1.2.3");
        assert_eq!(json["platform"]["os"], "windows");
        assert_eq!(json["system_config_enabled"], true);
        assert_eq!(json["detail_sampled"], false);
        assert!(!json["sensors"].as_array().unwrap().is_empty());

        let report = build(meta(), Some(snapshot(None)), true, vec![], vec![]);
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["telemetry"]["timestamp_ms"], 5);
    }

    #[test]
    fn included_telemetry_drops_process_names_and_addresses() {
        let report = build(meta(), Some(snapshot(Some(detail()))), true, vec![], vec![]);
        assert!(report.detail_sampled);
        let text = serde_json::to_string(&report).unwrap();
        assert!(!text.contains("secret"));
        assert!(!text.contains("203.0.113.9"));
        assert!(!text.contains("192.168.1.20"));
        assert!(text.contains("Wi-Fi"));
        assert!(text.contains("2000"));
    }

    #[test]
    fn report_without_a_snapshot_still_lists_controls_and_errors() {
        let controls = vec![ControlStatus {
            name: "display",
            supported: false,
            error: Some("no panel".into()),
        }];
        let errors = vec![ErrorEntry {
            timestamp_ms: 3,
            message: "boom".into(),
        }];
        let report = build(meta(), None, true, controls, errors);
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["sensors"].as_array().unwrap().len(), 0);
        assert_eq!(json["controls"][0]["error"], "no panel");
        assert_eq!(json["recent_errors"][0]["message"], "boom");
        assert!(json.get("telemetry").is_none());
    }
}
