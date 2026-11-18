use std::ffi::c_void;

use windows_sys::Win32::Foundation::{FreeLibrary, HMODULE};
use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

const CLOCK_GRAPHICS: u32 = 0;
const CLOCK_MEM: u32 = 2;

type Init = unsafe extern "C" fn() -> i32;
type Shutdown = unsafe extern "C" fn() -> i32;
type ByIndex = unsafe extern "C" fn(u32, *mut *mut c_void) -> i32;
type ClockInfo = unsafe extern "C" fn(*mut c_void, u32, *mut u32) -> i32;

pub struct Nvml {
    lib: HMODULE,
    shutdown: Shutdown,
    by_index: ByIndex,
    clock_info: ClockInfo,
}

unsafe impl Send for Nvml {}

impl Nvml {
    pub fn load() -> Option<Self> {
        let name: Vec<u16> = "nvml.dll\0".encode_utf16().collect();
        // SAFETY: each symbol is cast to its documented NVML signature; the library is freed on
        // every failure path and in Drop.
        unsafe {
            let lib = LoadLibraryW(name.as_ptr());
            if lib.is_null() {
                return None;
            }
            let sym = |n: &[u8]| GetProcAddress(lib, n.as_ptr());
            let loaded = (|| {
                let init: Init = std::mem::transmute(sym(b"nvmlInit_v2\0")?);
                let shutdown: Shutdown = std::mem::transmute(sym(b"nvmlShutdown\0")?);
                let by_index: ByIndex =
                    std::mem::transmute(sym(b"nvmlDeviceGetHandleByIndex_v2\0")?);
                let clock_info: ClockInfo = std::mem::transmute(sym(b"nvmlDeviceGetClockInfo\0")?);
                (init() == 0).then_some((shutdown, by_index, clock_info))
            })();
            match loaded {
                Some((shutdown, by_index, clock_info)) => Some(Self {
                    lib,
                    shutdown,
                    by_index,
                    clock_info,
                }),
                None => {
                    FreeLibrary(lib);
                    None
                }
            }
        }
    }

    /// Graphics and memory clocks in MHz of the NVML device at `index`.
    pub fn clocks(&self, index: u32) -> (Option<u32>, Option<u32>) {
        // SAFETY: the handle comes from NVML and is used immediately; out-pointers are locals.
        unsafe {
            let mut dev: *mut c_void = std::ptr::null_mut();
            if (self.by_index)(index, &mut dev) != 0 {
                return (None, None);
            }
            let read = |kind: u32| {
                let mut mhz = 0u32;
                ((self.clock_info)(dev, kind, &mut mhz) == 0).then_some(mhz)
            };
            (read(CLOCK_GRAPHICS), read(CLOCK_MEM))
        }
    }
}

impl Drop for Nvml {
    fn drop(&mut self) {
        // SAFETY: NVML was initialised in load and the library is freed once.
        unsafe {
            (self.shutdown)();
            FreeLibrary(self.lib);
        }
    }
}
