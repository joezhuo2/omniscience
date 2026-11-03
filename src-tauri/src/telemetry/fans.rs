#[cfg(windows)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(windows)]
use std::sync::{Arc, Mutex};
#[cfg(windows)]
use std::time::Duration;

#[cfg(any(windows, test))]
use super::types::FanSpeeds;

#[cfg(windows)]
const EVERY: Duration = Duration::from_secs(1);
#[cfg(any(windows, test))]
const RPM_PER_UNIT: u32 = 100;

#[cfg(any(windows, test))]
fn rpm(reading: u32) -> u32 {
    (reading & 0xFF) * RPM_PER_UNIT
}

#[cfg(any(windows, test))]
pub fn speeds(cpu: Option<u32>, gpu: Option<u32>) -> Option<FanSpeeds> {
    let (cpu_rpm, gpu_rpm) = (cpu.map(rpm), gpu.map(rpm));
    (cpu_rpm.is_some() || gpu_rpm.is_some()).then_some(FanSpeeds { cpu_rpm, gpu_rpm })
}

#[cfg(windows)]
pub struct FanSource {
    latest: Arc<Mutex<Option<FanSpeeds>>>,
    stop: Arc<AtomicBool>,
}

#[cfg(windows)]
impl FanSource {
    pub fn start() -> Self {
        let latest = Arc::new(Mutex::new(None));
        let stop = Arc::new(AtomicBool::new(false));
        let (out, done) = (latest.clone(), stop.clone());
        let spawned = std::thread::Builder::new()
            .name("fans".into())
            .spawn(move || {
                let Ok(asus) = crate::asus::Asus::open() else {
                    return;
                };
                while !done.load(Ordering::SeqCst) {
                    let read = |device| asus.get(device).ok();
                    let reading = speeds(read(crate::asus::CPU_FAN), read(crate::asus::GPU_FAN));
                    *out.lock().unwrap_or_else(|e| e.into_inner()) = reading;
                    std::thread::sleep(EVERY);
                }
            });
        if let Err(e) = spawned {
            crate::diagnostics::log_error(format_args!("spawn fan thread: {e}"));
        }
        Self { latest, stop }
    }

    pub fn latest(&self) -> Option<FanSpeeds> {
        *self.latest.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[cfg(windows)]
impl Drop for FanSource {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_units_of_100_rpm() {
        assert_eq!(rpm(0x2B), 4300);
        assert_eq!(rpm(1), 100);
        assert_eq!(rpm(0), 0);
    }

    #[test]
    fn ignores_bits_above_the_low_byte() {
        assert_eq!(rpm(0x0001_0023), 3500);
        assert_eq!(rpm(0x0000_FF00), 0);
        assert_eq!(rpm(0xFF), 25_500);
    }

    #[test]
    fn needs_at_least_one_fan() {
        assert_eq!(speeds(None, None), None);
    }

    #[test]
    fn keeps_each_fan_independent() {
        assert_eq!(
            speeds(Some(0x20), None),
            Some(FanSpeeds {
                cpu_rpm: Some(3200),
                gpu_rpm: None
            })
        );
        assert_eq!(
            speeds(None, Some(0x1C)),
            Some(FanSpeeds {
                cpu_rpm: None,
                gpu_rpm: Some(2800)
            })
        );
        assert_eq!(
            speeds(Some(0), Some(0x1C)),
            Some(FanSpeeds {
                cpu_rpm: Some(0),
                gpu_rpm: Some(2800)
            })
        );
    }

    #[cfg(windows)]
    #[test]
    fn source_stops_when_dropped_and_never_panics_without_a_sensor() {
        let source = FanSource::start();
        std::thread::sleep(Duration::from_millis(200));
        let _ = source.latest();
        drop(source);
    }
}
