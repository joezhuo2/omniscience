use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use super::types::HistoryEvent;

const HOUR_MS: u64 = 3_600_000;
const FILE: &str = "events.jsonl";
const MAX_EVENTS: usize = 2000;
const CPU_LIMIT_EVENT_PCT: f32 = 10.0;

#[derive(Debug, Default)]
pub struct EventLog {
    path: Option<PathBuf>,
    events: VecDeque<HistoryEvent>,
}

fn cutoff(now_ms: u64, hours: u32) -> u64 {
    now_ms.saturating_sub(u64::from(hours) * HOUR_MS)
}

fn write_all(path: &Path, events: &VecDeque<HistoryEvent>) -> std::io::Result<()> {
    let tmp = path.with_extension("jsonl.tmp");
    let mut out = std::io::BufWriter::new(File::create(&tmp)?);
    for e in events {
        serde_json::to_writer(&mut out, e)?;
        out.write_all(b"\n")?;
    }
    out.into_inner().map_err(|e| e.into_error())?.sync_all()?;
    std::fs::rename(tmp, path)
}

fn append(path: &Path, event: &HistoryEvent) -> std::io::Result<()> {
    let mut line = serde_json::to_vec(event)?;
    line.push(b'\n');
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?
        .write_all(&line)
}

impl EventLog {
    pub fn open(dir: Option<&Path>, now_ms: u64, hours: u32) -> Self {
        let Some(dir) = dir else {
            return Self::default();
        };
        let path = dir.join(FILE);
        let from = cutoff(now_ms, hours);
        let mut events: Vec<HistoryEvent> = File::open(&path)
            .map(|f| {
                BufReader::new(f)
                    .lines()
                    .map_while(Result::ok)
                    .filter_map(|line| serde_json::from_str::<HistoryEvent>(&line).ok())
                    .filter(|e| e.ts_ms >= from && e.ts_ms <= now_ms)
                    .collect()
            })
            .unwrap_or_default();
        events.sort_by_key(|e| e.ts_ms);
        let skip = events.len().saturating_sub(MAX_EVENTS);
        let events: VecDeque<_> = events.into_iter().skip(skip).collect();
        let result = std::fs::create_dir_all(dir).and_then(|()| write_all(&path, &events));
        if let Err(e) = result {
            crate::diagnostics::log_error(format_args!("events compact: {e}"));
        }
        Self {
            path: Some(path),
            events,
        }
    }

    pub fn record(&mut self, event: HistoryEvent, now_ms: u64, hours: u32) {
        self.events.push_back(event);
        let from = cutoff(now_ms, hours);
        let before = self.events.len();
        while self.events.front().is_some_and(|e| e.ts_ms < from) || self.events.len() > MAX_EVENTS
        {
            self.events.pop_front();
        }
        let Some(path) = &self.path else { return };
        let result = if self.events.len() == before {
            append(path, self.events.back().expect("just pushed"))
        } else {
            write_all(path, &self.events)
        };
        if let Err(e) = result {
            crate::diagnostics::log_error(format_args!("events write: {e}"));
        }
    }

    pub fn since(&self, since_ms: Option<u64>) -> Vec<HistoryEvent> {
        let from = since_ms.unwrap_or(0);
        let start = self.events.partition_point(|e| e.ts_ms < from);
        self.events.range(start..).cloned().collect()
    }
}

#[derive(Debug, Default)]
pub struct Watch {
    ac: Option<bool>,
    gpu_throttling: Vec<bool>,
    cpu_limited: bool,
}

impl Watch {
    pub fn power(&mut self, ac: Option<bool>) -> Option<&'static str> {
        let ac = ac?;
        let prev = self.ac.replace(ac)?;
        (prev != ac).then_some(if ac { "Plugged in" } else { "On battery" })
    }

    pub fn throttle<'a>(
        &mut self,
        gpus: impl Iterator<Item = (&'a str, bool)>,
        cpu_limit_pct: Option<f32>,
    ) -> Vec<String> {
        let mut labels = Vec::new();
        let mut now = Vec::new();
        for (i, (name, throttling)) in gpus.enumerate() {
            if throttling && !self.gpu_throttling.get(i).copied().unwrap_or(false) {
                labels.push(format!("{name} throttling"));
            }
            now.push(throttling);
        }
        self.gpu_throttling = now;
        let limited = cpu_limit_pct.is_some_and(|p| p >= CPU_LIMIT_EVENT_PCT);
        if let Some(p) = cpu_limit_pct.filter(|_| limited && !self.cpu_limited) {
            labels.push(format!("CPU limited by {p:.0}%"));
        }
        self.cpu_limited = limited;
        labels
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::types::HistoryEventKind;

    const T0: u64 = 1_700_000_000_000;

    fn ev(ts_ms: u64) -> HistoryEvent {
        HistoryEvent {
            ts_ms,
            kind: HistoryEventKind::Alert,
            label: format!("e{ts_ms}"),
        }
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("omniscience-events-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn power_fires_only_on_a_flip_after_the_first_reading() {
        let mut w = Watch::default();
        assert_eq!(w.power(None), None);
        assert_eq!(w.power(Some(true)), None);
        assert_eq!(w.power(Some(true)), None);
        assert_eq!(w.power(None), None);
        assert_eq!(w.power(Some(false)), Some("On battery"));
        assert_eq!(w.power(Some(false)), None);
        assert_eq!(w.power(Some(true)), Some("Plugged in"));
    }

    #[test]
    fn throttle_fires_on_rising_edges_and_rearms() {
        let mut w = Watch::default();
        let gpu = |t| [("RTX", t)].into_iter();
        assert!(w.throttle(gpu(false), Some(5.0)).is_empty());
        assert_eq!(
            w.throttle(gpu(true), Some(25.0)),
            ["RTX throttling", "CPU limited by 25%"]
        );
        assert!(w.throttle(gpu(true), Some(40.0)).is_empty());
        assert!(w.throttle(gpu(false), Some(9.0)).is_empty());
        assert_eq!(
            w.throttle(gpu(true), Some(10.0)),
            ["RTX throttling", "CPU limited by 10%"]
        );
        assert!(w.throttle(gpu(true), None).is_empty());
        assert_eq!(w.throttle(gpu(true), Some(10.0)), ["CPU limited by 10%"]);
    }

    #[test]
    fn keeps_the_window_and_the_cap_in_memory() {
        let mut log = EventLog::default();
        for i in 0..2100 {
            log.record(ev(T0 + i), T0 + i, 1);
        }
        assert_eq!(log.since(None).len(), MAX_EVENTS);
        assert_eq!(log.since(None)[0].ts_ms, T0 + 100);
        log.record(ev(T0 + 2 * HOUR_MS), T0 + 2 * HOUR_MS, 1);
        assert_eq!(log.since(None).len(), 1);
        assert_eq!(log.since(Some(T0 + 3 * HOUR_MS)).len(), 0);
    }

    #[test]
    fn persists_appends_and_prunes_on_load() {
        let dir = temp_dir("load");
        let mut log = EventLog::open(Some(&dir), T0, 24);
        log.record(ev(T0), T0, 24);
        log.record(ev(T0 + 1), T0 + 1, 24);
        let path = dir.join(FILE);
        let mut f = OpenOptions::new().append(true).open(&path).unwrap();
        f.write_all(b"{\"truncated\n").unwrap();
        drop(f);

        let reloaded = EventLog::open(Some(&dir), T0 + 2, 24);
        assert_eq!(reloaded.since(None), log.since(None));
        assert_eq!(std::fs::read_to_string(&path).unwrap().lines().count(), 2);

        let pruned = EventLog::open(Some(&dir), T0 + 1 + HOUR_MS, 1);
        assert_eq!(pruned.since(None).len(), 1);
        assert_eq!(std::fs::read_to_string(&path).unwrap().lines().count(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
