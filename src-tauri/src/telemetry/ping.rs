//! Round-trip latency to a user-chosen host, measured on its own thread so a slow or lost reply
//! never delays a telemetry tick. ICMP echo through `IcmpSendEcho` needs no admin rights.

use std::collections::VecDeque;
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

const WINDOW: usize = 30;

/// Mean absolute difference between consecutive successful round trips; `None` under two.
pub fn jitter_ms(results: &VecDeque<Option<f32>>) -> Option<f32> {
    let ok: Vec<f32> = results.iter().flatten().copied().collect();
    (ok.len() >= 2)
        .then(|| ok.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f32>() / (ok.len() - 1) as f32)
}

/// Lost replies as a percentage of all results; `None` before any result.
pub fn loss_pct(results: &VecDeque<Option<f32>>) -> Option<f32> {
    (!results.is_empty()).then(|| {
        results.iter().filter(|r| r.is_none()).count() as f32 / results.len() as f32 * 100.0
    })
}

/// Pings `target` every two seconds until dropped.
pub struct Pinger {
    target: Arc<str>,
    results: Arc<Mutex<VecDeque<Option<f32>>>>,
    stop: Arc<AtomicBool>,
}

impl Pinger {
    pub fn start(target: Arc<str>) -> Self {
        let latest = Arc::new(Mutex::new(VecDeque::with_capacity(WINDOW)));
        let stop = Arc::new(AtomicBool::new(false));
        let (host, out, done) = (target.clone(), latest.clone(), stop.clone());
        let spawned = std::thread::Builder::new()
            .name("ping".into())
            .spawn(move || {
                while !done.load(Ordering::SeqCst) {
                    let ms = resolve(&host).and_then(|a| ping(a, TIMEOUT_MS));
                    let mut r = out.lock().unwrap_or_else(|e| e.into_inner());
                    if r.len() == WINDOW {
                        r.pop_front();
                    }
                    r.push_back(ms);
                    drop(r);
                    std::thread::sleep(EVERY);
                }
            });
        if let Err(e) = spawned {
            crate::diagnostics::log_error(format_args!("spawn ping thread: {e}"));
        }
        Self {
            target,
            results: latest,
            stop,
        }
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    /// The last round trip in milliseconds, `None` before the first reply or after a lost one.
    pub fn latest(&self) -> Option<f32> {
        self.results
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .back()
            .copied()
            .flatten()
    }

    /// Jitter and loss over the last 30 results.
    pub fn quality(&self) -> (Option<f32>, Option<f32>) {
        let r = self.results.lock().unwrap_or_else(|e| e.into_inner());
        (jitter_ms(&r), loss_pct(&r))
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

    fn window(v: &[Option<f32>]) -> VecDeque<Option<f32>> {
        v.iter().copied().collect()
    }

    #[test]
    fn jitter_is_mean_step_between_successes() {
        assert_eq!(jitter_ms(&window(&[])), None);
        assert_eq!(jitter_ms(&window(&[Some(10.0)])), None);
        assert_eq!(jitter_ms(&window(&[None, Some(10.0), None])), None);
        assert_eq!(
            jitter_ms(&window(&[Some(10.0), Some(14.0), None, Some(12.0)])),
            Some(3.0)
        );
    }

    #[test]
    fn loss_is_lost_over_total() {
        assert_eq!(loss_pct(&window(&[])), None);
        assert_eq!(loss_pct(&window(&[Some(1.0), Some(2.0)])), Some(0.0));
        assert_eq!(
            loss_pct(&window(&[None, Some(2.0), None, None])),
            Some(75.0)
        );
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
