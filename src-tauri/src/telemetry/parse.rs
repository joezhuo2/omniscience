//! Pure parsing of performance-counter instance names and units. Platform-independent so it is
//! tested everywhere; only Windows feeds it real data.

use std::collections::HashMap;

use super::types::PowerBreakdown;

/// Milliwatts (Energy Meter `Power` counter) to watts.
pub fn mw_to_w(mw: f64) -> f32 {
    (mw / 1000.0) as f32
}

/// Tenths of Kelvin (`High Precision Temperature`) to °C, dropping implausible readings.
pub fn decikelvin_to_c(dk: f64) -> Option<f32> {
    let c = dk / 10.0 - 273.15;
    plausible_temp(c)
}

/// Unpopulated sensors report 0 or garbage.
pub fn plausible_temp(c: f64) -> Option<f32> {
    (c > 0.0 && c < 150.0).then(|| ((c * 10.0).round() / 10.0) as f32)
}

/// ACPI zone instance (`\_TZ.THRM`) to a display label (`ACPI THRM`).
pub fn zone_label(instance: &str) -> String {
    let name = instance.rsplit('.').next().unwrap_or(instance);
    format!(
        "ACPI {}",
        name.trim_matches(|c: char| !c.is_ascii_alphanumeric())
    )
}

/// Lowercase LUID key (`0x00000000_0x00012980`) from a GPU counter instance name such as
/// `pid_12_luid_0x00000000_0x00012980_phys_0_eng_3_engtype_3D` or
/// `luid_0x00000000_0x00012980_phys_0`.
pub fn luid_key(instance: &str) -> Option<String> {
    let lower = instance.to_ascii_lowercase();
    let rest = &lower[lower.find("luid_")? + 5..];
    let mut parts = rest.splitn(3, '_');
    let (high, low) = (parts.next()?, parts.next()?);
    let is_hex = |s: &str| {
        s.starts_with("0x") && s.len() > 2 && s[2..].chars().all(|c| c.is_ascii_hexdigit())
    };
    (is_hex(high) && is_hex(low)).then(|| format!("{high}_{low}"))
}

/// LUID key for an adapter LUID, matching [`luid_key`].
pub fn luid_key_from_parts(high: i32, low: u32) -> String {
    format!("0x{:08x}_0x{:08x}", high as u32, low)
}

/// GPU utilization per adapter, like Task Manager: per engine type, sum every instance's
/// utilization, then take the busiest engine type.
pub fn gpu_usage_by_adapter(items: &[(String, f64)]) -> HashMap<String, f32> {
    let mut per_type: HashMap<(String, &str), f64> = HashMap::new();
    for (name, value) in items {
        let Some(luid) = luid_key(name) else { continue };
        let engtype = name.rsplit_once("engtype_").map_or("other", |(_, t)| t);
        *per_type.entry((luid, engtype)).or_default() += value.max(0.0);
    }
    let mut out: HashMap<String, f32> = HashMap::new();
    for ((luid, _), sum) in per_type {
        let v = sum.min(100.0) as f32;
        let slot = out.entry(luid).or_insert(0.0);
        *slot = slot.max(v);
    }
    out
}

/// Process id from a per-process GPU counter instance such as
/// `pid_12_luid_0x00000000_0x00012980_phys_0_eng_3_engtype_3D`.
pub fn pid_of(instance: &str) -> Option<u32> {
    instance
        .strip_prefix("pid_")?
        .split('_')
        .next()?
        .parse()
        .ok()
}

/// GPU utilization per process, like Task Manager's GPU column: per engine (adapter and engine
/// type), sum the process's instances, then take its busiest engine.
pub fn gpu_usage_by_pid(items: &[(String, f64)]) -> HashMap<u32, f32> {
    let mut per_engine: HashMap<(u32, String, &str), f64> = HashMap::new();
    for (name, value) in items {
        let (Some(pid), Some(luid)) = (pid_of(name), luid_key(name)) else {
            continue;
        };
        let engtype = name.rsplit_once("engtype_").map_or("other", |(_, t)| t);
        *per_engine.entry((pid, luid, engtype)).or_default() += value.max(0.0);
    }
    let mut out: HashMap<u32, f32> = HashMap::new();
    for ((pid, _, _), sum) in per_engine {
        let slot = out.entry(pid).or_insert(0.0);
        *slot = slot.max(sum.min(100.0) as f32);
    }
    out
}

/// Sums counter values per adapter (e.g. `GPU Adapter Memory` usage across `phys_N`).
pub fn sum_by_adapter(items: &[(String, f64)]) -> HashMap<String, u64> {
    let mut out: HashMap<String, u64> = HashMap::new();
    for (name, value) in items {
        if let Some(luid) = luid_key(name) {
            *out.entry(luid).or_default() += value.max(0.0) as u64;
        }
    }
    out
}

/// Sums counter values per process (e.g. `GPU Process Memory` usage across adapters).
pub fn sum_by_pid(items: &[(String, f64)]) -> HashMap<u32, u64> {
    let mut out: HashMap<u32, u64> = HashMap::new();
    for (name, value) in items {
        if let Some(pid) = pid_of(name) {
            *out.entry(pid).or_default() += value.max(0.0) as u64;
        }
    }
    out
}

/// Readings from `\Energy Meter(*)\Power`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EnergyReadings {
    pub breakdown: PowerBreakdown,
    pub package_watts: Option<f32>,
    /// Per-core watts keyed by core index (`RAPL_Package0_Core{N}_CORE`).
    pub cores: HashMap<u32, f32>,
}

pub fn parse_energy(items: &[(String, f64)]) -> EnergyReadings {
    let mut r = EnergyReadings::default();
    let mut rapl_pkg = None;
    for (name, mw) in items {
        let w = mw_to_w(*mw);
        let lower = name.to_ascii_lowercase();
        match lower.as_str() {
            "apu power" => r.breakdown.apu = Some(w),
            "cpu power" => r.breakdown.cpu = Some(w),
            "gpu power" => r.breakdown.gpu = Some(w),
            "npu power" => r.breakdown.npu = Some(w),
            _ if lower.ends_with("_pkg") && lower.starts_with("rapl_") => {
                rapl_pkg = Some(rapl_pkg.unwrap_or(0.0) + w)
            }
            _ => {
                if let Some(core) = rapl_core_index(&lower) {
                    r.cores.insert(core, w);
                }
            }
        }
    }
    let b = &r.breakdown;
    r.package_watts = b.apu.or(rapl_pkg).or(match (b.cpu, b.gpu) {
        (Some(c), Some(g)) => Some(c + g),
        (c, g) => c.or(g),
    });
    r
}

/// Watts for logical CPU `logical`. RAPL reports physical cores; with SMT (twice as many logical
/// CPUs as RAPL cores) Windows numbers sibling threads adjacently, so both read their core's power.
pub fn core_watts(cores: &HashMap<u32, f32>, logical: u32, logical_count: usize) -> Option<f32> {
    if cores.is_empty() {
        return None;
    }
    let physical = if cores.len() * 2 == logical_count {
        logical / 2
    } else {
        logical
    };
    cores.get(&physical).copied()
}

/// Convert Windows `% Performance Limit` (available nominal performance) to loss.
/// 100% or boosted headroom means no loss. Zero/non-finite readings are unavailable,
/// not evidence that a running CPU has lost 100% of its performance.
pub fn performance_loss_pct(available: f64) -> Option<f32> {
    (available.is_finite() && available > 0.0)
        .then(|| (100.0 - available).max(0.0) as f32)
}

/// Logical CPU index from a `Processor Information` instance (`0,5` -> 5, `1,3` -> 67). Totals
/// (`_Total`, `0,_Total`) return `None`.
fn processor_index(instance: &str) -> Option<u32> {
    let (group, cpu) = instance.split_once(',')?;
    Some(group.parse::<u32>().ok()? * 64 + cpu.parse::<u32>().ok()?)
}

/// Live clock speeds, like Task Manager: base frequency x `% Processor Performance` (which goes
/// above 100% when boosting). Returns the overall clock and per-logical-CPU clocks in MHz.
pub fn clock_mhz(
    perf: &[(String, f64)],
    base: &[(String, f64)],
) -> (Option<u64>, HashMap<u32, u64>) {
    let base_of = |name: &str| {
        base.iter()
            .find(|(n, _)| n == name)
            .or_else(|| base.iter().find(|(n, _)| n == "_Total"))
            .map(|(_, mhz)| *mhz)
            .filter(|mhz| *mhz > 0.0)
    };
    let mut total = None;
    let mut cores = HashMap::new();
    for (name, pct) in perf {
        let Some(mhz) = base_of(name).map(|b| (b * pct / 100.0).round().max(0.0) as u64) else {
            continue;
        };
        if name == "_Total" {
            total = Some(mhz);
        } else if let Some(i) = processor_index(name) {
            cores.insert(i, mhz);
        }
    }
    (total, cores)
}

/// `rapl_package0_core12_core` -> 12.
fn rapl_core_index(lower: &str) -> Option<u32> {
    let rest = lower.strip_prefix("rapl_")?;
    let core = rest.split('_').find_map(|p| p.strip_prefix("core"))?;
    lower.ends_with("_core").then_some(())?;
    core.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items(v: &[(&str, f64)]) -> Vec<(String, f64)> {
        v.iter().map(|(n, x)| (n.to_string(), *x)).collect()
    }

    #[test]
    fn performance_limit_is_available_capacity_not_loss() {
        assert_eq!(performance_loss_pct(100.0), Some(0.0));
        assert_eq!(performance_loss_pct(120.0), Some(0.0));
        assert_eq!(performance_loss_pct(75.0), Some(25.0));
        assert_eq!(performance_loss_pct(99.0), Some(1.0));
        for value in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(performance_loss_pct(value), None);
        }
    }

    #[test]
    fn converts_units() {
        assert_eq!(mw_to_w(8223.5), 8.2235);
        assert_eq!(decikelvin_to_c(3231.5), Some(50.0));
        assert_eq!(decikelvin_to_c(0.0), None);
        assert_eq!(decikelvin_to_c(5000.0), None);
    }

    #[test]
    fn labels_acpi_zones() {
        assert_eq!(zone_label(r"\_TZ.THRM"), "ACPI THRM");
        assert_eq!(zone_label(r"\_TZ.TZ01"), "ACPI TZ01");
    }

    #[test]
    fn parses_luid_keys() {
        assert_eq!(
            luid_key("pid_12_luid_0x00000000_0x00012980_phys_0_eng_3_engtype_3D").as_deref(),
            Some("0x00000000_0x00012980")
        );
        assert_eq!(
            luid_key("luid_0x00000000_0x00014A02_phys_0").as_deref(),
            Some("0x00000000_0x00014a02")
        );
        assert_eq!(luid_key("_Total"), None);
        assert_eq!(luid_key("luid_zz_0x1"), None);
        assert_eq!(luid_key_from_parts(0, 0x14A02), "0x00000000_0x00014a02");
    }

    #[test]
    fn gpu_usage_takes_busiest_engine_type_per_adapter() {
        let v = items(&[
            (
                "pid_1_luid_0x00000000_0x0000AAAA_phys_0_eng_0_engtype_3D",
                20.0,
            ),
            (
                "pid_2_luid_0x00000000_0x0000AAAA_phys_0_eng_0_engtype_3D",
                15.0,
            ),
            (
                "pid_2_luid_0x00000000_0x0000AAAA_phys_0_eng_5_engtype_VideoDecode",
                30.0,
            ),
            (
                "pid_3_luid_0x00000000_0x0000BBBB_phys_0_eng_0_engtype_3D",
                250.0,
            ),
        ]);
        let u = gpu_usage_by_adapter(&v);
        assert_eq!(u["0x00000000_0x0000aaaa"], 35.0);
        assert_eq!(u["0x00000000_0x0000bbbb"], 100.0);
    }

    #[test]
    fn sums_adapter_memory() {
        let v = items(&[
            ("luid_0x00000000_0x0000AAAA_phys_0", 100.0),
            ("luid_0x00000000_0x0000AAAA_phys_1", 50.0),
        ]);
        assert_eq!(sum_by_adapter(&v)["0x00000000_0x0000aaaa"], 150);
    }

    #[test]
    fn parses_energy_meter_on_amd_apu() {
        let r = parse_energy(&items(&[
            ("Apu Power", 8223.6),
            ("CPU Power", 2804.2),
            ("GPU Power", 3232.6),
            ("NPU Power", 10.0),
            ("RAPL_Package0_Core0_CORE", 500.0),
            ("RAPL_Package0_Core11_CORE", 250.0),
        ]));
        assert_eq!(r.package_watts, Some(8.2236));
        assert_eq!(r.breakdown.gpu, Some(3.2326));
        assert_eq!(r.cores[&0], 0.5);
        assert_eq!(r.cores[&11], 0.25);
        assert_eq!(r.cores.len(), 2);
    }

    #[test]
    fn computes_live_clock_from_base_and_performance() {
        let perf = items(&[
            ("_Total", 150.0),
            ("0,_Total", 150.0),
            ("0,0", 100.0),
            ("0,5", 225.0),
        ]);
        let base = items(&[("_Total", 2000.0), ("0,0", 2000.0), ("0,5", 2000.0)]);
        let (total, cores) = clock_mhz(&perf, &base);
        assert_eq!(total, Some(3000));
        assert_eq!(cores[&0], 2000);
        assert_eq!(cores[&5], 4500);
        assert_eq!(cores.len(), 2);
        assert_eq!(clock_mhz(&perf, &[]), (None, HashMap::new()));
        assert_eq!(processor_index("1,3"), Some(67));
    }

    #[test]
    fn maps_smt_threads_to_their_physical_core() {
        let cores: HashMap<u32, f32> = (0..12).map(|c| (c, c as f32)).collect();
        assert_eq!(core_watts(&cores, 0, 24), Some(0.0));
        assert_eq!(core_watts(&cores, 1, 24), Some(0.0));
        assert_eq!(core_watts(&cores, 23, 24), Some(11.0));
        // Without SMT, one to one.
        assert_eq!(core_watts(&cores, 11, 12), Some(11.0));
        assert_eq!(core_watts(&HashMap::new(), 0, 24), None);
    }

    #[test]
    fn package_falls_back_to_rapl_then_cpu_plus_gpu() {
        let rapl = parse_energy(&items(&[
            ("RAPL_Package0_PKG", 15000.0),
            ("CPU Power", 1000.0),
        ]));
        assert_eq!(rapl.package_watts, Some(15.0));
        let sum = parse_energy(&items(&[("CPU Power", 1000.0), ("GPU Power", 2000.0)]));
        assert_eq!(sum.package_watts, Some(3.0));
        assert_eq!(parse_energy(&[]).package_watts, None);
    }

    #[test]
    fn reads_the_pid_of_a_process_instance() {
        assert_eq!(
            pid_of("pid_1234_luid_0x00000000_0x00012980_phys_0_eng_3_engtype_3D"),
            Some(1234)
        );
        assert_eq!(pid_of("luid_0x00000000_0x00012980_phys_0"), None);
        assert_eq!(pid_of("pid_x_luid"), None);
    }

    #[test]
    fn gpu_usage_per_pid_takes_the_busiest_engine() {
        let l = "luid_0x00000000_0x00012980_phys_0";
        let items = vec![
            (format!("pid_7_{l}_eng_0_engtype_3D"), 20.0),
            (format!("pid_7_{l}_eng_1_engtype_3D"), 15.0),
            (format!("pid_7_{l}_eng_5_engtype_VideoDecode"), 30.0),
            (format!("pid_9_{l}_eng_0_engtype_3D"), 2.5),
        ];
        let by_pid = gpu_usage_by_pid(&items);
        assert_eq!(by_pid.get(&7), Some(&35.0));
        assert_eq!(by_pid.get(&9), Some(&2.5));
    }

    #[test]
    fn sums_memory_per_pid_across_adapters() {
        let items = vec![
            ("pid_7_luid_0x00000000_0x00012980_phys_0".to_string(), 100.0),
            ("pid_7_luid_0x00000000_0x0001a000_phys_0".to_string(), 50.0),
            ("pid_8_luid_0x00000000_0x00012980_phys_0".to_string(), 10.0),
            ("luid_0x00000000_0x00012980_phys_0".to_string(), 999.0),
        ];
        let by_pid = sum_by_pid(&items);
        assert_eq!(by_pid.get(&7), Some(&150));
        assert_eq!(by_pid.get(&8), Some(&10));
        assert_eq!(by_pid.len(), 2);
    }
}
