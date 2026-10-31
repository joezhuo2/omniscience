use super::*;

#[test]
fn clamps_to_range_and_keeps_boost_at_or_above_sustained() {
    assert_eq!(normalize(45, 65), (45, 65));
    assert_eq!(normalize(0, 200), (5, 80));
    assert_eq!(normalize(70, 30), (70, 70));
    assert_eq!(normalize(100, 100), (80, 80));
}
