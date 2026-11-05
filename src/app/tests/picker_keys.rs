use super::*;
use crate::contracts::settings::wallpaper_kind::{STATIC, VIDEO, WE};
use crate::frontend::browser::Source;
use iced::keyboard::Modifiers;
use iced::keyboard::key::Named;

fn press(app: &mut App, key: keyboard::Key, modifiers: Modifiers) {
    let _ = update(app, Message::KeyPressed(key, modifiers));
}

fn tab(app: &mut App, modifiers: Modifiers) {
    press(app, keyboard::Key::Named(Named::Tab), modifiers);
}

fn character(app: &mut App, text: &str, modifiers: Modifiers) {
    press(app, keyboard::Key::Character(text.into()), modifiers);
}

fn source(app: &App) -> Option<Source> {
    app.source_browser.browser.as_ref().map(|browser| browser.source)
}

#[test]
fn tab_cycles_wallpaper_types_and_wraps() {
    let mut app = test_app();
    for expected in [STATIC, VIDEO, WE, "", STATIC] {
        tab(&mut app, Modifiers::default());
        assert_eq!(app.library_session.filters.kind, expected);
    }
    for expected in ["", WE] {
        tab(&mut app, Modifiers::SHIFT);
        assert_eq!(app.library_session.filters.kind, expected);
    }
}

#[test]
fn type_keys_skip_hidden_chips_and_respect_overlays() {
    let mut app = test_app();
    app.config.set_key("filterBar.show.type.video", json!(false));
    tab(&mut app, Modifiers::default());
    tab(&mut app, Modifiers::default());
    assert_eq!(app.library_session.filters.kind, WE);
    let _ = update(&mut app, Message::OpenPlaylists);
    tab(&mut app, Modifiers::default());
    assert_eq!(app.library_session.filters.kind, WE);
}

#[test]
fn tab_completes_in_search_instead_of_switching_types() {
    let mut app = test_app();
    let _ = update(&mut app, Message::OpenTagCloud);
    tab(&mut app, Modifiers::default());
    tab(&mut app, Modifiers::SHIFT);
    assert_eq!(app.library_session.filters.kind, "");
    assert!(app.tags.cloud_open);
}

#[test]
fn alt_arrows_cycle_sort_chips() {
    let mut app = test_app();
    let sorts: Vec<&str> = crate::frontend::ui::SORTS.iter().map(|(mode, _)| *mode).collect();
    let _ = update(&mut app, Message::SetSort(sorts[0].to_string()));
    for expected in sorts.iter().skip(1).chain(sorts.iter().take(1)) {
        press(&mut app, keyboard::Key::Named(Named::ArrowRight), Modifiers::ALT);
        assert_eq!(&app.library_session.filters.sort, expected);
    }
    press(&mut app, keyboard::Key::Named(Named::ArrowLeft), Modifiers::ALT);
    assert_eq!(app.library_session.filters.sort, *sorts.last().unwrap());
    app.config.set_key(&format!("filterBar.show.sort.{}", sorts[0]), json!(false));
    press(&mut app, keyboard::Key::Named(Named::ArrowRight), Modifiers::ALT);
    assert_eq!(app.library_session.filters.sort, sorts[1]);
    let _ = update(&mut app, Message::OpenPlaylists);
    press(&mut app, keyboard::Key::Named(Named::ArrowRight), Modifiers::ALT);
    assert_eq!(app.library_session.filters.sort, sorts[1]);
}

#[test]
fn ctrl_r_toggles_random_rotation_only_in_the_picker() {
    let mut app = test_app();
    character(&mut app, "r", Modifiers::CTRL);
    assert!(app.config.flag_default_config("general.randomRotate"));
    let _ = update(&mut app, Message::OpenPlaylists);
    character(&mut app, "r", Modifiers::CTRL);
    assert!(app.config.flag_default_config("general.randomRotate"));
    let _ = update(&mut app, Message::ClosePlaylists);
    character(&mut app, "r", Modifiers::CTRL);
    assert!(!app.config.flag_default_config("general.randomRotate"));
}

#[test]
fn ctrl_d_toggles_downloads_and_digits_switch_sources() {
    let mut app = test_app();
    app.config.set_key("sources.bing.enabled", json!(true));
    character(&mut app, "6", Modifiers::default());
    assert_eq!(source(&app), None);
    character(&mut app, "d", Modifiers::CTRL);
    assert_eq!(source(&app), Some(Source::Wallhaven));
    character(&mut app, "6", Modifiers::default());
    assert_eq!(source(&app), Some(Source::Bing));
    character(&mut app, "3", Modifiers::default());
    assert_eq!(source(&app), Some(Source::Bing));
    character(&mut app, "1", Modifiers::default());
    assert_eq!(source(&app), Some(Source::Wallhaven));
    character(&mut app, "d", Modifiers::CTRL);
    assert_eq!(source(&app), None);
    app.config.set_key("features.wallhaven", json!(false));
    app.config.set_key("features.steam", json!(false));
    character(&mut app, "d", Modifiers::CTRL);
    assert_eq!(source(&app), None);
}

#[test]
fn ctrl_tab_switches_search_mode_inside_search() {
    let mut app = test_app();
    tab(&mut app, Modifiers::CTRL);
    assert_eq!(app.tags.search_mode, SearchMode::Tags);
    let _ = update(&mut app, Message::OpenTagCloud);
    tab(&mut app, Modifiers::CTRL);
    assert_eq!(app.tags.search_mode, SearchMode::Describe);
    tab(&mut app, Modifiers::CTRL);
    assert_eq!(app.tags.search_mode, SearchMode::Tags);
}

#[test]
fn text_fields_keep_editing_keys_and_release_the_rest() {
    let reaches = |key: keyboard::Key, modifiers| {
        crate::app::input::reaches_bindings_from_text(&key, modifiers)
    };
    let ctrl_alt = Modifiers::CTRL | Modifiers::ALT;
    assert!(reaches(keyboard::Key::Character("d".into()), Modifiers::CTRL));
    assert!(reaches(keyboard::Key::Character("r".into()), Modifiers::CTRL));
    assert!(reaches(keyboard::Key::Named(Named::ArrowRight), Modifiers::ALT));
    assert!(reaches(keyboard::Key::Character("v".into()), ctrl_alt));
    for reserved in ["a", "c", "v", "x"] {
        assert!(!reaches(keyboard::Key::Character(reserved.into()), Modifiers::CTRL), "{reserved}");
    }
    for reserved in [Named::ArrowLeft, Named::ArrowRight, Named::Backspace, Named::Delete] {
        assert!(!reaches(keyboard::Key::Named(reserved), Modifiers::CTRL), "{reserved:?}");
    }
    assert!(!reaches(keyboard::Key::Character("r".into()), Modifiers::default()));
    assert!(!reaches(keyboard::Key::Named(Named::ArrowRight), Modifiers::SHIFT));
}

#[test]
fn search_field_still_runs_modifier_shortcuts() {
    let mut app = test_app();
    let _ = update(&mut app, Message::OpenTagCloud);
    character(&mut app, "r", Modifiers::CTRL);
    assert!(app.config.flag_default_config("general.randomRotate"));
    let sorts: Vec<&str> = crate::frontend::ui::SORTS.iter().map(|(mode, _)| *mode).collect();
    let started = app.library_session.filters.sort.clone();
    let next = sorts[(sorts.iter().position(|mode| *mode == started).unwrap() + 1) % sorts.len()];
    press(&mut app, keyboard::Key::Named(Named::ArrowRight), Modifiers::ALT);
    assert_eq!(app.library_session.filters.sort, next);
    assert!(app.tags.cloud_open);
    character(&mut app, "d", Modifiers::CTRL);
    assert_eq!(source(&app), Some(Source::Wallhaven));
    assert!(!app.tags.cloud_open);
}

#[test]
fn ui_state_reports_the_sort_chip_and_download_source() {
    let mut app = test_app();
    let state = |app: &App| -> serde_json::Value {
        serde_json::from_str(&crate::app::update::ui_state_json(app)).unwrap()
    };
    press(&mut app, keyboard::Key::Named(Named::ArrowRight), Modifiers::ALT);
    assert_eq!(state(&app)["filter"]["sort"], json!(app.library_session.filters.sort));
    assert_eq!(state(&app)["downloads"]["open"], json!(false));
    character(&mut app, "d", Modifiers::CTRL);
    assert_eq!(state(&app)["downloads"]["open"], json!(true));
    assert_eq!(state(&app)["downloads"]["source"], json!("wallhaven"));
}

#[test]
fn m_cycles_picker_modes_and_wraps() {
    let mut app = test_app();
    assert_eq!(app.config.display_mode(), "slices");
    for expected in ["depth", "hex", "wall", "sandy", "hand", "collection", "slices"] {
        character(&mut app, "m", Modifiers::default());
        assert_eq!(app.config.display_mode(), expected);
    }
    for expected in ["collection", "hand"] {
        character(&mut app, "M", Modifiers::SHIFT);
        assert_eq!(app.config.display_mode(), expected);
    }
}

#[test]
fn m_types_into_search_instead_of_switching_mode() {
    let mut app = test_app();
    let _ = update(&mut app, Message::OpenTagCloud);
    character(&mut app, "m", Modifiers::default());
    assert_eq!(app.config.display_mode(), "slices");
}

fn paged_app(mode: &str, count: usize) -> (App, std::time::Instant) {
    let mut app = test_app();
    let walls: Vec<Value> =
        (0..count).map(|i| wall(&format!("w{i}"), "static", (i * 7) as i64, i as i64)).collect();
    seed(&mut app, &walls);
    let _ = update(&mut app, Message::SetViewMode(mode.into()));
    let mut now = std::time::Instant::now();
    tick_frames(&mut app, &mut now, 30);
    (app, now)
}

fn named(app: &mut App, key: Named) {
    press(app, keyboard::Key::Named(key), Modifiers::default());
}

#[test]
fn home_and_end_jump_to_the_edges() {
    for mode in ["slices", "wall", "hex", "hand", "collection"] {
        let (mut app, _) = paged_app(mode, 60);
        named(&mut app, Named::End);
        assert_eq!(app.scene.current, 59, "{mode}");
        named(&mut app, Named::Home);
        assert_eq!(app.scene.current, 0, "{mode}");
    }
}

#[test]
fn page_keys_move_a_screenful_and_keep_the_grid_column() {
    let (mut app, _) = paged_app("wall", 200);
    let cols = app.scene.gp.cols.max(1);
    let visible = app.scene.render.hits.len();
    assert!(visible > cols, "grid shows more than one row: {visible} hits, {cols} cols");
    named(&mut app, Named::ArrowRight);
    let start = app.scene.current;
    named(&mut app, Named::PageDown);
    let step = app.scene.current - start;
    assert_eq!(step % cols, 0, "step {step} keeps the column of {cols}");
    assert_eq!(step, visible / cols * cols);
    named(&mut app, Named::PageUp);
    assert_eq!(app.scene.current, start);
    named(&mut app, Named::PageUp);
    assert_eq!(app.scene.current, 0);
}

#[test]
fn page_keys_step_by_visible_slices_and_clamp() {
    let (mut app, _) = paged_app("slices", 30);
    let visible = app.scene.render.hits.len().max(1);
    named(&mut app, Named::PageDown);
    assert_eq!(app.scene.current, visible.min(29));
    for _ in 0..30 {
        named(&mut app, Named::PageDown);
    }
    assert_eq!(app.scene.current, 29);
}

#[test]
fn page_and_edge_keys_stay_out_of_open_panels() {
    let (mut app, _) = paged_app("wall", 60);
    let _ = update(&mut app, Message::ToggleSettings);
    named(&mut app, Named::End);
    named(&mut app, Named::PageDown);
    assert_eq!(app.scene.current, 0);
}

fn applied_path(app: &App) -> Option<String> {
    drain_calls(app)
        .into_iter()
        .rev()
        .find(|(method, _)| method == "wall.apply")
        .and_then(|(_, params)| params.get("path").and_then(Value::as_str).map(str::to_string))
}

#[test]
fn random_wallpaper_key_applies_another_card() {
    let (mut app, _) = paged_app("slices", 12);
    let _ = update(&mut app, Message::ToggleSettings);
    let _ = update(
        &mut app,
        Message::Settings(crate::frontend::settings::SettingsMsg::KeybindCapture(
            "keys.randomApply".into(),
            0,
        )),
    );
    character(&mut app, "r", Modifiers::default());
    let _ = update(&mut app, Message::KeyReleased(keyboard::Key::Character("r".into())));
    let _ = update(&mut app, Message::ToggleSettings);
    assert!(!app.panels.settings.open);
    drain_calls(&app);
    let mut seen = std::collections::HashSet::new();
    for _ in 0..40 {
        let before = app.scene.current;
        character(&mut app, "r", Modifiers::default());
        assert_ne!(app.scene.current, before, "random never repeats the current card");
        let path = applied_path(&app).expect("random key applies a wallpaper");
        let current = &app.library_session.library.catalog().items
            [app.library_session.filtered[app.scene.current] as usize];
        assert!(path.ends_with(&current.name), "{path} applies the selected card {}", current.name);
        seen.insert(app.scene.current);
    }
    assert!(seen.len() > 3, "random picks spread across the library: {seen:?}");
}

#[test]
fn random_wallpaper_is_unbound_by_default() {
    let (mut app, _) = paged_app("slices", 12);
    drain_calls(&app);
    character(&mut app, "r", Modifiers::default());
    assert_eq!(app.scene.current, 0);
    assert_eq!(applied_path(&app), None);
}
