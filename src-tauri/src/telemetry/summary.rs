use std::collections::HashMap;
use std::fmt::Write;

use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use super::types::{DriveInfo, Snapshot};

pub struct SystemInfo {
    pub app_version: String,
    pub os: String,
    pub cpu: String,
    pub cores: Option<usize>,
    pub threads: usize,
    pub memory_bytes: u64,
    pub gpus: Vec<(String, u64)>,
    pub drives: Vec<DriveInfo>,
    pub snapshot: Option<Snapshot>,
}

fn gib(bytes: u64) -> String {
    format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
}

fn rate(bps: f64) -> String {
    format!("{:.2} MB/s", bps / 1_000_000.0)
}

fn opt(value: Option<f32>, unit: &str) -> String {
    value.map_or("n/a".into(), |v| format!("{v:.1}{unit}"))
}

pub fn render(info: &SystemInfo) -> String {
    let mut out = format!("Omniscience {}\n", info.app_version);
    let _ = writeln!(out, "OS: {}", info.os);
    let cores = info.cores.map_or(String::new(), |c| format!("{c} cores, "));
    let _ = writeln!(out, "CPU: {} ({cores}{} threads)", info.cpu, info.threads);
    let _ = writeln!(out, "Memory: {}", gib(info.memory_bytes));
    if info.gpus.is_empty() {
        out.push_str("GPU: none detected\n");
    }
    for (name, dedicated) in &info.gpus {
        let _ = writeln!(out, "GPU: {name} ({} dedicated)", gib(*dedicated));
    }
    for d in &info.drives {
        let _ = writeln!(
            out,
            "Drive {}: {} total, {} free",
            d.name,
            gib(d.total_bytes),
            gib(d.free_bytes)
        );
    }
    if let Some(s) = &info.snapshot {
        let c = &s.core;
        out.push_str("\nCurrent readings\n");
        let _ = writeln!(out, "CPU: {:.1}%", c.cpu.percent);
        let _ = writeln!(out, "CPU temperature: {}", opt(c.cpu_temp_c, " C"));
        let _ = writeln!(
            out,
            "Memory: {} / {}",
            gib(c.ram.used_bytes),
            gib(c.ram.total_bytes)
        );
        let _ = writeln!(out, "iGPU: {}", opt(c.igpu.map(|m| m.percent), "%"));
        let _ = writeln!(out, "dGPU: {}", opt(c.dgpu.map(|m| m.percent), "%"));
        let _ = writeln!(
            out,
            "Disk activity: {}",
            opt(c.disk.map(|m| m.percent), "%")
        );
        let _ = writeln!(
            out,
            "Network: {} down, {} up",
            rate(c.net.down_bps),
            rate(c.net.up_bps)
        );
        let _ = writeln!(out, "Package power: {}", opt(c.package_watts, " W"));
    }
    out
}

pub fn gather(snapshot: Option<Snapshot>) -> SystemInfo {
    let sys = System::new_with_specifics(
        RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::nothing())
            .with_memory(MemoryRefreshKind::nothing().with_ram()),
    );
    #[cfg(windows)]
    let gpus = super::windows::adapters()
        .into_iter()
        .map(|a| (a.name, a.dedicated_bytes))
        .collect();
    #[cfg(not(windows))]
    let gpus = Vec::new();
    SystemInfo {
        app_version: env!("CARGO_PKG_VERSION").into(),
        os: System::long_os_version().unwrap_or_else(|| std::env::consts::OS.into()),
        cpu: sys
            .cpus()
            .first()
            .map_or("unknown".into(), |c| c.brand().trim().to_string()),
        cores: System::physical_core_count(),
        threads: sys.cpus().len(),
        memory_bytes: sys.total_memory(),
        gpus,
        drives: super::drives::list(&HashMap::new()),
        snapshot,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::types::{
        CoreMetrics, Level, MemoryMetric, Metric, NetMetric, Thresholds,
    };

    const GB: u64 = 1_073_741_824;

    fn fixture(snapshot: Option<Snapshot>) -> SystemInfo {
        SystemInfo {
            app_version: "0.10.0".into(),
            os: "Windows 11 Home 26200".into(),
            cpu: "Example CPU 7".into(),
            cores: Some(8),
            threads: 16,
            memory_bytes: 32 * GB,
            gpus: vec![("Example GPU".into(), 8 * GB)],
            drives: vec![DriveInfo {
                name: "C:".into(),
                label: Some("Private label".into()),
                total_bytes: 500 * GB,
                free_bytes: 250 * GB,
                read_bps: None,
                write_bps: None,
            }],
            snapshot,
        }
    }

    fn snapshot() -> Snapshot {
        let t = Thresholds::default();
        Snapshot {
            timestamp_ms: 1,
            core: CoreMetrics {
                cpu: Metric::new(12.34, t),
                cpu_mhz: None,
                cpu_temp_c: Some(55.0),
                igpu_temp_c: None,
                dgpu_temp_c: None,
                ram: MemoryMetric {
                    used_bytes: 16 * GB,
                    total_bytes: 32 * GB,
                    percent: 50.0,
                    level: Level::Normal,
                },
                disk: Some(Metric::new(3.0, t)),
                net: NetMetric {
                    down_bps: 2_500_000.0,
                    up_bps: 100_000.0,
                    down_pct: 0.0,
                    up_pct: 0.0,
                    level: Level::Normal,
                },
                igpu: None,
                dgpu: Some(Metric::new(40.0, t)),
                package_watts: Some(18.5),
            },
            detail: None,
        }
    }

    #[test]
    fn renders_hardware_and_readings() {
        let text = render(&fixture(Some(snapshot())));
        assert_eq!(
            text,
            "Omniscience 0.10.0\n\
             OS: Windows 11 Home 26200\n\
             CPU: Example CPU 7 (8 cores, 16 threads)\n\
             Memory: 32.0 GB\n\
             GPU: Example GPU (8.0 GB dedicated)\n\
             Drive C:: 500.0 GB total, 250.0 GB free\n\
             \n\
             Current readings\n\
             CPU: 12.3%\n\
             CPU temperature: 55.0 C\n\
             Memory: 16.0 GB / 32.0 GB\n\
             iGPU: n/a\n\
             dGPU: 40.0%\n\
             Disk activity: 3.0%\n\
             Network: 2.50 MB/s down, 0.10 MB/s up\n\
             Package power: 18.5 W\n"
        );
        assert!(!text.contains("Private label"));
    }

    #[test]
    fn omits_readings_without_a_snapshot() {
        let text = render(&fixture(None));
        assert!(!text.contains("Current readings"));
        assert!(text.contains("Drive C:"));
    }

    #[test]
    #[ignore]
    fn summary_live() {
        println!("{}", render(&gather(None)));
    }
}
