//! ASUS ROG firmware endpoints via the installed ASUS System Control Interface.
//! The protocol identifiers are documented in Linux's asus-wmi.h:
//! https://github.com/torvalds/linux/blob/master/include/linux/platform_data/x86/asus-wmi.h
//! No kernel driver is installed by Omniscience.
use windows_sys::Win32::Foundation::{
    CloseHandle, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows_sys::Win32::System::IO::DeviceIoControl;

pub const GPU_ECO: u32 = 0x0009_0020;
pub const GPU_MUX: u32 = 0x0009_0016;

pub struct Asus(HANDLE);

impl Asus {
    pub fn open() -> Result<Self, String> {
        let name: Vec<u16> = r"\\.\ATKACPI".encode_utf16().chain(Some(0)).collect();
        // SAFETY: name is a terminated UTF-16 device path; no pointers are retained.
        let handle = unsafe {
            CreateFileW(
                name.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                std::ptr::null(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                std::ptr::null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(format!(
                "ASUS System Control Interface unavailable: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(Self(handle))
    }

    fn request(&self, method: u32, device: u32, value: u32) -> Result<u32, String> {
        let input = [method, 8, device, value];
        let mut output = [0u32; 4];
        let mut returned = 0;
        // SAFETY: input/output have the exact byte capacities passed to the synchronous ioctl.
        let ok = unsafe {
            DeviceIoControl(
                self.0,
                0x0022_240c,
                input.as_ptr().cast(),
                std::mem::size_of_val(&input) as u32,
                output.as_mut_ptr().cast(),
                std::mem::size_of_val(&output) as u32,
                &mut returned,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 || returned < 4 {
            return Err(format!(
                "ASUS firmware request failed: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(output[0])
    }

    pub fn get(&self, device: u32) -> Result<u32, String> {
        let raw = self.request(0x5354_5344, device, 0)?;
        if raw & 0xffff_0000 != 0x0001_0000 {
            return Err("This control is not supported by the ASUS firmware.".into());
        }
        Ok(raw & 0xffff)
    }

    pub fn set(&self, device: u32, value: u32) -> Result<(), String> {
        self.get(device)?;
        if self.request(0x5356_4544, device, value)? != 1 {
            return Err("The ASUS firmware refused this setting. Close GPU apps or check manufacturer controls.".into());
        }
        Ok(())
    }
}

impl Drop for Asus {
    fn drop(&mut self) {
        // SAFETY: this owns the handle created by open and closes it exactly once.
        unsafe {
            CloseHandle(self.0);
        }
    }
}
