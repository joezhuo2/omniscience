use super::*;

#[test]
fn clamps_to_range_and_keeps_boost_at_or_above_sustained() {
    assert_eq!(normalize(45, 65), (45, 65));
    assert_eq!(normalize(0, 200), (5, 80));
    assert_eq!(normalize(70, 30), (70, 70));
    assert_eq!(normalize(100, 100), (80, 80));
}

#[test]
fn migrates_the_single_saved_pair_to_both_sources() {
    let prefs: Preferences = serde_json::from_str(r#"{"spl":90,"sppt":10}"#).unwrap();
    let prefs = prefs.migrate();
    let expected = Some(Limits { spl: 80, sppt: 80 });
    assert_eq!(prefs.profile(true), expected);
    assert_eq!(prefs.profile(false), expected);
    assert!(
        !serde_json::to_string(&prefs)
            .unwrap()
            .contains("\"spl\":90")
    );
}

#[test]
fn keeps_existing_per_source_profiles_and_edits_one_source() {
    let json = r#"{"ac":{"spl":60,"sppt":70},"battery":{"spl":15,"sppt":20}}"#;
    let mut prefs = serde_json::from_str::<Preferences>(json).unwrap().migrate();
    assert_eq!(prefs.profile(false), Some(Limits { spl: 15, sppt: 20 }));
    prefs.set_profile(true, None);
    assert_eq!(prefs.profile(true), None);
    assert_eq!(prefs.profile(false), Some(Limits { spl: 15, sppt: 20 }));
}
