use super::types::CoreMetrics;

const THRESHOLD_PCT: f32 = 90.0;
const SUSTAIN_MS: u64 = 30_000;
const LABELS: [&str; 5] = ["CPU", "Memory", "Disk", "Integrated GPU", "Discrete GPU"];

#[derive(Debug, Default)]
pub struct Sustained {
    since: [Option<u64>; LABELS.len()],
    fired: [bool; LABELS.len()],
}

impl Sustained {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn check(&mut self, now_ms: u64, core: &CoreMetrics) -> Vec<&'static str> {
        self.update(
            now_ms,
            [
                Some(core.cpu.percent),
                Some(core.ram.percent),
                core.disk.map(|m| m.percent),
                core.igpu.map(|m| m.percent),
                core.dgpu.map(|m| m.percent),
            ],
        )
    }

    fn update(&mut self, now_ms: u64, readings: [Option<f32>; LABELS.len()]) -> Vec<&'static str> {
        let mut fire = Vec::new();
        for (i, reading) in readings.into_iter().enumerate() {
            if !reading.is_some_and(|p| p > THRESHOLD_PCT) {
                self.since[i] = None;
                self.fired[i] = false;
                continue;
            }
            let start = *self.since[i].get_or_insert(now_ms);
            if !self.fired[i] && now_ms.saturating_sub(start) >= SUSTAIN_MS {
                self.fired[i] = true;
                fire.push(LABELS[i]);
            }
        }
        fire
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HIGH: [Option<f32>; 5] = [Some(95.0), None, None, None, None];
    const LOW: [Option<f32>; 5] = [Some(50.0), None, None, None, None];

    #[test]
    fn fires_once_after_the_sustain_window() {
        let mut s = Sustained::default();
        assert!(s.update(0, HIGH).is_empty());
        assert!(s.update(29_000, HIGH).is_empty());
        assert_eq!(s.update(30_000, HIGH), ["CPU"]);
        assert!(s.update(60_000, HIGH).is_empty());
    }

    #[test]
    fn dropping_below_rearms_and_restarts_the_clock() {
        let mut s = Sustained::default();
        s.update(0, HIGH);
        assert_eq!(s.update(30_000, HIGH), ["CPU"]);
        s.update(31_000, LOW);
        assert!(s.update(32_000, HIGH).is_empty());
        assert!(s.update(61_000, HIGH).is_empty());
        assert_eq!(s.update(62_000, HIGH), ["CPU"]);
    }

    #[test]
    fn exactly_the_threshold_is_not_high() {
        let mut s = Sustained::default();
        let at = [Some(90.0), None, None, None, None];
        s.update(0, at);
        assert!(s.update(60_000, at).is_empty());
    }

    #[test]
    fn metrics_are_tracked_independently() {
        let mut s = Sustained::default();
        let mixed = [Some(95.0), Some(95.0), None, None, None];
        s.update(0, [Some(95.0), None, None, None, None]);
        s.update(20_000, mixed);
        assert_eq!(s.update(30_000, mixed), ["CPU"]);
        assert_eq!(s.update(50_000, mixed), ["Memory"]);
    }

    #[test]
    fn reset_forgets_progress() {
        let mut s = Sustained::default();
        s.update(0, HIGH);
        s.reset();
        assert!(s.update(30_000, HIGH).is_empty());
    }
}
