use super::*;

const PICKER: crate::domain::input::ActiveScopes =
    crate::domain::input::ActiveScopes { fields: false, search: false, downloads: false };

#[test]
fn key_event_routing() {
    use iced::event::Status;
    use keyboard::key::Named;
    let win = iced::window::Id::unique();
    let named = |key: Named| keyboard::Key::Named(key);
    assert!(matches!(
        on_event(key_press(named(Named::Enter), false), Status::Ignored, win),
        Some(Message::KeyPressed(key, _)) if key == named(Named::Enter)
    ));
    assert!(matches!(
        on_event(key_press(named(Named::Escape), false), Status::Ignored, win),
        Some(Message::KeyPressed(key, _)) if key == named(Named::Escape)
    ));
    assert!(on_event(key_press(named(Named::Enter), false), Status::Captured, win).is_none());
    assert!(matches!(
        on_event(key_press(named(Named::Escape), false), Status::Captured, win),
        Some(Message::KeyPressed(key, _)) if key == named(Named::Escape)
    ));
    assert!(matches!(
        on_event(key_press(named(Named::Tab), false), Status::Captured, win),
        Some(Message::KeyPressed(key, _)) if key == named(Named::Tab)
    ));
    assert!(settings_key_message(&named(Named::Home), keyboard::Modifiers::default()).is_none());
    assert!(settings_key_message(&named(Named::PageUp), keyboard::Modifiers::default()).is_none());
    assert!(matches!(
        settings_key_message(&keyboard::Key::Character("/".into()), keyboard::Modifiers::default()),
        Some(Message::Settings(crate::frontend::settings::SettingsMsg::Key(
            crate::frontend::settings::SettingsKey::Search
        )))
    ));
    assert!(matches!(
        settings_key_message(&keyboard::Key::Character("f".into()), keyboard::Modifiers::CTRL),
        Some(Message::Settings(crate::frontend::settings::SettingsMsg::Key(
            crate::frontend::settings::SettingsKey::Search
        )))
    ));
    assert!(
        settings_key_message(&keyboard::Key::Character("k".into()), keyboard::Modifiers::CTRL)
            .is_none()
    );
    assert!(matches!(
        settings_key_message(&named(Named::Tab), keyboard::Modifiers::CTRL),
        Some(Message::Settings(crate::frontend::settings::SettingsMsg::Key(
            crate::frontend::settings::SettingsKey::CategoryNext { backwards: false }
        )))
    ));
    assert!(matches!(
        settings_key_message(
            &named(Named::Tab),
            keyboard::Modifiers::CTRL | keyboard::Modifiers::SHIFT
        ),
        Some(Message::Settings(crate::frontend::settings::SettingsMsg::Key(
            crate::frontend::settings::SettingsKey::CategoryNext { backwards: true }
        )))
    ));
    assert!(matches!(
        settings_key_message(&keyboard::Key::Character(" ".into()), keyboard::Modifiers::default()),
        Some(Message::Settings(crate::frontend::settings::SettingsMsg::Key(
            crate::frontend::settings::SettingsKey::Activate
        )))
    ));
    let km = crate::domain::input::InputMap::default();
    let none = keyboard::Modifiers::default();
    let shift = keyboard::Modifiers::SHIFT;
    assert!(matches!(
        key_message(&km, &keyboard::Key::Character("s".into()), shift, PICKER),
        Some(Message::ToggleSettings)
    ));
    assert!(key_message(&km, &keyboard::Key::Character("s".into()), none, PICKER).is_none());
    assert!(matches!(
        key_message(&km, &named(Named::ArrowLeft), none, PICKER),
        Some(Message::KeyPrev)
    ));
    assert!(matches!(
        key_message(&km, &named(Named::ArrowRight), none, PICKER),
        Some(Message::KeyNext)
    ));
    assert!(matches!(
        key_message(&km, &named(Named::ArrowLeft), shift, PICKER),
        Some(Message::SetColorFilter(i64::MIN))
    ));
    assert!(matches!(
        key_message(&km, &named(Named::ArrowRight), shift, PICKER),
        Some(Message::SetColorFilter(i64::MAX))
    ));
    assert!(matches!(
        key_message(&km, &named(Named::ArrowUp), shift, PICKER),
        Some(Message::ToggleFilterBar)
    ));
    assert!(matches!(
        key_message(&km, &named(Named::ArrowDown), shift, PICKER),
        Some(Message::OpenTagCloud)
    ));
    assert!(matches!(
        key_message(&km, &keyboard::Key::Character("p".into()), none, PICKER),
        Some(Message::OpenPlaylists)
    ));
    assert!(matches!(
        key_message(&km, &keyboard::Key::Character("c".into()), none, PICKER),
        Some(Message::ToggleThemePanel)
    ));
    assert!(matches!(
        key_message(&km, &keyboard::Key::Character("f".into()), none, PICKER),
        Some(Message::KeyFavourite)
    ));
    assert!(key_message(&km, &keyboard::Key::Character("i".into()), none, PICKER).is_none());
    assert!(key_message(&km, &keyboard::Key::Character("e".into()), none, PICKER).is_none());
    assert!(matches!(
        key_message(&km, &keyboard::Key::Character("?".into()), shift, PICKER),
        Some(Message::ToggleHelp)
    ));
}

#[test]
fn colour_panel_shortcut_toggles_the_bottom_bar_panel() {
    let mut app = test_app();
    app.chrome.filter_bar_visible = false;
    assert!(!app.theme.bar_open);
    assert!(!app.chrome.filter_bar_visible);

    let _ = update(
        &mut app,
        Message::KeyPressed(keyboard::Key::Character("c".into()), keyboard::Modifiers::default()),
    );
    assert!(app.theme.bar_open);
    assert!(app.chrome.filter_bar_visible);

    let _ = update(
        &mut app,
        Message::KeyPressed(keyboard::Key::Character("c".into()), keyboard::Modifiers::default()),
    );
    assert!(!app.theme.bar_open);
}

#[test]
fn opened_event_carries_viewport() {
    let id = iced::window::Id::unique();
    let event = iced::Event::Window(iced::window::Event::Opened {
        position: None,
        size: iced::Size::new(2560.0, 1440.0),
    });
    assert!(matches!(
        on_event(event, iced::event::Status::Ignored, id),
        Some(Message::WindowOpened { id: opened, width: 2560.0, height: 1440.0 }) if opened == id
    ));
}

#[test]
fn esc_closes_overlays() {
    let mut app = test_app();
    app.panels.audio = Some(crate::frontend::audio_panel::AudioPanel::new());
    let _ = update(&mut app, Message::Exit);
    assert!(app.panels.audio.is_none());
    app.panels.playlists = Some(crate::frontend::playlists::Playlists::default());
    let _ = update(&mut app, Message::Exit);
    assert!(app.panels.playlists.is_none());
    app.input.help_open = true;
    app.panels.settings.open = true;
    let _ = update(&mut app, Message::Exit);
    assert!(!app.input.help_open);
    assert!(app.panels.settings.open);
    app.panels.settings.open = false;
}

#[test]
fn theme_designer_owns_keyboard() {
    use keyboard::key::Named;

    let mut app = test_app();
    app.panels.settings.open = true;
    app.panels.settings.section = 0;
    app.panels.settings.focus = crate::frontend::settings::SettingsFocus::Sections;
    crate::app::helpers::theme_designer_open(&mut app);
    assert!(app.panels.theme_designer.is_some());

    let before = (
        app.panels.settings.tab.clone(),
        app.panels.settings.section,
        app.panels.settings.focus,
        app.panels.settings.focused_control,
    );
    for key in [Named::Tab, Named::ArrowDown, Named::ArrowRight] {
        let _ = update(
            &mut app,
            Message::KeyPressed(keyboard::Key::Named(key), keyboard::Modifiers::default()),
        );
    }
    assert_eq!(
        (
            app.panels.settings.tab.clone(),
            app.panels.settings.section,
            app.panels.settings.focus,
            app.panels.settings.focused_control,
        ),
        before
    );

    let _ = update(
        &mut app,
        Message::KeyPressed(keyboard::Key::Named(Named::Escape), keyboard::Modifiers::default()),
    );
    assert!(app.panels.theme_designer.is_none());
    assert!(app.panels.settings.open);
}

#[test]
fn esc_closes_flip() {
    let mut app = test_app();
    seed(&mut app, &[wall("a", "static", 10, 1), wall("b", "static", 20, 2)]);
    app.scene.mode = Mode::Slices;
    app.scene.toggle_flip(0);
    assert!(app.scene.flip_open());
    let _ = update(&mut app, Message::Exit);
    assert!(!app.scene.flip_open());
}

#[test]
fn overlays_capture_input() {
    let mut app = test_app();
    seed(
        &mut app,
        &[wall("a", "static", 10, 1), wall("b", "static", 20, 2), wall("c", "static", 30, 3)],
    );
    app.panels.playlists = Some(crate::frontend::playlists::Playlists::default());
    let cur = app.scene.current;
    let color = app.library_session.filters.color;
    let bar = app.chrome.filter_bar_visible;
    let _ = update(&mut app, Message::KeyNext);
    assert_eq!(app.scene.current, cur);
    let _ = update(&mut app, Message::MouseMoved(100.0, 100.0));
    let _ = update(&mut app, Message::MouseMoved(300.0, 300.0));
    assert!(app.scene.hover.is_none());
    let _ = update(&mut app, Message::SetColorFilter(5));
    assert_eq!(app.library_session.filters.color, color);
    let _ = update(&mut app, Message::ToggleFilterBar);
    assert_eq!(app.chrome.filter_bar_visible, bar);
    let _ = update(&mut app, Message::OpenTagCloud);
    assert!(!app.tags.cloud_open);
    let _ = update(&mut app, Message::ToggleHelp);
    assert!(!app.input.help_open);
}

#[test]
fn random_rotate_toggle() {
    let mut app = test_app();
    assert!(!app.config.flag_default_config("general.randomRotate"));
    drain_calls(&app);
    let _ = update(&mut app, Message::ToggleRandomRotate);
    assert!(app.config.flag_default_config("general.randomRotate"));
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "wall.rotation_wake"));
    assert!(app.runtime_state.toast.is_some());
    let _ = update(&mut app, Message::ToggleRandomRotate);
    assert!(!app.config.flag_default_config("general.randomRotate"));
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "wall.rotation_wake"));
}

#[test]
fn shell_hover_preview() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0), wall("b.png", "static", 2, 0)]);
    assert!(app.shell_hover_enabled());
    for off in ["static", "off"] {
        app.config.set_key("theme.backend", json!(off));
        assert!(!app.shell_hover_enabled(), "{off}");
    }
    app.config.set_key("theme.backend", json!("noctalia"));
    assert!(app.shell_hover_enabled());
    app.config.set_key("noctalia.hoverPreview", json!(false));
    assert!(!app.shell_hover_enabled());
    app.config.set_key("noctalia.hoverPreview", json!(true));

    drain_calls(&app);
    app.step_shell_preview(0);
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, params)| {
        method == "wall.shell_preview" && params["path"].as_str() == Some("/thumbs/a.png.png")
    }));
    app.step_shell_preview(0);
    assert!(drain_calls(&app).is_empty());
    app.step_shell_preview(1);
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "wall.shell_preview"));
}

#[test]
fn hidden_picker_stops_preview() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0), wall("b.png", "static", 2, 0)]);
    app.scene.hover = Some(1);
    let t0 = Instant::now();
    frame(&mut app, t0, 1280.0, 720.0);
    frame(&mut app, t0 + Duration::from_millis(60), 1280.0, 720.0);
    frame(&mut app, t0 + Duration::from_millis(460), 1280.0, 720.0);
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "wall.shell_preview"));

    app.on_hidden();
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "wall.shell_preview_end"));

    frame(&mut app, Instant::now(), 1280.0, 720.0);
    assert!(!drain_calls(&app).iter().any(|(method, _)| method == "wall.shell_preview"));
}

#[test]
fn shell_hover_tick() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0), wall("b.png", "static", 2, 0)]);
    app.config.set_key("theme.backend", json!("noctalia"));
    app.config.set_key(skwd_config::keys::noctalia::HOVER_PREVIEW, json!(true));
    app.scene.hover = Some(1);
    drain_calls(&app);
    let t0 = Instant::now();
    frame(&mut app, t0, 1280.0, 720.0);
    assert!(!drain_calls(&app).iter().any(|(method, _)| method == "wall.shell_preview"));
    frame(&mut app, t0 + std::time::Duration::from_millis(60), 1280.0, 720.0);
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "wall.shell_preview"));
}

#[test]
fn tag_cloud_stays_open() {
    let mut app = test_app();
    seed(
        &mut app,
        &[
            wall("a.png", "static", 1, 0),
            wall("b.png", "static", 2, 0),
            wall("c.png", "static", 3, 0),
        ],
    );
    let _ = update(&mut app, Message::OpenTagCloud);
    assert!(app.tags.cloud_open);
    assert!(!app.tags.matching_tags_open);
    assert!(!app.menu_capturing());
    let _ = update(&mut app, Message::Tag(TagMsg::ToggleMatchingTags));
    assert!(app.tags.matching_tags_open);
    let _ = update(&mut app, Message::KeyNext);
    assert_eq!(app.scene.current, 1);
    drain_calls(&app);
    let _ = update(&mut app, Message::ApplyCurrent);
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "wall.apply"));
    assert!(app.tags.cloud_open);
    let _ = update(&mut app, Message::Exit);
    assert!(!app.tags.cloud_open);
    assert!(!app.tags.matching_tags_open);
    let _ = update(&mut app, Message::OpenTagCloud);
    assert!(!app.tags.matching_tags_open);
}

#[test]
fn tag_cloud_text_blocks_shortcuts() {
    let mut app = test_app();
    let _ = update(&mut app, Message::OpenTagCloud);
    assert!(app.tags.cloud_open);

    let _ = update(
        &mut app,
        Message::KeyPressed(keyboard::Key::Character("t".into()), keyboard::Modifiers::default()),
    );

    assert!(app.tags.cloud_open);
    assert!(!app.tags.mode);
}

#[test]
fn playlists_toggle_key() {
    let mut app = test_app();
    let _ = update(&mut app, Message::OpenPlaylists);
    assert!(app.panels.playlists.is_some());
    let _ = update(&mut app, Message::OpenPlaylists);
    assert!(app.panels.playlists.is_none());
    app.panels.settings.open = true;
    let _ = update(&mut app, Message::OpenPlaylists);
    assert!(app.panels.playlists.is_none());
    app.panels.settings.open = false;
}

#[test]
fn demo_open_replaces_desk_atomically() {
    let mut app = test_app();
    app.panels.settings.open = true;
    let command = |app: &mut App, value: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(value.to_string())),
        );
    };

    command(&mut app, "open effects brightness");
    assert!(!app.panels.settings.open);
    assert!(app.panels.effects.is_some());
    assert!(app.picker_obscured());

    command(&mut app, "open playlists");
    assert!(app.panels.effects.is_none());
    assert!(app.panels.playlists.is_some());
}

#[test]
fn grid_arrow_rows() {
    let mut app = test_app();
    let walls: Vec<Value> = (0..40).map(|i| wall(&format!("w{i}"), "static", i % 360, i)).collect();
    seed(&mut app, &walls);
    app.scene.mode = Mode::Grid;
    app.scene.gp.cols = 5;
    let cols = app.scene.gp.cols;
    assert_eq!(app.scene.current, 0);
    let _ = update(&mut app, Message::KeyDown);
    assert_eq!(app.scene.current, cols);
    assert!(app.scene.kb_nav);
    let _ = update(&mut app, Message::KeyUp);
    assert_eq!(app.scene.current, 0);
    let _ = update(&mut app, Message::KeyUp);
    assert_eq!(app.scene.current, 0);
    for _ in 0..7 {
        let _ = update(&mut app, Message::KeyDown);
    }
    assert_eq!(app.scene.current, 7 * cols);
    assert!(app.scene.camera_target() > 0.0);
}

#[test]
fn tag_cloud_scroll() {
    let mut app = test_app();
    let _ = update(&mut app, Message::Tag(TagMsg::CloudScroll(120.0)));
    assert_eq!(app.tags.cloud_scroll.target, 120.0);
    assert!(app.tags.cloud_scroll.x < 120.0);
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 120);
    assert!(app.tags.cloud_scroll.settled());
    assert!((app.tags.cloud_scroll.x - 120.0).abs() < 1.0);
    let _ = update(&mut app, Message::Tag(TagMsg::CloudScroll(-5.0)));
    assert_eq!(app.tags.cloud_scroll.target, 0.0);
    let _ = update(&mut app, Message::Tag(TagMsg::CloudScroll(80.0)));
    let _ = update(&mut app, Message::Tag(TagMsg::QueryInput("x".into())));
    assert_eq!(app.tags.cloud_scroll.x, 0.0);
    assert!(app.tags.cloud_scroll.settled());
}

#[test]
fn pane_scroll_springs() {
    let mut app = test_app();
    let _ = update(&mut app, Message::PaneWheel("settings", -1.0));
    assert_eq!(
        app.chrome.pane_scrolls["settings"].spring.target,
        crate::frontend::ui::PANE_WHEEL_STEP
    );
    let _ = update(&mut app, Message::PaneWheel("settings", -1.0));
    assert_eq!(
        app.chrome.pane_scrolls["settings"].spring.target,
        2.0 * crate::frontend::ui::PANE_WHEEL_STEP
    );
    let _ = update(&mut app, Message::PaneScrolled("settings", 0.0, 40.0));
    assert_eq!(app.chrome.pane_scrolls["settings"].spring.target, 40.0);
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 120);
    let pane = &app.chrome.pane_scrolls["settings"];
    assert!(pane.spring.settled() && !pane.dirty);
    assert!((pane.spring.x - 40.0).abs() < 1.0);
    let _ = update(&mut app, Message::PaneWheel("settings", 5.0));
    assert_eq!(app.chrome.pane_scrolls["settings"].spring.target, 0.0);
    tick_frames(&mut app, &mut now, 120);
    let _ = update(&mut app, Message::PaneScrolled("settings", 20.0, 40.0));
    assert_eq!(app.chrome.pane_scrolls["settings"].spring.x, 20.0);
    let _ = update(&mut app, Message::ToggleSettings);
    assert!(!app.chrome.pane_scrolls.contains_key("settings"));
    let _ = update(&mut app, Message::ToggleSettings);
}

#[test]
fn grid_wheel_magnitude() {
    let mut app = test_app();
    let walls: Vec<Value> = (0..40).map(|i| wall(&format!("w{i}"), "static", i % 360, i)).collect();
    seed(&mut app, &walls);
    app.scene.mode = Mode::Grid;
    app.scene.gp.cols = 5;
    let cell = app.scene.gp.cell_h();
    let _ = update(&mut app, Message::Wheel(-3.0));
    assert!((app.scene.camera_target() - 3.0 * cell).abs() < 0.5);
}

#[test]
fn folder_menu_scroll() {
    let mut app = test_app();
    app.library_session.folder_options = (0..30).map(|i| format!("folder-{i}")).collect();
    let _ = update(&mut app, Message::FolderMenuScroll(50.0));
    assert!(app.chrome.bar.menu_scroll.target > 0.0);
    assert_eq!(app.chrome.bar.menu_scroll.x, 0.0);
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 120);
    assert!(app.chrome.bar.menu_scroll.settled());
    let _ = update(&mut app, Message::FolderMenuToggle);
    assert_eq!(app.chrome.bar.menu_scroll.x, 0.0);
}

#[test]
fn browser_key_nav() {
    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    let mut browser = Browser::new(Source::Wallhaven);
    for i in 0..30 {
        browser.session.items.push(browser_item(&format!("w{i}")));
    }
    app.source_browser.browser = Some(browser);
    let cols = app.config.browser_grid(false).cols.max(1);
    let _ = update(&mut app, Message::KeyNext);
    assert_eq!(app.source_browser.browser.as_ref().unwrap().session.hover, Some(0));
    let _ = update(&mut app, Message::KeyNext);
    assert_eq!(app.source_browser.browser.as_ref().unwrap().session.hover, Some(1));
    let _ = update(&mut app, Message::KeyDown);
    assert_eq!(app.source_browser.browser.as_ref().unwrap().session.hover, Some(1 + cols));
    let _ = update(&mut app, Message::ApplyCurrent);
    assert_eq!(app.source_browser.browser.as_ref().unwrap().session.preview, Some(1 + cols));
    let _ = update(&mut app, Message::KeyNext);
    assert_eq!(app.source_browser.browser.as_ref().unwrap().session.preview, Some(2 + cols));
    let _ = update(&mut app, Message::KeyPrev);
    assert_eq!(app.source_browser.browser.as_ref().unwrap().session.preview, Some(1 + cols));
    let _ = update(&mut app, Message::Exit);
    assert!(app.source_browser.preview_closing);
}

#[test]
fn key_flip_favourite() {
    let mut app = test_app();
    seed(&mut app, &[wall("a", "static", 10, 1), wall("b", "static", 20, 2)]);
    app.scene.mode = Mode::Slices;
    let _ = update(&mut app, Message::KeyFlip);
    assert!(app.scene.flip_open());
    let _ = update(&mut app, Message::KeyFlip);
    assert!(!app.scene.flip_open());
    let key = app.library_session.library.catalog().items
        [app.library_session.filtered[app.scene.current] as usize]
        .key
        .clone();
    let _ = update(&mut app, Message::KeyFavourite);
    assert!(app.library_session.library.catalog().favourites.contains(&key));
    let _ = update(&mut app, Message::KeyFavourite);
    assert!(!app.library_session.library.catalog().favourites.contains(&key));
    app.panels.settings.open = true;
    let _ = update(&mut app, Message::KeyFavourite);
    assert!(!app.library_session.library.catalog().favourites.contains(&key));
    app.panels.settings.open = false;
}

#[test]
fn sandy_filter_storm() {
    let mut app = test_app();
    seed(
        &mut app,
        &[
            wall("a", "static", 10, 1),
            wall("b", "static", 20, 2),
            wall("c", "static", 30, 3),
            wall("v1", "video", 40, 4),
            wall("v2", "video", 50, 5),
        ],
    );
    app.scene.mode = Mode::Sandy;
    app.scene.set_current(2, app.library_session.filtered.len());
    app.scene.sandy_settle_now();
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 12);
    let old_store = app.library_session.filtered[app.scene.sandy_displayed()] as usize;
    let _ = update(&mut app, Message::SetTypeFilter("video".into()));
    assert_eq!(app.library_session.filtered.len(), 2);
    assert!(app.scene.sandy_ring_engaged());
    assert_eq!(app.scene.sandy_filter_from_idx(), Some(old_store));
    assert!(app.scene.filter_flip_running());
    assert!(app.scene.filter_flip_count() > 0);
    tick_frames(&mut app, &mut now, 600);
    assert!(!app.scene.sandy_ring_engaged());
    assert_eq!(app.scene.sandy_filter_from_idx(), None);
    assert!(!app.scene.filter_flip_running());
    assert_eq!(app.scene.filter_flip_count(), 0);
}

#[test]
fn filter_flip_all_modes() {
    for mode in [Mode::Slices, Mode::Grid, Mode::Hex] {
        let mut app = test_app();
        let walls: Vec<Value> =
            (0..12).map(|i| wall(&format!("w{i}"), "static", i * 25, i)).collect();
        seed(&mut app, &walls);
        app.scene.mode = mode;
        let mut now = Instant::now();
        tick_frames(&mut app, &mut now, 12);
        let _ = update(&mut app, Message::SetSort("date".into()));
        assert!(app.scene.filter_flip_running(), "{mode:?}");
        assert!(app.scene.filter_flip_count() > 0, "{mode:?}");
        tick_frames(&mut app, &mut now, 300);
        assert!(!app.scene.filter_flip_running(), "{mode:?}");
        assert_eq!(app.scene.filter_flip_count(), 0, "{mode:?}");
    }
}

#[test]
fn sandy_ring_mid_pick() {
    let mut app = test_app();
    seed(
        &mut app,
        &[wall("a", "static", 10, 1), wall("b", "static", 20, 2), wall("c", "static", 30, 3)],
    );
    app.scene.mode = Mode::Sandy;
    app.scene.sandy_settle_now();
    let _ = update(&mut app, Message::SetTypeFilter("static".into()));
    assert!(!app.scene.sandy_ring_engaged());
    let _ = update(&mut app, Message::SetSort("date".into()));
    assert!(app.scene.sandy_ring_engaged());
    app.scene.set_current(1, app.library_session.filtered.len());
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 600);
    assert!(!app.scene.sandy_ring_engaged());
    assert_eq!(app.scene.sandy_filter_from_idx(), None);
}

#[test]
fn sandy_storm_skips() {
    let mut app = test_app();
    seed(&mut app, &[wall("a", "static", 10, 1), wall("b", "static", 20, 2)]);
    app.scene.mode = Mode::Sandy;
    app.scene.sandy_settle_now();
    let _ = update(&mut app, Message::SetTypeFilter(String::new()));
    assert!(!app.scene.sandy_ring_engaged());
    let _ = update(&mut app, Message::SetTypeFilter("we".into()));
    assert!(app.library_session.filtered.is_empty());
    assert!(!app.scene.sandy_ring_engaged());
    let _ = update(&mut app, Message::SetTypeFilter(String::new()));
    assert!(app.scene.sandy_ring_engaged());
    app.scene.sandy_settle_now();
    app.scene.mode = Mode::Grid;
    let _ = update(&mut app, Message::SetTypeFilter("static".into()));
    assert!(!app.scene.sandy_ring_engaged());
}

#[test]
fn custom_keymap() {
    use keyboard::key::Named;
    let mut cfg = Config::from_data(json!({
        "keys": { "playlists": "x", "help": "ctrl+h", "favourite": "5" }
    }));
    cfg.config_path = std::env::temp_dir().join("skwd-keymsg-test.json");
    let km = crate::infrastructure::config::load_bindings(&cfg);
    let none = keyboard::Modifiers::default();
    assert!(matches!(
        key_message(&km, &keyboard::Key::Character("x".into()), none, PICKER),
        Some(Message::OpenPlaylists)
    ));
    assert!(key_message(&km, &keyboard::Key::Character("p".into()), none, PICKER).is_none());
    assert!(matches!(
        key_message(&km, &keyboard::Key::Character("h".into()), keyboard::Modifiers::CTRL, PICKER),
        Some(Message::ToggleHelp)
    ));
    assert!(key_message(&km, &keyboard::Key::Character("h".into()), none, PICKER).is_none());
    assert!(matches!(
        key_message(&km, &keyboard::Key::Character("5".into()), none, PICKER),
        Some(Message::KeyFavourite)
    ));
    assert!(matches!(
        key_message(&km, &keyboard::Key::Character("X".into()), keyboard::Modifiers::SHIFT, PICKER),
        Some(Message::OpenPlaylists)
    ));
    assert!(matches!(
        key_message(&km, &keyboard::Key::Named(Named::ArrowLeft), none, PICKER),
        Some(Message::KeyPrev)
    ));
    assert!(matches!(
        key_message(
            &km,
            &keyboard::Key::Named(Named::ArrowLeft),
            keyboard::Modifiers::SHIFT,
            PICKER
        ),
        Some(Message::SetColorFilter(i64::MIN))
    ));
    assert!(matches!(
        key_message(&km, &keyboard::Key::Named(Named::Escape), none, PICKER),
        Some(Message::Exit)
    ));
    assert!(matches!(
        key_message(&km, &keyboard::Key::Named(Named::Enter), none, PICKER),
        Some(Message::ApplyCurrent)
    ));
}

#[test]
fn rebind_live() {
    use crate::domain::input::{InputAction, KeyId, Mods};
    let mut app = test_app();
    assert_eq!(
        app.input.bindings.lookup_key(
            &KeyId::Char("p".into()),
            Mods::new(false, false, false),
            PICKER
        ),
        Some(InputAction::Playlists)
    );
    let _ = update(
        &mut app,
        Message::Settings(crate::frontend::settings::SettingsMsg::Input(
            "keys.playlists".into(),
            "x".into(),
        )),
    );
    assert_eq!(
        app.input.bindings.lookup_key(
            &KeyId::Char("x".into()),
            Mods::new(false, false, false),
            PICKER
        ),
        Some(InputAction::Playlists)
    );
    assert_eq!(
        app.input.bindings.lookup_key(
            &KeyId::Char("p".into()),
            Mods::new(false, false, false),
            PICKER
        ),
        None
    );
    let rows = crate::frontend::ui::help::help_rows(&app.input.bindings);
    assert!(rows.iter().any(|(key, help)| key == "X" && help.contains("playlists")));
    let _ = update(
        &mut app,
        Message::Settings(crate::frontend::settings::SettingsMsg::Input(
            "keys.playlists".into(),
            "junk+q".into(),
        )),
    );
    assert_eq!(
        app.input.bindings.lookup_key(
            &KeyId::Char("p".into()),
            Mods::new(false, false, false),
            PICKER
        ),
        Some(InputAction::Playlists)
    );
}

#[test]
fn keybind_capture_popup_flow() {
    use crate::domain::input::{InputAction, KeyId, Mods};
    use crate::frontend::settings::SettingsMsg;
    let mut app = test_app();
    let _ = update(&mut app, Message::ToggleSettings);
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.playlists".into())));
    assert!(app.panels.settings.keybind_capture.is_some());
    let _ = update(
        &mut app,
        Message::KeyPressed(keyboard::Key::Character("p".into()), keyboard::Modifiers::default()),
    );
    assert!(app.panels.settings.open);
    let _ = update(
        &mut app,
        Message::KeyPressed(keyboard::Key::Character("x".into()), keyboard::Modifiers::default()),
    );
    let _ = update(
        &mut app,
        Message::KeyPressed(
            keyboard::Key::Named(keyboard::key::Named::Enter),
            keyboard::Modifiers::default(),
        ),
    );
    assert!(app.panels.settings.keybind_capture.is_none());
    assert_eq!(app.config.str_path("keys.playlists"), "x");
    assert_eq!(
        app.input.bindings.lookup_key(
            &KeyId::Char("x".into()),
            Mods::new(false, false, false),
            PICKER
        ),
        Some(InputAction::Playlists)
    );
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.playlists".into())));
    let _ = update(
        &mut app,
        Message::KeyPressed(keyboard::Key::Character("z".into()), keyboard::Modifiers::default()),
    );
    let _ = update(
        &mut app,
        Message::KeyPressed(
            keyboard::Key::Named(keyboard::key::Named::Escape),
            keyboard::Modifiers::default(),
        ),
    );
    assert!(app.panels.settings.keybind_capture.is_none());
    assert!(app.panels.settings.open);
    assert_eq!(
        app.input.bindings.lookup_key(
            &KeyId::Char("x".into()),
            Mods::new(false, false, false),
            PICKER
        ),
        Some(InputAction::Playlists)
    );
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.playlists".into())));
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCaptureDefault));
    assert!(app.panels.settings.keybind_capture.is_none());
    assert_eq!(
        app.input.bindings.lookup_key(
            &KeyId::Char("p".into()),
            Mods::new(false, false, false),
            PICKER
        ),
        Some(InputAction::Playlists)
    );
}

#[test]
fn keybind_capture_binds_and_resets() {
    use crate::domain::input::{InputAction, KeyId, Mods, MouseButton, MouseSpec};
    use crate::frontend::settings::{ActionId, SettingsMsg};
    let mut app = test_app();
    let _ = update(&mut app, Message::ToggleSettings);

    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.flip".into())));
    let mods = keyboard::Modifiers::CTRL | keyboard::Modifiers::SHIFT | keyboard::Modifiers::ALT;
    let _ = update(&mut app, Message::KeyPressed(keyboard::Key::Character("D".into()), mods));
    let _ = update(
        &mut app,
        Message::KeyPressed(
            keyboard::Key::Named(keyboard::key::Named::Enter),
            keyboard::Modifiers::default(),
        ),
    );
    assert_eq!(app.config.str_path("keys.flip"), "ctrl+alt+shift+d");
    assert_eq!(
        app.input.bindings.lookup_key(
            &KeyId::Char("d".into()),
            Mods::new(true, true, true),
            PICKER
        ),
        Some(InputAction::Flip)
    );

    let _ =
        update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.playlists".into())));
    let _ = update(&mut app, Message::SetMods(Mods::new(false, true, false)));
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::KeybindCaptureClick(MouseButton::Middle)));
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCaptureApply));
    assert_eq!(app.config.str_path("keys.playlists"), "alt+middle-click");
    assert_eq!(
        app.input.bindings.lookup_mouse(
            MouseSpec { mods: Mods::new(false, true, false), button: MouseButton::Middle },
            PICKER
        ),
        Some(InputAction::Playlists)
    );

    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.select".into())));
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCaptureUnbind));
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCaptureApply));
    assert_eq!(app.config.str_path("keys.select"), "none");
    assert_eq!(
        app.input
            .bindings
            .lookup_mouse(MouseSpec { mods: Mods::NONE, button: MouseButton::Left }, PICKER),
        None
    );

    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::ResetKeybinds)));
    assert_eq!(app.config.str_path("keys.flip"), "ctrl+alt+shift+d");
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::ResetKeybinds)));
    assert_eq!(app.config.str_path("keys.flip"), "");
    assert_eq!(app.config.str_path("keys.select"), "");
    assert_eq!(
        app.input
            .bindings
            .lookup_mouse(MouseSpec { mods: Mods::NONE, button: MouseButton::Right }, PICKER),
        Some(InputAction::Flip)
    );
    assert_eq!(
        app.input
            .bindings
            .lookup_mouse(MouseSpec { mods: Mods::NONE, button: MouseButton::Left }, PICKER),
        Some(InputAction::Select)
    );
}

#[test]
fn capture_steals_trigger() {
    use crate::domain::input::{InputAction, KeyId, Mods};
    use crate::frontend::settings::SettingsMsg;
    let mut app = test_app();
    let _ = update(&mut app, Message::ToggleSettings);
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.settings".into())));
    let _ = update(
        &mut app,
        Message::KeyPressed(keyboard::Key::Character("p".into()), keyboard::Modifiers::default()),
    );
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCaptureApply));
    assert_eq!(
        app.input.bindings.lookup_key(&KeyId::Char("p".into()), Mods::NONE, PICKER),
        Some(InputAction::Settings)
    );
    assert_eq!(app.config.str_path("keys.playlists"), "none");
}

#[test]
fn key_effects_toggle() {
    let mut app = test_app();
    seed(&mut app, &[wall("a", "static", 10, 1)]);
    let _ = update(&mut app, Message::KeyEffects);
    assert!(app.panels.effects.is_some());
    let _ = update(&mut app, Message::KeyEffects);
    assert!(app.panels.effects.is_none());
}

#[test]
fn external_config_adopt() {
    let mut app = test_app();
    let external = json!({
        "keys": { "favourite": "x" },
        "components": { "wallpaperSelector": { "sandyDuration": 777.0 } },
        "transition": { "durationMs": 750.0 }
    });
    std::fs::write(&app.config.config_path, external.to_string()).unwrap();
    assert!(app.adopt_external_config());
    assert_eq!(app.config.sandy_duration(), 777.0);
    assert_eq!(app.config.num_path("transition.durationMs"), 750.0);
    let none = keyboard::Modifiers::default();
    assert!(matches!(
        key_message(&app.input.bindings, &keyboard::Key::Character("x".into()), none, PICKER),
        Some(Message::KeyFavourite)
    ));
    assert!(!app.adopt_external_config());
}

#[test]
fn captured_left_release_routes() {
    let win = iced::window::Id::unique();
    let released =
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left));
    assert!(matches!(
        on_event(released, iced::event::Status::Captured, win),
        Some(Message::PointerUp)
    ));
}

#[test]
fn pointer_release_ends_drag() {
    let mut app = test_app();
    let mut editor = crate::frontend::schedule_editor::ScheduleEditor::new(Vec::new(), false, true);
    editor.add_rule();
    editor.add_rule();
    editor.rows[0].name = String::from("first");
    editor.rows[1].name = String::from("second");
    editor.drag_start(0);
    app.panels.schedule = Some(editor);

    let _ = update(&mut app, Message::PointerUp);

    let editor = app.panels.schedule.as_mut().expect("schedule editor stays open");
    assert!(editor.drag.is_none());
    editor.drag_over(1);
    assert_eq!(editor.rows[0].name, "first");
}

fn hand_app(count: usize) -> (App, Instant) {
    let mut app = test_app();
    let walls: Vec<Value> =
        (0..count).map(|i| wall(&format!("w{i}"), "static", (i * 20) as i64, i as i64)).collect();
    seed(&mut app, &walls);
    app.scene.mode = Mode::Hand;
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 3);
    (app, now)
}

fn hit_center(app: &App, index: usize) -> (f32, f32) {
    let hit = app.scene.render.hits.iter().find(|hit| hit.index == index).expect("hand hit");
    (hit.cx, hit.cy)
}

#[test]
fn hand_click_selects_then_applies() {
    use crate::domain::input::MouseButton;
    let (mut app, mut now) = hand_app(8);
    assert_eq!(app.scene.current, 0);
    let (x, y) = hit_center(&app, 1);
    let _ = update(&mut app, Message::Click(x, y, MouseButton::Left));
    assert_eq!(app.scene.current, 1);
    assert!(app.scene.hand_dragging());
    let _ = update(&mut app, Message::PointerUp);
    assert!(!app.scene.hand_dragging());
    assert!(!drain_calls(&app).iter().any(|(method, _)| method == "wall.apply"));
    tick_frames(&mut app, &mut now, 40);
    let (x, y) = hit_center(&app, 1);
    let _ = update(&mut app, Message::Click(x, y, MouseButton::Left));
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "wall.apply"));
}

#[test]
fn hand_right_click_flips_and_esc_closes() {
    use crate::domain::input::MouseButton;
    let (mut app, mut now) = hand_app(8);
    let (x, y) = hit_center(&app, 0);
    let _ = update(&mut app, Message::Click(x, y, MouseButton::Right));
    assert!(app.scene.flip_open());
    tick_frames(&mut app, &mut now, 150);
    assert!(app.scene.render.back.is_some());
    let _ = update(&mut app, Message::Exit);
    assert!(!app.scene.flip_open());
}

#[test]
fn hand_v_key_reveals_and_esc_folds_the_row() {
    let (mut app, mut now) = hand_app(8);
    let press = |app: &mut App, key: &str| {
        let _ = update(
            app,
            Message::KeyPressed(
                keyboard::Key::Character(key.into()),
                keyboard::Modifiers::default(),
            ),
        );
    };
    press(&mut app, "v");
    assert!(app.scene.hand_reveal_open());
    tick_frames(&mut app, &mut now, 20);
    assert!(app.scene.is_animating());
    press(&mut app, "v");
    assert!(!app.scene.hand_reveal_open());
    tick_frames(&mut app, &mut now, 200);
    assert!(!app.scene.is_animating());
    press(&mut app, "v");
    assert!(app.scene.hand_reveal_open());
    let _ = update(&mut app, Message::Exit);
    assert!(!app.scene.hand_reveal_open(), "escape folds the row instead of quitting");
    let _ = update(&mut app, Message::ToggleSettings);
    press(&mut app, "v");
    assert!(!app.scene.hand_reveal_open(), "typing in settings never reaches the picker");
}

#[test]
fn hand_wheel_past_edge_deals_next_hand() {
    let (mut app, mut now) = hand_app(12);
    for _ in 0..4 {
        let _ = update(&mut app, Message::Wheel(-1.0));
    }
    assert_eq!(app.scene.current, 4);
    assert!(!app.scene.hand_dealing());
    let _ = update(&mut app, Message::Wheel(-1.0));
    assert_eq!(app.scene.current, 5);
    assert!(app.scene.hand_dealing());
    assert_eq!(app.scene.hand_offset(), 5);
    tick_frames(&mut app, &mut now, 300);
    assert!(!app.scene.hand_dealing());
    assert!(app.scene.render.hits.iter().any(|hit| hit.index == 5));
}

#[test]
fn hand_filter_change_deals() {
    let (mut app, mut now) = hand_app(8);
    let _ = update(&mut app, Message::SetSort("date".into()));
    assert!(app.scene.hand_dealing());
    assert!(!app.scene.filter_flip_running());
    tick_frames(&mut app, &mut now, 300);
    assert!(!app.scene.hand_dealing());
}

#[test]
fn hand_hover_only_lifts_and_click_selects() {
    use crate::domain::input::MouseButton;
    let (mut app, mut now) = hand_app(8);
    app.scene.set_current(2, 8);
    tick_frames(&mut app, &mut now, 40);
    let (x, y) = hit_center(&app, 4);
    let _ = update(&mut app, Message::MouseMoved(1.0, 1.0));
    let _ = update(&mut app, Message::MouseMoved(x, y));
    assert_eq!(app.scene.current, 2);
    assert_eq!(app.scene.hover, Some(4));
    let _ = update(&mut app, Message::Click(x, y, MouseButton::Left));
    assert_eq!(app.scene.current, 4);
    assert!(!drain_calls(&app).iter().any(|(method, _)| method == "wall.apply"));
}

#[test]
fn hand_wheel_closes_the_flip_and_moves_on() {
    let (mut app, mut now) = hand_app(8);
    app.scene.set_current(2, 8);
    tick_frames(&mut app, &mut now, 30);
    let _ = update(&mut app, Message::KeyFlip);
    tick_frames(&mut app, &mut now, 120);
    assert!(app.scene.flip_open());
    assert_eq!(app.scene.flipped(), Some(2));
    let _ = update(&mut app, Message::Wheel(-1.0));
    assert_eq!(app.scene.current, 3);
    assert!(!app.scene.flip_open());
    assert_eq!(app.scene.flipped(), Some(2));
    tick_frames(&mut app, &mut now, 150);
    assert_eq!(app.scene.flipped(), None);
    let _ = update(&mut app, Message::KeyFlip);
    assert!(app.scene.flip_open());
    let _ = update(&mut app, Message::KeyPrev);
    assert_eq!(app.scene.current, 2);
    assert!(!app.scene.flip_open());
}

#[test]
fn choose_displays_shortcut_can_be_rebound_and_reset() {
    use crate::domain::input::{InputAction, KeyId, Mods, MouseButton, MouseSpec};
    use crate::frontend::settings::SettingsMsg;
    let mut app = test_app();
    seed(&mut app, &[wall("wallpaper", "static", 1, 0)]);
    let _ = update(&mut app, Message::ToggleSettings);
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.effects".into())));
    assert_eq!(app.panels.settings.keybind_capture.as_ref().unwrap().title_key, "keybind-effects");
    let _ = update(
        &mut app,
        Message::KeyPressed(keyboard::Key::Character("m".into()), keyboard::Modifiers::SHIFT),
    );
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCaptureApply));
    assert_eq!(app.config.str_path("keys.effects"), "shift+m");
    assert_eq!(
        app.input.bindings.lookup_key(
            &KeyId::Char("m".into()),
            Mods::new(false, false, true),
            PICKER
        ),
        Some(InputAction::Effects)
    );
    assert_ne!(
        app.input.bindings.lookup_mouse(
            MouseSpec { mods: Mods::new(true, false, false), button: MouseButton::Left },
            PICKER
        ),
        Some(InputAction::Effects)
    );
    let _ = update(&mut app, Message::ToggleSettings);
    let _ = update(
        &mut app,
        Message::KeyPressed(keyboard::Key::Character("m".into()), keyboard::Modifiers::SHIFT),
    );
    assert_eq!(
        app.panels.effects.as_ref().unwrap().mode(),
        crate::frontend::effects::EffectsMode::Displays
    );
    app.panels.effects = None;
    let _ = update(&mut app, Message::ToggleSettings);
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.effects".into())));
    let _ = update(&mut app, Message::SetMods(Mods::new(false, true, false)));
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::KeybindCaptureClick(MouseButton::Middle)));
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCaptureApply));
    assert_eq!(app.config.str_path("keys.effects"), "alt+middle-click");
    assert_eq!(
        app.input.bindings.lookup_mouse(
            MouseSpec { mods: Mods::new(false, true, false), button: MouseButton::Middle },
            PICKER
        ),
        Some(InputAction::Effects)
    );
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.effects".into())));
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCaptureDefault));
    assert_eq!(
        app.input.bindings.lookup_mouse(
            MouseSpec { mods: Mods::new(true, false, false), button: MouseButton::Left },
            PICKER
        ),
        Some(InputAction::Effects)
    );
}

#[test]
fn outside_click_dismisses_one_overlay_at_a_time() {
    use crate::domain::input::MouseButton;
    let mut app = test_app();
    app.panels.settings.open = true;
    app.input.help_open = true;
    let _ = update(&mut app, Message::Click(0.0, 0.0, MouseButton::Left));
    assert!(!app.input.help_open);
    assert!(app.panels.settings.open);
    let _ = update(&mut app, Message::Click(0.0, 0.0, MouseButton::Left));
    assert!(!app.panels.settings.open);
    assert!(drain_calls(&app).iter().all(|(method, _)| method != "wall.apply"));
}

#[test]
fn outside_click_closes_card_back_in_every_layout() {
    use crate::domain::input::MouseButton;
    for mode in [Mode::Slices, Mode::Hex, Mode::Grid, Mode::Sandy, Mode::Hand] {
        let mut app = test_app();
        seed(&mut app, &[wall("a", "static", 10, 1)]);
        app.scene.mode = mode;
        let mut now = Instant::now();
        tick_frames(&mut app, &mut now, 150);
        let _ = update(&mut app, Message::KeyFlip);
        tick_frames(&mut app, &mut now, 150);
        assert!(app.scene.render.back.is_some(), "{mode:?}");
        let _ = update(&mut app, Message::Click(0.0, 0.0, MouseButton::Left));
        assert!(!app.scene.flip_open(), "{mode:?}");
        assert!(!app.detail_open(), "{mode:?}");
        assert!(drain_calls(&app).iter().all(|(method, _)| method != "wall.apply"));
    }
}

#[test]
fn outside_right_click_keeps_picker_and_overlay_open() {
    use crate::domain::input::MouseButton;
    let mut app = test_app();
    let _ = update(&mut app, Message::Click(0.0, 0.0, MouseButton::Right));
    app.input.help_open = true;
    let _ = update(&mut app, Message::Click(0.0, 0.0, MouseButton::Right));
    assert!(app.input.help_open);
}
