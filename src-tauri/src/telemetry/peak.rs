//! Rolling peak used to scale the network rings.

use std::collections::VecDeque;

/// Highest value over the last `window` samples, never below `floor`.
#[derive(Debug, Clone)]
pub struct RollingPeak {
    window: usize,
    floor: f64,
    samples: VecDeque<f64>,
}

impl RollingPeak {
    pub fn new(window: usize, floor: f64) -> Self {
        Self {
            window: window.max(1),
            floor,
            samples: VecDeque::with_capacity(window.max(1)),
        }
    }

    /// Records `value` and returns it as a percent (0-100) of the current peak.
    pub fn push(&mut self, value: f64) -> f32 {
        if self.samples.len() == self.window {
            self.samples.pop_front();
        }
        let value = if value.is_finite() { value.max(0.0) } else { 0.0 };
        self.samples.push_back(value);
        (value / self.peak() * 100.0).clamp(0.0, 100.0) as f32
    }

    pub fn peak(&self) -> f64 {
        self.samples.iter().copied().fold(self.floor, f64::max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MB: f64 = 1_000_000.0;

    #[test]
    fn floor_applies_when_quiet() {
        let mut p = RollingPeak::new(60, MB);
        assert_eq!(p.push(250_000.0), 25.0);
    }

    #[test]
    fn new_peak_reads_full() {
        let mut p = RollingPeak::new(60, MB);
        p.push(2.0 * MB);
        assert_eq!(p.push(4.0 * MB), 100.0);
        assert_eq!(p.push(2.0 * MB), 50.0);
    }

    #[test]
    fn old_peak_expires_after_window() {
        let mut p = RollingPeak::new(3, MB);
        p.push(10.0 * MB);
        p.push(2.0 * MB);
        p.push(2.0 * MB);
        // The 10 MB sample falls out now.
        assert_eq!(p.push(2.0 * MB), 100.0);
    }

    #[test]
    fn ignores_negative_and_nan() {
        let mut p = RollingPeak::new(5, MB);
        assert_eq!(p.push(-5.0), 0.0);
        assert_eq!(p.push(f64::NAN), 0.0);
        assert_eq!(p.peak(), MB);
    }
}
