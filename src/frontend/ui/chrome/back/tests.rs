#![cfg(test)]

use iced::{Point, Rectangle, Size};

use crate::frontend::scene::BackPanel;

use super::background::{embedded_section_radii, masthead_divider};
use super::layout::back_layout;

#[test]
fn copy_action_availability() {
    for (static_img, revealable) in [(true, true), (true, false), (false, true), (false, false)] {
        let panel = BackPanel { static_img, revealable, ..BackPanel::default() };
        let actions = back_layout(&panel).actions;
        assert_eq!(actions.copy_image.is_some(), static_img && revealable);
        assert_eq!(actions.copy_path.is_some(), revealable);
    }
}

#[test]
fn copy_actions_mirror() {
    let panel = BackPanel {
        cx: 500.0,
        cy: 400.0,
        hw: 220.0,
        hh: 250.0,
        static_img: true,
        revealable: true,
        ..BackPanel::default()
    };
    let ltr = super::layout::back_layout_for(&panel, false).actions;
    let rtl = super::layout::back_layout_for(&panel, true).actions;
    for (original, mirrored) in [
        (ltr.copy_image.unwrap(), rtl.copy_image.unwrap()),
        (ltr.copy_path.unwrap(), rtl.copy_path.unwrap()),
    ] {
        assert_eq!(
            mirrored,
            (panel.cx * 2.0 - original.0 - original.2, original.1, original.2, original.3)
        );
    }
}

#[test]
fn copy_actions_fit() {
    for (embedded, hw, hh) in [
        (true, 160.0, 300.0),
        (true, 450.0, 220.0),
        (false, 220.0, 250.0),
        (true, 160.0, 160.0),
        (true, 220.0, 170.0),
    ] {
        let panel = BackPanel {
            embedded,
            cx: 500.0,
            cy: 400.0,
            hw,
            hh,
            static_img: true,
            revealable: true,
            overview_available: true,
            tags: vec!["nature".into(), "landscape".into()],
            ..BackPanel::default()
        };
        for rtl in [false, true] {
            let layout = super::layout::back_layout_for(&panel, rtl);
            let actions = [
                layout.actions.copy_image.unwrap(),
                layout.actions.copy_path.unwrap(),
                layout.actions.effects.unwrap(),
                layout.actions.reveal.unwrap(),
                layout.actions.overview.unwrap(),
                layout.actions.playlist,
                layout.actions.delete,
            ];
            for (idx, rect) in actions.iter().enumerate() {
                assert!(rect.0 >= layout.card.0 && rect.1 >= layout.card.1);
                assert!(rect.0 + rect.2 <= layout.card.0 + layout.card.2 + 0.01);
                assert!(
                    rect.1 + rect.3 <= layout.card.1 + layout.card.3 + 0.01,
                    "{embedded} {hw} {hh}"
                );
                for other in &actions[idx + 1..] {
                    assert!(
                        rect.0 + rect.2 <= other.0 + 0.01
                            || other.0 + other.2 <= rect.0 + 0.01
                            || rect.1 + rect.3 <= other.1 + 0.01
                            || other.1 + other.3 <= rect.1 + 0.01
                    );
                }
            }
        }
    }
}

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
        assert!(back_layout(&panel).actions.reset_thumbnail.is_none());
        panel.reset_thumbnail = true;
        let layout = back_layout(&panel);
        let reset = layout.actions.reset_thumbnail.expect("generated thumbnail action");
        assert!(reset.2 > 0.0 && reset.3 > 0.0);
        for other in [
            layout.actions.playlist,
            layout.actions.delete,
            layout.actions.scene_properties.unwrap(),
            layout.actions.overview.unwrap(),
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

#[test]
fn reveal_action_fits_cards() {
    for (embedded, hw, hh) in [(true, 160.0, 300.0), (true, 450.0, 220.0), (false, 220.0, 250.0)] {
        let mut panel = BackPanel {
            embedded,
            cx: 500.0,
            cy: 400.0,
            hw,
            hh,
            reset_thumbnail: true,
            overview_available: true,
            ..BackPanel::default()
        };
        assert!(back_layout(&panel).actions.reveal.is_none());
        panel.revealable = true;
        let layout = back_layout(&panel);
        let reveal = layout.actions.reveal.expect("reveal action");
        assert!(reveal.2 > 0.0 && reveal.3 > 0.0);
        for other in [
            layout.actions.playlist,
            layout.actions.delete,
            layout.actions.reset_thumbnail.unwrap(),
            layout.actions.overview.unwrap(),
        ] {
            assert!(
                reveal.0 + reveal.2 <= other.0 + 0.01
                    || other.0 + other.2 <= reveal.0 + 0.01
                    || reveal.1 + reveal.3 <= other.1 + 0.01
                    || other.1 + other.3 <= reveal.1 + 0.01
            );
        }
    }
}

#[test]
fn actions_rule_and_rail_divider_follow_the_leading_edge() {
    assert_eq!(super::actions::actions_rule(false, 100.0, 300.0), (174.0, 300.0));
    assert_eq!(super::actions::actions_rule(true, 100.0, 300.0), (100.0, 226.0));
    let sheet = Rectangle::new(Point::new(40.0, 60.0), Size::new(300.0, 500.0));
    let (from, to) = super::background::rail_divider(sheet, false);
    assert_eq!((from, to), (Point::new(340.0, 60.0), Point::new(340.0, 560.0)));
    let (from, to) = super::background::rail_divider(sheet, true);
    assert_eq!((from, to), (Point::new(40.0, 60.0), Point::new(40.0, 560.0)));
}
