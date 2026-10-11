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
}
