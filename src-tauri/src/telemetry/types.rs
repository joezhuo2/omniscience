//! The telemetry contract shared with the frontend. TypeScript bindings are generated from these
//! types by `ts-rs` into `src/types/generated/` when `cargo test` runs.

use serde::Serialize;
use ts_rs::TS;

/// Load band a reading falls in. The frontend maps it to colors and never re-derives it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Level {
    Normal,
    Warning,
    Critical,
}

/// Warning and critical thresholds, in percent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thresholds {
    pub warn: f32,
    pub crit: f32,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            warn: 70.0,
            crit: 90.0,
        }
    }
}

/// `<= warn` is normal, `<= crit` is warning, above is critical.
pub fn classify(percent: f32, t: Thresholds) -> Level {
    if percent > t.crit {
        Level::Critical
    } else if percent > t.warn {
        Level::Warning
    } else {
        Level::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Metric {
    pub percent: f32,
    pub level: Level,
}

impl Metric {
    pub fn new(percent: f32, t: Thresholds) -> Self {
        let percent = percent.clamp(0.0, 100.0);
        Self {
            percent,
            level: classify(percent, t),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct MemoryMetric {
    #[ts(type = "number")]
    pub used_bytes: u64,
    #[ts(type = "number")]
    pub total_bytes: u64,
    pub percent: f32,
    pub level: Level,
}

/// Network throughput. Percentages are relative to each direction's rolling 60 s peak.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct NetMetric {
    pub down_bps: f64,
    pub up_bps: f64,
    pub down_pct: f32,
    pub up_pct: f32,
    pub level: Level,
}

/// One network interface's throughput.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct NetAdapter {
    pub name: String,
    pub down_bps: f64,
    pub up_bps: f64,
    /// First IPv4 address, if any.
    pub ipv4: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct CoreMetrics {
    pub cpu: Metric,
    /// Current average clock in MHz (live on Windows, nominal elsewhere).
    #[ts(type = "number | null")]
    pub cpu_mhz: Option<u64>,
    pub ram: MemoryMetric,
    pub disk: Option<Metric>,
    pub net: NetMetric,
    pub igpu: Option<Metric>,
    pub dgpu: Option<Metric>,
    pub package_watts: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct CoreSample {
    pub id: u32,
    pub usage_pct: f32,
    #[ts(type = "number")]
    pub freq_mhz: u64,
    pub watts: Option<f32>,
    pub level: Level,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct TempSample {
    pub label: String,
    pub celsius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
// Only the Windows adapter probe classifies GPUs.
#[cfg_attr(not(windows), allow(dead_code))]
pub enum GpuKind {
    Integrated,
    Discrete,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct GpuDetail {
    pub name: String,
    pub kind: GpuKind,
    pub usage_pct: Option<f32>,
    /// Dedicated memory in use (discrete) or shared memory in use (integrated).
    #[ts(type = "number | null")]
    pub mem_used_bytes: Option<u64>,
    /// Dedicated memory size (discrete) or `None` (integrated, shared memory has no fixed size).
    #[ts(type = "number | null")]
    pub mem_total_bytes: Option<u64>,
    pub temp_c: Option<f32>,
    /// Fan speed, when the driver reports one and the fan is spinning.
    pub fan_rpm: Option<u32>,
    /// Hot enough that the GPU is slowing itself down (thermal throttling).
    pub throttling: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct PowerBreakdown {
    pub apu: Option<f32>,
    pub cpu: Option<f32>,
    pub gpu: Option<f32>,
    pub npu: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct ProcessGroup {
    /// Lowercase executable name without `.exe`; the key `kill_process_group` takes.
    pub key: String,
    /// Executable name as reported by the OS.
    pub name: String,
    pub pids: Vec<u32>,
    /// Percent of total CPU (all logical processors), like Task Manager.
    pub cpu_pct: f32,
    #[ts(type = "number")]
    pub mem_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct DetailMetrics {
    pub cores: Vec<CoreSample>,
    pub temps: Vec<TempSample>,
    pub gpus: Vec<GpuDetail>,
    pub power: Option<PowerBreakdown>,
    /// How far power or thermal limits hold the CPU below its rated speed, in percent (0 when
    /// unthrottled).
    pub cpu_limit_pct: Option<f32>,
    /// Interfaces that have carried traffic, busiest first.
    pub net_adapters: Vec<NetAdapter>,
    /// Last round trip to the ping target in milliseconds; `None` when off, pending or lost.
    pub ping_ms: Option<f32>,
    pub processes: Vec<ProcessGroup>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Snapshot {
    #[ts(type = "number")]
    pub timestamp_ms: u64,
    pub core: CoreMetrics,
    /// Present only while expanded, from the second detail sample on.
    pub detail: Option<DetailMetrics>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
// Only Windows reports access-denied kills.
#[cfg_attr(not(windows), allow(dead_code))]
pub enum KillOutcome {
    Killed,
    Protected,
    AccessDenied,
    NotFound,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct KillResult {
    pub pid: u32,
    pub outcome: KillOutcome,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct KillReport {
    pub key: String,
    pub results: Vec<KillResult>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: Thresholds = Thresholds {
        warn: 70.0,
        crit: 90.0,
    };

    #[test]
    fn classify_boundaries() {
        assert_eq!(classify(0.0, T), Level::Normal);
        assert_eq!(classify(70.0, T), Level::Normal);
        assert_eq!(classify(70.1, T), Level::Warning);
        assert_eq!(classify(90.0, T), Level::Warning);
        assert_eq!(classify(90.1, T), Level::Critical);
        assert_eq!(classify(100.0, T), Level::Critical);
    }

    #[test]
    fn classify_respects_custom_thresholds() {
        let t = Thresholds {
            warn: 50.0,
            crit: 60.0,
        };
        assert_eq!(classify(55.0, t), Level::Warning);
        assert_eq!(classify(61.0, t), Level::Critical);
    }

    #[test]
    fn metric_clamps_out_of_range_percent() {
        assert_eq!(Metric::new(130.0, T).percent, 100.0);
        assert_eq!(Metric::new(-3.0, T).percent, 0.0);
    }

    #[test]
    fn level_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&Level::Critical).unwrap(),
            "\"critical\""
        );
        assert_eq!(
            serde_json::to_string(&KillOutcome::AccessDenied).unwrap(),
            "\"access_denied\""
        );
    }
}
