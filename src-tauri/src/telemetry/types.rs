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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum BatteryState {
    Charging,
    Discharging,
    /// On AC power but not charging (full, or held by a charge limit).
    Idle,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Battery {
    pub percent: f32,
    pub state: BatteryState,
    /// Charge or drain rate, when the driver reports one.
    pub watts: Option<f32>,
    /// Time to empty (discharging) or to full (charging), in seconds.
    #[ts(type = "number | null")]
    pub secs_remaining: Option<u64>,
    pub design_capacity_mwh: Option<u32>,
    pub full_charge_capacity_mwh: Option<u32>,
    pub wear_pct: Option<f32>,
    pub cycle_count: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
#[cfg_attr(not(windows), allow(dead_code))]
pub struct FanSpeeds {
    pub cpu_rpm: Option<u32>,
    pub gpu_rpm: Option<u32>,
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
    /// Hottest ACPI thermal zone; a system-temperature proxy, not a CPU package sensor.
    pub cpu_temp_c: Option<f32>,
    pub igpu_temp_c: Option<f32>,
    pub dgpu_temp_c: Option<f32>,
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
    pub core_clock_mhz: Option<u32>,
    pub mem_clock_mhz: Option<u32>,
    pub encode_pct: Option<f32>,
    pub decode_pct: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct DriveInfo {
    pub name: String,
    pub label: Option<String>,
    #[ts(type = "number")]
    pub total_bytes: u64,
    #[ts(type = "number")]
    pub free_bytes: u64,
    pub read_bps: Option<f64>,
    pub write_bps: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct PhysicalDisk {
    pub name: String,
    pub kind: Option<String>,
    pub health: Option<String>,
    pub temp_c: Option<f32>,
    pub wear_pct: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct WifiInfo {
    pub ssid: String,
    pub signal_pct: u8,
    pub link_mbps: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct MemoryDetail {
    #[ts(type = "number")]
    pub committed_bytes: u64,
    #[ts(type = "number")]
    pub commit_limit_bytes: u64,
    #[ts(type = "number | null")]
    pub cached_bytes: Option<u64>,
    #[ts(type = "number | null")]
    pub compressed_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
#[allow(dead_code)]
pub enum PowerEventKind {
    Sleep,
    Wake,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct PowerEvent {
    #[ts(type = "number")]
    pub ts_ms: u64,
    pub kind: PowerEventKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct BluetoothBattery {
    pub name: String,
    pub percent: u8,
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
    /// Utilization of the group's busiest GPU engines, in percent (capped at 100).
    pub gpu_pct: f32,
    /// Dedicated GPU memory in use across adapters.
    #[ts(type = "number")]
    pub gpu_mem_bytes: u64,
    /// Physical disk bytes read + written per second; None when collection is unavailable.
    pub disk_bps: Option<f64>,
    /// TCP/UDP payload bytes sent + received per second; None when collection is unavailable.
    pub net_bps: Option<f64>,
    pub members: Vec<ProcessMember>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct ProcessMember {
    pub pid: u32,
    pub cpu_pct: f32,
    #[ts(type = "number")]
    pub mem_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ProcessPriority {
    Idle,
    BelowNormal,
    Normal,
    AboveNormal,
    High,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct ProcessDetails {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
    pub command_line: Option<String>,
    pub priority: Option<ProcessPriority>,
    pub efficiency: Option<bool>,
    pub suspended: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ActionOutcome {
    Ok,
    Protected,
    AccessDenied,
    NotFound,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct ActionResult {
    pub pid: u32,
    pub outcome: ActionOutcome,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
#[allow(dead_code)]
pub struct StartupApp {
    pub id: String,
    pub name: String,
    pub command: String,
    #[ts(type = "'user' | 'machine' | 'folder'")]
    pub source: String,
    pub enabled: bool,
    pub can_toggle: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct DetailMetrics {
    pub cores: Vec<CoreSample>,
    pub temps: Vec<TempSample>,
    pub gpus: Vec<GpuDetail>,
    pub power: Option<PowerBreakdown>,
    /// Loss of available nominal CPU performance: 100 minus Windows `% Performance Limit`.
    /// Zero means full headroom; missing/zero raw counters yield None. This is not a direct
    /// thermal measurement and can reflect Windows power policy.
    pub cpu_limit_pct: Option<f32>,
    /// Interfaces that have carried traffic, busiest first.
    pub net_adapters: Vec<NetAdapter>,
    /// Last round trip to the ping target in milliseconds; `None` when off, pending or lost.
    pub ping_ms: Option<f32>,
    /// Public IPv4 address, when lookups are on and one has succeeded.
    pub public_ip: Option<String>,
    /// `None` on machines without a battery.
    pub battery: Option<Battery>,
    pub fans: Option<FanSpeeds>,
    /// Sent only on ticks that refresh the list (one in three) while the processes section is
    /// open; `None` otherwise, and the frontend keeps the last list.
    pub processes: Option<Vec<ProcessGroup>>,
    pub drives: Vec<DriveInfo>,
    pub disks: Vec<PhysicalDisk>,
    pub wifi: Option<WifiInfo>,
    pub ping_jitter_ms: Option<f32>,
    pub ping_loss_pct: Option<f32>,
    #[ts(type = "number | null")]
    pub data_today_bytes: Option<u64>,
    #[ts(type = "number | null")]
    pub data_month_bytes: Option<u64>,
    pub metered: Option<bool>,
    pub memory: Option<MemoryDetail>,
    #[ts(type = "number | null")]
    pub boot_ms: Option<u64>,
    pub power_events: Vec<PowerEvent>,
    pub bluetooth: Vec<BluetoothBattery>,
    pub energy_today_wh: Option<f32>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum HistoryEventKind {
    PowerSource,
    Throttle,
    Alert,
}

#[derive(Debug, Clone, PartialEq, Serialize, serde::Deserialize, TS)]
#[ts(export)]
pub struct HistoryEvent {
    #[ts(type = "number")]
    pub ts_ms: u64,
    pub kind: HistoryEventKind,
    pub label: String,
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
