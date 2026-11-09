use super::*;
use skwd_config::FilterBarVisibility;

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
            crate::frontend::settings::SettingsKey::Space
        )))
    ));
    let km = crate::domain::input::InputMap::default();
    let none = keyboard::Modifiers::default();
    let shift = keyboard::Modifiers::SHIFT;
    assert!(matches!(
        key_message(&km, &keyboard::Key::Character("s".into()), shift, PICKER),
        Some(Message::ToggleSettings)
    ));
    let char_message = |text: &str, modifiers| {
        key_message(&km, &keyboard::Key::Character(text.into()), modifiers, PICKER)
    };
    assert!(matches!(char_message("w", none), Some(Message::KeyUp)));
    assert!(matches!(char_message("a", none), Some(Message::KeyPrev)));
    assert!(matches!(char_message("s", none), Some(Message::KeyDown)));
    assert!(matches!(char_message("d", none), Some(Message::KeyNext)));
    assert!(matches!(char_message("/", none), Some(Message::OpenTagCloud)));
    assert!(matches!(char_message("m", none), Some(Message::CycleMode { backwards: false })));
    assert!(matches!(char_message("M", shift), Some(Message::CycleMode { backwards: true })));
    assert!(char_message("h", none).is_none());
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
        Some(Message::KeyColourStudio)
    ));
    assert!(matches!(
        key_message(&km, &keyboard::Key::Character("C".into()), shift, PICKER),
        Some(Message::ToggleThemePanel)
    ));
    assert!(matches!(
        key_message(&km, &keyboard::Key::Character("f".into()), none, PICKER),
        Some(Message::KeyFavourite)
    ));
    assert!(key_message(&km, &keyboard::Key::Character("i".into()), none, PICKER).is_none());
    assert!(matches!(
        key_message(&km, &keyboard::Key::Character("e".into()), none, PICKER),
        Some(Message::OpenWidgets)
    ));
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
        Message::KeyPressed(keyboard::Key::Character("C".into()), keyboard::Modifiers::SHIFT),
    );
    assert!(app.theme.bar_open);
    assert!(app.chrome.filter_bar_visible);

    let _ = update(
        &mut app,
        Message::KeyPressed(keyboard::Key::Character("C".into()), keyboard::Modifiers::SHIFT),
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
    app.theme.armed = true;
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
    app.theme.armed = true;
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
    app.chrome.bar.menu = Some(crate::frontend::ui::MenuKind::Folders);
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
fn browser_page_and_edge_keys_clamp() {
    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    let mut browser = Browser::new(Source::Wallhaven);
    for i in 0..30 {
        browser.session.items.push(browser_item(&format!("w{i}")));
    }
    app.source_browser.browser = Some(browser);
    let hover = |app: &App| app.source_browser.browser.as_ref().unwrap().session.hover;
    let _ = update(&mut app, Message::KeyEdge { last: true });
    assert_eq!(hover(&app), Some(29));
    let _ = update(&mut app, Message::KeyPage { backwards: false });
    assert_eq!(hover(&app), Some(29));
    let _ = update(&mut app, Message::KeyEdge { last: false });
    assert_eq!(hover(&app), Some(0));
    let _ = update(&mut app, Message::KeyPage { backwards: true });
    assert_eq!(hover(&app), Some(0));
    let _ = update(&mut app, Message::KeyPage { backwards: false });
    assert!(hover(&app).is_some_and(|index| index > 0));
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
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::KeybindCapture("keys.playlists".into(), 0)),
    );
    assert!(app.panels.settings.keybind_capture.is_some());
    let _ = update(
        &mut app,
        Message::KeyPressed(keyboard::Key::Character("p".into()), keyboard::Modifiers::default()),
    );
    assert!(app.panels.settings.open);
    assert!(app.panels.settings.keybind_capture.is_none());
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::KeybindCapture("keys.playlists".into(), 0)),
    );
    let _ = update(
        &mut app,
        Message::KeyPressed(keyboard::Key::Character("x".into()), keyboard::Modifiers::default()),
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
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::KeybindCapture("keys.playlists".into(), 0)),
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
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::KeybindCapture("keys.playlists".into(), 0)),
    );
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

    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.flip".into(), 0)));
    let mods = keyboard::Modifiers::CTRL | keyboard::Modifiers::SHIFT | keyboard::Modifiers::ALT;
    let _ = update(&mut app, Message::KeyPressed(keyboard::Key::Character("D".into()), mods));
    assert_eq!(app.config.str_path("keys.flip"), "ctrl+alt+shift+d");
    assert_eq!(
        app.input.bindings.lookup_key(
            &KeyId::Char("d".into()),
            Mods::new(true, true, true),
            PICKER
        ),
        Some(InputAction::Flip)
    );

    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::KeybindCapture("keys.playlists".into(), 0)),
    );
    let _ = update(&mut app, Message::SetMods(Mods::new(false, true, false)));
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::KeybindCaptureClick(MouseButton::Middle)));
    assert_eq!(app.config.str_path("keys.playlists"), "alt+middle-click");
    assert_eq!(
        app.input.bindings.lookup_mouse(
            MouseSpec { mods: Mods::new(false, true, false), button: MouseButton::Middle },
            PICKER
        ),
        Some(InputAction::Playlists)
    );

    let _ =
        update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.select".into(), 0)));
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCaptureUnbind));
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
        update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.settings".into(), 0)));
    let _ = update(
        &mut app,
        Message::KeyPressed(keyboard::Key::Character("p".into()), keyboard::Modifiers::default()),
    );
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
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.effects".into(), 0)));
    assert_eq!(app.panels.settings.keybind_capture.as_ref().unwrap().action, InputAction::Effects);
    let _ = update(
        &mut app,
        Message::KeyPressed(keyboard::Key::Character("m".into()), keyboard::Modifiers::SHIFT),
    );
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
    let _ = update(&mut app, Message::KeyReleased(keyboard::Key::Character("m".into())));
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
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.effects".into(), 0)));
    let _ = update(&mut app, Message::SetMods(Mods::new(false, true, false)));
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::KeybindCaptureClick(MouseButton::Middle)));
    assert_eq!(app.config.str_path("keys.effects"), "alt+middle-click");
    assert_eq!(
        app.input.bindings.lookup_mouse(
            MouseSpec { mods: Mods::new(false, true, false), button: MouseButton::Middle },
            PICKER
        ),
        Some(InputAction::Effects)
    );
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.effects".into(), 0)));
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
    for mode in [Mode::Slices, Mode::Depth, Mode::Hex, Mode::Grid, Mode::Sandy, Mode::Hand] {
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

#[test]
fn collection_click_previews_escape_returns_and_enter_applies() {
    use crate::domain::input::MouseButton;
    let mut app = test_app();
    let walls: Vec<Value> = (0..20).map(|i| wall(&format!("c{i}"), "static", i * 10, i)).collect();
    seed(&mut app, &walls);
    let _ = update(&mut app, Message::SetViewMode(String::from("collection")));
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 120);
    let hit = app.scene.render.hits.iter().find(|hit| hit.index == 3).unwrap();
    let quad = hit.quad.unwrap();
    let x = (quad[0][0] + quad[1][0]) * 0.5;
    let y = quad[0][1] + 5.0;
    let _ = update(&mut app, Message::Click(x, y, MouseButton::Left));
    tick_frames(&mut app, &mut now, 90);
    assert_eq!(app.scene.current, 3);
    assert_eq!(app.scene.render.hits[0].index, 3);
    assert_eq!(app.scene.render.chrome.iter().filter(|c| c.opacity > 0.0).count(), 1);
    assert!(!drain_calls(&app).iter().any(|(method, _)| method == "wall.apply"));
    let _ = update(&mut app, Message::Exit);
    tick_frames(&mut app, &mut now, 90);
    assert!(app.scene.render.chrome.iter().all(|c| c.opacity == 0.0));
    let _ = update(&mut app, Message::KeyDown);
    assert_eq!(app.scene.current, 4);
    let _ = update(&mut app, Message::KeyUp);
    assert_eq!(app.scene.current, 3);
    let _ = update(&mut app, Message::KeyFlip);
    tick_frames(&mut app, &mut now, 120);
    assert!(app.scene.render.back.is_some());
    let _ = update(&mut app, Message::Exit);
    tick_frames(&mut app, &mut now, 120);
    let _ = update(&mut app, Message::ApplyCurrent);
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "wall.apply"));
}

#[test]
fn collection_click_after_escape_or_on_another_card_only_previews() {
    use crate::domain::input::MouseButton;
    let mut app = test_app();
    let walls: Vec<Value> = (0..8).map(|i| wall(&format!("c{i}"), "static", i * 10, i)).collect();
    seed(&mut app, &walls);
    let _ = update(&mut app, Message::SetViewMode(String::from("collection")));
    let mut now = Instant::now();
    for dismiss in [false, true] {
        app.scene.close_collection();
        tick_frames(&mut app, &mut now, 120);
        app.scene.select_collection(0, walls.len());
        tick_frames(&mut app, &mut now, 120);
        if dismiss {
            let _ = update(&mut app, Message::Exit);
            tick_frames(&mut app, &mut now, 120);
        }
        let idx = if dismiss { 0 } else { 3 };
        let hit = app.scene.render.hits.iter().find(|hit| hit.index == idx).unwrap();
        let quad = hit.quad.unwrap();
        let x = (quad[0][0] + quad[1][0]) * 0.5;
        let y = quad[0][1] + 2.0;
        assert_eq!(app.scene.render.hits.iter().find(|hit| hit.contains(x, y)).unwrap().index, idx);
        drain_calls(&app);
        let _ = update(&mut app, Message::Click(x, y, MouseButton::Left));
        assert_eq!(app.scene.current, idx);
        assert!(app.scene.collection_open_for(idx));
        assert!(!drain_calls(&app).iter().any(|(method, _)| method == "wall.apply"));
    }
}

#[test]
fn collection_second_click_applies_the_raised_wallpaper() {
    use crate::domain::input::MouseButton;
    let mut app = test_app();
    let walls: Vec<Value> = (0..8)
        .map(|i| {
            let mut item = wall(&format!("c{i}"), "static", i * 10, i);
            item["path"] = json!(format!("/wp/c{i}.png"));
            item
        })
        .collect();
    seed(&mut app, &walls);
    app.config.save_key(skwd_config::keys::general::CLOSE_ON_SELECTION, json!(false));
    let _ = update(&mut app, Message::SetViewMode(String::from("collection")));
    let mut now = Instant::now();
    for settled in [false, true] {
        for idx in [0, 3] {
            app.scene.close_collection();
            app.scene.set_current(0, walls.len());
            tick_frames(&mut app, &mut now, 120);
            let hit = app.scene.render.hits.iter().find(|hit| hit.index == idx).unwrap();
            let quad = hit.quad.unwrap();
            let x = (quad[0][0] + quad[1][0]) * 0.5;
            let y = quad[0][1] + 5.0;
            assert_eq!(
                app.scene.render.hits.iter().find(|hit| hit.contains(x, y)).unwrap().index,
                idx
            );
            let expected = app.library_session.library.catalog().items
                [app.library_session.filtered[idx] as usize]
                .path
                .clone();
            drain_calls(&app);
            let _ = update(&mut app, Message::Click(x, y, MouseButton::Left));
            assert_eq!(app.scene.current, idx);
            assert!(!drain_calls(&app).iter().any(|(method, _)| method == "wall.apply"));
            let (x, y) = if settled {
                tick_frames(&mut app, &mut now, 120);
                hit_center(&app, idx)
            } else {
                (x, y)
            };
            let _ = update(&mut app, Message::Click(x, y, MouseButton::Left));
            let calls = drain_calls(&app);
            let applies: Vec<_> =
                calls.iter().filter(|(method, _)| method == "wall.apply").collect();
            assert_eq!(applies.len(), 1, "settled={settled}, index={idx}");
            assert_eq!(applies[0].1["path"], json!(expected));
        }
    }
}

#[test]
fn depth_click_applies_the_visible_card_without_waiting_for_navigation() {
    use crate::domain::input::MouseButton;
    for moving in [false, true] {
        for current in [false, true] {
            let mut app = test_app();
            let walls: Vec<Value> = (0..12)
                .map(|i| {
                    let mut item = wall(&format!("d{i}"), "static", i * 10, i);
                    item["path"] = json!(format!("/wp/d{i}.png"));
                    item
                })
                .collect();
            seed(&mut app, &walls);
            app.config.set_key("components.wallpaperSelector.depthNavigationMs", json!(4000));
            app.config.set_key("components.wallpaperSelector.depthSpacingPx", json!(320));
            let _ = update(&mut app, Message::SetViewMode(String::from("depth")));
            let mut now = Instant::now();
            tick_frames(&mut app, &mut now, 300);
            if moving {
                app.scene.set_current(1, app.library_session.filtered.len());
                tick_frames(&mut app, &mut now, 3);
                assert!((app.scene.camera_pos() - app.scene.camera_target()).abs() > 0.1);
            }
            let idx = if current { app.scene.current } else { 2 };
            let (x, y) = hit_center(&app, idx);
            assert_eq!(
                app.scene.render.hits.iter().find(|hit| hit.contains(x, y)).unwrap().index,
                idx
            );
            let expected = app.library_session.library.catalog().items
                [app.library_session.filtered[idx] as usize]
                .clone();
            drain_calls(&app);
            let camera = app.scene.camera_pos();
            let _ = update(&mut app, Message::Click(x, y, MouseButton::Left));
            let calls = drain_calls(&app);
            let applies: Vec<_> =
                calls.iter().filter(|(method, _)| method == "wall.apply").collect();
            assert_eq!(applies.len(), 1, "moving={moving}, current={current}");
            assert_eq!(applies[0].1["path"], json!(expected.path));
            assert_eq!(app.scene.current, idx);
            assert_eq!(app.scene.camera_pos(), camera);
        }
    }
}

#[test]
fn hover_selection_can_be_turned_off() {
    for (mode, turned_off) in [("slices", true), ("hex", true), ("slices", false)] {
        let mut app = test_app();
        let walls: Vec<Value> =
            (0..20).map(|i| wall(&format!("w{i}"), "static", (i * 9) as i64, i as i64)).collect();
        seed(&mut app, &walls);
        let _ = update(&mut app, Message::SetViewMode(mode.into()));
        let mut now = std::time::Instant::now();
        tick_frames(&mut app, &mut now, 30);
        app.config.save_key(skwd_config::keys::general::HOVER_SELECTS, json!(!turned_off));
        let target = app
            .scene
            .render
            .hits
            .iter()
            .find(|hit| hit.index != app.scene.current)
            .copied()
            .expect("another card is visible");
        let _ = update(&mut app, Message::MouseMoved(1.0, 1.0));
        let _ = update(&mut app, Message::MouseMoved(target.cx, target.cy));
        if turned_off {
            assert_eq!(app.scene.current, 0, "{mode}: hover leaves the selection alone");
            assert_eq!(app.scene.hover, Some(target.index), "{mode}: hover still highlights");
        } else {
            assert_eq!(app.scene.current, target.index, "{mode}: hover selects by default");
        }
    }
}

fn setting(app: &mut App, key: &str, on: bool) {
    let toggle = crate::frontend::settings::SettingsMsg::Toggle(key.to_owned(), on);
    let _ = update(app, Message::Settings(toggle));
}

fn reshow(app: &mut App) {
    app.runtime_state.overlay = Some(iced::window::Id::unique());
    app.runtime_state.stashed = true;
    let _ = crate::app::warm::reappear(app);
}

fn tagged_app() -> App {
    let mut app = test_app();
    let mut sunset = wall("a", "static", 10, 1);
    sunset["tags"] = json!("sunset");
    seed(&mut app, &[sunset, wall("b", "static", 20, 2)]);
    app
}

#[test]
fn bar_flag_on_reshow() {
    let mut app = test_app();
    let _ = update(&mut app, Message::ToggleFilterBar);
    assert!(!app.chrome.filter_bar_visible);
    reshow(&mut app);
    assert!(app.chrome.filter_bar_visible);
    let _ = update(&mut app, Message::ToggleFilterBar);
    assert!(!app.chrome.filter_bar_visible);
}

#[test]
fn dialog_return_bar() {
    let mut app = test_app();
    let _ = update(&mut app, Message::ToggleFilterBar);
    app.runtime_state.overlay = Some(iced::window::Id::unique());
    let _ = crate::app::warm::stash(&mut app);
    let _ = crate::app::warm::unstash(&mut app);
    assert!(!app.chrome.filter_bar_visible);
}

fn hover_app() -> App {
    let mut app = test_app();
    let _ = update(&mut app, Message::Viewport(1920.0, 1080.0));
    bar_mode(&mut app, FilterBarVisibility::Hover);
    leave_bar(&mut app);
    app
}

fn setting_text(app: &mut App, path: &str, value: &str) {
    let pick = crate::frontend::settings::SettingsMsg::Pick(path.to_owned(), value.to_owned());
    let _ = update(app, Message::Settings(pick));
}

fn bar_mode(app: &mut App, mode: FilterBarVisibility) {
    let path = skwd_config::keys::general::FILTER_BAR_VISIBILITY.to_owned();
    let pick = crate::frontend::settings::SettingsMsg::Pick(path, mode.key().to_owned());
    let _ = update(app, Message::Settings(pick));
}

fn bar_zone(app: &App) -> iced::Rectangle {
    crate::app::update::arrange::placement::drawn(app, crate::frontend::arrange::Target::Bar)
}

fn point_at(app: &mut App, x: f32, y: f32) {
    let _ = update(app, Message::MouseMoved(x, y));
}

fn hover_bar(app: &mut App) {
    let center = bar_zone(app).center();
    point_at(app, center.x, center.y);
}

fn leave_bar(app: &mut App) {
    let zone = bar_zone(app);
    let (vw, vh) = app.scene.viewport;
    let below = (zone.y + zone.height + vh) * 0.5;
    point_at(app, vw * 0.5, if zone.y > vh * 0.5 { zone.y * 0.5 } else { below });
}

#[test]
fn bar_mode_live() {
    let mut app = test_app();
    for (mode, shown) in [
        (FilterBarVisibility::Hover, false),
        (FilterBarVisibility::Always, true),
        (FilterBarVisibility::Hidden, false),
        (FilterBarVisibility::Always, true),
    ] {
        bar_mode(&mut app, mode);
        assert_eq!(app.chrome.filter_bar_shown(), shown, "{mode:?}");
    }
}

#[test]
fn hover_reveals_bar() {
    let mut app = hover_app();
    hover_bar(&mut app);
    assert!(app.chrome.filter_bar_shown());
    assert_eq!(app.chrome.filter_bar_reveal.target, 1.0);
    leave_bar(&mut app);
    assert!(!app.chrome.filter_bar_shown());
    assert_eq!(app.chrome.filter_bar_reveal.target, 0.0);
}

#[test]
fn hover_keeps_open_menu() {
    let mut app = hover_app();
    hover_bar(&mut app);
    app.chrome.bar.menu = Some(crate::frontend::ui::MenuKind::Folders);
    leave_bar(&mut app);
    assert!(app.chrome.filter_bar_shown());
    app.chrome.bar.menu = None;
    leave_bar(&mut app);
    assert!(!app.chrome.filter_bar_shown());
}

#[test]
fn hidden_ignores_pointer() {
    let mut app = hover_app();
    bar_mode(&mut app, FilterBarVisibility::Hidden);
    hover_bar(&mut app);
    assert!(!app.chrome.filter_bar_shown());
}

#[test]
fn keybind_toggles_modes() {
    for mode in FilterBarVisibility::ALL {
        let mut app = test_app();
        bar_mode(&mut app, mode);
        let before = app.chrome.filter_bar_shown();
        let _ = update(&mut app, Message::ToggleFilterBar);
        assert_eq!(app.chrome.filter_bar_shown(), !before, "{mode:?}");
        let _ = update(&mut app, Message::ToggleFilterBar);
        assert_eq!(app.chrome.filter_bar_shown(), before, "{mode:?}");
    }
}

#[test]
fn keybind_sticks_over_hover() {
    let mut app = hover_app();
    let _ = update(&mut app, Message::ToggleFilterBar);
    leave_bar(&mut app);
    assert!(app.chrome.filter_bar_shown());
    hover_bar(&mut app);
    let _ = update(&mut app, Message::ToggleFilterBar);
    assert!(!app.chrome.filter_bar_shown());
    let center = bar_zone(&app).center();
    point_at(&mut app, center.x + 5.0, center.y);
    assert!(!app.chrome.filter_bar_shown());
    leave_bar(&mut app);
    hover_bar(&mut app);
    assert!(app.chrome.filter_bar_shown());
}

fn use_profile(app: &mut App, document: &serde_json::Value) {
    use skwd_config::picker_layout::{LayoutProfile, Size};
    let size = Size { width: 1920.0, height: 1080.0 };
    app.config.set_layout_profile("DP-1", &LayoutProfile::capture(document, size)).unwrap();
    app.config.select_layout_output(Some(String::from("DP-1")), size);
    app.runtime_state.picker_output = Some(String::from("DP-1"));
    app.daemon.output_statuses = vec![crate::contracts::daemon::OutputStatus {
        name: String::from("DP-1"),
        connected: true,
        width: 1920,
        height: 1080,
        logical_width: 1920,
        logical_height: 1080,
        ..crate::contracts::daemon::OutputStatus::default()
    }];
}

#[test]
fn hover_follows_placement() {
    use skwd_config::picker_layout::{Alignment, Component, Offset, Position, Reference};
    let mut app = hover_app();
    let home = bar_zone(&app);
    use_profile(&mut app, &json!({}));
    let corner = Position {
        reference: Reference::Screen,
        alignment: Alignment::BottomRight,
        offset: Offset { x: 0.0, y: 0.0 },
    };
    app.config.set_position(Component::Bar, Some(corner)).unwrap();
    bar_mode(&mut app, FilterBarVisibility::Hover);
    let placed = bar_zone(&app);
    assert!(!placed.contains(home.center()));
    point_at(&mut app, home.center().x, home.center().y);
    assert!(!app.chrome.filter_bar_shown());
    point_at(&mut app, placed.center().x, placed.center().y);
    assert!(app.chrome.filter_bar_shown());
}

#[test]
fn profile_switch_applies_mode() {
    let mut app = hover_app();
    assert!(!app.chrome.filter_bar_shown());
    use_profile(&mut app, &json!({"general": {"filterBarVisibility": "always"}}));
    let _ = update(&mut app, Message::Viewport(1920.0, 1080.0));
    assert!(app.chrome.filter_bar_shown());
}

#[test]
fn hover_follows_orientation() {
    let mut app = hover_app();
    let across = bar_zone(&app);
    let path = skwd_config::keys::filter_bar::ORIENTATION.to_owned();
    let pick = crate::frontend::settings::SettingsMsg::Pick(path, String::from("vertical"));
    let _ = update(&mut app, Message::Settings(pick));
    for _ in 0..30 {
        app.chrome.tick_filter_bar(0.05);
    }
    assert!(app.chrome.filter_bar_vertical);
    let down = bar_zone(&app);
    assert!(!across.contains(down.center()));
    point_at(&mut app, down.center().x, down.center().y);
    assert!(app.chrome.filter_bar_shown());
}

#[test]
fn hover_follows_resize() {
    let mut app = hover_app();
    let wide = bar_zone(&app);
    let _ = update(&mut app, Message::Viewport(1280.0, 720.0));
    let small = bar_zone(&app);
    assert!(!wide.contains(small.center()));
    point_at(&mut app, small.center().x, small.center().y);
    assert!(app.chrome.filter_bar_shown());
}

fn placed_zone(app: &mut App) -> iced::Rectangle {
    app.place_hover_zone();
    app.chrome.filter_bar_hover_zone
}

fn settle_bar(app: &mut App) {
    for _ in 0..30 {
        app.chrome.tick_filter_bar(0.05);
    }
}

#[test]
fn zone_horizontal() {
    let mut app = hover_app();
    assert_eq!(placed_zone(&mut app), bar_zone(&app));
}

#[test]
fn zone_offset() {
    use skwd_config::keys::filter_bar;
    let mut app = hover_app();
    setting_text(&mut app, filter_bar::OFFSET_X, "40");
    assert_eq!(placed_zone(&mut app), bar_zone(&app));
}

#[test]
fn zone_vertical() {
    use skwd_config::keys::filter_bar;
    let mut app = hover_app();
    setting_text(&mut app, filter_bar::ORIENTATION, "vertical");
    settle_bar(&mut app);
    assert_eq!(placed_zone(&mut app), bar_zone(&app));
}

#[test]
fn zone_arranged() {
    use skwd_config::picker_layout::{Alignment, Component, Offset, Position, Reference};
    let mut app = hover_app();
    use_profile(&mut app, &json!({}));
    let corner = Position {
        reference: Reference::Screen,
        alignment: Alignment::TopRight,
        offset: Offset { x: -20.0, y: 30.0 },
    };
    app.config.set_position(Component::Bar, Some(corner)).unwrap();
    bar_mode(&mut app, FilterBarVisibility::Hover);
    settle_bar(&mut app);
    assert_eq!(placed_zone(&mut app), bar_zone(&app));
}

#[test]
fn hover_follows_folders() {
    let mut app = hover_app();
    let empty = placed_zone(&mut app);
    seed(&mut app, &[wall("nature/a.png", "static", 1, 0), wall("zebra/b.png", "static", 2, 0)]);
    let full = bar_zone(&app);
    let end = iced::Point::new(full.x + full.width - 1.0, full.center().y);
    assert!(!empty.contains(end));
    point_at(&mut app, end.x, end.y);
    assert!(app.chrome.filter_bar_shown());
}

fn profiled_app(global: bool, profile: bool) -> App {
    crate::app::tests::app_from(json!({
        "general": { "filterBarAlwaysVisible": global },
        "pickerLayouts": { "version": 1, "outputs": { "DP-1": {
            "viewport": { "width": 1920.0, "height": 1080.0 },
            "values": { "general.filterBarAlwaysVisible": profile },
            "positions": {}
        }}}
    }))
}

fn launch(mut app: App) -> Vec<f32> {
    let id = iced::window::Id::unique();
    let _ = update(&mut app, Message::WindowOpened { id, width: 1920.0, height: 1080.0 });
    assert!(app.runtime_state.picker_output.is_none());
    crate::app::warm::sync_output(&mut app, Some(String::from("DP-1")));
    let mut reveals = Vec::new();
    for _ in 0..10 {
        reveals.push(app.chrome.filter_bar_fade());
        app.chrome.tick_filter_bar(0.016);
    }
    reveals
}

#[test]
fn late_output_starts_resolved() {
    assert!(launch(profiled_app(true, false)).iter().all(|reveal| *reveal == 0.0));
    assert!(launch(profiled_app(false, true)).iter().all(|reveal| *reveal == 1.0));
}

#[test]
fn forced_bar_starts_shown() {
    let mut app = profiled_app(true, false);
    app.chrome.filter_bar_forced = true;
    assert!(launch(app).iter().all(|reveal| *reveal == 1.0));
}

fn unopened(launch: &str) -> App {
    let mut config = test_app().config;
    config.set_key(skwd_config::keys::launch::ANIMATION, json!(launch));
    App::with_config_using(config, |_| crate::infrastructure::ipc::DaemonClient::recording())
}

fn open_window(app: &mut App) {
    let id = iced::window::Id::unique();
    let _ = update(app, Message::WindowOpened { id, width: 1280.0, height: 720.0 });
}

#[test]
fn entrance_waits_for_window() {
    let mut app = unopened("fade");
    let mut now = Instant::now();
    for _ in 0..10 {
        tick_frames(&mut app, &mut now, 1);
        assert_eq!(app.scene.open_fade(), 0.0);
    }
    open_window(&mut app);
    let mut last = app.scene.open_fade();
    for _ in 0..60 {
        tick_frames(&mut app, &mut now, 1);
        assert!(app.scene.open_fade() >= last);
        last = app.scene.open_fade();
    }
    assert!(last > 0.99);
}

#[test]
fn reload_keeps_waiting() {
    let mut app = unopened("fade");
    app.config.set_key(skwd_config::keys::general::OPEN_FADE_FROM, json!(60));
    app.apply_layout();
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 5);
    assert_eq!(app.scene.open_fade(), 0.0);
}

#[test]
fn browser_wall_visible() {
    let app = unopened("fade");
    assert_eq!(app.source_browser.wall.scene.open_fade(), 1.0);
}

#[test]
fn instant_launch_shows() {
    let mut app = unopened("none");
    open_window(&mut app);
    assert_eq!(app.scene.open_fade(), 1.0);
}

#[test]
fn reshow_snaps_bar() {
    let mut app = hover_app();
    let _ = update(&mut app, Message::ToggleFilterBar);
    settle_bar(&mut app);
    assert_eq!(app.chrome.filter_bar_fade(), 1.0);
    reshow(&mut app);
    assert_eq!(app.chrome.filter_bar_fade(), 0.0);
}

#[test]
fn jitter_keeps_hide() {
    let mut app = hover_app();
    hover_bar(&mut app);
    for _ in 0..3 {
        let _ = update(&mut app, Message::ToggleFilterBar);
    }
    assert!(!app.chrome.filter_bar_shown());
    let center = bar_zone(&app).center();
    point_at(&mut app, center.x + 5.0, center.y);
    assert!(!app.chrome.filter_bar_shown());
}

#[test]
fn pointer_exit_hides_bar() {
    let mut app = hover_app();
    hover_bar(&mut app);
    let _ = update(&mut app, Message::MouseLeft);
    assert!(!app.chrome.filter_bar_shown());
}

#[test]
fn settings_stop_hover() {
    let mut app = hover_app();
    hover_bar(&mut app);
    let _ = update(&mut app, Message::ToggleSettings);
    assert!(!app.chrome.filter_bar_shown());
}

#[test]
fn bar_command_hides_hover() {
    let mut app = hover_app();
    hover_bar(&mut app);
    let command = crate::infrastructure::runtime::Wake::Command(String::from("bar toggle"));
    let _ = update(&mut app, Message::Daemon(command));
    assert!(!app.chrome.filter_bar_shown());
    let center = bar_zone(&app).center();
    point_at(&mut app, center.x + 5.0, center.y);
    assert!(!app.chrome.filter_bar_shown());
}

#[test]
fn search_flag_live() {
    let mut app = test_app();
    setting(&mut app, skwd_config::keys::general::SEARCH_ALWAYS_VISIBLE, true);
    assert!(app.tags.cloud_open);
    setting(&mut app, skwd_config::keys::general::SEARCH_ALWAYS_VISIBLE, false);
    assert!(!app.tags.cloud_open);
}

#[test]
fn flags_follow_external_config() {
    let mut app = test_app();
    let flags =
        json!({ "general": { "filterBarVisibility": "hover", "searchAlwaysVisible": true } });
    std::fs::write(&app.config.config_path, flags.to_string()).unwrap();
    assert!(app.adopt_external_config());
    assert!(!app.chrome.filter_bar_visible);
    assert!(app.tags.cloud_open);
    let flags =
        json!({ "general": { "filterBarVisibility": "always", "searchAlwaysVisible": false } });
    std::fs::write(&app.config.config_path, flags.to_string()).unwrap();
    assert!(app.adopt_external_config());
    assert!(app.chrome.filter_bar_visible);
    assert!(!app.tags.cloud_open);
}

#[test]
fn search_flag_reshow_escape() {
    let mut app = tagged_app();
    setting(&mut app, skwd_config::keys::general::SEARCH_ALWAYS_VISIBLE, true);
    let _ = update(&mut app, Message::Tag(TagMsg::CloudClick(String::from("sunset"), false)));
    let _ = update(&mut app, Message::OpenTagCloud);
    assert!(!app.tags.cloud_open);
    reshow(&mut app);
    assert!(app.tags.cloud_open);
    let _ = update(&mut app, Message::Exit);
    assert!(!app.tags.cloud_open);
    assert_eq!(app.library_session.filters.tags, ["sunset"]);
}
