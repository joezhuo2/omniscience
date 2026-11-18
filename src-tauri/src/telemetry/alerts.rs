use std::collections::HashMap;
use std::sync::Arc;

use super::ping::Pinger;
use super::types::CoreMetrics;
use super::{drives, usage};
use crate::settings::{AlertRule, AlertSettings};

const RULES: usize = 12;
const DAY_MS: i64 = 86_400_000;
const DRIVE_EVERY_MS: u64 = 60_000;
const USAGE: &str = "High sustained usage";

#[derive(Debug, Clone, PartialEq)]
pub struct Notice {
    pub title: &'static str,
    pub body: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
enum Echo {
    #[default]
    Unknown,
    Lost,
    Ms(f32),
}

#[derive(Debug, Default)]
struct Inputs<'a> {
    cpu: f32,
    memory: f32,
    disk: Option<f32>,
    igpu: Option<f32>,
    dgpu: Option<f32>,
    cpu_temp: Option<f32>,
    gpu_temp: Option<f32>,
    free_pcts: &'a [(String, f32)],
    echo: Echo,
    target: &'a str,
    charge: Option<(f32, bool)>,
}

#[derive(Debug, Default)]
struct Timer {
    since: Option<u64>,
    fired: bool,
}

impl Timer {
    fn update(&mut self, now_ms: u64, active: bool, duration_s: u32) -> bool {
        if !active {
            *self = Self::default();
            return false;
        }
        let start = *self.since.get_or_insert(now_ms);
        if !self.fired && now_ms.saturating_sub(start) >= u64::from(duration_s) * 1000 {
            self.fired = true;
            return true;
        }
        false
    }
}

fn seconds(d: u32) -> String {
    format!("{d} second{}", if d == 1 { "" } else { "s" })
}

fn above(what: &str, threshold: f32, unit: &str, d: u32) -> String {
    if d == 0 {
        format!("{what} is above {threshold}{unit}.")
    } else {
        format!(
            "{what} has been above {threshold}{unit} for {}.",
            seconds(d)
        )
    }
}

fn over(
    r: &AlertRule,
    title: &'static str,
    what: &str,
    unit: &str,
    v: Option<f32>,
) -> Option<Notice> {
    let v = v?;
    (r.enabled && v > r.threshold).then(|| Notice {
        title,
        body: above(what, r.threshold, unit, r.duration_s),
    })
}

fn lowest_below(free_pcts: &[(String, f32)], threshold: f32) -> Option<&(String, f32)> {
    free_pcts
        .iter()
        .filter(|(_, p)| *p < threshold)
        .min_by(|a, b| a.1.total_cmp(&b.1))
}

fn hits(a: &AlertSettings, i: &Inputs) -> [Option<Notice>; RULES] {
    let ping = i.echo;
    [
        over(&a.cpu, USAGE, "CPU", "%", Some(i.cpu)),
        over(&a.memory, USAGE, "Memory", "%", Some(i.memory)),
        over(&a.disk, USAGE, "Disk", "%", i.disk),
        over(&a.igpu, USAGE, "Integrated GPU", "%", i.igpu),
        over(&a.dgpu, USAGE, "Discrete GPU", "%", i.dgpu),
        over(&a.cpu_temp, "High temperature", "CPU", "°C", i.cpu_temp),
        over(&a.gpu_temp, "High temperature", "GPU", "°C", i.gpu_temp),
        a.disk_space
            .enabled
            .then(|| lowest_below(i.free_pcts, a.disk_space.threshold))
            .flatten()
            .map(|(name, pct)| Notice {
                title: "Low disk space",
                body: format!("{name} has {pct:.0}% free."),
            }),
        (a.network_down.enabled && ping == Echo::Lost).then(|| Notice {
            title: "Network down",
            body: if a.network_down.duration_s == 0 {
                format!("No reply from {}.", i.target)
            } else {
                format!(
                    "No reply from {} for {}.",
                    i.target,
                    seconds(a.network_down.duration_s)
                )
            },
        }),
        match ping {
            Echo::Ms(ms) => over(&a.ping_spike, "High ping", "Ping", " ms", Some(ms)),
            _ => None,
        },
        i.charge
            .filter(|(p, ac)| a.battery_full.enabled && *ac && *p >= a.battery_full.threshold)
            .map(|(p, _)| Notice {
                title: "Battery charged",
                body: format!("Unplug the charger: battery at {p:.0}%."),
            }),
        i.charge
            .filter(|(p, ac)| a.battery_low.enabled && !*ac && *p <= a.battery_low.threshold)
            .map(|(p, _)| Notice {
                title: "Battery low",
                body: format!("Battery at {p:.0}%. Plug in the charger."),
            }),
    ]
}

fn rules(a: &AlertSettings) -> [&AlertRule; RULES] {
    [
        &a.cpu,
        &a.memory,
        &a.disk,
        &a.igpu,
        &a.dgpu,
        &a.cpu_temp,
        &a.gpu_temp,
        &a.disk_space,
        &a.network_down,
        &a.ping_spike,
        &a.battery_full,
        &a.battery_low,
    ]
}

fn in_quiet_hours(minute: u16, start: u16, end: u16) -> bool {
    match start.cmp(&end) {
        std::cmp::Ordering::Equal => false,
        std::cmp::Ordering::Less => (start..end).contains(&minute),
        std::cmp::Ordering::Greater => minute >= start || minute < end,
    }
}

fn suppressed(a: &AlertSettings, now_ms: u64, minute: u16) -> bool {
    a.snooze_until_ms > now_ms
        || (a.quiet_hours && in_quiet_hours(minute, a.quiet_start_min, a.quiet_end_min))
}

fn local_minute(now_ms: u64) -> u16 {
    ((now_ms as i64 + usage::local_offset_ms()).rem_euclid(DAY_MS) / 60_000) as u16
}

#[derive(Default)]
pub struct Engine {
    timers: [Timer; RULES],
    pinger: Option<Pinger>,
    free_pcts: Vec<(String, f32)>,
    drives_at: Option<u64>,
}

impl Engine {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    fn sync_pinger(&mut self, want: Option<&Arc<str>>) {
        match want {
            None => self.pinger = None,
            Some(t) if self.pinger.as_ref().is_none_or(|p| p.target() != &**t) => {
                self.pinger = Some(Pinger::start(t.clone()));
            }
            Some(_) => {}
        }
    }

    fn refresh_drives(&mut self, now_ms: u64) {
        if self
            .drives_at
            .is_some_and(|at| now_ms.saturating_sub(at) < DRIVE_EVERY_MS)
        {
            return;
        }
        self.drives_at = Some(now_ms);
        self.free_pcts = drives::list(&HashMap::new())
            .into_iter()
            .map(|d| {
                let pct = d.free_bytes as f32 / d.total_bytes as f32 * 100.0;
                (d.name, pct)
            })
            .collect();
    }

    pub fn check(
        &mut self,
        now_ms: u64,
        core: &CoreMetrics,
        charge: Option<(f32, bool)>,
        a: &AlertSettings,
        ping_target: &Arc<str>,
    ) -> Vec<Notice> {
        let pinging = (a.network_down.enabled || a.ping_spike.enabled) && !ping_target.is_empty();
        self.sync_pinger(pinging.then_some(ping_target));
        if a.disk_space.enabled {
            self.refresh_drives(now_ms);
        } else {
            self.drives_at = None;
        }
        let echo =
            self.pinger
                .as_ref()
                .map_or(Echo::Unknown, |p| match (p.latest(), p.quality().1) {
                    (Some(ms), _) => Echo::Ms(ms),
                    (None, Some(_)) => Echo::Lost,
                    (None, None) => Echo::Unknown,
                });
        let inputs = Inputs {
            cpu: core.cpu.percent,
            memory: core.ram.percent,
            disk: core.disk.map(|m| m.percent),
            igpu: core.igpu.map(|m| m.percent),
            dgpu: core.dgpu.map(|m| m.percent),
            cpu_temp: core.cpu_temp_c,
            gpu_temp: [core.igpu_temp_c, core.dgpu_temp_c]
                .into_iter()
                .flatten()
                .reduce(f32::max),
            free_pcts: &self.free_pcts,
            echo,
            target: ping_target,
            charge,
        };
        let fired: Vec<Notice> = self
            .timers
            .iter_mut()
            .zip(rules(a))
            .zip(hits(a, &inputs))
            .filter_map(|((timer, rule), hit)| {
                let due = timer.update(now_ms, hit.is_some(), rule.duration_s);
                hit.filter(|_| due)
            })
            .collect();
        if fired.is_empty() || suppressed(a, now_ms, local_minute(now_ms)) {
            return Vec::new();
        }
        fired
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(enabled: bool, threshold: f32, duration_s: u32) -> AlertRule {
        AlertRule {
            enabled,
            threshold,
            duration_s,
        }
    }

    fn only_cpu(threshold: f32, duration_s: u32) -> AlertSettings {
        let off = rule(false, 0.0, 0);
        AlertSettings {
            cpu: rule(true, threshold, duration_s),
            memory: off,
            disk: off,
            igpu: off,
            dgpu: off,
            ..AlertSettings::default()
        }
    }

    fn cpu_at(v: f32) -> Inputs<'static> {
        Inputs {
            cpu: v,
            ..Inputs::default()
        }
    }

    fn step(t: &mut [Timer; RULES], a: &AlertSettings, now: u64, i: &Inputs) -> Vec<usize> {
        let h = hits(a, i);
        let r = rules(a);
        (0..RULES)
            .filter(|&n| t[n].update(now, h[n].is_some(), r[n].duration_s))
            .collect()
    }

    #[test]
    fn fires_once_after_the_sustain_window() {
        let a = only_cpu(90.0, 30);
        let mut t = Default::default();
        assert!(step(&mut t, &a, 0, &cpu_at(95.0)).is_empty());
        assert!(step(&mut t, &a, 29_000, &cpu_at(95.0)).is_empty());
        assert_eq!(step(&mut t, &a, 30_000, &cpu_at(95.0)), [0]);
        assert!(step(&mut t, &a, 60_000, &cpu_at(95.0)).is_empty());
    }

    #[test]
    fn dropping_below_rearms_and_restarts_the_clock() {
        let a = only_cpu(90.0, 30);
        let mut t = Default::default();
        step(&mut t, &a, 0, &cpu_at(95.0));
        assert_eq!(step(&mut t, &a, 30_000, &cpu_at(95.0)), [0]);
        step(&mut t, &a, 31_000, &cpu_at(50.0));
        assert!(step(&mut t, &a, 32_000, &cpu_at(95.0)).is_empty());
        assert!(step(&mut t, &a, 61_000, &cpu_at(95.0)).is_empty());
        assert_eq!(step(&mut t, &a, 62_000, &cpu_at(95.0)), [0]);
    }

    #[test]
    fn exactly_the_threshold_is_not_high() {
        let a = only_cpu(90.0, 30);
        let mut t = Default::default();
        step(&mut t, &a, 0, &cpu_at(90.0));
        assert!(step(&mut t, &a, 60_000, &cpu_at(90.0)).is_empty());
    }

    #[test]
    fn rules_are_tracked_independently() {
        let mut a = only_cpu(90.0, 30);
        a.memory = rule(true, 90.0, 30);
        let mut t = Default::default();
        step(&mut t, &a, 0, &cpu_at(95.0));
        let both = Inputs {
            cpu: 95.0,
            memory: 95.0,
            ..Inputs::default()
        };
        step(&mut t, &a, 20_000, &both);
        assert_eq!(step(&mut t, &a, 30_000, &both), [0]);
        assert_eq!(step(&mut t, &a, 50_000, &both), [1]);
    }

    #[test]
    fn zero_duration_fires_immediately() {
        let a = only_cpu(90.0, 0);
        let mut t = Default::default();
        assert_eq!(step(&mut t, &a, 5, &cpu_at(95.0)), [0]);
        assert!(step(&mut t, &a, 6, &cpu_at(95.0)).is_empty());
    }

    #[test]
    fn disabled_rules_never_fire() {
        let a = only_cpu(90.0, 0);
        let mut t = Default::default();
        let i = Inputs {
            memory: 99.0,
            ..Inputs::default()
        };
        assert!(step(&mut t, &a, 0, &i).is_empty());
    }

    #[test]
    fn engine_reset_forgets_progress() {
        let mut e = Engine::default();
        e.timers[0].update(0, true, 30);
        e.reset();
        assert!(!e.timers[0].update(30_000, true, 30));
    }

    #[test]
    fn temperature_rules_use_their_threshold() {
        let mut a = only_cpu(90.0, 0);
        a.cpu_temp = rule(true, 80.0, 0);
        a.gpu_temp = rule(true, 70.0, 0);
        let i = Inputs {
            cpu_temp: Some(81.0),
            gpu_temp: Some(70.0),
            ..Inputs::default()
        };
        let h = hits(&a, &i);
        assert_eq!(h[5].as_ref().unwrap().body, "CPU is above 80°C.");
        assert!(h[6].is_none());
    }

    #[test]
    fn low_free_space_names_the_emptiest_drive() {
        let drives = vec![
            ("C:".to_string(), 8.0),
            ("D:".to_string(), 3.0),
            ("E:".to_string(), 50.0),
        ];
        assert_eq!(lowest_below(&drives, 10.0).unwrap().0, "D:");
        assert!(lowest_below(&drives, 3.0).is_none());
        let mut a = only_cpu(90.0, 0);
        a.disk_space = rule(true, 10.0, 0);
        let i = Inputs {
            free_pcts: &drives[..1],
            ..Inputs::default()
        };
        let n = hits(&a, &i)[7].clone().unwrap();
        assert_eq!(
            (n.title, n.body.as_str()),
            ("Low disk space", "C: has 8% free.")
        );
    }

    #[test]
    fn network_rules_follow_the_echo() {
        let mut a = only_cpu(90.0, 0);
        a.network_down = rule(true, 0.0, 30);
        a.ping_spike = rule(true, 200.0, 10);
        let lost = Inputs {
            echo: Echo::Lost,
            target: "1.1.1.1",
            ..Inputs::default()
        };
        let h = hits(&a, &lost);
        assert_eq!(
            h[8].as_ref().unwrap().body,
            "No reply from 1.1.1.1 for 30 seconds."
        );
        assert!(h[9].is_none());
        let slow = Inputs {
            echo: Echo::Ms(250.0),
            ..Inputs::default()
        };
        let h = hits(&a, &slow);
        assert!(h[8].is_none());
        assert_eq!(
            h[9].as_ref().unwrap().body,
            "Ping has been above 200 ms for 10 seconds."
        );
        let unknown = hits(&a, &Inputs::default());
        assert!(unknown[8].is_none() && unknown[9].is_none());
    }

    #[test]
    fn battery_rules_need_the_right_power_source() {
        let mut a = only_cpu(90.0, 0);
        a.battery_full = rule(true, 80.0, 0);
        a.battery_low = rule(true, 20.0, 0);
        let at = |p, ac| Inputs {
            charge: Some((p, ac)),
            ..Inputs::default()
        };
        let h = hits(&a, &at(80.0, true));
        assert_eq!(
            h[10].as_ref().unwrap().body,
            "Unplug the charger: battery at 80%."
        );
        assert!(h[11].is_none());
        assert!(hits(&a, &at(80.0, false))[10].is_none());
        let h = hits(&a, &at(20.0, false));
        assert_eq!(h[11].as_ref().unwrap().title, "Battery low");
        assert!(h[10].is_none());
        assert!(hits(&a, &at(20.0, true))[11].is_none());
        assert!(
            hits(&a, &Inputs::default())[10..]
                .iter()
                .all(Option::is_none)
        );
    }

    #[test]
    fn usage_messages_carry_the_real_threshold_and_duration() {
        let a = only_cpu(85.0, 60);
        assert_eq!(
            hits(&a, &cpu_at(90.0))[0].as_ref().unwrap().body,
            "CPU has been above 85% for 60 seconds."
        );
        assert_eq!(seconds(1), "1 second");
    }

    #[test]
    fn quiet_hours_window() {
        assert!(in_quiet_hours(600, 540, 1020));
        assert!(!in_quiet_hours(1020, 540, 1020));
        assert!(in_quiet_hours(540, 540, 1020));
        assert!(in_quiet_hours(23 * 60, 22 * 60, 7 * 60));
        assert!(in_quiet_hours(3 * 60, 22 * 60, 7 * 60));
        assert!(!in_quiet_hours(7 * 60, 22 * 60, 7 * 60));
        assert!(!in_quiet_hours(12 * 60, 22 * 60, 7 * 60));
        assert!(!in_quiet_hours(100, 300, 300));
    }

    #[test]
    fn snooze_and_quiet_hours_suppress() {
        let mut a = AlertSettings::default();
        assert!(!suppressed(&a, 1000, 0));
        a.snooze_until_ms = 1001;
        assert!(suppressed(&a, 1000, 0));
        assert!(!suppressed(&a, 1001, 0));
        a.snooze_until_ms = 0;
        a.quiet_hours = true;
        a.quiet_start_min = 60;
        a.quiet_end_min = 120;
        assert!(suppressed(&a, 0, 90));
        assert!(!suppressed(&a, 0, 30));
        a.quiet_hours = false;
        assert!(!suppressed(&a, 0, 90));
    }
}
