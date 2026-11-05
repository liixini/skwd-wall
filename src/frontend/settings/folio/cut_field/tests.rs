use super::*;

#[test]
fn wipe_respects_cut_corners() {
    let size = Size::new(WIDTH, HEIGHT);
    for right_to_left in [false, true] {
        assert!(wipe_vertices(size, 1.0, 0.0, right_to_left).is_empty());
        assert_eq!(wipe_vertices(size, 1.0, 1.0, right_to_left).len(), 6);
        for point in wipe_vertices(size, 1.0, 0.5, right_to_left) {
            assert!((0.0..=WIDTH).contains(&point.x));
            assert!((0.0..=HEIGHT).contains(&point.y));
            if point.x < CUT {
                assert!(point.y >= CUT - point.x);
            }
            if point.x > WIDTH - CUT {
                assert!(point.y <= HEIGHT + WIDTH - CUT - point.x);
            }
        }
    }
}
