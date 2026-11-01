use super::*;

fn migrated(json: &str) -> Preferences {
    serde_json::from_str::<Preferences>(json).unwrap().migrate()
}

#[test]
fn migrates_saved_auto_and_manual_modes_to_both_sources() {
    let auto = migrated(r#"{"mode":"auto"}"#);
    assert_eq!(auto.profile(true), Some(PowerMode::Performance));
    assert_eq!(auto.profile(false), Some(PowerMode::Efficiency));
    let manual = migrated(r#"{"mode":"balanced"}"#);
    assert_eq!(manual.profile(true), Some(PowerMode::Balanced));
    assert_eq!(manual.profile(false), Some(PowerMode::Balanced));
    let fresh = migrated("{}");
    assert_eq!((fresh.profile(true), fresh.profile(false)), (None, None));
    let saved = migrated(r#"{"ac":"efficiency","battery":"performance"}"#);
    assert_eq!(saved.profile(true), Some(PowerMode::Efficiency));
    assert_eq!(saved.profile(false), Some(PowerMode::Performance));
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
    assert!(serde_json::from_str::<PowerMode>("\"auto\"").is_err());
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
