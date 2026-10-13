#![cfg(test)]

#[test]
fn chrome_signature_buckets() {
    use crate::frontend::scene::{BackPanel, Chrome};
    use crate::frontend::theme::Palette;
    use crate::frontend::ui::chrome_signature;
    use iced::Color;

    let pal = Palette::default();
    let chrome = Chrome {
        view: 0,
        cx: 10.0,
        cy: 20.0,
        hw: 5.0,
        hh: 6.0,
        skew: 0.0,
        edge_tilt: 0.0,
        kind: 1,
        has_video: true,
        favourite: false,
        hidden: false,
        radius: 8.0,
        opacity: 1.0,
    };
    let base = chrome_signature(&[chrome], None, &pal);
    assert_eq!(base, chrome_signature(&[chrome], None, &pal));

    let mut fav = chrome;
    fav.favourite = true;
    assert_ne!(base, chrome_signature(&[fav], None, &pal));

    let mut dim = chrome;
    dim.opacity = 0.5;
    assert_ne!(base, chrome_signature(&[dim], None, &pal));

    let back = BackPanel { progress: 0.5, ..Default::default() };
    let with_back = chrome_signature(&[chrome], Some(&back), &pal);
    assert_ne!(base, with_back);
    let disabled = BackPanel {
        removal: crate::frontend::scene::RemovalAction::UnsubscribeUnavailable,
        ..back.clone()
    };
    let enabled =
        BackPanel { removal: crate::frontend::scene::RemovalAction::Unsubscribe, ..back.clone() };
    assert_ne!(
        chrome_signature(&[chrome], Some(&disabled), &pal),
        chrome_signature(&[chrome], Some(&enabled), &pal)
    );

    let jitter = BackPanel { progress: 0.5 + 0.001, ..Default::default() };
    assert_eq!(with_back, chrome_signature(&[chrome], Some(&jitter), &pal));

    let step = BackPanel { progress: 0.5 + 0.02, ..Default::default() };
    assert_ne!(with_back, chrome_signature(&[chrome], Some(&step), &pal));

    let coordinated = BackPanel { progress: 0.5, coordinated_flip: true, ..Default::default() };
    assert_ne!(with_back, chrome_signature(&[chrome], Some(&coordinated), &pal));

    let generated = BackPanel { progress: 0.5, reset_thumbnail: true, ..Default::default() };
    assert_ne!(with_back, chrome_signature(&[chrome], Some(&generated), &pal));

    let mut pal2 = pal;
    pal2.primary = Color::from_rgb(0.1, 0.2, 0.3);
    assert_ne!(base, chrome_signature(&[chrome], None, &pal2));

    let fill = BackPanel { progress: 0.5, fav_fill: 0.6, ..Default::default() };
    assert_ne!(with_back, chrome_signature(&[chrome], Some(&fill), &pal));

    let morph = BackPanel { progress: 0.5, add_open: 0.5, ..Default::default() };
    assert_ne!(with_back, chrome_signature(&[chrome], Some(&morph), &pal));
    let pop = BackPanel { progress: 0.5, chip_pop: 0.5, pop_idx: 2, ..Default::default() };
    assert_ne!(with_back, chrome_signature(&[chrome], Some(&pop), &pal));

    let rounded = BackPanel { progress: 0.5, radii: [18.0, 12.0, 8.0, 4.0], ..Default::default() };
    assert_ne!(with_back, chrome_signature(&[chrome], Some(&rounded), &pal));
}

#[test]
fn back_actions_wrap() {
    use crate::frontend::scene::BackPanel;
    let narrow = BackPanel {
        cx: 100.0,
        cy: 300.0,
        hw: 80.0,
        hh: 150.0,
        progress: 1.0,
        embedded: true,
        ..Default::default()
    };
    let lay = super::back_layout(&narrow);
    let card_left = narrow.cx - narrow.hw;
    let card_right = narrow.cx + narrow.hw;
    for rect in [lay.actions.playlist, lay.actions.delete] {
        assert!(rect.0 >= card_left - 0.01 && rect.0 + rect.2 <= card_right + 0.01);
    }
    let ys: std::collections::HashSet<u32> =
        [lay.actions.playlist.1, lay.actions.delete.1].iter().map(|y| y.to_bits()).collect();
    assert!(ys.len() > 1);

    let wide = BackPanel {
        cx: 600.0,
        cy: 400.0,
        hw: 500.0,
        hh: 280.0,
        progress: 1.0,
        embedded: true,
        ..Default::default()
    };
    let wide_lay = super::back_layout(&wide);
    assert_eq!(wide_lay.actions.playlist.1, wide_lay.actions.delete.1);
    assert!(wide_lay.actions.overview.is_none());

    let overview = BackPanel { overview_available: true, ..wide };
    assert!(super::back_layout(&overview).actions.overview.is_some());
}

fn rects_overlap(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> bool {
    a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.3 && b.1 < a.1 + a.3
}

#[test]
fn back_layout_no_overlap() {
    use crate::frontend::scene::BackPanel;
    let panel = BackPanel {
        cx: 400.0,
        cy: 300.0,
        hw: 200.0,
        hh: 150.0,
        progress: 1.0,
        embedded: true,
        tags: vec![
            String::from("nature"),
            String::from("sunset"),
            String::from("golden hour"),
            String::from("mountains"),
        ],
        kind_label: String::from("static"),
        fields: vec![(String::from("Size"), String::from("3840x2160"))],
        ..Default::default()
    };
    let lay = super::back_layout(&panel);
    let mut rects: Vec<(&str, (f32, f32, f32, f32))> = Vec::new();
    assert_eq!(lay.tags.len(), panel.tags.len());
    for rect in &lay.tags {
        rects.push(("tag", *rect));
    }
    rects.push(("add", lay.add));
    rects.push(("playlist", lay.actions.playlist));
    rects.push(("delete", lay.actions.delete));
    if let Some(over) = lay.actions.overview {
        rects.push(("overview", over));
    }
    for (_, rect) in &rects {
        assert!(rect.2 > 0.0 && rect.3 > 0.0);
    }
    for i in 0..rects.len() {
        for j in i + 1..rects.len() {
            assert!(
                !rects_overlap(rects[i].1, rects[j].1),
                "{} {} overlap",
                rects[i].0,
                rects[j].0
            );
        }
    }
    let (fx, fy, _) = lay.fav;
    let fav_zone = (fx - 30.0, fy - 26.0, 60.0, 52.0);
    for (name, rect) in &rects {
        assert!(!rects_overlap(fav_zone, *rect), "fav zone hits {name}");
    }
    for (name, rect) in &rects {
        assert!(
            rect.0 >= lay.content_left - 0.01 && rect.0 + rect.2 <= lay.content_right + 0.01,
            "{name} escapes content bounds"
        );
        assert!(
            rect.1 >= lay.sheet.1 - 0.01 && rect.1 + rect.3 <= lay.card.1 + lay.card.3 + 0.01,
            "{name} escapes sheet vertically"
        );
    }
    assert!(lay.sheet.1 > lay.card.1);
}

#[test]
fn landscape_caption_deck() {
    use crate::frontend::scene::BackPanel;
    let panel = BackPanel {
        cx: 640.0,
        cy: 360.0,
        hw: 462.0,
        hh: 260.0,
        skew: 72.0,
        progress: 1.0,
        embedded: true,
        static_img: true,
        overview_available: true,
        title: String::from("wallhaven-6ly5g6"),
        kind_label: String::from("Image"),
        fields: vec![
            (String::from("Resolution"), String::from("1920 x 1080")),
            (String::from("Size"), String::from("169 KB")),
            (String::from("Modified"), String::from("2026-08-10")),
            (String::from("Applied"), String::from("1 times")),
        ],
        tags: vec![String::from("anime")],
        ..Default::default()
    };
    let layout = super::back_layout(&panel);
    assert!(layout.sheet.3 <= layout.card.3 * 0.25);
    assert!(layout.title_right < layout.action_left);
    assert!(layout.tags.iter().all(|tag| tag.0 + tag.2 <= layout.action_left));
    for action in [layout.actions.overview, layout.actions.effects]
        .into_iter()
        .flatten()
        .chain([layout.actions.playlist, layout.actions.delete])
    {
        assert!(action.0 >= layout.action_left);
        assert!(action.1 >= layout.facts_rule_y);
        assert!(action.1 + action.3 <= layout.sheet.1 + layout.sheet.3 + 0.01);
    }
}

#[test]
fn back_rise_stagger() {
    for slot in [0.0f32, 0.7, 1.4, 2.6, 3.6, 4.5, 9.0] {
        let (a0, d0) = super::back_rise(0.0, slot);
        assert_eq!(a0, 0.0);
        assert_eq!(d0, 14.0);
        let (a1, d1) = super::back_rise(1.0, slot);
        assert!((a1 - 1.0).abs() < 1e-6 && d1.abs() < 1e-6, "slot {slot} unfinished");
        let mut prev = 0.0;
        for i in 0..=20 {
            let (a, _) = super::back_rise(i as f32 / 20.0, slot);
            assert!(a >= prev - 1e-6);
            prev = a;
        }
    }
    let (early, _) = super::back_rise(0.3, 0.0);
    let (late, _) = super::back_rise(0.3, 3.6);
    assert!(early > late);
}

#[test]
fn tag_input_morph() {
    use crate::frontend::scene::BackPanel;
    let closed = BackPanel {
        cx: 400.0,
        cy: 300.0,
        hw: 300.0,
        hh: 220.0,
        progress: 1.0,
        embedded: true,
        tags: vec![String::from("nature"), String::from("sunset")],
        ..Default::default()
    };
    let open = BackPanel { add_open: 1.0, ..closed.clone() };
    let closed_lay = super::back_layout(&closed);
    let open_lay = super::back_layout(&open);
    assert!(open_lay.add.2 > closed_lay.add.2);
    assert_eq!(open_lay.tags.len(), open.tags.len());
    let mut rects = open_lay.tags.clone();
    rects.push(open_lay.add);
    rects.push(open_lay.actions.playlist);
    rects.push(open_lay.actions.delete);
    for i in 0..rects.len() {
        for j in i + 1..rects.len() {
            assert!(!rects_overlap(rects[i], rects[j]));
        }
    }
    let mid = BackPanel { add_open: 0.5, ..closed };
    let mid_lay = super::back_layout(&mid);
    assert!(mid_lay.add.2 > closed_lay.add.2 && mid_lay.add.2 < open_lay.add.2);
    assert!(open_lay.actions.delete.1 + open_lay.actions.delete.3 > open_lay.actions_label_cy);
}

#[test]
fn dense_tag_overflow() {
    use crate::frontend::scene::BackPanel;
    let tags: Vec<String> = (0..28).map(|index| format!("descriptive tag {index}")).collect();
    let panel = BackPanel {
        cx: 640.0,
        cy: 360.0,
        hw: 462.0,
        hh: 260.0,
        skew: 72.0,
        progress: 1.0,
        embedded: true,
        tags,
        kind_label: String::from("Image"),
        fields: vec![
            (String::from("Resolution"), String::from("4492 x 1560")),
            (String::from("Size"), String::from("4.2 MB")),
            (String::from("Modified"), String::from("2026-08-06")),
        ],
        ..Default::default()
    };
    let layout = super::back_layout(&panel);
    let overflow = layout.tag_overflow.expect("tag overflow");
    assert_eq!(layout.tags.len() + overflow.4, panel.tags.len());
    assert!(layout.tags.len() < panel.tags.len());
    for rectangle in layout
        .tags
        .iter()
        .copied()
        .chain([(overflow.0, overflow.1, overflow.2, overflow.3), layout.add])
    {
        assert!(rectangle.0 >= layout.content_left - 0.01);
        assert!(rectangle.0 + rectangle.2 <= layout.action_left + 0.01);
        assert!(rectangle.1 >= layout.sheet.1 - 0.01);
        assert!(rectangle.1 + rectangle.3 <= layout.sheet.1 + layout.sheet.3 + 0.01);
    }
}

#[test]
fn sloped_deck_skew_inset() {
    use crate::frontend::scene::BackPanel;
    let panel = |skew: f32| BackPanel {
        cx: 640.0,
        cy: 360.0,
        hw: 462.0,
        hh: 260.0,
        skew,
        progress: 1.0,
        embedded: true,
        kind_label: String::from("Image"),
        fields: vec![(String::from("Resolution"), String::from("1920 x 1080"))],
        tags: vec![String::from("anime")],
        ..Default::default()
    };
    let straight = super::back_layout(&panel(0.0));
    let sloped = super::back_layout(&panel(72.0));
    assert!((sloped.content_left - straight.content_left - 72.0).abs() < 0.5, "leading edge");
    assert!((straight.content_right - sloped.content_right - 72.0).abs() < 0.5, "trailing edge");
}

#[test]
fn cramped_tag_row_overflow() {
    use crate::frontend::scene::BackPanel;
    let panel = BackPanel {
        cx: 700.0,
        cy: 400.0,
        hw: 700.0,
        hh: 150.0,
        progress: 1.0,
        add_open: 1.0,
        kind_label: String::from("Image"),
        fields: vec![
            (String::from("Resolution"), String::from("3840 x 2160")),
            (String::from("Size"), String::from("8.2 MB")),
            (String::from("Modified"), String::from("2026-08-06")),
            (String::from("Applied"), String::from("3 times")),
        ],
        tags: (0..12).map(|index| format!("descriptive tag {index}")).collect(),
        ..Default::default()
    };
    let layout = super::back_layout(&panel);
    assert!(layout.tags.is_empty());
    let overflow = layout.tag_overflow.expect("tag overflow");
    assert_eq!(overflow.4, panel.tags.len());
    assert_eq!(overflow.1, layout.add.1);
    let half_row = (layout.content_right - layout.content_left) * 0.5 + 0.5;
    assert!(overflow.2 <= half_row);
    assert!(layout.add.2 <= half_row);
    assert!(layout.add.2 >= 24.0);
    assert!(overflow.0 >= layout.content_left - 0.01);
    assert!(layout.add.0 + layout.add.2 <= layout.content_right + 0.01);
}

#[test]
fn external_back_left_rail() {
    use crate::frontend::scene::BackPanel;
    let panel = BackPanel {
        cx: 800.0,
        cy: 450.0,
        hw: 720.0,
        hh: 390.0,
        progress: 1.0,
        static_img: true,
        overview_available: true,
        kind_label: String::from("Image"),
        fields: vec![
            (String::from("Resolution"), String::from("3840 x 2160")),
            (String::from("Size"), String::from("8.2 MB")),
            (String::from("Modified"), String::from("2026-08-26")),
            (String::from("Applied"), String::from("9 times")),
        ],
        tags: vec![String::from("nature"), String::from("editorial"), String::from("monochrome")],
        ..Default::default()
    };
    let layout = super::back_layout(&panel);
    assert!(!panel.embedded);
    assert!(layout.sheet.2 < layout.card.2 * 0.43);
    assert_eq!(layout.masthead.0, layout.sheet.0);
    assert_eq!(layout.masthead.2, layout.sheet.2);
    assert_eq!(layout.masthead.1 + layout.masthead.3, layout.sheet.1);
    assert_eq!(layout.sheet.1 + layout.sheet.3, layout.card.1 + layout.card.3);
    assert!(layout.title_left >= layout.content_left);
    assert!(layout.title_right <= layout.content_right);
    assert_eq!(layout.action_deck.0, layout.sheet.0);
    assert_eq!(layout.action_deck.2, layout.sheet.2);
    assert!(layout.action_deck.1 + layout.action_deck.3 <= layout.card.1 + layout.card.3);
    let rail_right = layout.sheet.0 + layout.sheet.2;
    let (fav_x, _, _) = layout.fav;
    assert!(fav_x <= layout.content_right);
    for rectangle in [
        layout.actions.playlist,
        layout.actions.delete,
        layout.actions.effects.expect("effects"),
        layout.actions.overview.expect("overview"),
    ] {
        assert!(rectangle.0 >= layout.action_left - 0.01);
        assert!(rectangle.0 + rectangle.2 <= layout.action_right + 0.01);
        assert!(rectangle.0 + rectangle.2 <= rail_right + 0.01);
        assert!(rectangle.1 >= layout.action_deck.1);
        assert!(rectangle.1 + rectangle.3 <= layout.action_deck.1 + layout.action_deck.3);
    }
    for rectangle in layout.tags.iter().copied().chain([layout.add]) {
        assert!(rectangle.0 >= layout.content_left - 0.01);
        assert!(rectangle.0 + rectangle.2 <= layout.content_right + 0.01);
        assert!(rectangle.1 + rectangle.3 <= layout.action_deck.1 + 0.01);
    }
}

#[test]
fn compact_back_sections() {
    use crate::frontend::scene::BackPanel;
    let panel = BackPanel {
        cx: 360.0,
        cy: 200.0,
        hw: 320.0,
        hh: 168.0,
        progress: 1.0,
        static_img: true,
        overview_available: true,
        fields: vec![
            (String::from("Resolution"), String::from("2560 x 1440")),
            (String::from("Size"), String::from("5.2 MB")),
            (String::from("Modified"), String::from("2026-07-23")),
            (String::from("Applied"), String::from("9 times")),
        ],
        tags: (0..8).map(|index| format!("tag {index}")).collect(),
        ..Default::default()
    };
    let layout = super::back_layout(&panel);
    assert!(layout.facts[1].0 > layout.facts[0].0);
    assert_eq!(layout.facts[2].0, layout.facts[0].0);
    assert!(layout.facts_rule_y < layout.tags_label_cy);
    assert!(layout.tags.iter().all(|tag| tag.1 + tag.3 <= layout.action_deck.1 + 0.01));
    assert!(layout.add.1 + layout.add.3 <= layout.action_deck.1 + 0.01);
}

fn chrome_at(view: u8, skew: f32) -> crate::frontend::scene::Chrome {
    crate::frontend::scene::Chrome {
        view,
        cx: 400.0,
        cy: 300.0,
        hw: 120.0,
        hh: 80.0,
        skew,
        edge_tilt: 0.0,
        kind: 1,
        has_video: true,
        favourite: true,
        hidden: false,
        radius: 8.0,
        opacity: 1.0,
    }
}

#[test]
fn slice_badges_swap_column_under_rtl() {
    let badge = 40.0;
    let sloped = chrome_at(0, 30.0);
    let (indicator, badge_x) = super::badges::slice_corners(false, &sloped, badge);
    assert_eq!(indicator, sloped.cx + sloped.hw - 21.0);
    assert_eq!(badge_x, sloped.cx + sloped.hw - badge - 30.0 - 8.0);
    let (indicator, badge_x) = super::badges::slice_corners(true, &sloped, badge);
    assert_eq!(indicator, sloped.cx - sloped.hw + 30.0 + 21.0);
    assert_eq!(badge_x, sloped.cx - sloped.hw + 8.0);

    let leaning = chrome_at(0, -30.0);
    let (indicator, badge_x) = super::badges::slice_corners(false, &leaning, badge);
    assert_eq!(indicator, leaning.cx - leaning.hw + 21.0);
    assert_eq!(badge_x, leaning.cx - leaning.hw + 30.0 + 8.0);
    let (indicator, badge_x) = super::badges::slice_corners(true, &leaning, badge);
    assert_eq!(indicator, leaning.cx + leaning.hw - 30.0 - 21.0);
    assert_eq!(badge_x, leaning.cx + leaning.hw - badge - 8.0);
}

#[test]
fn grid_and_hex_marks_mirror_about_card_center() {
    let badge = 36.0;
    let grid = chrome_at(1, 0.0);
    let (badge_x, indicator, fav) = super::badges::grid_corners(false, &grid, badge);
    assert_eq!(badge_x, grid.cx - grid.hw + 4.0);
    assert_eq!(indicator, grid.cx - grid.hw + 13.0);
    assert_eq!(fav, grid.cx + grid.hw - 11.0);
    let (badge_x, indicator, fav) = super::badges::grid_corners(true, &grid, badge);
    assert_eq!(badge_x + badge, grid.cx + grid.hw - 4.0);
    assert_eq!(indicator, grid.cx + grid.hw - 13.0);
    assert_eq!(fav, grid.cx - grid.hw + 11.0);

    let hex = chrome_at(2, 0.0);
    let ltr = super::badges::hex_indicator_x(false, &hex);
    let rtl = super::badges::hex_indicator_x(true, &hex);
    assert_eq!(ltr, hex.cx + hex.hw * 0.5 - 14.0);
    assert_eq!(rtl, hex.cx * 2.0 - ltr);
}

fn mirrored_rect(card: (f32, f32, f32, f32), rect: (f32, f32, f32, f32)) -> (f32, f32, f32, f32) {
    (card.0 + card.2 - (rect.0 - card.0) - rect.2, rect.1, rect.2, rect.3)
}

fn assert_rect_mirrored(
    name: &str,
    card: (f32, f32, f32, f32),
    ltr: (f32, f32, f32, f32),
    rtl: (f32, f32, f32, f32),
) {
    let expected = mirrored_rect(card, ltr);
    for (a, b) in
        [(expected.0, rtl.0), (expected.1, rtl.1), (expected.2, rtl.2), (expected.3, rtl.3)]
    {
        assert!((a - b).abs() < 0.01, "{name} not mirrored: {ltr:?} -> {rtl:?}");
    }
}

#[test]
fn back_layout_mirrors_about_the_card() {
    use crate::frontend::scene::BackPanel;
    let panels = [
        BackPanel {
            cx: 640.0,
            cy: 360.0,
            hw: 462.0,
            hh: 260.0,
            skew: 72.0,
            progress: 1.0,
            embedded: true,
            static_img: true,
            overview_available: true,
            tags: vec![String::from("anime"), String::from("sunset"), String::from("city")],
            fields: vec![(String::from("Size"), String::from("1 MB"))],
            ..Default::default()
        },
        BackPanel {
            cx: 400.0,
            cy: 300.0,
            hw: 200.0,
            hh: 150.0,
            progress: 1.0,
            embedded: true,
            tags: vec![String::from("nature"), String::from("golden hour")],
            fields: vec![(String::from("Size"), String::from("3840x2160"))],
            ..Default::default()
        },
        BackPanel {
            cx: 800.0,
            cy: 450.0,
            hw: 720.0,
            hh: 390.0,
            progress: 1.0,
            static_img: true,
            overview_available: true,
            reset_thumbnail: true,
            revealable: true,
            tags: vec![String::from("nature"), String::from("editorial")],
            fields: vec![(String::from("Resolution"), String::from("3840 x 2160"))],
            ..Default::default()
        },
    ];
    for panel in &panels {
        let ltr = super::back_layout_for(panel, false);
        let rtl = super::back_layout_for(panel, true);
        assert!(!ltr.rtl && rtl.rtl);
        let card = ltr.card;
        assert_eq!(rtl.card, card);
        let card_right = card.0 + card.2;
        for (name, a, b) in [
            ("masthead", ltr.masthead, rtl.masthead),
            ("sheet", ltr.sheet, rtl.sheet),
            ("deck", ltr.action_deck, rtl.action_deck),
            ("add", ltr.add, rtl.add),
            ("playlist", ltr.actions.playlist, rtl.actions.playlist),
            ("delete", ltr.actions.delete, rtl.actions.delete),
        ] {
            assert_rect_mirrored(name, card, a, b);
        }
        for (a, b) in ltr.tags.iter().zip(&rtl.tags) {
            assert_rect_mirrored("tag", card, *a, *b);
        }
        for (a, b) in ltr.facts.iter().zip(&rtl.facts) {
            assert_rect_mirrored("fact", card, *a, *b);
        }
        for (a, b) in [
            (ltr.actions.overview, rtl.actions.overview),
            (ltr.actions.effects, rtl.actions.effects),
            (ltr.actions.reset_thumbnail, rtl.actions.reset_thumbnail),
            (ltr.actions.reveal, rtl.actions.reveal),
        ] {
            assert_eq!(a.is_some(), b.is_some());
            if let (Some(a), Some(b)) = (a, b) {
                assert_rect_mirrored("action", card, a, b);
            }
        }
        assert!((rtl.content_left - (card.0 + card_right - ltr.content_right)).abs() < 0.01);
        assert!((rtl.content_right - (card.0 + card_right - ltr.content_left)).abs() < 0.01);
        assert!((rtl.title_left - (card.0 + card_right - ltr.title_right)).abs() < 0.01);
        assert!((rtl.action_right - (card.0 + card_right - ltr.action_left)).abs() < 0.01);
        assert!((rtl.fav.0 - (card.0 + card_right - ltr.fav.0)).abs() < 0.01);
        assert_eq!(rtl.fav.1, ltr.fav.1);
        assert_eq!(rtl.kicker_cy, ltr.kicker_cy);
        assert_eq!(rtl.title_cy, ltr.title_cy);
        assert_eq!(rtl.facts_rule_y, ltr.facts_rule_y);
        assert_eq!(rtl.tags_label_cy, ltr.tags_label_cy);
        assert_eq!(rtl.actions_label_cy, ltr.actions_label_cy);
        if rtl.tags.len() > 1 && rtl.tags[0].1 == rtl.tags[1].1 {
            assert!(rtl.tags[1].0 + rtl.tags[1].2 <= rtl.tags[0].0 + 0.01);
        }
        for rect in rtl.tags.iter().chain([&rtl.add, &rtl.actions.playlist, &rtl.actions.delete]) {
            assert!(
                rect.0 >= rtl.content_left - 0.01 && rect.0 + rect.2 <= rtl.content_right + 0.01
            );
        }
    }
    let external = super::back_layout_for(&panels[2], true);
    assert!((external.sheet.0 + external.sheet.2 - (panels[2].cx + panels[2].hw)).abs() < 0.01);
    assert!(external.action_left >= external.sheet.0);
}

#[test]
fn mirrored_back_hit_resolves_same_target() {
    use crate::frontend::scene::BackPanel;
    let panel = BackPanel {
        cx: 640.0,
        cy: 360.0,
        hw: 462.0,
        hh: 260.0,
        skew: 72.0,
        edge_tilt: 40.0,
        progress: 1.0,
        embedded: true,
        static_img: true,
        overview_available: true,
        tags: vec![String::from("anime"), String::from("sunset")],
        fields: vec![(String::from("Size"), String::from("1 MB"))],
        ..Default::default()
    };
    let flat = BackPanel { skew: 0.0, edge_tilt: 0.0, ..panel.clone() };
    for panel in [&panel, &flat] {
        let ltr = super::back_layout_for(panel, false);
        let rtl = super::back_layout_for(panel, true);
        for (a, b) in [
            (ltr.actions.delete, rtl.actions.delete),
            (ltr.actions.playlist, rtl.actions.playlist),
            (ltr.add, rtl.add),
            (ltr.tags[1], rtl.tags[1]),
            (ltr.actions.overview.unwrap(), rtl.actions.overview.unwrap()),
        ] {
            let (lx, ly, lw, lh) = super::back_bounds(panel, &ltr, a);
            let (rx, ry, rw, rh) = super::back_bounds(panel, &rtl, b);
            assert!((lw - rw).abs() < 0.01 && (lh - rh).abs() < 0.01);
            if panel.skew == 0.0 && panel.edge_tilt == 0.0 {
                assert!((ly - ry).abs() < 0.01);
                assert!((rx - (panel.cx * 2.0 - lx - lw)).abs() < 0.01);
            }
            let (cx, cy) = (rx + rw * 0.5, ry + rh * 0.5);
            assert!(super::back_contains(panel, &rtl, b, cx, cy));
            assert!(!super::back_contains(panel, &ltr, a, cx, cy));
            assert!(!super::back_contains(panel, &rtl, b, lx + lw * 0.5, ly + lh * 0.5));
        }
    }
}

#[test]
fn hide_beside_favourite() {
    use crate::frontend::scene::BackPanel;
    let panel = BackPanel {
        cx: 400.0,
        cy: 300.0,
        hw: 200.0,
        hh: 150.0,
        progress: 1.0,
        ..Default::default()
    };
    let ltr = super::back_layout_for(&panel, false);
    let rtl = super::back_layout_for(&panel, true);
    let button = ltr.masthead.3 - super::BACK_TOGGLE_INSET;
    assert_eq!(ltr.hide.1, ltr.fav.1);
    assert!(ltr.fav.0 - ltr.hide.0 > button);
    assert!(rtl.hide.0 - rtl.fav.0 > button);
}
