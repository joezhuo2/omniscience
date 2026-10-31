use super::*;

fn mode(width: u32, height: u32, hz: u32) -> Mode {
    Mode { width, height, hz }
}

#[test]
fn lists_distinct_resolutions_descending_and_rates_ascending() {
    let modes = [
        mode(1920, 1080, 60),
        mode(2560, 1600, 120),
        mode(2560, 1600, 60),
        mode(2560, 1600, 60),
    ];
    assert_eq!(resolutions(&modes), vec![[2560, 1600], [1920, 1080]]);
    assert_eq!(rates_at(&modes, 2560, 1600), vec![60, 120]);
    assert!(rates_at(&modes, 800, 600).is_empty());
}

#[test]
fn auto_prefers_60_on_battery_and_highest_on_ac() {
    assert_eq!(auto_rate(&[48, 60, 120], false), Some(60));
    assert_eq!(auto_rate(&[48, 90, 120], false), Some(90));
    assert_eq!(auto_rate(&[30, 48], false), Some(48));
    assert_eq!(auto_rate(&[60, 120], true), Some(120));
    assert_eq!(auto_rate(&[], true), None);
}

#[test]
fn resolution_change_keeps_rate_or_uses_highest_or_auto() {
    assert_eq!(resolution_rate(&[60, 120], 120, None), Some(120));
    assert_eq!(resolution_rate(&[60, 90], 120, None), Some(90));
    assert_eq!(resolution_rate(&[60, 120], 120, Some(false)), Some(60));
    assert_eq!(resolution_rate(&[60, 120], 60, Some(true)), Some(120));
}
