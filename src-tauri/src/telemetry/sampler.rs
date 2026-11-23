//! Builds one [`Snapshot`] per tick. The core tier is always sampled; the detail tier exists only
//! while the dashboard is expanded and is dropped (memory released) on collapse. Within the detail
//! tier, a section's costly sources run only while that section is open; slow readings are cached
//! across collapses.

use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use sysinfo::{Networks, System};

use super::battery;
use super::cached::Cached;
use super::peak::RollingPeak;
use super::ping::Pinger;
use super::processes;
use super::public_ip::PublicIp;
use super::types::{
    BluetoothBattery, CoreMetrics, CoreSample, DetailMetrics, DriveInfo, FanSpeeds, MemoryMetric,
    Metric, MetricLimits, NetAdapter, NetMetric, PhysicalDisk, PowerEvent, Snapshot, Thresholds,
    classify,
};

/// Network rings are scaled to the busiest second of the last minute, never below 1 MB/s.
const NET_WINDOW: usize = 60;
const NET_FLOOR_BPS: f64 = 1_000_000.0;

/// While collapsed, GPU engine counters (hundreds of instances) are read one tick in this many.
const GPU_EVERY_COLLAPSED: u32 = 3;
/// Drive space for a collapsed Drives card's summary is re-read one tick in this many.
const DRIVES_EVERY_COLLAPSED: u32 = 10;

fn round1(v: f32) -> f32 {
    (v * 10.0).round() / 10.0
}

/// Dashboard sections open (not collapsed) in some window, so their data is worth collecting.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Sections {
    pub gpu_power: bool,
    pub cpu_cores: bool,
    pub network: bool,
    pub drives: bool,
    pub memory: bool,
    pub system: bool,
    pub bluetooth: bool,
    pub fans: bool,
    pub processes: bool,
}

impl Sections {
    #[cfg(test)]
    pub const ALL: Self = Self {
        gpu_power: true,
        cpu_cores: true,
        network: true,
        drives: true,
        memory: true,
        system: true,
        bluetooth: true,
        fans: true,
        processes: true,
    };

    /// Unknown IDs (config and custom cards, history) are ignored.
    pub fn from_ids<'a>(ids: impl IntoIterator<Item = &'a str>) -> Self {
        let mut s = Self::default();
        for id in ids {
            match id {
                "gpu-power" => s.gpu_power = true,
                "cpu-cores" => s.cpu_cores = true,
                "network" => s.network = true,
                "drives" => s.drives = true,
                "memory" => s.memory = true,
                "system" => s.system = true,
                "bluetooth" => s.bluetooth = true,
                "fans" => s.fans = true,
                "processes" => s.processes = true,
                _ => {}
            }
        }
        s
    }
}

/// Creates `slot` while `on`, drops it while off.
fn keep<T>(slot: &mut Option<T>, on: bool, make: impl FnOnce() -> Option<T>) {
    if !on {
        *slot = None;
    } else if slot.is_none() {
        *slot = make();
    }
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
    disks: Cached<Vec<PhysicalDisk>>,
    power_events: Cached<Vec<PowerEvent>>,
    metered: Cached<Option<bool>>,
    bluetooth: Cached<Vec<BluetoothBattery>>,
    /// Last fan reading, kept while the fan source is stopped.
    fans: Option<FanSpeeds>,
    drives: Vec<DriveInfo>,
    drives_wait: u32,
}

/// The process list is refreshed (and sent) on one tick in this many.
const PROCESS_EVERY: u32 = 3;

/// Detail-tier state, alive only while expanded. Each `Option` source lives while its section is
/// open.
struct Detail {
    #[cfg(windows)]
    gpu_mem: Option<super::windows::Query>,
    #[cfg(windows)]
    cpu_limit: Option<super::windows::Query>,
    #[cfg(windows)]
    drive_io: Option<super::windows::Query>,
    #[cfg(windows)]
    mem: Option<super::windows::Query>,
    #[cfg(windows)]
    fans: Option<super::fans::FanSource>,
    /// Loaded the first time GPU & Power opens; the inner `None` means no NVIDIA driver.
    #[cfg(windows)]
    nvml: Option<Option<super::nvml::Nvml>>,
    #[cfg(windows)]
    wifi: Option<super::wifi::Wifi>,
    battery: battery::Devices,
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
            disks: Cached::new(Duration::from_secs(300), super::slow::disks),
            power_events: Cached::new(Duration::from_secs(600), super::slow::power_events),
            metered: Cached::new(Duration::from_secs(300), super::slow::metered),
            bluetooth: Cached::new(Duration::from_secs(30), super::bluetooth::batteries),
            fans: None,
            drives: Vec::new(),
            drives_wait: 0,
        }
    }

    /// Host to ping while expanded (empty for none), and whether to look up the public IP.
    pub fn set_network(&mut self, ping_target: Arc<str>, public_ip: bool) {
        self.ping_target = ping_target;
        if public_ip != self.public_ip.is_some() {
            self.public_ip = public_ip.then(PublicIp::default);
        }
    }

    /// Drops detail state and stops the background probes; cached readings are kept.
    pub fn suspend_detail(&mut self) {
        self.detail = None;
        self.disks.set_active(false);
        self.power_events.set_active(false);
        self.metered.set_active(false);
        self.bluetooth.set_active(false);
    }

    /// `sections`: which dashboard sections are open, so their data is worth reading.
    pub fn sample(&mut self, t: MetricLimits, expanded: bool, sections: Sections) -> Snapshot {
        if !expanded {
            self.suspend_detail();
        }
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();

        #[cfg(windows)]
        let win = self
            .win
            .sample(if expanded { 1 } else { GPU_EVERY_COLLAPSED });

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
            let d = self.sample_detail(t.cpu, sections, &win);
            #[cfg(not(windows))]
            let d = self.sample_detail(t.cpu, sections);
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
        s: Sections,
        #[cfg(windows)] win: &win::CoreSample,
    ) -> Option<DetailMetrics> {
        let detail = self.detail.get_or_insert_with(|| Detail {
            #[cfg(windows)]
            gpu_mem: None,
            #[cfg(windows)]
            cpu_limit: None,
            #[cfg(windows)]
            drive_io: None,
            #[cfg(windows)]
            mem: None,
            #[cfg(windows)]
            fans: None,
            #[cfg(windows)]
            nvml: None,
            #[cfg(windows)]
            wifi: None,
            battery: battery::Devices::find(),
            pinger: None,
            procs: None,
            warm: false,
        });
        #[cfg(windows)]
        {
            use super::windows::Query;
            keep(&mut detail.gpu_mem, s.gpu_power, || {
                Query::new(&win::GPU_MEM_COUNTERS)
            });
            keep(&mut detail.cpu_limit, s.cpu_cores, || {
                Query::new(&win::CPU_LIMIT_COUNTERS)
            });
            keep(&mut detail.drive_io, s.drives, || {
                Query::new(&win::DRIVE_COUNTERS)
            });
            keep(&mut detail.mem, s.memory, || {
                Query::new(&win::MEMORY_COUNTERS)
            });
            keep(&mut detail.wifi, s.network, super::wifi::Wifi::open);
            if s.gpu_power && detail.nvml.is_none() {
                let nvidia = self.win.adapters.iter().any(|a| a.name.contains("NVIDIA"));
                detail.nvml = Some(nvidia.then(super::nvml::Nvml::load).flatten());
            }
            // Also runs until the first reading, which decides whether the Fans card shows at all.
            keep(&mut detail.fans, s.fans || self.fans.is_none(), || {
                Some(super::fans::FanSource::start())
            });
            if let Some(f) = detail.fans.as_ref().and_then(|f| f.latest()) {
                self.fans = Some(f);
            }
        }
        self.disks.set_active(s.drives);
        self.power_events.set_active(s.system);
        self.metered.set_active(s.network);
        // Also probes while no device is known, which decides whether the Bluetooth card shows.
        let no_devices = self.bluetooth.latest().is_none_or(|b| b.is_empty());
        self.bluetooth.set_active(s.bluetooth || no_devices);

        let target = &self.ping_target;
        if detail.pinger.as_ref().map(Pinger::target) != Some(&**target) {
            detail.pinger = (!target.is_empty()).then(|| Pinger::start(target.clone()));
        }
        let procs = if s.processes {
            let p = detail.procs.get_or_insert_with(Procs::start);
            p.due().then_some(p)
        } else {
            detail.procs = None;
            None
        };
        if s.cpu_cores {
            self.sys.refresh_cpu_frequency();
        }
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

        if s.network
            && let Some(ip) = self.public_ip.as_mut()
        {
            ip.refresh();
        }

        #[cfg(windows)]
        let (temps, gpus, power, cpu_limit, memory, drive_rates) = (
            win.temps.clone(),
            win::gpus(
                win,
                &self.win.adapters,
                detail.gpu_mem.as_ref(),
                detail.nvml.as_ref().and_then(Option::as_ref),
            ),
            win::power(win),
            win::cpu_limit(detail.cpu_limit.as_ref()),
            win::memory(detail.mem.as_ref()),
            win::drive_rates(detail.drive_io.as_ref()),
        );
        #[cfg(not(windows))]
        let (temps, gpus, power, cpu_limit, memory, drive_rates) = (
            Vec::new(),
            Vec::new(),
            None,
            None,
            None,
            std::collections::HashMap::new(),
        );

        if s.drives || self.drives_wait == 0 {
            self.drives = super::drives::list(&drive_rates);
            self.drives_wait = DRIVES_EVERY_COLLAPSED;
        }
        self.drives_wait = self.drives_wait.saturating_sub(1);

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

        let quality = detail.pinger.as_ref().map_or((None, None), Pinger::quality);

        #[cfg(windows)]
        let wifi = detail.wifi.as_ref().and_then(super::wifi::Wifi::current);
        #[cfg(not(windows))]
        let wifi = None;

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
            fans: self.fans,
            processes: groups,
            drives: self.drives.clone(),
            disks: self.disks.latest().unwrap_or_default(),
            wifi,
            data_today_bytes: None,
            data_month_bytes: None,
            metered: self.metered.latest().flatten(),
            memory,
            boot_ms: Some(System::boot_time() * 1000),
            power_events: self.power_events.latest().unwrap_or_default(),
            bluetooth: self.bluetooth.latest().unwrap_or_default(),
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
    use std::sync::Arc;

    use super::super::nvml::Nvml;
    use super::super::parse::{self, EnergyReadings};
    use super::super::types::{GpuDetail, GpuKind, MemoryDetail, PowerBreakdown, TempSample};
    use super::super::windows::{Adapter, AdapterPerf, Query, adapter_perf, adapters, commit};
    use super::round1;

    const DISK_IDLE: usize = 0;
    const ENERGY: usize = 1;
    const CPU_PERF: usize = 2;
    const CPU_BASE: usize = 3;
    const THERMAL: usize = 4;
    const CORE_COUNTERS: [&str; 5] = [
        r"\PhysicalDisk(_Total)\% Idle Time",
        r"\Energy Meter(*)\Power",
        r"\Processor Information(*)\% Processor Performance",
        r"\Processor Information(*)\Processor Frequency",
        r"\Thermal Zone Information(*)\High Precision Temperature",
    ];

    /// Hundreds of instances (per process and engine), so it has its own, less frequent query.
    const GPU_ENGINE_COUNTERS: [&str; 1] = [r"\GPU Engine(*)\Utilization Percentage"];

    const GPU_DEDICATED: usize = 0;
    const GPU_SHARED: usize = 1;
    pub const GPU_MEM_COUNTERS: [&str; 2] = [
        r"\GPU Adapter Memory(*)\Dedicated Usage",
        r"\GPU Adapter Memory(*)\Shared Usage",
    ];

    pub const CPU_LIMIT_COUNTERS: [&str; 1] =
        [r"\Processor Information(_Total)\% Performance Limit"];

    const DRIVE_READ: usize = 0;
    const DRIVE_WRITE: usize = 1;
    pub const DRIVE_COUNTERS: [&str; 2] = [
        r"\LogicalDisk(*)\Disk Read Bytes/sec",
        r"\LogicalDisk(*)\Disk Write Bytes/sec",
    ];

    const MEM_CACHED: std::ops::Range<usize> = 0..4;
    const MEM_COMPRESSED: usize = 4;
    pub const MEMORY_COUNTERS: [&str; 5] = [
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
        gpu: Option<Query>,
        /// Last GPU engine read, reused on ticks that skip it.
        engines: Arc<[(String, f64)]>,
        usage: HashMap<String, f32>,
        gpu_wait: u32,
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
        pub gpu_engines: Arc<[(String, f64)]>,
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
                gpu: Query::new(&GPU_ENGINE_COUNTERS),
                engines: Arc::new([]),
                usage: HashMap::new(),
                gpu_wait: 0,
                adapters: adapters(),
            }
        }

        /// Reads GPU engines once every `gpu_every` calls; the rest reuse the last read. The
        /// counter is a rate, so a longer gap still yields the average over it.
        fn sample_gpu(&mut self, gpu_every: u32) {
            self.gpu_wait = self.gpu_wait.min(gpu_every.saturating_sub(1));
            if self.gpu_wait > 0 {
                self.gpu_wait -= 1;
                return;
            }
            self.gpu_wait = gpu_every.saturating_sub(1);
            if let Some(q) = self.gpu.as_ref().filter(|q| q.collect_ok()) {
                let engines = q.items(0).unwrap_or_default();
                self.usage = parse::gpu_usage_by_adapter(&engines);
                self.engines = engines.into();
            }
        }

        pub fn sample(&mut self, gpu_every: u32) -> CoreSample {
            self.sample_gpu(gpu_every);
            let Some(q) = self.query.as_mut().filter(|q| q.collect_ok()) else {
                return CoreSample {
                    temps: Vec::new(),
                    gpu_perf: HashMap::new(),
                    disk_pct: None,
                    igpu_pct: None,
                    dgpu_pct: None,
                    gpu_usage: HashMap::new(),
                    gpu_engines: Arc::new([]),
                    energy: EnergyReadings::default(),
                    cpu_mhz: None,
                    core_mhz: HashMap::new(),
                };
            };
            let disk_pct = q.items(DISK_IDLE).and_then(|v| {
                v.first()
                    .map(|(_, idle)| (100.0 - idle).clamp(0.0, 100.0) as f32)
            });
            let gpu_usage = &self.usage;
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
                gpu_usage: gpu_usage.clone(),
                gpu_engines: self.engines.clone(),
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

    fn collected(query: Option<&Query>) -> Option<&Query> {
        query.filter(|q| q.collect_ok())
    }

    pub fn cpu_limit(query: Option<&Query>) -> Option<f32> {
        collected(query)?
            .items(0)?
            .first()
            .and_then(|(_, pct)| parse::performance_loss_pct(*pct))
    }

    pub fn drive_rates(query: Option<&Query>) -> HashMap<String, (f64, f64)> {
        collected(query).map_or_else(HashMap::new, |q| {
            parse::drive_rates(
                &q.items(DRIVE_READ).unwrap_or_default(),
                &q.items(DRIVE_WRITE).unwrap_or_default(),
            )
        })
    }

    /// Commit charge always (cheap, shown in the collapsed summary); cache and compression only
    /// with `query`.
    pub fn memory(query: Option<&Query>) -> Option<MemoryDetail> {
        let q = collected(query);
        let first = |i: usize| {
            q.and_then(|q| q.items(i))
                .and_then(|v| v.first().map(|(_, b)| b.max(0.0) as u64))
        };
        let cached_bytes = MEM_CACHED.filter_map(first).reduce(|a, b| a + b);
        let compressed_bytes = first(MEM_COMPRESSED);
        commit().map(|(committed_bytes, commit_limit_bytes)| MemoryDetail {
            committed_bytes,
            commit_limit_bytes,
            cached_bytes,
            compressed_bytes,
        })
    }

    pub fn power(core: &CoreSample) -> Option<PowerBreakdown> {
        let b = &core.energy.breakdown;
        (b.apu.is_some() || b.cpu.is_some() || b.gpu.is_some() || b.npu.is_some())
            .then(|| b.clone())
    }

    /// Adapters with core-tier readings; memory needs `mem_query` and clocks need `nvml`, both of
    /// which exist only while GPU & Power is open.
    pub fn gpus(
        core: &CoreSample,
        adapters: &[Adapter],
        mem_query: Option<&Query>,
        nvml: Option<&Nvml>,
    ) -> Vec<GpuDetail> {
        let (dedicated, shared) = collected(mem_query).map_or_else(Default::default, |q| {
            (
                q.items(GPU_DEDICATED)
                    .map(|v| parse::sum_by_adapter(&v))
                    .unwrap_or_default(),
                q.items(GPU_SHARED)
                    .map(|v| parse::sum_by_adapter(&v))
                    .unwrap_or_default(),
            )
        });
        let decode = parse::engine_pct_by_adapter(&core.gpu_engines, "VideoDecode");
        let encode = parse::engine_pct_by_adapter(&core.gpu_engines, "VideoEncode");
        let mut nvidia = 0;
        adapters
            .iter()
            .map(|a| {
                let usage = core.gpu_usage.get(&a.luid_key).copied().unwrap_or(0.0);
                let discrete = a.kind == GpuKind::Discrete;
                let unavailable = AdapterPerf::default();
                let perf = core.gpu_perf.get(&a.luid_key).unwrap_or(&unavailable);
                let clocks = if a.name.contains("NVIDIA") {
                    nvidia += 1;
                    match nvml {
                        Some(n) => guarded_perf(a.kind, usage, || {
                            let (core_mhz, mem_mhz) = n.clocks(nvidia - 1);
                            AdapterPerf {
                                core_clock_mhz: core_mhz,
                                mem_clock_mhz: mem_mhz,
                                ..Default::default()
                            }
                        }),
                        None => AdapterPerf::default(),
                    }
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
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_snapshot_has_no_detail() {
        let mut s = Sampler::new();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        let snap = s.sample(MetricLimits::default(), false, Sections::ALL);
        assert!(snap.detail.is_none());
        assert!(snap.core.ram.total_bytes > 0);
        assert!((0.0..=100.0).contains(&snap.core.cpu.percent));
    }

    #[test]
    fn detail_appears_on_second_expanded_tick_and_drops_on_collapse() {
        let mut s = Sampler::new();
        let t = MetricLimits::default();
        assert!(
            s.sample(t, true, Sections::ALL).detail.is_none(),
            "first tick only primes"
        );
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        let d = s
            .sample(t, true, Sections::ALL)
            .detail
            .expect("second tick has detail");
        assert!(!d.cores.is_empty());
        assert!(
            !d.processes
                .expect("second tick refreshes processes")
                .is_empty()
        );
        assert!(s.sample(t, false, Sections::ALL).detail.is_none());
        assert!(s.detail.is_none(), "detail state released on collapse");
    }

    #[test]
    fn processes_refresh_every_third_tick_and_stop_when_closed() {
        let mut s = Sampler::new();
        let t = MetricLimits::default();
        s.sample(t, true, Sections::ALL);
        let sent: Vec<bool> = (0..4)
            .map(|_| {
                std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
                let d = s.sample(t, true, Sections::ALL).detail.expect("detail");
                d.processes.is_some()
            })
            .collect();
        assert_eq!(sent, [true, false, false, true]);
        let d = s
            .sample(t, true, Sections::default())
            .detail
            .expect("detail");
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
        s.sample(t, true, Sections::ALL);
        std::thread::sleep(std::time::Duration::from_secs(1));
        let snap = s.sample(t, true, Sections::ALL);
        println!("{}", serde_json::to_string_pretty(&snap).unwrap());
    }
}
