//! Per-process physical disk and TCP/UDP bytes from a private, real-time ETW session.
//! Only alive in the detail tier. Access denied means unavailable, never invented zeroes.

use std::collections::HashMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::Instant;

use ferrisetw::{
    EventRecord,
    parser::Parser,
    provider::{
        Provider,
        kernel_providers::{self, KernelProvider},
    },
    schema_locator::SchemaLocator,
    trace::{KernelTrace, TraceTrait},
};
use windows_sys::Win32::{
    Foundation::CloseHandle,
    System::Threading::{GetProcessIdOfThread, OpenThread, THREAD_QUERY_LIMITED_INFORMATION},
};

#[derive(Default)]
struct Counts {
    // Bytes since the previous sample; drained every tick to bound memory.
    bytes: HashMap<u32, (u64, u64)>,
    threads: HashMap<u32, u32>,
}

impl Counts {
    fn add(&mut self, pid: u32, disk: u64, net: u64) {
        if pid == 0 || self.bytes.len() >= 32768 && !self.bytes.contains_key(&pid) {
            return;
        }
        let count = self.bytes.entry(pid).or_default();
        count.0 = count.0.saturating_add(disk);
        count.1 = count.1.saturating_add(net);
    }
}

pub struct ProcessIo {
    trace: Option<KernelTrace>,
    worker: Option<std::thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
    counts: Arc<Mutex<Counts>>,
    last: Instant,
}

impl ProcessIo {
    pub fn start() -> Option<Self> {
        static SERIAL: AtomicU64 = AtomicU64::new(0);
        let counts = Arc::new(Mutex::new(Counts::default()));
        let net_counts = counts.clone();
        let network = move |record: &EventRecord, schemas: &SchemaLocator| {
            // Send/receive IPv4 and IPv6, excluding reconnect/retransmit/control events.
            if !matches!(record.opcode(), 10 | 11 | 26 | 27) {
                return;
            }
            let Ok(schema) = schemas.event_schema(record) else {
                return;
            };
            let parser = Parser::create(record, &schema);
            if let (Ok(pid), Ok(bytes)) = (
                parser.try_parse::<u32>("PID"),
                parser.try_parse::<u32>("size"),
            ) {
                net_counts
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .add(pid, 0, bytes.into());
            }
        };
        let tcp = Provider::kernel(&kernel_providers::TCP_IP_PROVIDER)
            .add_callback(network.clone())
            .build();
        let udp_provider = KernelProvider::new(
            "bf3a50c5-a9c9-4988-a005-2df0b7c80f80".into(),
            kernel_providers::TCP_IP_PROVIDER.flags,
        );
        let udp = Provider::kernel(&udp_provider)
            .add_callback(network)
            .build();
        let thread_counts = counts.clone();
        let threads = Provider::kernel(&kernel_providers::THREAD_PROVIDER)
            .add_callback(move |record: &EventRecord, schemas: &SchemaLocator| {
                if !matches!(record.opcode(), 1..=4) {
                    return;
                }
                let Ok(schema) = schemas.event_schema(record) else {
                    return;
                };
                let parser = Parser::create(record, &schema);
                if let (Ok(pid), Ok(tid)) = (
                    parser.try_parse::<u32>("ProcessId"),
                    parser.try_parse::<u32>("TThreadId"),
                ) {
                    let mut state = thread_counts.lock().unwrap_or_else(|e| e.into_inner());
                    if matches!(record.opcode(), 1 | 3) {
                        if state.threads.len() >= 65536 {
                            state.threads.clear();
                        }
                        state.threads.insert(tid, pid);
                    } else {
                        state.threads.remove(&tid);
                    }
                }
            })
            .build();
        let disk_counts = counts.clone();
        let disk = Provider::kernel(&kernel_providers::DISK_IO_PROVIDER)
            .add_callback(move |record: &EventRecord, schemas: &SchemaLocator| {
                if !matches!(record.opcode(), 10 | 11) {
                    return;
                }
                let Ok(schema) = schemas.event_schema(record) else {
                    return;
                };
                let parser = Parser::create(record, &schema);
                let (Ok(tid), Ok(bytes)) = (
                    parser.try_parse::<u32>("IssuingThreadId"),
                    parser.try_parse::<u32>("TransferSize"),
                ) else {
                    return;
                };
                let mut state = disk_counts.lock().unwrap_or_else(|e| e.into_inner());
                let pid = state
                    .threads
                    .get(&tid)
                    .copied()
                    .or_else(|| thread_owner(tid));
                if let Some(pid) = pid {
                    state.add(pid, bytes.into(), 0);
                }
            })
            .build();
        let (trace, handle) = KernelTrace::new()
            .named(format!(
                "Omniscience-ProcessIo-{}-{}",
                std::process::id(),
                SERIAL.fetch_add(1, Ordering::Relaxed)
            ))
            .enable(tcp)
            .enable(udp)
            .enable(threads)
            .enable(disk)
            .start()
            .ok()?;
        let running = Arc::new(AtomicBool::new(true));
        let worker_running = running.clone();
        let worker = std::thread::Builder::new()
            .name("process-io".into())
            .spawn(move || {
                let _ = KernelTrace::process_from_handle(handle);
                worker_running.store(false, Ordering::Release);
            })
            .ok()?;
        Some(Self {
            trace: Some(trace),
            worker: Some(worker),
            running,
            counts,
            last: Instant::now(),
        })
    }

    pub fn sample(&mut self) -> Option<HashMap<u32, (f64, f64)>> {
        if !self.running.load(Ordering::Acquire) {
            return None;
        }
        let now = Instant::now();
        let seconds = now.duration_since(self.last).as_secs_f64();
        self.last = now;
        let mut state = self.counts.lock().unwrap_or_else(|e| e.into_inner());
        Some(
            std::mem::take(&mut state.bytes)
                .into_iter()
                .map(|(pid, (disk, net))| {
                    (
                        pid,
                        (
                            disk as f64 / seconds.max(0.001),
                            net as f64 / seconds.max(0.001),
                        ),
                    )
                })
                .collect(),
        )
    }
}

impl Drop for ProcessIo {
    fn drop(&mut self) {
        // Stop the owned session before joining its blocking consumer thread.
        drop(self.trace.take());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn thread_owner(tid: u32) -> Option<u32> {
    // SAFETY: query-only access, handle checked and closed on every path.
    unsafe {
        let handle = OpenThread(THREAD_QUERY_LIMITED_INFORMATION, 0, tid);
        if handle.is_null() {
            return None;
        }
        let pid = GetProcessIdOfThread(handle);
        CloseHandle(handle);
        (pid != 0).then_some(pid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulates_only_attributed_payload_bytes() {
        let mut counts = Counts::default();
        counts.add(0, 99, 99);
        counts.add(42, 100, 20);
        counts.add(42, 50, 30);
        counts.add(7, 0, 5);
        assert_eq!(counts.bytes.get(&42), Some(&(150, 50)));
        assert_eq!(counts.bytes.get(&7), Some(&(0, 5)));
        assert!(!counts.bytes.contains_key(&0));
    }
}
