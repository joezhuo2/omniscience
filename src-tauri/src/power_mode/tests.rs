use super::*;

#[test]
fn automatic_uses_performance_on_ac_and_efficiency_on_battery() {
    assert_eq!(target(PowerMode::Auto, true), PowerMode::Performance);
    assert_eq!(target(PowerMode::Auto, false), PowerMode::Efficiency);
    for mode in [
        PowerMode::Efficiency,
        PowerMode::Balanced,
        PowerMode::Performance,
    ] {
        assert_eq!(target(mode, true), mode);
        assert_eq!(target(mode, false), mode);
    }
}

#[test]
fn guids_round_trip_and_unknown_is_none() {
    for mode in [
        PowerMode::Efficiency,
        PowerMode::Balanced,
        PowerMode::Performance,
    ] {
        assert_eq!(mode_of(guid_of(mode)), Some(mode));
    }
    assert_eq!(
        mode_of("DED574B5-45A0-4F42-8737-46345C09C238"),
        Some(PowerMode::Performance)
    );
    assert_eq!(mode_of("3af9b8d9-7c97-431d-ad78-34a8bfa6f3d0"), None);
}

#[test]
fn rejects_unknown_mode_names() {
    assert!(serde_json::from_str::<PowerMode>("\"turbo\"").is_err());
    assert_eq!(
        serde_json::from_str::<PowerMode>("\"auto\"").unwrap(),
        PowerMode::Auto
    );
}

#[test]
#[cfg(windows)]
fn guid_parse_and_format_round_trip() {
    assert_eq!(guid_string(&parse_guid(PERFORMANCE)), PERFORMANCE);
    assert_eq!(guid_string(&parse_guid(EFFICIENCY)), EFFICIENCY);
}

#[test]
#[cfg(windows)]
#[ignore = "read-only Windows power mode probe"]
fn power_mode_live() {
    println!("{:?}", read_current());
}
