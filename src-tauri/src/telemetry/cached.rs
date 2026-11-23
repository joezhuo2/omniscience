//! A value refreshed on a background thread only while wanted, and kept between uses.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

type Slot<T> = Arc<Mutex<(Option<T>, Option<Instant>)>>;

pub struct Cached<T> {
    slot: Slot<T>,
    every: Duration,
    read: fn(&AtomicBool) -> Option<T>,
    /// Stop flag of the running worker; `None` while inactive.
    worker: Option<Arc<AtomicBool>>,
}

impl<T: Clone + Send + 'static> Cached<T> {
    pub fn new(every: Duration, read: fn(&AtomicBool) -> Option<T>) -> Self {
        Self {
            slot: Arc::new(Mutex::new((None, None))),
            every,
            read,
            worker: None,
        }
    }

    /// Starts or stops refreshing. A restart reads at once only if the value is older than `every`.
    pub fn set_active(&mut self, active: bool) {
        if active == self.worker.is_some() {
            return;
        }
        if let Some(stop) = self.worker.take() {
            stop.store(true, Ordering::Relaxed);
            return;
        }
        let stop = Arc::new(AtomicBool::new(false));
        let (slot, every, read, done) = (self.slot.clone(), self.every, self.read, stop.clone());
        let spawned = std::thread::Builder::new()
            .name("cached-probe".into())
            .spawn(move || {
                while !done.load(Ordering::Relaxed) {
                    let last = slot.lock().unwrap_or_else(|e| e.into_inner()).1;
                    let wait = last.map_or(Duration::ZERO, |t| every.saturating_sub(t.elapsed()));
                    let until = Instant::now() + wait;
                    while Instant::now() < until {
                        if done.load(Ordering::Relaxed) {
                            return;
                        }
                        std::thread::sleep(Duration::from_millis(200).min(until - Instant::now()));
                    }
                    if done.load(Ordering::Relaxed) {
                        return;
                    }
                    let value = read(&done);
                    let mut guard = slot.lock().unwrap_or_else(|e| e.into_inner());
                    if value.is_some() {
                        guard.0 = value;
                    }
                    // Failures also wait a full period, so a broken source is not retried hot.
                    guard.1 = Some(Instant::now());
                }
            });
        match spawned {
            Ok(_) => self.worker = Some(stop),
            Err(e) => crate::diagnostics::log_error(format_args!("spawn probe thread: {e}")),
        }
    }

    pub fn latest(&self) -> Option<T> {
        self.slot
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .0
            .clone()
    }
}

impl<T> Drop for Cached<T> {
    fn drop(&mut self) {
        if let Some(stop) = &self.worker {
            stop.store(true, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    static CALLS: AtomicU32 = AtomicU32::new(0);

    fn count(_: &AtomicBool) -> Option<u32> {
        Some(CALLS.fetch_add(1, Ordering::SeqCst) + 1)
    }

    fn wait_for(c: &Cached<u32>, want: Option<u32>) {
        let start = Instant::now();
        while c.latest() != want && start.elapsed() < Duration::from_secs(5) {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(c.latest(), want);
    }

    #[test]
    fn reads_only_while_active_and_keeps_the_value() {
        let mut c = Cached::new(Duration::from_secs(60), count);
        assert_eq!(c.latest(), None);
        c.set_active(true);
        wait_for(&c, Some(1));
        c.set_active(false);
        c.set_active(true);
        std::thread::sleep(Duration::from_millis(300));
        assert_eq!(c.latest(), Some(1), "fresh value is reused, not re-read");
        assert_eq!(CALLS.load(Ordering::SeqCst), 1);
        c.set_active(false);
    }
}
