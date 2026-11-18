//! Builds one [`Snapshot`] per tick. The core tier is always sampled; the detail tier exists only
//! while the dashboard is expanded and is dropped (memory released) on collapse.

use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use sysinfo::{Networks, System};

use super::battery;
use super::peak::RollingPeak;
use super::ping::Pinger;
use super::processes;
use super::public_ip::PublicIp;
use super::types::{
    CoreMetrics, CoreSample, DetailMetrics, MemoryMetric, Metric, MetricLimits, NetAdapter,
    NetMetric, Snapshot, Thresholds, classify,
};

/// Network rings are scaled to the busiest second of the last minute, never below 1 MB/s.
const NET_WINDOW: usize = 60;
const NET_FLOOR_BPS: f64 = 1_000_000.0;

fn round1(v: f32) -> f32 {
    (v * 10.0).round() / 10.0
}

pub struct Sampler {
    sys: System,
    networks: Networks,
    last_net: Instant,
    peak_down: RollingPeak,
    peak_up: RollingPeak,
    /// Per-interface rates from the last network sample.
    net_adapters: Vec<NetAdapter>,
    ping_target: Arc<str>,
    /// `Some` while public IP lookups are on.
    public_ip: Option<PublicIp>,
    #[cfg(windows)]
    win: win::Core,
    detail: Option<Detail>,
}

/// The process list is refreshed (and sent) on one tick in this many.
const PROCESS_EVERY: u32 = 3;

/// Detail-tier state, alive only while expanded.
struct Detail {
    #[cfg(windows)]
    win: Option<super::windows::Query>,
    #[cfg(windows)]
    fans: super::fans::FanSource,
    #[cfg(windows)]
    nvml: Option<super::nvml::Nvml>,
    battery: battery::Devices,
    slow: super::slow::SlowProbe,
    pinger: Option<Pinger>,
    /// `Some` while the processes section is open.
    procs: Option<Procs>,
    /// Process CPU% needs two samples; the first detail tick only primes.
    warm: bool,
}

/// Process-list state: the costliest part of the detail tier.
struct Procs {
    sys: System,
    #[cfg(windows)]
    io: Option<super::process_io::ProcessIo>,
    #[cfg(windows)]
    gpu_mem: Option<super::windows::Query>,
    /// Ticks to skip before the next refresh.
    wait: u32,
}

impl Procs {
    fn start() -> Self {
        Self {
            sys: processes::new_system(),
            #[cfg(windows)]
            io: super::process_io::ProcessIo::start(),
            #[cfg(windows)]
            gpu_mem: super::windows::Query::new(&win::PROCESS_COUNTERS),
            // `new_system` just refreshed; another refresh this tick would read ~0% CPU.
            wait: 1,
        }
    }

    /// Whether this tick refreshes the list.
    fn due(&mut self) -> bool {
        if self.wait > 0 {
            self.wait -= 1;
            return false;
        }
        self.wait = PROCESS_EVERY - 1;
        processes::refresh(&mut self.sys);
        true
    }
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
            net_adapters: Vec::new(),
            ping_target: "".into(),
            public_ip: None,
            #[cfg(windows)]
            win: win::Core::new(),
            detail: None,
        }
    }

    /// Host to ping while expanded (empty for none), and whether to look up the public IP.
    pub fn set_network(&mut self, ping_target: Arc<str>, public_ip: bool) {
        self.ping_target = ping_target;
        if public_ip != self.public_ip.is_some() {
            self.public_ip = public_ip.then(PublicIp::default);
        }
    }

    pub fn suspend_detail(&mut self) {
        self.detail = None;
    }

    /// `processes`: whether the processes section is open, so the process list is worth reading.
    pub fn sample(&mut self, t: MetricLimits, expanded: bool, processes: bool) -> Snapshot {
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

        let net = self.sample_net(t.net);

        #[cfg(windows)]
        let (disk, igpu, dgpu, package_watts) = (
            win.disk_pct.map(|p| Metric::new(round1(p), t.disk)),
            win.igpu_pct.map(|p| Metric::new(round1(p), t.igpu)),
            win.dgpu_pct.map(|p| Metric::new(round1(p), t.dgpu)),
            win.energy.package_watts,
        );
        #[cfg(not(windows))]
        let (disk, igpu, dgpu, package_watts) = (None, None, None, None);

        #[cfg(windows)]
        let cpu_mhz = win.cpu_mhz;
        #[cfg(not(windows))]
        let cpu_mhz = {
            let cpus = self.sys.cpus();
            (!cpus.is_empty())
                .then(|| cpus.iter().map(|c| c.frequency()).sum::<u64>() / cpus.len() as u64)
                .filter(|mhz| *mhz > 0)
        };

        let core = CoreMetrics {
            cpu: Metric::new(round1(self.sys.global_cpu_usage()), t.cpu),
            cpu_mhz,
            #[cfg(windows)]
            cpu_temp_c: win.temps.iter().map(|t| t.celsius).reduce(f32::max),
            #[cfg(not(windows))]
            cpu_temp_c: None,
            #[cfg(windows)]
            igpu_temp_c: win.gpu_temperature(super::types::GpuKind::Integrated, &self.win.adapters),
            #[cfg(not(windows))]
            igpu_temp_c: None,
            #[cfg(windows)]
            dgpu_temp_c: win.gpu_temperature(super::types::GpuKind::Discrete, &self.win.adapters),
            #[cfg(not(windows))]
            dgpu_temp_c: None,
            ram: MemoryMetric {
                used_bytes: used,
                total_bytes: total,
                percent: round1(ram_pct),
                level: classify(ram_pct, t.ram),
            },
            disk,
            net,
            igpu,
            dgpu,
            package_watts: package_watts.map(round1),
        };

        let detail = if expanded {
            #[cfg(windows)]
            let d = self.sample_detail(t.cpu, processes, &win);
            #[cfg(not(windows))]
            let d = self.sample_detail(t.cpu, processes);
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
        self.net_adapters.clear();
        for (name, data) in &self.networks {
            let lower = name.to_ascii_lowercase();
            if lower == "lo" || lower.contains("loopback") {
                continue;
            }
            down += data.received();
            up += data.transmitted();
            if data.total_received() + data.total_transmitted() > 0 {
                self.net_adapters.push(NetAdapter {
                    name: name.clone(),
                    down_bps: (data.received() as f64 / elapsed).round(),
                    up_bps: (data.transmitted() as f64 / elapsed).round(),
                    ipv4: data
                        .ip_networks()
                        .iter()
                        .find(|n| n.addr.is_ipv4())
                        .map(|n| n.addr.to_string()),
                });
            }
        }
        self.net_adapters
            .sort_by(|a, b| (b.down_bps + b.up_bps).total_cmp(&(a.down_bps + a.up_bps)));
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
        want_processes: bool,
        #[cfg(windows)] win: &win::CoreSample,
    ) -> Option<DetailMetrics> {
        let detail = self.detail.get_or_insert_with(|| Detail {
            #[cfg(windows)]
            win: super::windows::Query::new(&win::DETAIL_COUNTERS),
            #[cfg(windows)]
            fans: super::fans::FanSource::start(),
            #[cfg(windows)]
            nvml: self
                .win
                .adapters
                .iter()
                .any(|a| a.name.contains("NVIDIA"))
                .then(super::nvml::Nvml::load)
                .flatten(),
            battery: battery::Devices::find(),
            slow: super::slow::SlowProbe::start(),
            pinger: None,
            procs: None,
            warm: false,
        });
        let target = &self.ping_target;
        if detail.pinger.as_ref().map(Pinger::target) != Some(&**target) {
            detail.pinger = (!target.is_empty()).then(|| Pinger::start(target.clone()));
        }
        let procs = if want_processes {
            let p = detail.procs.get_or_insert_with(Procs::start);
            p.due().then_some(p)
        } else {
            detail.procs = None;
            None
        };
        self.sys.refresh_cpu_frequency();
        if !detail.warm {
            detail.warm = true;
            return None;
        }

        #[cfg(windows)]
        let n_logical = self.sys.cpus().len();
        let cores = self
            .sys
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, c)| CoreSample {
                id: i as u32,
                usage_pct: round1(c.cpu_usage()),
                #[cfg(windows)]
                freq_mhz: win
                    .core_mhz
                    .get(&(i as u32))
                    .copied()
                    .unwrap_or(c.frequency()),
                #[cfg(not(windows))]
                freq_mhz: c.frequency(),
                #[cfg(windows)]
                watts: super::parse::core_watts(&win.energy.cores, i as u32, n_logical).map(round1),
                #[cfg(not(windows))]
                watts: None,
                level: classify(c.cpu_usage(), t),
            })
            .collect();

        if let Some(ip) = self.public_ip.as_mut() {
            ip.refresh();
        }

        #[cfg(windows)]
        let (temps, gpus, power, cpu_limit, memory, drive_rates) = {
            let r = win::detail(
                detail.win.as_mut(),
                win,
                &self.win.adapters,
                detail.nvml.as_ref(),
            );
            (
                r.temps,
                r.gpus,
                r.power,
                r.cpu_limit,
                r.memory,
                r.drive_rates,
            )
        };
        #[cfg(not(windows))]
        let (temps, gpus, power, cpu_limit, memory, drive_rates) = (
            Vec::new(),
            Vec::new(),
            None,
            None,
            None,
            std::collections::HashMap::new(),
        );

        let cpus = self.sys.cpus().len();
        // Search must include quiet apps too; the frontend filters, sorts and limits rows.
        let groups = procs.map(|state| {
            #[cfg_attr(not(windows), allow(unused_mut))]
            let mut list = processes::read(&state.sys);
            #[cfg(windows)]
            {
                let (usage, mem) = win::process_gpu(state.gpu_mem.as_ref(), win);
                let rates = state
                    .io
                    .as_mut()
                    .and_then(super::process_io::ProcessIo::sample);
                for p in &mut list {
                    p.gpu_pct = usage.get(&p.pid).copied().unwrap_or(0.0);
                    p.gpu_mem_bytes = mem.get(&p.pid).copied().unwrap_or(0);
                    if let Some(rates) = &rates {
                        let (disk, net) = rates.get(&p.pid).copied().unwrap_or_default();
                        p.disk_bps = Some(disk);
                        p.net_bps = Some(net);
                    }
                }
            }
            processes::groups(&list, cpus)
        });

        let slow = detail.slow.latest();
        let quality = detail.pinger.as_ref().map_or((None, None), Pinger::quality);

        #[cfg(windows)]
        let wifi = super::wifi::current();
        #[cfg(not(windows))]
        let wifi = None;

        #[cfg(windows)]
        let fans = detail.fans.latest();
        #[cfg(not(windows))]
        let fans = None;

        Some(DetailMetrics {
            cores,
            temps,
            gpus,
            power,
            cpu_limit_pct: cpu_limit.map(round1),
            net_adapters: self.net_adapters.clone(),
            ping_ms: detail.pinger.as_ref().and_then(Pinger::latest),
            ping_jitter_ms: quality.0.map(round1),
            ping_loss_pct: quality.1.map(round1),
            public_ip: self
                .public_ip
                .as_ref()
                .and_then(PublicIp::latest)
                .map(|ip| ip.to_string()),
            battery: battery::estimate(&detail.battery.read()),
            fans,
            processes: groups,
            drives: super::drives::list(&drive_rates),
            disks: slow.disks,
            wifi,
            data_today_bytes: None,
            data_month_bytes: None,
            metered: slow.metered,
            memory,
            boot_ms: Some(System::boot_time() * 1000),
            power_events: slow.power_events,
            bluetooth: slow.bluetooth,
            energy_today_wh: None,
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

    use super::super::nvml::Nvml;
    use super::super::parse::{self, EnergyReadings};
    use super::super::types::{GpuDetail, GpuKind, MemoryDetail, PowerBreakdown, TempSample};
    use super::super::windows::{Adapter, AdapterPerf, Query, adapter_perf, adapters, commit};
    use super::round1;

    const DISK_IDLE: usize = 0;
    const GPU_ENGINE: usize = 1;
    const ENERGY: usize = 2;
    const CPU_PERF: usize = 3;
    const CPU_BASE: usize = 4;
    const THERMAL: usize = 5;
    const CORE_COUNTERS: [&str; 6] = [
        r"\PhysicalDisk(_Total)\% Idle Time",
        r"\GPU Engine(*)\Utilization Percentage",
        r"\Energy Meter(*)\Power",
        r"\Processor Information(*)\% Processor Performance",
        r"\Processor Information(*)\Processor Frequency",
        r"\Thermal Zone Information(*)\High Precision Temperature",
    ];

    const GPU_DEDICATED: usize = 0;
    const GPU_SHARED: usize = 1;
    const CPU_LIMIT: usize = 2;
    const DRIVE_READ: usize = 3;
    const DRIVE_WRITE: usize = 4;
    const MEM_CACHED: std::ops::Range<usize> = 5..9;
    const MEM_COMPRESSED: usize = 9;
    pub const DETAIL_COUNTERS: [&str; 10] = [
        r"\GPU Adapter Memory(*)\Dedicated Usage",
        r"\GPU Adapter Memory(*)\Shared Usage",
        r"\Processor Information(_Total)\% Performance Limit",
        r"\LogicalDisk(*)\Disk Read Bytes/sec",
        r"\LogicalDisk(*)\Disk Write Bytes/sec",
        r"\Memory\Standby Cache Normal Priority Bytes",
        r"\Memory\Standby Cache Reserve Bytes",
        r"\Memory\Standby Cache Core Bytes",
        r"\Memory\Modified Page List Bytes",
        r"\Process(Memory Compression)\Working Set",
    ];

    /// Per-process instances are expensive to expand, so this query lives only with the
    /// process list.
    const GPU_PROCESS_MEM: usize = 0;
    pub const PROCESS_COUNTERS: [&str; 1] = [r"\GPU Process Memory(*)\Dedicated Usage"];

    /// A dGPU below this usage is treated as asleep: no D3DKMT sensor query, which could wake it.
    const DGPU_ACTIVE_PCT: f32 = 0.5;

    fn guarded_perf(kind: GpuKind, usage: f32, read: impl FnOnce() -> AdapterPerf) -> AdapterPerf {
        if kind != GpuKind::Discrete || usage >= DGPU_ACTIVE_PCT {
            read()
        } else {
            AdapterPerf::default()
        }
    }

    #[test]
    fn sleeping_discrete_gpu_is_never_queried() {
        for usage in [0.0, 0.49] {
            assert_eq!(
                guarded_perf(GpuKind::Discrete, usage, || panic!("woke sleeping GPU")),
                AdapterPerf::default()
            );
        }
        for (kind, usage) in [(GpuKind::Integrated, 0.0), (GpuKind::Discrete, 0.5)] {
            let mut called = false;
            let perf = guarded_perf(kind, usage, || {
                called = true;
                AdapterPerf {
                    temp_c: Some(55.0),
                    ..Default::default()
                }
            });
            assert!(called);
            assert_eq!(perf.temp_c, Some(55.0));
        }
    }

    pub struct Core {
        query: Option<Query>,
        pub adapters: Vec<Adapter>,
    }

    pub struct CoreSample {
        pub temps: Vec<TempSample>,
        pub gpu_perf: HashMap<String, AdapterPerf>,
        pub disk_pct: Option<f32>,
        pub igpu_pct: Option<f32>,
        pub dgpu_pct: Option<f32>,
        pub gpu_usage: HashMap<String, f32>,
        /// Raw `GPU Engine` instances, kept for the per-process breakdown.
        pub gpu_engines: Vec<(String, f64)>,
        pub energy: EnergyReadings,
        pub cpu_mhz: Option<u64>,
        pub core_mhz: HashMap<u32, u64>,
    }

    impl CoreSample {
        pub fn gpu_temperature(&self, kind: GpuKind, adapters: &[Adapter]) -> Option<f32> {
            adapters
                .iter()
                .filter(|a| a.kind == kind)
                .filter_map(|a| self.gpu_perf.get(&a.luid_key).and_then(|p| p.temp_c))
                .reduce(f32::max)
        }
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
                    temps: Vec::new(),
                    gpu_perf: HashMap::new(),
                    disk_pct: None,
                    igpu_pct: None,
                    dgpu_pct: None,
                    gpu_usage: HashMap::new(),
                    gpu_engines: Vec::new(),
                    energy: EnergyReadings::default(),
                    cpu_mhz: None,
                    core_mhz: HashMap::new(),
                };
            };
            let disk_pct = q.items(DISK_IDLE).and_then(|v| {
                v.first()
                    .map(|(_, idle)| (100.0 - idle).clamp(0.0, 100.0) as f32)
            });
            let gpu_engines = q.items(GPU_ENGINE).unwrap_or_default();
            let gpu_usage = parse::gpu_usage_by_adapter(&gpu_engines);
            let temps = q
                .items(THERMAL)
                .unwrap_or_default()
                .iter()
                .filter_map(|(name, dk)| {
                    parse::decikelvin_to_c(*dk).map(|celsius| TempSample {
                        label: parse::zone_label(name),
                        celsius,
                    })
                })
                .collect();
            let gpu_perf = self
                .adapters
                .iter()
                .map(|a| {
                    let usage = gpu_usage.get(&a.luid_key).copied().unwrap_or(0.0);
                    let perf = guarded_perf(a.kind, usage, || adapter_perf(a.luid));
                    (a.luid_key.clone(), perf)
                })
                .collect();
            let energy = q
                .items(ENERGY)
                .map(|v| parse::parse_energy(&v))
                .unwrap_or_default();
            let (cpu_mhz, core_mhz) = match (q.items(CPU_PERF), q.items(CPU_BASE)) {
                (Some(perf), Some(base)) => parse::clock_mhz(&perf, &base),
                _ => (None, HashMap::new()),
            };
            // An adapter with no engine instances (asleep) reads 0%.
            let busiest = |kind: GpuKind| {
                self.adapters
                    .iter()
                    .filter(|a| a.kind == kind)
                    .map(|a| gpu_usage.get(&a.luid_key).copied().unwrap_or(0.0))
                    .reduce(f32::max)
            };
            CoreSample {
                temps,
                gpu_perf,
                disk_pct,
                igpu_pct: busiest(GpuKind::Integrated),
                dgpu_pct: busiest(GpuKind::Discrete),
                gpu_usage,
                gpu_engines,
                energy,
                cpu_mhz,
                core_mhz,
            }
        }
    }

    /// Per-process GPU utilization and dedicated memory; collects the [`PROCESS_COUNTERS`] query.
    pub fn process_gpu(
        query: Option<&Query>,
        core: &CoreSample,
    ) -> (HashMap<u32, f32>, HashMap<u32, u64>) {
        let usage = parse::gpu_usage_by_pid(&core.gpu_engines);
        let mem = query
            .filter(|q| q.collect_ok())
            .and_then(|q| q.items(GPU_PROCESS_MEM))
            .map(|v| parse::sum_by_pid(&v))
            .unwrap_or_default();
        (usage, mem)
    }

    pub struct DetailRead {
        pub temps: Vec<TempSample>,
        pub gpus: Vec<GpuDetail>,
        pub power: Option<PowerBreakdown>,
        pub cpu_limit: Option<f32>,
        pub drive_rates: HashMap<String, (f64, f64)>,
        pub memory: Option<MemoryDetail>,
    }

    pub fn detail(
        query: Option<&mut Query>,
        core: &CoreSample,
        adapters: &[Adapter],
        nvml: Option<&Nvml>,
    ) -> DetailRead {
        let (mut dedicated, mut shared) = (HashMap::new(), HashMap::new());
        let mut cpu_limit = None;
        let mut drive_rates = HashMap::new();
        let mut cached = None;
        let mut compressed = None;
        if let Some(q) = query.filter(|q| q.collect_ok()) {
            drive_rates = parse::drive_rates(
                &q.items(DRIVE_READ).unwrap_or_default(),
                &q.items(DRIVE_WRITE).unwrap_or_default(),
            );
            cached = MEM_CACHED
                .filter_map(|i| {
                    q.items(i)
                        .and_then(|v| v.first().map(|(_, b)| b.max(0.0) as u64))
                })
                .reduce(|a, b| a + b);
            compressed = q
                .items(MEM_COMPRESSED)
                .and_then(|v| v.first().map(|(_, b)| b.max(0.0) as u64));
            dedicated = q
                .items(GPU_DEDICATED)
                .map(|v| parse::sum_by_adapter(&v))
                .unwrap_or_default();
            shared = q
                .items(GPU_SHARED)
                .map(|v| parse::sum_by_adapter(&v))
                .unwrap_or_default();
            cpu_limit = q.items(CPU_LIMIT).and_then(|v| {
                v.first()
                    .and_then(|(_, pct)| parse::performance_loss_pct(*pct))
            });
        }
        let decode = parse::engine_pct_by_adapter(&core.gpu_engines, "VideoDecode");
        let encode = parse::engine_pct_by_adapter(&core.gpu_engines, "VideoEncode");
        let mut nvidia = 0;
        let gpus = adapters
            .iter()
            .map(|a| {
                let usage = core.gpu_usage.get(&a.luid_key).copied().unwrap_or(0.0);
                let discrete = a.kind == GpuKind::Discrete;
                let unavailable = AdapterPerf::default();
                let perf = core.gpu_perf.get(&a.luid_key).unwrap_or(&unavailable);
                let clocks = if a.name.contains("NVIDIA") {
                    nvidia += 1;
                    guarded_perf(a.kind, usage, || {
                        let (core_mhz, mem_mhz) =
                            nvml.map_or((None, None), |n| n.clocks(nvidia - 1));
                        AdapterPerf {
                            core_clock_mhz: core_mhz,
                            mem_clock_mhz: mem_mhz,
                            ..Default::default()
                        }
                    })
                } else {
                    AdapterPerf::default()
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
                    temp_c: perf.temp_c,
                    fan_rpm: perf.fan_rpm,
                    throttling: perf.throttling,
                    core_clock_mhz: clocks.core_clock_mhz,
                    mem_clock_mhz: clocks.mem_clock_mhz,
                    encode_pct: encode.get(&a.luid_key).map(|p| round1(*p)),
                    decode_pct: decode.get(&a.luid_key).map(|p| round1(*p)),
                }
            })
            .collect();
        let b = &core.energy.breakdown;
        let power = (b.apu.is_some() || b.cpu.is_some() || b.gpu.is_some() || b.npu.is_some())
            .then(|| b.clone());
        let memory = commit().map(|(committed_bytes, commit_limit_bytes)| MemoryDetail {
            committed_bytes,
            commit_limit_bytes,
            cached_bytes: cached,
            compressed_bytes: compressed,
        });
        DetailRead {
            temps: core.temps.clone(),
            gpus,
            power,
            cpu_limit,
            drive_rates,
            memory,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_snapshot_has_no_detail() {
        let mut s = Sampler::new();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        let snap = s.sample(MetricLimits::default(), false, true);
        assert!(snap.detail.is_none());
        assert!(snap.core.ram.total_bytes > 0);
        assert!((0.0..=100.0).contains(&snap.core.cpu.percent));
    }

    #[test]
    fn detail_appears_on_second_expanded_tick_and_drops_on_collapse() {
        let mut s = Sampler::new();
        let t = MetricLimits::default();
        assert!(
            s.sample(t, true, true).detail.is_none(),
            "first tick only primes"
        );
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        let d = s
            .sample(t, true, true)
            .detail
            .expect("second tick has detail");
        assert!(!d.cores.is_empty());
        assert!(
            !d.processes
                .expect("second tick refreshes processes")
                .is_empty()
        );
        assert!(s.sample(t, false, true).detail.is_none());
        assert!(s.detail.is_none(), "detail state released on collapse");
    }

    #[test]
    fn processes_refresh_every_third_tick_and_stop_when_closed() {
        let mut s = Sampler::new();
        let t = MetricLimits::default();
        s.sample(t, true, true);
        let sent: Vec<bool> = (0..4)
            .map(|_| {
                std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
                let d = s.sample(t, true, true).detail.expect("detail");
                d.processes.is_some()
            })
            .collect();
        assert_eq!(sent, [true, false, false, true]);
        let d = s.sample(t, true, false).detail.expect("detail");
        assert!(d.processes.is_none());
        assert!(
            s.detail.as_ref().is_some_and(|d| d.procs.is_none()),
            "process state released"
        );
    }

    /// Machine-dependent: `cargo test snapshot_live -- --ignored --nocapture` prints a real
    /// expanded snapshot, to check sensors on new hardware.
    #[test]
    #[ignore]
    fn snapshot_live() {
        let mut s = Sampler::new();
        let t = MetricLimits::default();
        s.sample(t, true, true);
        std::thread::sleep(std::time::Duration::from_secs(1));
        let snap = s.sample(t, true, true);
        println!("{}", serde_json::to_string_pretty(&snap).unwrap());
    }
}
