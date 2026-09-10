#![cfg(test)]

use iced::{Point, Rectangle, Size};

use crate::frontend::scene::BackPanel;

use super::background::{embedded_section_radii, masthead_divider};
use super::layout::back_layout;

#[test]
fn masthead_divider_embedded() {
    let masthead = Rectangle::new(Point::new(190.0, 105.0), Size::new(900.0, 32.0));
    assert_eq!(masthead_divider(masthead, true), None);
    let (from, to) = masthead_divider(masthead, false).expect("divider");
    assert_eq!(from, Point::new(190.0, 137.0));
    assert_eq!(to, Point::new(1090.0, 137.0));
}

#[test]
fn embedded_sections_preserve_matching_square_and_rounded_corners() {
    let panel = BackPanel {
        embedded: true,
        cx: 500.0,
        cy: 400.0,
        hw: 260.0,
        hh: 340.0,
        edge_tilt: 72.0,
        radii: [48.0, 0.0, 76.0, 0.0],
        ..BackPanel::default()
    };
    let layout = back_layout(&panel);
    let rect = |tuple: (f32, f32, f32, f32)| {
        Rectangle::new(Point::new(tuple.0, tuple.1), Size::new(tuple.2, tuple.3))
    };
    let masthead = embedded_section_radii(&panel, &layout, rect(layout.masthead), true, false);
    let sheet = embedded_section_radii(&panel, &layout, rect(layout.sheet), false, true);

    assert!(masthead[0] > 0.0);
    assert_eq!(masthead[1], 0.0);
    assert_eq!(masthead[2..], [0.0, 0.0]);
    assert_eq!(sheet[..2], [0.0, 0.0]);
    assert!(sheet[2] > 0.0);
    assert_eq!(sheet[3], 0.0);
}

#[test]
fn generated_thumbnail_action_fits_all_card_presentations() {
    for (embedded, hw, hh) in [(true, 160.0, 300.0), (true, 450.0, 220.0), (false, 220.0, 250.0)] {
        let mut panel = BackPanel {
            embedded,
            cx: 500.0,
            cy: 400.0,
            hw,
            hh,
            scene_properties: true,
            overview_available: true,
            ..BackPanel::default()
        };
        assert!(back_layout(&panel).reset_thumbnail.is_none());
        panel.reset_thumbnail = true;
        let layout = back_layout(&panel);
        let reset = layout.reset_thumbnail.expect("generated thumbnail action");
        assert!(reset.2 > 0.0 && reset.3 > 0.0);
        for other in [
            layout.playlist,
            layout.delete,
            layout.scene_properties.unwrap(),
            layout.overview.unwrap(),
        ] {
            assert!(
                reset.0 + reset.2 <= other.0 + 0.01
                    || other.0 + other.2 <= reset.0 + 0.01
                    || reset.1 + reset.3 <= other.1 + 0.01
                    || other.1 + other.3 <= reset.1 + 0.01
            );
        }
    }
}
