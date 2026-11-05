use iced::Point;

use super::{EMBLEM_RADII, METRICS, divider};

#[test]
fn divider_mirrors_rtl() {
    assert_eq!(
        divider(false, 318.0, 1180.0, 700.0),
        [Point::new(318.0, 0.0), Point::new(318.0, 700.0)]
    );
    assert_eq!(
        divider(true, 318.0, 1180.0, 700.0),
        [Point::new(862.0, 0.0), Point::new(862.0, 700.0)]
    );
}

#[test]
fn emblem_radii_fit() {
    for radius in EMBLEM_RADII {
        assert!(radius <= METRICS.emblem / 2.0);
    }
}
