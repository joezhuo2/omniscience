use std::path::Path;

use super::types::CoreMetrics;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct UsageTotals {
    pub data_today_bytes: u64,
    pub data_month_bytes: u64,
    pub energy_today_wh: Option<f32>,
}

#[derive(Debug, Default)]
pub struct Usage;

impl Usage {
    pub fn open(_dir: Option<&Path>) -> Self {
        Self
    }

    pub fn record(&mut self, _now_ms: u64, _core: &CoreMetrics) {}

    pub fn totals(&self, _now_ms: u64) -> UsageTotals {
        UsageTotals::default()
    }
}
