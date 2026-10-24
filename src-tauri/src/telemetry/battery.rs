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

/// Charge, AC and Windows' time estimate from `GetSystemPowerStatus`.
#[cfg(windows)]
pub fn read() -> Reading {
    use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
    const NO_BATTERY: u8 = 128;
    const CHARGING: u8 = 8;
    const UNKNOWN: u8 = 255;
    let mut s = SYSTEM_POWER_STATUS::default();
    // SAFETY: plain query into a valid out pointer.
    if unsafe { GetSystemPowerStatus(&mut s) } == 0
        || s.BatteryFlag == UNKNOWN
        || s.BatteryFlag & NO_BATTERY != 0
        || s.BatteryLifePercent == UNKNOWN
    {
        return Reading::default();
    }
    let (capacity_mwh, full_mwh, rate_mw) = driver_status();
    Reading {
        percent: Some(f32::from(s.BatteryLifePercent)),
        on_ac: s.ACLineStatus == 1,
        charging: s.BatteryFlag & CHARGING != 0,
        os_secs_left: (s.BatteryLifeTime != u32::MAX).then_some(u64::from(s.BatteryLifeTime)),
        capacity_mwh,
        full_mwh,
        rate_mw,
    }
}

/// Remaining capacity, full-charge capacity (mWh) and rate (mW), each summed over the batteries
/// that report it, from the battery class driver's IOCTLs. No admin rights needed.
#[cfg(windows)]
fn driver_status() -> (Option<u32>, Option<u32>, Option<i32>) {
    use windows_sys::Win32::Devices::DeviceAndDriverInstallation::{
        DIGCF_DEVICEINTERFACE, DIGCF_PRESENT, GUID_DEVCLASS_BATTERY, SP_DEVICE_INTERFACE_DATA,
        SP_DEVICE_INTERFACE_DETAIL_DATA_W, SetupDiDestroyDeviceInfoList,
        SetupDiEnumDeviceInterfaces, SetupDiGetClassDevsW, SetupDiGetDeviceInterfaceDetailW,
    };
    use windows_sys::Win32::Foundation::{
        CloseHandle, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    };
    use windows_sys::Win32::System::IO::DeviceIoControl;
    use windows_sys::Win32::System::Power::{
        BATTERY_CAPACITY_RELATIVE, BATTERY_INFORMATION, BATTERY_QUERY_INFORMATION, BATTERY_STATUS,
        BATTERY_UNKNOWN_CAPACITY, BATTERY_UNKNOWN_RATE, BATTERY_WAIT_STATUS, BatteryInformation,
        IOCTL_BATTERY_QUERY_INFORMATION, IOCTL_BATTERY_QUERY_STATUS, IOCTL_BATTERY_QUERY_TAG,
    };

    /// Sends one IOCTL with a fixed-size input and output.
    unsafe fn ioctl<I, O>(h: HANDLE, code: u32, input: &I, output: &mut O) -> bool {
        let mut returned = 0;
        // SAFETY: both buffers are live references of the sizes passed.
        unsafe {
            DeviceIoControl(
                h,
                code,
                (input as *const I).cast(),
                std::mem::size_of::<I>() as u32,
                (output as *mut O).cast(),
                std::mem::size_of::<O>() as u32,
                &mut returned,
                std::ptr::null_mut(),
            ) != 0
        }
    }

    let (mut capacity, mut full, mut rate) = (None::<u32>, None::<u32>, None::<i32>);
    // SAFETY: the device info set and every opened handle are released before returning; the
    // interface detail buffer is u64-aligned and as large as SetupDi asked for.
    unsafe {
        let set = SetupDiGetClassDevsW(
            &GUID_DEVCLASS_BATTERY,
            std::ptr::null(),
            std::ptr::null_mut(),
            DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
        );
        if set == -1 {
            return (None, None, None);
        }
        for index in 0..8 {
            let mut iface = SP_DEVICE_INTERFACE_DATA {
                cbSize: std::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as u32,
                ..Default::default()
            };
            if SetupDiEnumDeviceInterfaces(
                set,
                std::ptr::null(),
                &GUID_DEVCLASS_BATTERY,
                index,
                &mut iface,
            ) == 0
            {
                break;
            }
            let mut needed = 0;
            SetupDiGetDeviceInterfaceDetailW(
                set,
                &iface,
                std::ptr::null_mut(),
                0,
                &mut needed,
                std::ptr::null_mut(),
            );
            let mut buf = vec![0u64; (needed as usize).div_ceil(8).max(1)];
            let detail = buf.as_mut_ptr().cast::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>();
            (*detail).cbSize = std::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as u32;
            if SetupDiGetDeviceInterfaceDetailW(
                set,
                &iface,
                detail,
                needed,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ) == 0
            {
                continue;
            }
            let h = CreateFileW(
                (*detail).DevicePath.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                std::ptr::null(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                std::ptr::null_mut(),
            );
            if h == INVALID_HANDLE_VALUE {
                continue;
            }
            let mut tag = 0u32;
            let mut info = BATTERY_INFORMATION::default();
            let mut status = BATTERY_STATUS::default();
            let ok = ioctl(h, IOCTL_BATTERY_QUERY_TAG, &0u32, &mut tag)
                && tag != 0
                && ioctl(
                    h,
                    IOCTL_BATTERY_QUERY_INFORMATION,
                    &BATTERY_QUERY_INFORMATION {
                        BatteryTag: tag,
                        InformationLevel: BatteryInformation,
                        AtRate: 0,
                    },
                    &mut info,
                )
                && ioctl(
                    h,
                    IOCTL_BATTERY_QUERY_STATUS,
                    &BATTERY_WAIT_STATUS {
                        BatteryTag: tag,
                        ..Default::default()
                    },
                    &mut status,
                );
            CloseHandle(h);
            // Relative batteries report percentages, not mWh and mW.
            if !ok || info.Capabilities & BATTERY_CAPACITY_RELATIVE != 0 {
                continue;
            }
            if status.Capacity != BATTERY_UNKNOWN_CAPACITY {
                *capacity.get_or_insert(0) += status.Capacity;
                *full.get_or_insert(0) += info.FullChargedCapacity;
            }
            if status.Rate != BATTERY_UNKNOWN_RATE as i32 {
                *rate.get_or_insert(0) += status.Rate;
            }
        }
        SetupDiDestroyDeviceInfoList(set);
    }
    (capacity, full, rate)
}

#[cfg(not(windows))]
pub fn read() -> Reading {
    Reading::default()
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

    /// Machine-dependent: `cargo test battery_live -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn battery_live() {
        let r = read();
        println!("{r:?}
{:?}", estimate(&r));
    }
}
