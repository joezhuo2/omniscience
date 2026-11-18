#[cfg(any(windows, test))]
fn hex_from_abgr(value: u32) -> String {
    let [r, g, b, _] = value.to_le_bytes();
    format!("#{r:02x}{g:02x}{b:02x}")
}

#[cfg(windows)]
fn read() -> Option<u32> {
    use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};

    let key: Vec<u16> = r"Software\Microsoft\Windows\DWM"
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let value: Vec<u16> = "AccentColor\0".encode_utf16().collect();
    let mut data = 0u32;
    let mut size = size_of::<u32>() as u32;
    // SAFETY: key and value are NUL-terminated; data is a DWORD-sized buffer matching size.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            (&mut data as *mut u32).cast(),
            &mut size,
        )
    };
    (status == 0).then_some(data)
}

#[tauri::command]
pub fn get_accent_color() -> Option<String> {
    #[cfg(windows)]
    return read().map(hex_from_abgr);
    #[cfg(not(windows))]
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_abgr_to_rgb_hex() {
        assert_eq!(hex_from_abgr(0xFFD77800), "#0078d7");
        assert_eq!(hex_from_abgr(0xFF0A1B2C), "#2c1b0a");
        assert_eq!(hex_from_abgr(0), "#000000");
    }

    #[test]
    #[ignore]
    fn accent_live() {
        println!("{:?}", get_accent_color());
    }
}
