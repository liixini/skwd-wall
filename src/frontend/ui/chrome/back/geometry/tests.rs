use super::*;

fn panel() -> BackPanel {
    BackPanel {
        embedded: true,
        cx: 500.0,
        cy: 300.0,
        hw: 250.0,
        hh: 200.0,
        edge_tilt: 80.0,
        ..BackPanel::default()
    }
}

#[test]
fn positive_edge_tilt_raises_right_side_without_slanting_sides() {
    let panel = panel();
    let layout = super::super::layout::back_layout(&panel);
    let geometry = BackGeometry::new(&panel, &layout);
    let card = rectangle(layout.card);
    let top_left = geometry.point(card.position());
    let top_right = geometry.point(Point::new(card.x + card.width, card.y));
    let bottom_left = geometry.point(Point::new(card.x, card.y + card.height));
    let bottom_right = geometry.point(Point::new(card.x + card.width, card.y + card.height));

    assert!((top_left.y - top_right.y - panel.edge_tilt).abs() < 0.01);
    assert!((bottom_left.y - bottom_right.y - panel.edge_tilt).abs() < 0.01);
    assert!((top_left.x - bottom_left.x).abs() < 0.01);
    assert!((top_right.x - bottom_right.x).abs() < 0.01);
}

#[test]
fn transformed_hit_target_uses_the_same_inverse_geometry() {
    let panel = panel();
    let layout = super::super::layout::back_layout(&panel);
    let target = layout.actions.delete;
    let geometry = BackGeometry::new(&panel, &layout);
    let logical_center = Point::new(target.0 + target.2 * 0.5, target.1 + target.3 * 0.5);
    let drawn_center = geometry.point(logical_center);

    assert!(back_contains(&panel, &layout, target, drawn_center.x, drawn_center.y));
    assert!(!back_contains(
        &panel,
        &layout,
        target,
        drawn_center.x,
        drawn_center.y + target.3 * 2.0,
    ));
}

#[test]
fn text_rotates_to_the_transformed_horizontal_axis() {
    let panel = panel();
    let layout = super::super::layout::back_layout(&panel);
    let geometry = BackGeometry::new(&panel, &layout);
    let left = geometry.point(Point::new(layout.content_left, layout.title_cy));
    let right = geometry.point(Point::new(layout.content_right, layout.title_cy));
    let expected = (right.y - left.y).atan2(right.x - left.x);

    assert!((geometry.text_angle() - expected).abs() < 0.000_001);
    assert!(geometry.text_angle() < 0.0);
}
