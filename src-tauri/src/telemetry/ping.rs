//! Round-trip latency to a user-chosen host, measured on its own thread so a slow or lost reply
//! never delays a telemetry tick. ICMP echo through `IcmpSendEcho` needs no admin rights.

use std::net::{Ipv4Addr, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const EVERY: Duration = Duration::from_secs(2);
const TIMEOUT_MS: u32 = 1000;

/// First IPv4 address of `host` (a name or a literal address).
pub fn resolve(host: &str) -> Option<Ipv4Addr> {
    let host = host.trim();
    if host.is_empty() {
        return None;
    }
    (host, 0)
        .to_socket_addrs()
        .ok()?
        .find_map(|a| match a.ip() {
            std::net::IpAddr::V4(v4) => Some(v4),
            std::net::IpAddr::V6(_) => None,
        })
}

/// One echo round trip in milliseconds, or `None` on timeout or error.
#[cfg(windows)]
pub fn ping(addr: Ipv4Addr, timeout_ms: u32) -> Option<f32> {
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        ICMP_ECHO_REPLY, IcmpCloseHandle, IcmpCreateFile, IcmpSendEcho,
    };
    let payload = [0u8; 32];
    // Room for the reply header, the echoed payload and the 8-byte ICMP error slack.
    let mut replies = [ICMP_ECHO_REPLY::default(); 3];
    // SAFETY: the handle is closed before returning; the buffers outlive the call and their sizes
    // are passed alongside them.
    unsafe {
        let handle = IcmpCreateFile();
        if handle == INVALID_HANDLE_VALUE {
            return None;
        }
        let n = IcmpSendEcho(
            handle,
            u32::from_ne_bytes(addr.octets()),
            payload.as_ptr().cast(),
            payload.len() as u16,
            std::ptr::null(),
            replies.as_mut_ptr().cast(),
            std::mem::size_of_val(&replies) as u32,
            timeout_ms,
        );
        IcmpCloseHandle(handle);
        (n > 0 && replies[0].Status == 0).then_some(replies[0].RoundTripTime as f32)
    }
}

#[cfg(not(windows))]
pub fn ping(_addr: Ipv4Addr, _timeout_ms: u32) -> Option<f32> {
    None
}

/// Pings `target` every two seconds until dropped.
pub struct Pinger {
    target: Arc<str>,
    latest: Arc<Mutex<Option<f32>>>,
    stop: Arc<AtomicBool>,
}

impl Pinger {
    pub fn start(target: Arc<str>) -> Self {
        let latest = Arc::new(Mutex::new(None));
        let stop = Arc::new(AtomicBool::new(false));
        let (host, out, done) = (target.clone(), latest.clone(), stop.clone());
        let spawned = std::thread::Builder::new()
            .name("ping".into())
            .spawn(move || {
                while !done.load(Ordering::SeqCst) {
                    let ms = resolve(&host).and_then(|a| ping(a, TIMEOUT_MS));
                    *out.lock().unwrap_or_else(|e| e.into_inner()) = ms;
                    std::thread::sleep(EVERY);
                }
            });
        if let Err(e) = spawned {
            eprintln!("spawn ping thread: {e}");
        }
        Self {
            target,
            latest,
            stop,
        }
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    /// The last round trip in milliseconds, `None` before the first reply or after a lost one.
    pub fn latest(&self) -> Option<f32> {
        *self.latest.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl Drop for Pinger {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_literal_addresses() {
        assert_eq!(resolve("127.0.0.1"), Some(Ipv4Addr::LOCALHOST));
        assert_eq!(resolve(" 1.1.1.1 "), Some(Ipv4Addr::new(1, 1, 1, 1)));
    }

    #[test]
    fn rejects_an_empty_host() {
        assert_eq!(resolve(""), None);
        assert_eq!(resolve("  "), None);
    }

    #[cfg(windows)]
    #[test]
    fn pings_localhost() {
        assert!(ping(Ipv4Addr::LOCALHOST, TIMEOUT_MS).is_some());
    }
}
