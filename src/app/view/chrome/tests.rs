#![cfg(test)]

use serde_json::json;

use super::{bar_intent_message, filter_bar_footprint, filter_bar_geometry};
use crate::app::tests::test_app;

#[test]
fn theme_mode_controls_share_the_effective_persisted_value() {
    use crate::contracts::settings::SettingsSource;
    use crate::infrastructure::config::Config;

    for (root, expected) in [
        (json!({}), "dark"),
        (json!({"matugen": {"mode": "light"}}), "light"),
        (json!({"matugen": {"mode": "dark"}}), "dark"),
        (json!({"theme": {"mode": "light"}, "matugen": {"mode": "dark"}}), "light"),
        (json!({"theme": {"mode": "dark"}, "matugen": {"mode": "light"}}), "dark"),
        (json!({"theme": {"mode": "auto"}, "matugen": {"mode": "light"}}), "auto"),
        (json!({"theme": {"mode": ""}, "matugen": {"mode": "light"}}), "light"),
    ] {
        let mut app = test_app();
        app.config = Config::from_data(root);
        assert_eq!(SettingsSource::text(&app.config, skwd_config::keys::theme::MODE), expected);
        assert_eq!(super::theme_bar_model(&app, "skwd-iris", false, 1).mode, expected);
    }
}

#[test]
fn pinned_mode_precedes_the_effective_global_mode() {
    use crate::infrastructure::ipc::IpcMsg;

    for (pinned, value, expected) in [
        (true, json!("dark"), "dark"),
        (true, json!("auto"), "auto"),
        (true, json!(""), "light"),
        (true, json!(false), "light"),
        (false, json!("dark"), "light"),
    ] {
        let mut app = test_app();
        app.config.set_key(skwd_config::keys::matugen::MODE, json!("light"));
        let _ = app.daemon.client.call("subscribe", json!({"events": ["skwd."]}));
        let list_id = app.daemon.client.call("wall.list", json!({}));
        app.handle_ipc(IpcMsg::Connected { list_id });
        app.handle_ipc(IpcMsg::Response {
            id: list_id,
            result: Some(json!({"wallpapers": [{
                "name": "pinned.png", "type": "static", "thumb": "/thumbs/pinned.png",
                "hue": 1, "mtime": 0
            }]})),
            error: None,
        });
        app.scene.set_current(0, app.library_session.filtered.len());
        let key = app.library_session.library.catalog().items[0].key.clone();
        app.config.set_key(
            skwd_config::keys::theme::WALLPAPER_PROFILES,
            json!([{"key": key, "settingsPinned": pinned, "settings": {"theme.mode": value}}]),
        );
        assert_eq!(super::theme_bar_model(&app, "skwd-iris", false, 1).mode, expected);
    }
}

#[test]
fn committed_theme_modes_reopen_in_the_bar_and_settings() {
    use crate::app::{App, Message, update};
    use crate::contracts::settings::SettingsSource;
    use crate::frontend::theme_designer::ThemeMsg;
    use crate::infrastructure::config::Config;

    let mut app = test_app();
    app.config.set_key(skwd_config::keys::matugen::MODE, json!("light"));
    for mode in ["dark", "light", "auto"] {
        let _ = update(
            &mut app,
            Message::Theme(ThemeMsg::Option(crate::contracts::picker::theme_setting::MODE, mode)),
        );
        let saved: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&app.config.config_path).unwrap()).unwrap();
        assert_eq!(saved["theme"]["mode"], mode);
        assert_eq!(saved["matugen"]["mode"], "light");
        let mut reopened = Config::from_data(json!({}));
        reopened.config_path.clone_from(&app.config.config_path);
        assert!(reopened.reload());
        app = App::with_config_using(reopened, |_| {
            crate::infrastructure::ipc::DaemonClient::recording()
        });
        assert_eq!(super::theme_bar_model(&app, "skwd-iris", false, 1).mode, mode);
        assert_eq!(SettingsSource::text(&app.config, skwd_config::keys::theme::MODE), mode);
    }
}

#[test]
fn filter_bar_intents_translate_at_composition() {
    use crate::app::Message;
    use crate::frontend::ui::{BarAction, BarIntent};

    assert!(matches!(
        bar_intent_message(BarIntent::Hover(Some(2), Some(3))),
        Message::BarHover(Some(2), Some(3))
    ));
    assert!(matches!(
        bar_intent_message(BarIntent::Activate(BarAction::Resolution(String::from("4k")))),
        Message::SetResolution(value) if value == "4k"
    ));
    assert!(matches!(
        bar_intent_message(BarIntent::SelectFolder(String::from("nature"))),
        Message::SetFolder(value) if value == "nature"
    ));
    assert!(matches!(
        bar_intent_message(BarIntent::StepVolume(-10)),
        Message::Audio(crate::frontend::audio_panel::AudioMsg::VolumeStep(-10))
    ));
    assert!(matches!(
        bar_intent_message(BarIntent::Theme(
            crate::frontend::theme_designer::ThemeMsg::BackendMenu
        )),
        Message::Theme(crate::frontend::theme_designer::ThemeMsg::BackendMenu)
    ));
}

#[test]
fn footprint_tracks_controls_and_scale() {
    let mut app = test_app();
    let (_, base_w, base_h) = filter_bar_footprint(&app, 5000.0, 3000.0);
    assert!(base_w > 0.0 && base_h > 0.0);

    for key in ["orient", "resolution", "colors", "theme", "random", "favourites", "folder"] {
        app.config.set_key(&format!("filterBar.show.{key}"), json!(false));
    }
    let (_, trimmed_w, _) = filter_bar_footprint(&app, 5000.0, 3000.0);
    assert!(trimmed_w < base_w, "{trimmed_w} vs {base_w}");

    app.config.set_key(skwd_config::keys::general::UI_SCALE, json!(2.0));
    let (_, scaled_w, scaled_h) = filter_bar_footprint(&app, 5000.0, 3000.0);
    assert!(scaled_w > trimmed_w && scaled_h > base_h, "{scaled_w}x{scaled_h}");
}

#[test]
fn bar_position_stays_fixed() {
    use skwd_config::keys::{filter_bar, selector};

    let mut app = test_app();
    let viewport = (5000.0, 3000.0);
    let original =
        filter_bar_geometry(&app, viewport.0, viewport.1, crate::app::view::BarExtent::Expanded).1;
    app.config.set_key(filter_bar::POSITION_X, json!(25.0));
    app.config.set_key(filter_bar::POSITION_Y, json!(40.0));
    assert_eq!(
        filter_bar_geometry(&app, viewport.0, viewport.1, crate::app::view::BarExtent::Expanded).1,
        original
    );

    app.config.set_key(filter_bar::POSITION_ENABLED, json!(true));
    let anchored =
        filter_bar_geometry(&app, viewport.0, viewport.1, crate::app::view::BarExtent::Expanded).1;
    assert_eq!((anchored.x, anchored.y), (1250.0, 1200.0));
    app.config.set_key(filter_bar::OFFSET_X, json!(100.0));
    app.config.set_key(selector::GRID_COLUMNS, json!(4));
    app.config.set_key(selector::GRID_ROWS, json!(4));
    app.apply_layout();
    let after =
        filter_bar_geometry(&app, viewport.0, viewport.1, crate::app::view::BarExtent::Expanded).1;
    assert_eq!((after.x, after.y), (anchored.x, anchored.y));
}

#[test]
fn bar_position_fits_output() {
    use skwd_config::keys::{filter_bar, general};

    let mut app = test_app();
    app.config.set_key(filter_bar::POSITION_ENABLED, json!(true));
    app.config.set_key(filter_bar::POSITION_X, json!(60.0));
    app.config.set_key(filter_bar::POSITION_Y, json!(90.0));
    for scale in [1.0, 2.0] {
        app.config.set_key(general::UI_SCALE, json!(scale));
        let (_, bar) =
            filter_bar_geometry(&app, 650.0, 400.0, crate::app::view::BarExtent::Expanded);
        assert!(bar.x + bar.width <= 650.0);
        assert!(bar.y + bar.height <= 400.0);
        assert_eq!(app.config.filter_bar_position(), Some((0.6, 0.9)));
    }
}

#[test]
fn bar_menu_stays_put() {
    for mode in
        [crate::frontend::scene::layout::Mode::Grid, crate::frontend::scene::layout::Mode::Sandy]
    {
        for y in [0.0, 90.0] {
            let mut app = positioned_bar(mode, y);
            for scale in [1.0, 2.0] {
                app.config.set_key(skwd_config::keys::general::UI_SCALE, json!(scale));
                assert_bar_menu_position(&mut app, (650.0, 400.0), Some(y > 0.0));
            }
        }
    }
}

fn positioned_bar(mode: crate::frontend::scene::layout::Mode, y: f32) -> crate::app::App {
    use skwd_config::keys::filter_bar;

    let mut app = test_app();
    app.scene.set_mode(mode);
    app.library_session.folder_options =
        vec![String::from("nature"), String::from("nature/forest"), String::from("cities")];
    app.config.set_key(filter_bar::POSITION_ENABLED, json!(true));
    app.config.set_key(filter_bar::POSITION_X, json!(60.0));
    app.config.set_key(filter_bar::POSITION_Y, json!(y));
    app
}

fn assert_bar_menu_position(app: &mut crate::app::App, viewport: (f32, f32), up: Option<bool>) {
    let (closed, bounds) =
        filter_bar_geometry(app, viewport.0, viewport.1, crate::app::view::BarExtent::Expanded);
    let first = closed.items.first().unwrap();
    let (x, y) = (bounds.x + first.x, bounds.y + first.y);
    app.chrome.bar.menu = Some(crate::frontend::ui::MenuKind::Folders);
    let (open, bounds) =
        filter_bar_geometry(app, viewport.0, viewport.1, crate::app::view::BarExtent::Expanded);
    let first = open.items.first().unwrap();
    assert!(open.height > closed.height);
    if let Some(up) = up {
        assert_eq!(open.menu_up, up);
    }
    assert!(bounds.x >= 0.0);
    assert!(bounds.x + bounds.width <= viewport.0);
    assert!(bounds.y >= 0.0);
    assert!(bounds.y + bounds.height <= viewport.1);
    assert!((bounds.x + first.x - x).abs() < 0.001);
    assert!((bounds.y + first.y - y).abs() < 0.001);
    app.chrome.bar.menu = None;
}

#[test]
fn long_menu_fits_output() {
    for mode in
        [crate::frontend::scene::layout::Mode::Grid, crate::frontend::scene::layout::Mode::Sandy]
    {
        let mut app = positioned_bar(mode, 50.0);
        app.config.set_key(skwd_config::keys::general::UI_SCALE, json!(2.0));
        app.scene.viewport = (650.0, 400.0);
        app.library_session.folder_options =
            (0..20).map(|index| format!("folder-{index}")).collect();
        assert_bar_menu_position(&mut app, (650.0, 400.0), None);
        app.chrome.bar.menu = Some(crate::frontend::ui::MenuKind::Folders);
        let (model, _) =
            filter_bar_geometry(&app, 650.0, 400.0, crate::app::view::BarExtent::Expanded);
        assert!(model.menu_visible_rows < crate::frontend::ui::MENU_MAX_ROWS);
        let _ = crate::app::update(&mut app, crate::app::Message::FolderMenuScroll(10000.0));
        let expected = (20 - model.menu_visible_rows) as f32
            * crate::frontend::ui::MENU_ROW_H
            * app.config.ui_scale();
        assert_eq!(app.chrome.bar.menu_scroll.target, expected);
    }
}

#[test]
fn vertical_menu_fits_output() {
    for y in [0.0, 90.0] {
        let mut app = positioned_bar(crate::frontend::scene::layout::Mode::Grid, y);
        app.chrome.filter_bar_vertical = true;
        app.scene.viewport = (650.0, 400.0);
        let viewport = app.scene.viewport;
        for scale in [1.0, 2.0] {
            app.config.set_key(skwd_config::keys::general::UI_SCALE, json!(scale));
            assert_bar_menu_position(&mut app, viewport, Some(y > 0.0));
        }
    }
}

#[test]
fn open_menus_keep_footprint() {
    let mut app = test_app();
    app.library_session.folder_options =
        vec![String::from("nature"), String::from("nature/forest"), String::from("cities")];
    let closed = filter_bar_footprint(&app, 1200.0, 700.0);

    app.chrome.bar.menu = Some(crate::frontend::ui::MenuKind::Folders);
    assert_eq!(filter_bar_footprint(&app, 1200.0, 700.0), closed);

    app.theme.bar_open = true;
    let theme_closed = filter_bar_footprint(&app, 1200.0, 700.0);
    app.chrome.bar.menu = Some(crate::frontend::ui::MenuKind::Backends);
    assert_eq!(filter_bar_footprint(&app, 1200.0, 700.0), theme_closed);
}

#[test]
fn optional_controls_persist() {
    use crate::app::{Message, update};
    use crate::frontend::settings::SettingsMsg;
    use crate::frontend::ui::{BarAction, build_bar_with_tasks, verticalize_bar};
    use crate::infrastructure::config::Config;
    use skwd_config::keys::filter_bar;

    for path in [
        filter_bar::SHOW_TAG_CLOUD,
        filter_bar::SHOW_ORIENT,
        filter_bar::SHOW_PLAYLISTS,
        filter_bar::SHOW_DOWNLOAD,
        filter_bar::SHOW_COLOUR_STUDIO,
        filter_bar::SHOW_WIDGETS,
    ] {
        let mut app = test_app();
        app.config.set_key(filter_bar::SHOW_COLOUR_STUDIO, json!(true));
        app.config.set_key(filter_bar::SHOW_WIDGETS, json!(true));
        let original_filters = app.library_session.filters.clone();
        for visible in [true, false, true] {
            let _ = update(&mut app, Message::Settings(SettingsMsg::Toggle(path.into(), visible)));
            let mut reopened = Config::from_data(json!({}));
            reopened.config_path.clone_from(&app.config.config_path);
            assert!(reopened.reload());
            app.config = reopened;
            assert_eq!(
                app.config.filter_show(path.strip_prefix("filterBar.show.").unwrap()),
                visible
            );
            let show = super::bar_show(&app, &app.config);
            for downloads_enabled in [false, true] {
                for vertical in [false, true] {
                    let mut model = build_bar_with_tasks(
                        &app.library_session.filters,
                        &[],
                        false,
                        0,
                        0,
                        1.0,
                        downloads_enabled,
                        false,
                        false,
                        &show,
                        5000.0,
                        false,
                        false,
                        None,
                        &[],
                    );
                    if vertical {
                        verticalize_bar(&mut model, 500.0, 1.0);
                    }
                    for (key, action) in [
                        (filter_bar::SHOW_TAG_CLOUD, BarAction::TagCloud),
                        (filter_bar::SHOW_ORIENT, BarAction::Orient(String::new())),
                        (filter_bar::SHOW_PLAYLISTS, BarAction::Playlists),
                        (filter_bar::SHOW_DOWNLOAD, BarAction::Download),
                        (filter_bar::SHOW_COLOUR_STUDIO, BarAction::ColourStudio),
                        (filter_bar::SHOW_WIDGETS, BarAction::Widgets),
                    ] {
                        let present = model.items.iter().any(|item| {
                            item.action.as_ref().is_some_and(|shown| {
                                std::mem::discriminant(shown) == std::mem::discriminant(&action)
                            })
                        });
                        let expected = (key != path || visible)
                            && (key != filter_bar::SHOW_DOWNLOAD || downloads_enabled);
                        assert_eq!(
                            present, expected,
                            "{path}={visible}, {key}, vertical={vertical}"
                        );
                    }
                    let settings = model
                        .items
                        .iter()
                        .position(|item| matches!(item.action, Some(BarAction::Settings)))
                        .expect("settings is always on the bar");
                    assert!(matches!(
                        model.items.get(settings + 1).and_then(|item| item.action.as_ref()),
                        Some(BarAction::Stash)
                    ));
                }
            }
            assert_eq!(app.library_session.filters.orient, original_filters.orient);
            assert_eq!(app.library_session.filters.tags, original_filters.tags);
        }
    }
}

#[test]
fn filter_bar_eye_stashes() {
    use crate::app::Message;
    use crate::frontend::ui::{BarAction, BarIntent};
    assert!(matches!(bar_intent_message(BarIntent::Activate(BarAction::Stash)), Message::Stash));
}

#[test]
fn studio_buttons_open_panels() {
    use crate::app::Message;
    use crate::frontend::ui::{BarAction, BarIntent};

    assert!(matches!(
        bar_intent_message(BarIntent::Activate(BarAction::ColourStudio)),
        Message::KeyColourStudio
    ));
    assert!(matches!(
        bar_intent_message(BarIntent::Activate(BarAction::Widgets)),
        Message::OpenWidgets
    ));

    let mut app = test_app();
    assert!(!super::bar_show(&app, &app.config).colour_studio);
    assert!(!super::bar_show(&app, &app.config).widgets);
    app.config.set_key(skwd_config::keys::filter_bar::SHOW_COLOUR_STUDIO, json!(true));
    app.config.set_key(skwd_config::keys::filter_bar::SHOW_WIDGETS, json!(true));
    assert!(super::bar_show(&app, &app.config).colour_studio);
    assert!(super::bar_show(&app, &app.config).widgets);
}
