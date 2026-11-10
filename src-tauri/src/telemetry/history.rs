use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::types::CoreMetrics;

const MINUTE_MS: u64 = 60_000;
const HOUR_MS: u64 = 60 * MINUTE_MS;
const FILE: &str = "history.jsonl";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HistorySample {
    #[ts(type = "number")]
    pub ts_ms: u64,
    pub cpu_pct: f32,
    pub cpu_temp_c: Option<f32>,
    pub ram_pct: f32,
    pub igpu_pct: Option<f32>,
    pub dgpu_pct: Option<f32>,
    pub igpu_temp_c: Option<f32>,
    pub dgpu_temp_c: Option<f32>,
    pub disk_pct: Option<f32>,
    pub net_down_bps: f64,
    pub net_up_bps: f64,
    pub battery_pct: Option<f32>,
    pub package_watts: Option<f32>,
}

#[derive(Debug, Default, Clone, Copy)]
struct Mean {
    sum: f64,
    n: u32,
}

impl Mean {
    fn add(&mut self, v: Option<f64>) {
        if let Some(v) = v.filter(|v| v.is_finite()) {
            self.sum += v;
            self.n += 1;
        }
    }

    fn get(&self) -> Option<f64> {
        (self.n > 0).then(|| self.sum / f64::from(self.n))
    }

    fn get_f32(&self) -> Option<f32> {
        self.get().map(|v| v as f32)
    }
}

fn max(a: Option<f32>, b: Option<f32>) -> Option<f32> {
    match (a, b.filter(|v| v.is_finite())) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (a, b) => a.or(b),
    }
}

#[derive(Debug, Default, Clone)]
struct Minute {
    start_ms: u64,
    cpu: Mean,
    ram: Mean,
    igpu: Mean,
    dgpu: Mean,
    disk: Mean,
    down: Mean,
    up: Mean,
    battery: Mean,
    watts: Mean,
    cpu_temp: Option<f32>,
    igpu_temp: Option<f32>,
    dgpu_temp: Option<f32>,
}

impl Minute {
    fn add(&mut self, core: &CoreMetrics, battery_pct: Option<f32>) {
        let f = |v: Option<f32>| v.map(f64::from);
        self.cpu.add(Some(f64::from(core.cpu.percent)));
        self.ram.add(Some(f64::from(core.ram.percent)));
        self.igpu.add(f(core.igpu.map(|m| m.percent)));
        self.dgpu.add(f(core.dgpu.map(|m| m.percent)));
        self.disk.add(f(core.disk.map(|m| m.percent)));
        self.down.add(Some(core.net.down_bps));
        self.up.add(Some(core.net.up_bps));
        self.battery.add(f(battery_pct));
        self.watts.add(f(core.package_watts));
        self.cpu_temp = max(self.cpu_temp, core.cpu_temp_c);
        self.igpu_temp = max(self.igpu_temp, core.igpu_temp_c);
        self.dgpu_temp = max(self.dgpu_temp, core.dgpu_temp_c);
    }

    fn sample(&self) -> HistorySample {
        HistorySample {
            ts_ms: self.start_ms,
            cpu_pct: self.cpu.get_f32().unwrap_or_default(),
            cpu_temp_c: self.cpu_temp,
            ram_pct: self.ram.get_f32().unwrap_or_default(),
            igpu_pct: self.igpu.get_f32(),
            dgpu_pct: self.dgpu.get_f32(),
            igpu_temp_c: self.igpu_temp,
            dgpu_temp_c: self.dgpu_temp,
            disk_pct: self.disk.get_f32(),
            net_down_bps: self.down.get().unwrap_or_default(),
            net_up_bps: self.up.get().unwrap_or_default(),
            battery_pct: self.battery.get_f32(),
            package_watts: self.watts.get_f32(),
        }
    }
}

#[derive(Debug, Default)]
pub struct History {
    path: Option<PathBuf>,
    samples: VecDeque<HistorySample>,
    pending: Option<Minute>,
    file_lines: usize,
}

fn cutoff(now_ms: u64, hours: u32) -> u64 {
    now_ms.saturating_sub(u64::from(hours) * HOUR_MS)
}

fn write_all(path: &Path, samples: &VecDeque<HistorySample>) -> std::io::Result<()> {
    let tmp = path.with_extension("jsonl.tmp");
    let mut out = std::io::BufWriter::new(File::create(&tmp)?);
    for s in samples {
        serde_json::to_writer(&mut out, s)?;
        out.write_all(b"\n")?;
    }
    out.into_inner().map_err(|e| e.into_error())?.sync_all()?;
    std::fs::rename(tmp, path)
}

impl History {
    pub fn open(dir: Option<&Path>, now_ms: u64, hours: u32) -> Self {
        let Some(dir) = dir else {
            return Self::default();
        };
        let path = dir.join(FILE);
        let from = cutoff(now_ms, hours);
        let mut samples: Vec<HistorySample> = File::open(&path)
            .map(|f| {
                BufReader::new(f)
                    .lines()
                    .map_while(Result::ok)
                    .filter_map(|line| serde_json::from_str::<HistorySample>(&line).ok())
                    .filter(|s| s.ts_ms >= from && s.ts_ms <= now_ms)
                    .collect()
            })
            .unwrap_or_default();
        samples.sort_by_key(|s| s.ts_ms);
        samples.dedup_by_key(|s| s.ts_ms);
        let mut history = Self {
            path: Some(path),
            samples: samples.into(),
            pending: None,
            file_lines: 0,
        };
        history.compact();
        history
    }

    fn compact(&mut self) {
        let Some(path) = &self.path else { return };
        let dir = path.parent().unwrap_or(Path::new("."));
        let result = std::fs::create_dir_all(dir).and_then(|()| write_all(path, &self.samples));
        match result {
            Ok(()) => self.file_lines = self.samples.len(),
            Err(e) => crate::diagnostics::log_error(format_args!("history compact: {e}")),
        }
    }

    pub fn record(&mut self, ts_ms: u64, core: &CoreMetrics, battery_pct: Option<f32>, hours: u32) {
        let start_ms = ts_ms - ts_ms % MINUTE_MS;
        if let Some(done) = self.pending.take_if(|m| m.start_ms != start_ms) {
            self.push(done.sample(), ts_ms, hours);
        }
        self.pending
            .get_or_insert_with(|| Minute {
                start_ms,
                ..Minute::default()
            })
            .add(core, battery_pct);
    }

    fn push(&mut self, sample: HistorySample, now_ms: u64, hours: u32) {
        if self.samples.back().is_some_and(|s| s.ts_ms >= sample.ts_ms) {
            return;
        }
        self.samples.push_back(sample);
        let from = cutoff(now_ms, hours);
        while self.samples.front().is_some_and(|s| s.ts_ms < from) {
            self.samples.pop_front();
        }
        let Some(path) = &self.path else {
            return;
        };
        if self.file_lines > 2 * self.samples.len() + 60 {
            self.compact();
            return;
        }
        if let Err(e) = append(path, self.samples.back().expect("just pushed")) {
            crate::diagnostics::log_error(format_args!("history append: {e}"));
        } else {
            self.file_lines += 1;
        }
    }

    pub fn since(&self, since_ms: Option<u64>) -> Vec<HistorySample> {
        let from = since_ms.unwrap_or(0);
        let start = self.samples.partition_point(|s| s.ts_ms < from);
        self.samples.range(start..).cloned().collect()
    }
}

fn append(path: &Path, sample: &HistorySample) -> std::io::Result<()> {
    let mut line = serde_json::to_vec(sample)?;
    line.push(b'\n');
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?
        .write_all(&line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::types::{Level, MemoryMetric, Metric, NetMetric, Thresholds};

    fn core(cpu: f32, temp: Option<f32>, down: f64) -> CoreMetrics {
        let t = Thresholds::default();
        CoreMetrics {
            cpu: Metric::new(cpu, t),
            cpu_mhz: None,
            cpu_temp_c: temp,
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
                down_bps: down,
                up_bps: 0.0,
                down_pct: 0.0,
                up_pct: 0.0,
                level: Level::Normal,
            },
            igpu: Some(Metric::new(10.0, t)),
            dgpu: None,
            package_watts: None,
        }
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("omniscience-history-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    const T0: u64 = 1_700_000_040_000;

    #[test]
    fn folds_a_minute_into_averages_and_max_temperatures() {
        let mut h = History::default();
        h.record(T0, &core(20.0, Some(50.0), 100.0), Some(80.0), 24);
        h.record(T0 + 1_000, &core(40.0, Some(70.0), 300.0), None, 24);
        h.record(T0 + 2_000, &core(60.0, None, 200.0), Some(82.0), 24);
        assert!(h.since(None).is_empty());
        h.record(T0 + MINUTE_MS, &core(0.0, None, 0.0), None, 24);
        let s = &h.since(None)[0];
        assert_eq!(s.ts_ms, T0);
        assert_eq!(s.cpu_pct, 40.0);
        assert_eq!(s.cpu_temp_c, Some(70.0));
        assert_eq!(s.net_down_bps, 200.0);
        assert_eq!(s.battery_pct, Some(81.0));
        assert_eq!(s.igpu_pct, Some(10.0));
        assert_eq!(s.dgpu_pct, None);
        assert_eq!(s.disk_pct, None);
        assert_eq!(s.package_watts, None);
    }

    #[test]
    fn prunes_samples_older_than_retention_on_append() {
        let mut h = History::default();
        for m in 0..130 {
            h.record(T0 + m * MINUTE_MS, &core(1.0, None, 0.0), None, 1);
        }
        let all = h.since(None);
        let newest = T0 + 129 * MINUTE_MS;
        assert!(all.iter().all(|s| s.ts_ms >= newest - HOUR_MS));
        assert_eq!(all.len(), 60);
        assert_eq!(h.since(Some(newest - 10 * MINUTE_MS)).len(), 10);
    }

    #[test]
    fn persists_and_compacts_on_load() {
        let dir = temp_dir("load");
        let mut h = History::open(Some(&dir), T0, 24);
        for m in 0..4 {
            h.record(T0 + m * MINUTE_MS, &core(m as f32, None, 0.0), None, 24);
        }
        assert_eq!(h.since(None).len(), 3);
        let path = dir.join(FILE);
        let mut f = OpenOptions::new().append(true).open(&path).unwrap();
        f.write_all(b"{\"truncated\n").unwrap();
        drop(f);

        let reloaded = History::open(Some(&dir), T0 + 4 * MINUTE_MS, 24);
        assert_eq!(reloaded.since(None), h.since(None));
        let lines = std::fs::read_to_string(&path).unwrap().lines().count();
        assert_eq!(lines, 3);

        let later = T0 + 2 * MINUTE_MS + HOUR_MS;
        let pruned = History::open(Some(&dir), later, 1);
        assert_eq!(pruned.since(None).len(), 1);
        assert_eq!(std::fs::read_to_string(&path).unwrap().lines().count(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rewrites_the_file_when_it_outgrows_memory() {
        let dir = temp_dir("grow");
        let mut h = History::open(Some(&dir), T0, 1);
        for m in 0..400 {
            h.record(T0 + m * MINUTE_MS, &core(1.0, None, 0.0), None, 1);
        }
        let lines = std::fs::read_to_string(dir.join(FILE))
            .unwrap()
            .lines()
            .count();
        assert!(lines <= 2 * 60 + 61, "{lines} lines");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
