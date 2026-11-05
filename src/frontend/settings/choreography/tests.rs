use super::*;

#[test]
fn outgoing_stays_legible() {
    let progress = SETTINGS_MOTION.roll.at(70.0 / SETTINGS_MOTION.content_ms);
    assert!(progress < 0.15);
    assert!(progress > 0.0);
    assert_eq!(SETTINGS_MOTION.roll.at(1.0), 1.0);
}

#[test]
fn staggered_tracks_finish() {
    assert_eq!(SETTINGS_MOTION.entrance(0.0, 0, true), 0.0);
    assert_eq!(SETTINGS_MOTION.entrance(0.0, 0, false), 0.0);
    assert!(SETTINGS_MOTION.entrance(0.2, 0, false) > 0.0);
    assert_eq!(SETTINGS_MOTION.entrance(0.2, 9, false), 0.0);
    assert_eq!(SETTINGS_MOTION.entrance(1.0, 30, true), 1.0);
    assert_eq!(SETTINGS_MOTION.entrance(1.0, 20, false), 1.0);
    for index in [33, 60, 250] {
        for navigation in [false, true] {
            assert_eq!(SETTINGS_MOTION.entrance(1.0, index, navigation), 1.0);
            assert!(SETTINGS_MOTION.entrance(0.99, index, navigation) > 0.9);
        }
    }
}
