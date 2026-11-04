use super::*;
#[test]
#[cfg(windows)]
#[ignore = "read-only ASUS driver probe on a supported reference laptop"]
fn gpu_support_live() {
    let (mode, error) = read_hardware();
    assert!(mode.is_some(), "ASUS GPU state unavailable: {error:?}");
    println!("Firmware mode: {mode:?}");
}

#[test]
fn automatic_uses_integrated_on_battery_and_hybrid_on_ac() {
    assert_eq!(target(GpuMode::Auto, false), GpuMode::Min);
    assert_eq!(target(GpuMode::Auto, true), GpuMode::Med);
    for mode in [GpuMode::Med, GpuMode::Min] {
        assert_eq!(target(mode, false), mode);
        assert_eq!(target(mode, true), mode);
    }
}
struct FakeGpu {
    eco: std::cell::Cell<u32>,
    mux: std::cell::Cell<Option<u32>>,
    writes: std::cell::RefCell<Vec<(u32, u32)>>,
}
impl FakeGpu {
    fn new(eco: u32, mux: Option<u32>) -> Self {
        Self {
            eco: eco.into(),
            mux: mux.into(),
            writes: Default::default(),
        }
    }
}
impl GpuDevice for FakeGpu {
    fn get(&self, device: u32) -> Result<u32, String> {
        match device {
            0x0009_0020 => Ok(self.eco.get()),
            0x0009_0016 => self.mux.get().ok_or_else(|| "No MUX".into()),
            _ => Err("Unsupported endpoint".into()),
        }
    }
    fn set(&self, device: u32, value: u32) -> Result<(), String> {
        self.writes.borrow_mut().push((device, value));
        match device {
            0x0009_0020 => self.eco.set(value),
            0x0009_0016 => self.mux.set(Some(value)),
            _ => return Err("Unsupported endpoint".into()),
        }
        Ok(())
    }
}

#[test]
fn leaving_discrete_display_mode_requires_reboot_before_disabling_gpu() {
    let gpu = FakeGpu::new(0, Some(0));
    assert!(apply_device(&gpu, GpuMode::Min, false).unwrap());
    assert_eq!(*gpu.writes.borrow(), vec![(0x0009_0016, 1)]);
    assert_eq!(gpu.eco.get(), 0);
}

#[test]
fn no_mode_routes_the_display_through_the_dgpu() {
    for mode in [GpuMode::Auto, GpuMode::Med, GpuMode::Min] {
        for ac in [false, true] {
            let gpu = FakeGpu::new(1, Some(1));
            assert!(!apply_device(&gpu, mode, ac).unwrap());
            assert!(gpu.writes.borrow().iter().all(|w| w.0 != 0x0009_0016));
        }
    }
}

#[test]
fn saved_max_preference_loads_as_hybrid() {
    let prefs: Preferences = serde_json::from_str(r#"{"mode":"max"}"#).unwrap();
    assert_eq!(prefs.mode, Some(GpuMode::Med));
}

#[test]
fn discrete_display_reports_med_with_a_way_back() {
    let (mode, note) = classify(0, Some(0));
    assert_eq!(mode, GpuMode::Med);
    assert!(note.unwrap().contains("Select Med"));
    assert_eq!(classify(1, Some(1)), (GpuMode::Min, None));
    assert_eq!(classify(0, None), (GpuMode::Med, None));
}

#[test]
fn unknown_mux_state_never_disables_the_gpu() {
    let gpu = FakeGpu::new(0, Some(2));
    assert!(apply_device(&gpu, GpuMode::Min, false).is_err());
    assert!(gpu.writes.borrow().is_empty());
}

#[test]
fn auto_switches_only_the_eco_endpoint_and_skips_unchanged_writes() {
    let gpu = FakeGpu::new(0, Some(1));
    assert!(!apply_device(&gpu, GpuMode::Auto, false).unwrap());
    assert_eq!(*gpu.writes.borrow(), vec![(0x0009_0020, 1)]);
    assert!(!apply_device(&gpu, GpuMode::Auto, false).unwrap());
    assert_eq!(gpu.writes.borrow().len(), 1);
    assert!(!apply_device(&gpu, GpuMode::Auto, true).unwrap());
    assert_eq!(gpu.eco.get(), 0);
}

#[test]
fn unconfigured_preferences_never_enable_automation() {
    let prefs: Preferences = serde_json::from_str("{}").unwrap();
    assert_eq!(prefs.mode, None);
    assert_eq!(prefs.pending_boot, None);
}
