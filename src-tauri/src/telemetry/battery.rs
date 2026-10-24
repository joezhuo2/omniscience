//! Battery charge, drain rate and time estimates for laptops.

use super::types::{Battery, BatteryState};

/// What the OS and the battery driver report, before any estimate.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Reading {
    pub percent: Option<f32>,
    pub on_ac: bool,
    pub charging: bool,
    /// Windows' own time-left estimate, when it has one.
    pub os_secs_left: Option<u64>,
    /// Remaining and full-charge capacity in mWh, summed over batteries.
    pub capacity_mwh: Option<u32>,
    pub full_mwh: Option<u32>,
    /// Charge (+) or discharge (-) rate in mW.
    pub rate_mw: Option<i32>,
}

/// Turns a reading into what the dashboard shows. `None` without a battery.
pub fn estimate(r: &Reading) -> Option<Battery> {
    let percent = r.percent?;
    let state = if r.charging {
        BatteryState::Charging
    } else if r.on_ac {
        BatteryState::Idle
    } else {
        BatteryState::Discharging
    };
    let rate = r.rate_mw.filter(|mw| *mw != 0).map(i32::unsigned_abs);
    let secs = |mwh: u32| rate.map(|mw| u64::from(mwh) * 3600 / u64::from(mw));
    let secs_remaining = match state {
        BatteryState::Discharging => r.capacity_mwh.and_then(secs).or(r.os_secs_left),
        BatteryState::Charging => r
            .full_mwh
            .zip(r.capacity_mwh)
            .and_then(|(full, now)| secs(full.saturating_sub(now))),
        BatteryState::Idle => None,
    };
    Some(Battery {
        percent: percent.clamp(0.0, 100.0),
        state,
        watts: rate.map(|mw| (mw as f32 / 100.0).round() / 10.0),
        secs_remaining,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DRAINING: Reading = Reading {
        percent: Some(80.0),
        on_ac: false,
        charging: false,
        os_secs_left: Some(9000),
        capacity_mwh: Some(40_000),
        full_mwh: Some(50_000),
        rate_mw: Some(-10_000),
    };

    #[test]
    fn no_battery_no_metric() {
        assert_eq!(estimate(&Reading::default()), None);
    }

    #[test]
    fn time_left_from_capacity_and_drain() {
        let b = estimate(&DRAINING).unwrap();
        assert_eq!(b.state, BatteryState::Discharging);
        assert_eq!(b.watts, Some(10.0));
        assert_eq!(b.secs_remaining, Some(4 * 3600));
    }

    #[test]
    fn falls_back_to_the_os_estimate_without_a_rate() {
        let r = Reading {
            rate_mw: None,
            ..DRAINING
        };
        assert_eq!(estimate(&r).unwrap().secs_remaining, Some(9000));
    }

    #[test]
    fn time_to_full_while_charging() {
        let r = Reading {
            on_ac: true,
            charging: true,
            rate_mw: Some(20_000),
            ..DRAINING
        };
        let b = estimate(&r).unwrap();
        assert_eq!(b.state, BatteryState::Charging);
        assert_eq!(b.secs_remaining, Some(1800));
    }

    #[test]
    fn plugged_in_and_not_charging_is_idle() {
        let r = Reading {
            on_ac: true,
            rate_mw: Some(0),
            ..DRAINING
        };
        let b = estimate(&r).unwrap();
        assert_eq!(b.state, BatteryState::Idle);
        assert_eq!((b.watts, b.secs_remaining), (None, None));
    }
}
