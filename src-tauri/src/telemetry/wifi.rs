use std::ffi::c_void;

use windows_sys::Win32::NetworkManagement::WiFi::{
    WLAN_CONNECTION_ATTRIBUTES, WLAN_INTERFACE_INFO_LIST, WlanCloseHandle, WlanEnumInterfaces,
    WlanFreeMemory, WlanOpenHandle, WlanQueryInterface, wlan_interface_state_connected,
    wlan_intf_opcode_current_connection,
};

use super::parse;
use super::types::WifiInfo;

pub fn current() -> Option<WifiInfo> {
    // SAFETY: the client handle is closed and every WLAN allocation freed before returning; the
    // interface list and attributes are only read while alive and within their reported counts.
    unsafe {
        let (mut version, mut handle) = (0u32, std::ptr::null_mut());
        if WlanOpenHandle(2, std::ptr::null(), &mut version, &mut handle) != 0 {
            return None;
        }
        let mut list: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
        let mut found = None;
        if WlanEnumInterfaces(handle, std::ptr::null(), &mut list) == 0 && !list.is_null() {
            let infos = std::slice::from_raw_parts(
                (*list).InterfaceInfo.as_ptr(),
                (*list).dwNumberOfItems as usize,
            );
            for info in infos {
                if info.isState != wlan_interface_state_connected {
                    continue;
                }
                let mut size = 0u32;
                let mut data: *mut c_void = std::ptr::null_mut();
                let r = WlanQueryInterface(
                    handle,
                    &info.InterfaceGuid,
                    wlan_intf_opcode_current_connection,
                    std::ptr::null(),
                    &mut size,
                    &mut data,
                    std::ptr::null_mut(),
                );
                if r == 0 && !data.is_null() {
                    let a = &(*data.cast::<WLAN_CONNECTION_ATTRIBUTES>()).wlanAssociationAttributes;
                    found = Some(WifiInfo {
                        ssid: parse::ssid_text(&a.dot11Ssid.ucSSID, a.dot11Ssid.uSSIDLength),
                        signal_pct: a.wlanSignalQuality.min(100) as u8,
                        link_mbps: parse::wifi_link_mbps(a.ulRxRate, a.ulTxRate),
                    });
                }
                if !data.is_null() {
                    WlanFreeMemory(data);
                }
                if found.is_some() {
                    break;
                }
            }
        }
        if !list.is_null() {
            WlanFreeMemory(list.cast());
        }
        WlanCloseHandle(handle, std::ptr::null());
        found
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn queries_without_crashing() {
        let _ = super::current();
    }
}
