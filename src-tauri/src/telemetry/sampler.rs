//! Builds one [`Snapshot`] per tick. The core tier is always sampled; the detail tier exists only
//! while the dashboard is expanded and is dropped (memory released) on collapse.

use std::time::{Instant, SystemTime, UNIX_EPOCH};

use sysinfo::{Networks, System};

use super::peak::RollingPeak;
use super::processes;
use super::types::{
    CoreMetrics, CoreSample, DetailMetrics, MemoryMetric, Metric, NetMetric, Snapshot, Thresholds,
    classify,
};

/// Network rings are scaled to the busiest second of the last minute, never below 1 MB/s.
const NET_WINDOW: usize = 60;
const NET_FLOOR_BPS: f64 = 1_000_000.0;
/// Top process groups sent per sort order (CPU and RAM); the frontend shows 8 of the union.
const TOP_PROCESSES: usize = 8;

fn round1(v: f32) -> f32 {
    (v * 10.0).round() / 10.0
}

pub struct Sampler {
    sys: System,
    networks: Networks,
    last_net: Instant,
    peak_down: RollingPeak,
    peak_up: RollingPeak,
    #[cfg(windows)]
    win: win::Core,
    detail: Option<Detail>,
}

/// Detail-tier state, alive only while expanded.
struct Detail {
    procs: System,
    #[cfg(windows)]
    win: Option<super::windows::Query>,
    /// Process CPU% needs two samples; the first detail tick only primes.
    warm: bool,
}

impl Sampler {
    pub fn new() -> Self {
        let mut sys = System::new();
        // CPU usage is a delta: prime the first sample.
        sys.refresh_cpu_usage();
        Self {
            sys,
            networks: Networks::new_with_refreshed_list(),
            last_net: Instant::now(),
            peak_down: RollingPeak::new(NET_WINDOW, NET_FLOOR_BPS),
            peak_up: RollingPeak::new(NET_WINDOW, NET_FLOOR_BPS),
            #[cfg(windows)]
            win: win::Core::new(),
            detail: None,
        }
    }

    pub fn sample(&mut self, t: Thresholds, expanded: bool) -> Snapshot {
        if !expanded {
            self.detail = None;
        }
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();

        #[cfg(windows)]
        let win = self.win.sample();

        let total = self.sys.total_memory();
        let used = self.sys.used_memory();
        let ram_pct = if total > 0 {
            used as f32 / total as f32 * 100.0
        } else {
            0.0
        };

        let net = self.sample_net(t);

        #[cfg(windows)]
        let (disk, igpu, dgpu, package_watts) = (
            win.disk_pct.map(|p| Metric::new(round1(p), t)),
            win.igpu_pct.map(|p| Metric::new(round1(p), t)),
            win.dgpu_pct.map(|p| Metric::new(round1(p), t)),
            win.energy.package_watts,
        );
        #[cfg(not(windows))]
        let (disk, igpu, dgpu, package_watts) = (None, None, None, None);

        let core = CoreMetrics {
            cpu: Metric::new(round1(self.sys.global_cpu_usage()), t),
            ram: MemoryMetric {
                used_bytes: used,
                total_bytes: total,
                percent: round1(ram_pct),
                level: classify(ram_pct, t),
            },
            disk,
            net,
            igpu,
            dgpu,
            package_watts: package_watts.map(round1),
        };

        let detail = if expanded {
            #[cfg(windows)]
            let d = self.sample_detail(t, &win);
            #[cfg(not(windows))]
            let d = self.sample_detail(t);
            d
        } else {
            None
        };

        Snapshot {
            timestamp_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_millis() as u64),
            core,
            detail,
        }
    }

    fn sample_net(&mut self, t: Thresholds) -> NetMetric {
        self.networks.refresh(true);
        let elapsed = self.last_net.elapsed().as_secs_f64().max(0.001);
        self.last_net = Instant::now();
        let (mut down, mut up) = (0u64, 0u64);
        for (name, data) in &self.networks {
            let lower = name.to_ascii_lowercase();
            if lower == "lo" || lower.contains("loopback") {
                continue;
            }
            down += data.received();
            up += data.transmitted();
        }
        let down_bps = down as f64 / elapsed;
        let up_bps = up as f64 / elapsed;
        let down_pct = round1(self.peak_down.push(down_bps));
        let up_pct = round1(self.peak_up.push(up_bps));
        NetMetric {
            down_bps: down_bps.round(),
            up_bps: up_bps.round(),
            down_pct,
            up_pct,
            level: classify(down_pct.max(up_pct), t),
        }
    }

    fn sample_detail(
        &mut self,
        t: Thresholds,
        #[cfg(windows)] win: &win::CoreSample,
    ) -> Option<DetailMetrics> {
        let detail = self.detail.get_or_insert_with(|| Detail {
            procs: processes::new_system(),
            #[cfg(windows)]
            win: super::windows::Query::new(&win::DETAIL_COUNTERS),
            warm: false,
        });
        if detail.warm {
            processes::refresh(&mut detail.procs);
        }
        self.sys.refresh_cpu_frequency();
        if !detail.warm {
            detail.warm = true;
            return None;
        }

        let n_logical = self.sys.cpus().len();
        let cores = self
            .sys
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, c)| CoreSample {
                id: i as u32,
                usage_pct: round1(c.cpu_usage()),
                freq_mhz: c.frequency(),
                #[cfg(windows)]
                watts: super::parse::core_watts(&win.energy.cores, i as u32, n_logical).map(round1),
                #[cfg(not(windows))]
                watts: None,
                level: classify(c.cpu_usage(), t),
            })
            .collect();

        let procs = processes::read(&detail.procs);
        let n = self.sys.cpus().len();
        let mut groups = processes::top_groups(&procs, n, false, TOP_PROCESSES);
        for g in processes::top_groups(&procs, n, true, TOP_PROCESSES) {
            if !groups.iter().any(|x| x.key == g.key) {
                groups.push(g);
            }
        }

        #[cfg(windows)]
        let (temps, gpus, power) = win::detail(detail.win.as_mut(), win, &self.win.adapters);
        #[cfg(not(windows))]
        let (temps, gpus, power) = (Vec::new(), Vec::new(), None);

        Some(DetailMetrics {
            cores,
            temps,
            gpus,
            power,
            processes: groups,
        })
    }
}

impl Default for Sampler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(windows)]
mod win {
    use std::collections::HashMap;

    use super::super::parse::{self, EnergyReadings};
    use super::super::types::{GpuDetail, GpuKind, PowerBreakdown, TempSample};
    use super::super::windows::{Adapter, Query, adapter_temp, adapters};

    const DISK_IDLE: usize = 0;
    const GPU_ENGINE: usize = 1;
    const ENERGY: usize = 2;
    const CORE_COUNTERS: [&str; 3] = [
        r"\PhysicalDisk(_Total)\% Idle Time",
        r"\GPU Engine(*)\Utilization Percentage",
        r"\Energy Meter(*)\Power",
    ];

    const THERMAL: usize = 0;
    const GPU_DEDICATED: usize = 1;
    const GPU_SHARED: usize = 2;
    pub const DETAIL_COUNTERS: [&str; 3] = [
        r"\Thermal Zone Information(*)\High Precision Temperature",
        r"\GPU Adapter Memory(*)\Dedicated Usage",
        r"\GPU Adapter Memory(*)\Shared Usage",
    ];

    /// A dGPU below this usage is treated as asleep: no D3DKMT temperature query, which could
    /// wake it.
    const DGPU_ACTIVE_PCT: f32 = 0.5;

    pub struct Core {
        query: Option<Query>,
        pub adapters: Vec<Adapter>,
    }

    pub struct CoreSample {
        pub disk_pct: Option<f32>,
        pub igpu_pct: Option<f32>,
        pub dgpu_pct: Option<f32>,
        pub gpu_usage: HashMap<String, f32>,
        pub energy: EnergyReadings,
    }

    impl Core {
        pub fn new() -> Self {
            Self {
                query: Query::new(&CORE_COUNTERS),
                adapters: adapters(),
            }
        }

        pub fn sample(&mut self) -> CoreSample {
            let Some(q) = self.query.as_mut().filter(|q| q.collect_ok()) else {
                return CoreSample {
                    disk_pct: None,
                    igpu_pct: None,
                    dgpu_pct: None,
                    gpu_usage: HashMap::new(),
                    energy: EnergyReadings::default(),
                };
            };
            let disk_pct = q.items(DISK_IDLE).and_then(|v| {
                v.first()
                    .map(|(_, idle)| (100.0 - idle).clamp(0.0, 100.0) as f32)
            });
            let gpu_usage = q
                .items(GPU_ENGINE)
                .map(|v| parse::gpu_usage_by_adapter(&v))
                .unwrap_or_default();
            let energy = q
                .items(ENERGY)
                .map(|v| parse::parse_energy(&v))
                .unwrap_or_default();
            // An adapter with no engine instances (asleep) reads 0%.
            let busiest = |kind: GpuKind| {
                self.adapters
                    .iter()
                    .filter(|a| a.kind == kind)
                    .map(|a| gpu_usage.get(&a.luid_key).copied().unwrap_or(0.0))
                    .reduce(f32::max)
            };
            CoreSample {
                disk_pct,
                igpu_pct: busiest(GpuKind::Integrated),
                dgpu_pct: busiest(GpuKind::Discrete),
                gpu_usage,
                energy,
            }
        }
    }

    pub fn detail(
        query: Option<&mut Query>,
        core: &CoreSample,
        adapters: &[Adapter],
    ) -> (Vec<TempSample>, Vec<GpuDetail>, Option<PowerBreakdown>) {
        let (mut temps, mut dedicated, mut shared) = (Vec::new(), HashMap::new(), HashMap::new());
        if let Some(q) = query.filter(|q| q.collect_ok()) {
            temps = q
                .items(THERMAL)
                .unwrap_or_default()
                .iter()
                .filter_map(|(name, dk)| {
                    parse::decikelvin_to_c(*dk).map(|c| TempSample {
                        label: parse::zone_label(name),
                        celsius: c,
                    })
                })
                .collect();
            dedicated = q
                .items(GPU_DEDICATED)
                .map(|v| parse::sum_by_adapter(&v))
                .unwrap_or_default();
            shared = q
                .items(GPU_SHARED)
                .map(|v| parse::sum_by_adapter(&v))
                .unwrap_or_default();
        }
        let gpus = adapters
            .iter()
            .map(|a| {
                let usage = core.gpu_usage.get(&a.luid_key).copied().unwrap_or(0.0);
                let discrete = a.kind == GpuKind::Discrete;
                let temp_c = if !discrete || usage >= DGPU_ACTIVE_PCT {
                    adapter_temp(a.luid)
                } else {
                    None
                };
                GpuDetail {
                    name: a.name.clone(),
                    kind: a.kind,
                    usage_pct: Some((usage * 10.0).round() / 10.0),
                    mem_used_bytes: if discrete {
                        dedicated.get(&a.luid_key).copied()
                    } else {
                        shared.get(&a.luid_key).copied()
                    },
                    mem_total_bytes: discrete.then_some(a.dedicated_bytes),
                    temp_c,
                }
            })
            .collect();
        let b = &core.energy.breakdown;
        let power = (b.apu.is_some() || b.cpu.is_some() || b.gpu.is_some() || b.npu.is_some())
            .then(|| b.clone());
        (temps, gpus, power)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_snapshot_has_no_detail() {
        let mut s = Sampler::new();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        let snap = s.sample(Thresholds::default(), false);
        assert!(snap.detail.is_none());
        assert!(snap.core.ram.total_bytes > 0);
        assert!((0.0..=100.0).contains(&snap.core.cpu.percent));
    }

    #[test]
    fn detail_appears_on_second_expanded_tick_and_drops_on_collapse() {
        let mut s = Sampler::new();
        let t = Thresholds::default();
        assert!(s.sample(t, true).detail.is_none(), "first tick only primes");
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        let d = s.sample(t, true).detail.expect("second tick has detail");
        assert!(!d.cores.is_empty());
        assert!(!d.processes.is_empty());
        assert!(s.sample(t, false).detail.is_none());
        assert!(s.detail.is_none(), "detail state released on collapse");
    }

    /// Machine-dependent: `cargo test snapshot_live -- --ignored --nocapture` prints a real
    /// expanded snapshot, to check sensors on new hardware.
    #[test]
    #[ignore]
    fn snapshot_live() {
        let mut s = Sampler::new();
        let t = Thresholds::default();
        s.sample(t, true);
        std::thread::sleep(std::time::Duration::from_secs(1));
        let snap = s.sample(t, true);
        println!("{}", serde_json::to_string_pretty(&snap).unwrap());
    }
}
