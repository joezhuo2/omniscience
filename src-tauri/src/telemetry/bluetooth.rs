use std::sync::atomic::AtomicBool;

use super::types::BluetoothBattery;

#[cfg(any(windows, test))]
fn battery(name: String, percent: u32) -> Option<BluetoothBattery> {
    (!name.is_empty()).then(|| BluetoothBattery {
        name,
        percent: percent.min(100) as u8,
    })
}

/// Battery levels of present Bluetooth devices, read from their device properties.
#[cfg(windows)]
pub fn batteries(_stop: &AtomicBool) -> Option<Vec<BluetoothBattery>> {
    use windows_sys::Win32::Devices::DeviceAndDriverInstallation::{
        DIGCF_PRESENT, GUID_DEVCLASS_BLUETOOTH, SP_DEVINFO_DATA, SPDRP_FRIENDLYNAME,
        SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW,
        SetupDiGetDevicePropertyW, SetupDiGetDeviceRegistryPropertyW,
    };
    use windows_sys::Win32::Devices::Properties::{DEVPROP_TYPE_BYTE, DEVPROP_TYPE_UINT32};
    use windows_sys::Win32::Foundation::DEVPROPKEY;
    use windows_sys::core::GUID;

    const BATTERY_LEVEL: DEVPROPKEY = DEVPROPKEY {
        fmtid: GUID::from_u128(0x104ea319_6ee2_4701_bd47_8ddbf425bbe5),
        pid: 2,
    };
    let mut out = Vec::new();
    // SAFETY: the device set is destroyed before returning; every buffer is a local whose size is
    // passed with it.
    unsafe {
        let set = SetupDiGetClassDevsW(
            &GUID_DEVCLASS_BLUETOOTH,
            std::ptr::null(),
            std::ptr::null_mut(),
            DIGCF_PRESENT,
        );
        if set == -1 {
            return None;
        }
        let mut info: SP_DEVINFO_DATA = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<SP_DEVINFO_DATA>() as u32;
        let mut i = 0;
        while SetupDiEnumDeviceInfo(set, i, &mut info) != 0 {
            i += 1;
            let (mut kind, mut value) = (0u32, [0u8; 4]);
            if SetupDiGetDevicePropertyW(
                set,
                &info,
                &BATTERY_LEVEL,
                &mut kind,
                value.as_mut_ptr(),
                value.len() as u32,
                std::ptr::null_mut(),
                0,
            ) == 0
            {
                continue;
            }
            let percent = match kind {
                DEVPROP_TYPE_BYTE => value[0] as u32,
                DEVPROP_TYPE_UINT32 => u32::from_le_bytes(value),
                _ => continue,
            };
            let mut name = [0u16; 256];
            let got = SetupDiGetDeviceRegistryPropertyW(
                set,
                &info,
                SPDRP_FRIENDLYNAME,
                std::ptr::null_mut(),
                name.as_mut_ptr().cast(),
                std::mem::size_of_val(&name) as u32,
                std::ptr::null_mut(),
            ) != 0;
            let len = name.iter().position(|&c| c == 0).unwrap_or(name.len());
            let name = if got {
                String::from_utf16_lossy(&name[..len]).trim().to_string()
            } else {
                String::new()
            };
            out.extend(battery(name, percent));
        }
        SetupDiDestroyDeviceInfoList(set);
    }
    Some(out)
}

#[cfg(not(windows))]
pub fn batteries(_stop: &AtomicBool) -> Option<Vec<BluetoothBattery>> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_percent_and_needs_a_name() {
        assert_eq!(battery("Pad".into(), 250).map(|b| b.percent), Some(100));
        assert_eq!(battery(String::new(), 50), None);
    }

    #[cfg(windows)]
    #[test]
    fn reads_without_crashing() {
        assert!(batteries(&AtomicBool::new(false)).is_some());
    }
}
