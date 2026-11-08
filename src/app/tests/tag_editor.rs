use super::*;

#[test]
fn tag_editor_commits() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    app.scene.set_flipped_for_test(Some(0));
    app.tags.editing = true;
    super::begin_card_tag_edit(&mut app, &[]);
    let _ = update(&mut app, Message::Tag(TagMsg::InputChanged(String::from("beach "))));
    assert_eq!(
        app.library_session.library.catalog().tags.get("a.png").map(Vec::as_slice),
        Some(&[String::from("beach")][..])
    );
    assert!(app.tags.input.is_empty());
    assert_eq!(app.tags.card_locked, ["beach"]);
    assert!(app.tags.editing);

    app.scene.toggle_flip(0);
    app.tags.input = String::from("Calm");
    let _ = update(&mut app, Message::Exit);
    assert_eq!(
        app.library_session.library.catalog().tags["a.png"],
        vec![String::from("beach"), String::from("calm")]
    );
    assert!(!app.tags.editing);
    assert!(app.scene.flip_open());
    let _ = update(&mut app, Message::Exit);
    assert!(!app.scene.flip_open());
}

#[test]
fn tag_editor_locks_on_space() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    app.scene.set_flipped_for_test(Some(0));
    app.tags.editing = true;
    super::begin_card_tag_edit(&mut app, &[]);

    let _ = update(&mut app, Message::Tag(TagMsg::InputChanged(String::from("beach calm "))));
    assert_eq!(app.library_session.library.catalog().tags["a.png"], ["beach", "calm"]);
    assert!(app.tags.input.is_empty());
    assert_eq!(app.tags.card_locked, ["beach", "calm"]);

    let _ = update(&mut app, Message::Tag(TagMsg::InputChanged(String::from("ultraviolet"))));
    assert_eq!(app.library_session.library.catalog().tags["a.png"], ["beach", "calm"]);
    assert_eq!(app.tags.input, "ultraviolet");
    let _ = update(&mut app, Message::Tag(TagMsg::InputChanged(String::from("ultraviolet "))));
    assert_eq!(
        app.library_session.library.catalog().tags["a.png"],
        ["beach", "calm", "ultraviolet"]
    );
    assert!(app.tags.input.is_empty());
    assert_eq!(app.tags.card_locked, ["beach", "calm", "ultraviolet"]);
}

#[test]
fn known_partial_waits_for_space() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    edit_catalog(&mut app, |catalog| {
        catalog.tag_vocab = [String::from("a"), String::from("aircraft")].into_iter().collect();
    });
    app.scene.set_flipped_for_test(Some(0));
    app.tags.editing = true;
    super::begin_card_tag_edit(&mut app, &[]);

    let _ = update(&mut app, Message::Tag(TagMsg::InputChanged(String::from("a"))));
    assert!(app.library_session.library.catalog().tags.get("a.png").is_none_or(Vec::is_empty));
    assert_eq!(app.tags.input, "a");

    let _ = update(&mut app, Message::Tag(TagMsg::InputChanged(String::from("a "))));
    assert_eq!(app.library_session.library.catalog().tags["a.png"], ["a"]);
}

#[test]
fn tag_focus_deferred() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    app.scene.viewport = (800.0, 600.0);
    app.scene.set_visible(true);
    app.scene.toggle_flip(0);
    app.tags.editing = true;
    app.tags.focus_pending = true;
    app.scene.set_tag_editing(true);
    let t0 = Instant::now();
    frame(&mut app, t0, 800.0, 600.0);
    assert!(app.tags.focus_pending);
    for i in 1..=60u64 {
        frame(&mut app, t0 + Duration::from_millis(i * 16), 800.0, 600.0);
    }
    let dbg = (
        app.scene.render.back.as_ref().map(|back| back.add_open),
        app.scene.render.instances.len(),
        app.scene.render.vis,
        app.scene.flipped(),
    );
    assert!(!app.tags.focus_pending, "back={dbg:?}");
}

#[test]
fn tag_chip_clicks() {
    let mut app = test_app();
    let mut a = wall("a.png", "static", 1, 0);
    a["tags"] = json!("calm,vibrant,serene");
    seed(&mut app, &[a]);
    app.scene.viewport = (800.0, 600.0);
    app.scene.set_visible(true);
    app.scene.toggle_flip(0);
    let t0 = Instant::now();
    for i in 0..=30u64 {
        frame(&mut app, t0 + Duration::from_millis(i * 16), 800.0, 600.0);
    }
    let chip = |app: &App, idx: usize| {
        let panel = app.scene.render.back.clone().expect("back panel present");
        let layout = crate::frontend::ui::back_layout(&panel);
        let (rx, ry, rw, rh) = layout.tags[idx];
        (rx + rw - 17.0, ry + rh / 2.0)
    };
    let (x, y) = chip(&app, 0);
    let _ = update(&mut app, Message::Click(x, y, crate::domain::input::MouseButton::Left));
    assert_eq!(
        app.library_session.library.catalog().tags["a.png"],
        vec![String::from("vibrant"), String::from("serene")]
    );
    frame(&mut app, t0 + Duration::from_millis(496), 800.0, 600.0);
    let (x, y) = chip(&app, 0);
    let _ = update(&mut app, Message::Click(x, y, crate::domain::input::MouseButton::Right));
    assert_eq!(app.library_session.library.catalog().tags["a.png"], vec![String::from("serene")]);
    frame(&mut app, t0 + Duration::from_millis(512), 800.0, 600.0);
    app.tags.editing = true;
    let (x, y) = chip(&app, 0);
    let _ = update(&mut app, Message::Click(x, y, crate::domain::input::MouseButton::Left));
    assert!(app.library_session.library.catalog().tags["a.png"].is_empty());
}

#[test]
fn dense_tag_summary_discloses_inventory() {
    let mut app = test_app();
    let tags: Vec<String> = (0..28).map(|index| format!("folio-tag-{index:02}")).collect();
    let mut item = wall("a.png", "static", 1, 0);
    item["tags"] = json!(tags.join(","));
    seed(&mut app, &[item]);
    app.scene.viewport = (800.0, 600.0);
    app.scene.set_visible(true);
    app.scene.toggle_flip(0);
    let t0 = Instant::now();
    for i in 0..=30u64 {
        frame(&mut app, t0 + Duration::from_millis(i * 16), 800.0, 600.0);
    }

    let panel = app.scene.render.back.clone().expect("back panel present");
    let layout = crate::frontend::ui::back_layout(&panel);
    let overflow = layout.tag_overflow.expect("overflow chip");
    let _ = update(
        &mut app,
        Message::Click(
            overflow.0 + overflow.2 * 0.5,
            overflow.1 + overflow.3 * 0.5,
            crate::domain::input::MouseButton::Left,
        ),
    );
    assert!(app.tags.card_drawer_open);
    assert!(!app.tags.editing);
    assert_eq!(app.tags.card_locked.len(), 28);
    assert_eq!(app.tags.card_key.as_deref(), Some("a.png"));

    let _ = update(&mut app, Message::Tag(TagMsg::InputChanged(String::from("drawer-added "))));
    assert!(app.tags.card_locked.contains(&String::from("drawer-added")));

    let _ = update(&mut app, Message::Tag(TagMsg::Remove(20)));
    assert_eq!(app.tags.card_locked.len(), 28);
    assert!(!app.library_session.library.catalog().tags["a.png"].contains(&tags[20]));

    let _ = update(&mut app, Message::Exit);
    assert!(!app.tags.card_drawer_open);
    assert!(app.scene.flip_open());
}

#[test]
fn cloud_fit_widths() {
    use super::view::cloud_fit_width;
    assert_eq!(cloud_fit_width(None, Some(1200.0), 760.0, 2000.0), 874.0);
    assert_eq!(cloud_fit_width(None, Some(500.0), 760.0, 2000.0), 760.0);
    assert_eq!(cloud_fit_width(Some(900.0), Some(1400.0), 760.0, 2000.0), 900.0);
    assert_eq!(cloud_fit_width(None, Some(3000.0), 760.0, 1000.0), 874.0);
    assert_eq!(cloud_fit_width(None, None, 760.0, 2000.0), 760.0);
}

#[test]
fn scaled_cloud_widths() {
    use super::view::cloud_fit_width;
    for (base, scale, expected) in [
        (760.0_f32, 1.3, 987.99994),
        (760.0, 1.5, 1140.0),
        (760.0, 2.0, 1520.0),
        (540.0, 2.0, 1080.0),
    ] {
        for content in [500.0, 1200.0, 3000.0] {
            assert_eq!(cloud_fit_width(None, Some(content), base * scale, 1920.0), expected);
        }
    }
}

#[test]
fn cloud_width_scale_range() {
    use super::view::cloud_fit_width;
    for scale in [0.5, 0.75, 1.0, 1.2, 1.21, 1.22, 1.3, 1.5, 2.0] {
        for base in [360.0, 540.0, 760.0, 2200.0] {
            let width = base * scale;
            for content in [0.0, 500.0, 1200.0, 5000.0] {
                let fitted = cloud_fit_width(None, Some(content), width, 5000.0);
                assert!(fitted.is_finite() && (width..=4960.0).contains(&fitted));
            }
        }
    }
}

#[test]
fn scaled_cloud_fits_viewport() {
    use super::view::cloud_fit_width;
    let width = 760.0_f32 * 1.3;
    for content in [None, Some(500.0), Some(1200.0)] {
        assert_eq!(cloud_fit_width(None, content, width, 1000.0), 960.0);
        assert_eq!(cloud_fit_width(Some(900.0), content, width, 1000.0), 900.0);
    }
}

#[test]
fn tag_search_height_tracks_rows() {
    use super::view::cloud_fit_height;
    assert_eq!(cloud_fit_height(false, false, 3, 8, 768.0, 1.0), 126.0);
    assert_eq!(cloud_fit_height(false, true, 3, 8, 768.0, 1.0), 126.0);
    assert_eq!(cloud_fit_height(true, false, 2, 1, 768.0, 1.0), 161.0);
    assert_eq!(cloud_fit_height(true, false, 2, 8, 768.0, 1.0), 194.0);
    assert_eq!(cloud_fit_height(true, true, 2, 8, 768.0, 1.0), 194.0);
    assert_eq!(cloud_fit_height(true, false, 3, 8, 768.0, 1.0), 227.0);
    assert_eq!(cloud_fit_height(true, false, 1, 8, 768.0, 1.0), 161.0);
}
