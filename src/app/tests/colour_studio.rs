use super::*;
use crate::frontend::colour_studio::{ColourStudio, StudioMsg};
use skwd_config::keys::theme;
use skwd_palette::studio::from_theme;

const ART: &str = "/wp/dusk.png";
const STALE_MODE: &str = "time";

fn snapshot() -> Value {
    let dark = from_theme(&skwd_palette::preset("nord").expect("nord")).to_wire();
    let light = from_theme(&skwd_palette::preset("graphite-light").expect("graphite")).to_wire();
    json!({
        "image": ART,
        "seed": "#5e81ac",
        "grey": false,
        "wheel": [],
        "methods": [{"key": "tonal", "dark": dark, "light": light}]
    })
}

fn studio_request(app: &App) -> Option<u64> {
    let key = app.panels.colour_studio.as_ref().map(|studio| studio.wallpaper.key.clone());
    app.daemon
        .pending
        .iter()
        .filter(|(_, kind)| {
            matches!(kind, Pending::Studio { key: asked, area: None, schemes: false } if Some(asked) == key.as_ref())
        })
        .map(|(id, _)| *id)
        .max()
}

fn opened(current: bool) -> App {
    let mut app = test_app();
    let mut dusk = static_wall("dusk.png", 1);
    dusk["path"] = json!(ART);
    seed(&mut app, &[dusk]);
    if current {
        app.daemon.current_wallpaper_art = Some(ART.into());
    }
    let _ = update(
        &mut app,
        Message::KeyPressed(keyboard::Key::Character("c".into()), keyboard::Modifiers::default()),
    );
    app
}

#[test]
fn key_opens_and_requests() {
    let app = opened(true);
    assert!(app.panels.colour_studio.is_some());
    assert!(!app.theme.bar_open);
    let calls = drain_calls(&app);
    let (_, params) = calls
        .iter()
        .find(|(method, _)| method == wall_proto::rpc::THEME_STUDIO)
        .expect("studio call");
    assert_eq!(params["image"], ART);
}

#[test]
fn snapshot_fills_studio() {
    let mut app = opened(true);
    let id = studio_request(&app).expect("pending");
    respond(&mut app, id, snapshot());
    let studio = app.panels.colour_studio.as_ref().expect("studio");
    assert!(studio.model.snapshot.is_some());
    assert!(studio.palette().is_some());
}

#[test]
fn failure_toasts() {
    let mut app = opened(true);
    let id = studio_request(&app).expect("pending");
    reject(&mut app, id, "no pixels");
    let studio = app.panels.colour_studio.as_ref().expect("studio");
    assert!(studio.palette().is_none());
    let toast = app.runtime_state.toast.as_ref().map(|(text, _)| text.as_str());
    assert!(toast.is_some_and(|text| text.contains("no pixels")));
}

#[test]
fn apply_current_rethemes() {
    let mut app = opened(true);
    let id = studio_request(&app).expect("pending");
    respond(&mut app, id, snapshot());
    drain_calls(&app);
    let _ = update(&mut app, Message::Studio(StudioMsg::Apply));
    assert_eq!(app.config.str_path(theme::ENGINE), wall_proto::STUDIO_ENGINE);
    assert_eq!(app.config.str_path(theme::STUDIO_METHOD), "tonal");
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, _)| method == wall_proto::rpc::WALL_RETHEME));
    assert!(!calls.iter().any(|(method, _)| method == "wall.apply"));
    assert!(app.panels.colour_studio.as_ref().is_some_and(ColourStudio::undoable));
}

#[test]
fn apply_other_applies_wallpaper() {
    let mut app = opened(false);
    let id = studio_request(&app).expect("pending");
    respond(&mut app, id, snapshot());
    drain_calls(&app);
    let _ = update(&mut app, Message::Studio(StudioMsg::Apply));
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, params)| method == "wall.apply" && params["path"] == ART));
    assert!(!calls.iter().any(|(method, _)| method == wall_proto::rpc::WALL_RETHEME));
}

#[test]
fn escape_closes() {
    let mut app = opened(true);
    let _ = update(
        &mut app,
        Message::KeyPressed(
            keyboard::Key::Named(keyboard::key::Named::Escape),
            keyboard::Modifiers::default(),
        ),
    );
    assert!(app.panels.colour_studio.is_none());
}

#[test]
fn stale_snapshot_ignored() {
    let mut app = opened(true);
    let id = studio_request(&app).expect("pending");
    let _ = update(&mut app, Message::Studio(StudioMsg::Close));
    respond(&mut app, id, snapshot());
    assert!(app.panels.colour_studio.is_none());
}

fn loaded(current: bool) -> App {
    let mut app = opened(current);
    let id = studio_request(&app).expect("pending");
    respond(&mut app, id, snapshot());
    drain_calls(&app);
    app
}

#[test]
fn apply_keeps_theme_mode() {
    let mut app = test_app();
    app.config.set_key(theme::MODE, json!("auto"));
    let mut dusk = static_wall("dusk.png", 1);
    dusk["path"] = json!(ART);
    seed(&mut app, &[dusk]);
    app.daemon.current_wallpaper_art = Some(ART.into());
    let _ = update(&mut app, Message::KeyColourStudio);
    let id = studio_request(&app).expect("pending");
    respond(&mut app, id, snapshot());
    let _ = update(&mut app, Message::Studio(StudioMsg::Apply));
    assert_eq!(app.config.str_path(theme::MODE), "auto");
    assert_eq!(app.config.str_path(theme::STUDIO_MODE), "");
    let _ = update(
        &mut app,
        Message::Studio(StudioMsg::Mode(crate::domain::studio::ModeChoice::Light)),
    );
    let _ = update(&mut app, Message::Studio(StudioMsg::Apply));
    assert_eq!(app.config.str_path(theme::MODE), "auto");
    assert_eq!(app.config.str_path(theme::STUDIO_MODE), theme::MODE_LIGHT);
}

#[test]
fn time_round_trips() {
    use crate::domain::studio::ModeChoice;

    let mut app = loaded(true);
    app.config.set_key(theme::MODE, json!(theme::MODE_LIGHT));
    let _ = update(&mut app, Message::Studio(StudioMsg::Mode(ModeChoice::ByTime)));
    let selected = app.panels.colour_studio.as_ref().expect("studio");
    assert_eq!(selected.model.mode, crate::infrastructure::theme::studio_time().0);
    assert_eq!(selected.model.mode_choice, ModeChoice::ByTime);
    let _ = update(&mut app, Message::Studio(StudioMsg::Apply));
    assert_eq!(app.config.str_path(theme::STUDIO_MODE), theme::STUDIO_MODE_BY_TIME);
    assert_eq!(app.config.str_path(theme::MODE), theme::MODE_LIGHT);
    let _ = update(&mut app, Message::Studio(StudioMsg::Undo));
    assert_eq!(app.config.str_path(theme::STUDIO_MODE), "");
    let _ = update(&mut app, Message::Studio(StudioMsg::Mode(ModeChoice::ByTime)));
    let _ = update(&mut app, Message::Studio(StudioMsg::Apply));
    let _ = update(&mut app, Message::Studio(StudioMsg::Close));
    let _ = update(&mut app, Message::KeyColourStudio);
    let reopened = app.panels.colour_studio.as_ref().expect("reopened");
    assert_eq!(reopened.model.mode_choice, ModeChoice::ByTime);
    assert_eq!(reopened.model.mode, crate::infrastructure::theme::studio_time().0);
}

#[test]
fn pinned_studio_survives_apply() {
    use crate::domain::studio::ModeChoice;

    for current in [false, true] {
        let mut app = test_app();
        app.config.set_key(theme::MODE, json!(theme::MODE_DARK));
        app.config.set_key(theme::STUDIO_MODE, json!(theme::MODE_DARK));
        let mut tuning = skwd_palette::studio::Tuning {
            warmth: 0.3,
            colour: 0.7,
            contrast: 0.2,
            ..skwd_palette::studio::Tuning::default()
        };
        tuning.shades[skwd_palette::studio::Role::Accent.index()] = 2;
        let area = skwd_palette::studio::Area { left: 0.1, top: 0.2, right: 0.6, bottom: 0.9 };
        let mut settings = json!({});
        settings[theme::POLICY] = json!(theme::POLICY_WALLPAPER);
        settings[theme::ENGINE] = json!(wall_proto::STUDIO_ENGINE);
        settings[theme::STUDIO_MODE] = json!(theme::STUDIO_MODE_BY_TIME);
        settings[theme::STUDIO_METHOD] = json!(skwd_palette::studio::AREA_KEY);
        settings[theme::STUDIO_WARMTH] = json!(tuning.warmth);
        settings[theme::STUDIO_COLOUR] = json!(tuning.colour);
        settings[theme::STUDIO_CONTRAST] = json!(tuning.contrast);
        settings[theme::STUDIO_SHADES] = json!(tuning.shades);
        settings[theme::STUDIO_AREA] = json!(area.edges());
        settings[theme::STUDIO_AREA_IMAGE] = json!(ART);
        app.config.set_key(
            theme::WALLPAPER_PROFILES,
            json!([{
                "key": "static:dusk.png", "settingsPinned": true,
                "settings": settings
            }]),
        );
        let mut dusk = static_wall("dusk.png", 1);
        dusk["path"] = json!(ART);
        seed(&mut app, &[dusk]);
        app.daemon.current_wallpaper_art = current.then(|| ART.into());
        let _ = update(&mut app, Message::KeyColourStudio);
        let studio = app.panels.colour_studio.as_ref().expect("studio");
        assert_eq!(studio.model.mode_choice, ModeChoice::ByTime);
        assert_eq!(studio.model.mode, crate::infrastructure::theme::studio_time().0);
        assert_eq!(studio.model.source, crate::domain::studio::area());
        assert_eq!(studio.model.tuning, tuning);
        assert_eq!(studio.saved_area(), Some(area));
        assert_eq!(studio.area_image, Some(("static:dusk.png".into(), ART.into())));
        let area_id = app
            .daemon
            .pending
            .iter()
            .find_map(|(id, kind)| {
                matches!(kind, Pending::Studio { area: Some(_), .. }).then_some(*id)
            })
            .expect("area request");
        let id = studio_request(&app).expect("pending");
        respond(&mut app, id, snapshot());
        respond(&mut app, area_id, snapshot());
        let _ = update(&mut app, Message::Studio(StudioMsg::Apply));
        let profiles = app.config.array_slice(theme::WALLPAPER_PROFILES);
        for (key, expected) in settings.as_object().expect("settings") {
            assert_eq!(&profiles[0]["settings"][key], expected, "{key}");
        }
        assert_eq!(app.config.str_path(theme::MODE), theme::MODE_DARK);
        assert!(app.panels.colour_studio.as_ref().is_some_and(ColourStudio::undoable));
    }
}

#[test]
fn apply_updates_pinned_profile() {
    let mut app = loaded(true);
    let profile = json!([{"key": "static:dusk.png", "settingsPinned": true, "settings": {}}]);
    app.config.set_key(theme::WALLPAPER_PROFILES, profile);
    let _ = update(&mut app, Message::Studio(StudioMsg::Pick("theme:nord".into())));
    let _ = update(&mut app, Message::Studio(StudioMsg::Apply));
    let profiles = app.config.array_slice(theme::WALLPAPER_PROFILES);
    assert_eq!(profiles[0]["settings"][theme::STUDIO_METHOD], "theme:nord");
}

#[test]
fn apply_needs_the_wallpaper() {
    let mut app = loaded(false);
    edit_catalog(&mut app, |catalog| catalog.items.clear());
    app.refilter();
    let _ = update(&mut app, Message::Studio(StudioMsg::Apply));
    assert_ne!(app.config.str_path(theme::ENGINE), wall_proto::STUDIO_ENGINE);
    let studio = app.panels.colour_studio.as_ref().expect("studio");
    assert!(!studio.undoable());
    assert!(app.runtime_state.toast.is_some());
}

#[test]
fn undo_restores_previous_colours() {
    let mut app = loaded(true);
    let engine = app.config.str_path(theme::ENGINE);
    let before = app.theme.base_palette;
    let _ = update(&mut app, Message::Studio(StudioMsg::Apply));
    assert_eq!(app.config.str_path(theme::ENGINE), wall_proto::STUDIO_ENGINE);
    drain_calls(&app);
    let _ = update(&mut app, Message::Studio(StudioMsg::Undo));
    assert_eq!(app.config.str_path(theme::ENGINE), engine);
    assert_eq!(app.theme.base_palette.studio(), before.studio());
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, _)| method == wall_proto::rpc::WALL_RETHEME));
    let studio = app.panels.colour_studio.as_ref().expect("studio");
    assert!(studio.applied.is_none());
}

#[test]
fn edits_offer_use_again() {
    let mut app = loaded(true);
    let _ = update(&mut app, Message::Studio(StudioMsg::Apply));
    let _ = update(
        &mut app,
        Message::Studio(StudioMsg::Step(crate::frontend::colour_studio::Row::Warmth, 1, false)),
    );
    let studio = app.panels.colour_studio.as_ref().expect("studio");
    assert!(!studio.undoable());
}

#[test]
fn saved_colours_persist() {
    let mut app = loaded(true);
    let _ = update(&mut app, Message::Studio(StudioMsg::SaveAs));
    let _ = update(&mut app, Message::Studio(StudioMsg::Save));
    let saved = app.config.array_slice(theme::STUDIO_SAVED);
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0]["method"], "tonal");
}

#[test]
fn typing_fills_name() {
    let mut app = loaded(true);
    let _ = update(&mut app, Message::Studio(StudioMsg::SaveAs));
    let _ = update(
        &mut app,
        Message::KeyPressed(keyboard::Key::Character("d".into()), keyboard::Modifiers::default()),
    );
    let _ = update(
        &mut app,
        Message::KeyPressed(
            keyboard::Key::Named(keyboard::key::Named::Enter),
            keyboard::Modifiers::default(),
        ),
    );
    assert_eq!(app.config.array_slice(theme::STUDIO_SAVED)[0]["name"], "d");
    assert!(app.panels.colour_studio.is_some());
}

fn stripes(app: &mut App) {
    let (width, height) = (16, 10);
    let rgba = (0..width * height)
        .flat_map(|index| [if index % 2 == 0 { 220 } else { 30 }, 80, 40, 255])
        .collect();
    let decoded = crate::frontend::colour_studio::Decoded {
        pixels: crate::domain::studio::Pixels { rgba, width, height, aspect: 1.6 },
        grey: iced::widget::image::Handle::from_rgba(1, 1, vec![0, 0, 0, 255]),
    };
    let key = app.panels.colour_studio.as_ref().expect("studio").wallpaper.key.clone();
    crate::app::update::colour_studio::pixels(app, &key, Some(decoded));
}

#[test]
fn area_commit_requests() {
    use crate::frontend::colour_studio::Phase;
    let mut app = loaded(true);
    stripes(&mut app);
    drain_calls(&app);
    let point =
        |x: f32, y: f32| skwd_palette::studio::Area { left: x, top: y, right: x, bottom: y };
    for msg in [
        StudioMsg::StartPick,
        StudioMsg::Drag(point(0.1, 0.1), Phase::Down),
        StudioMsg::Drag(point(0.6, 0.7), Phase::Drag),
        StudioMsg::Drag(point(0.6, 0.7), Phase::Up),
    ] {
        let _ = update(&mut app, Message::Studio(msg));
    }
    let calls = drain_calls(&app);
    assert!(
        calls.iter().any(|(method, params)| method == wall_proto::rpc::THEME_STUDIO
            && params.get("area").is_some())
    );
    let _ = update(&mut app, Message::Studio(StudioMsg::Apply));
    assert_eq!(app.config.str_path(theme::STUDIO_METHOD), "area");
    assert_eq!(app.config.array_slice(theme::STUDIO_AREA).len(), 4);
}

#[test]
fn saved_entries_round_trip() {
    use crate::app::update::colour_studio::{saved_entry, saved_value};
    let plain = skwd_palette::studio::Tuning::default();
    let mut shades = plain.shades;
    shades[skwd_palette::studio::Role::Accent.index()] = 2;
    let tuning = skwd_palette::studio::Tuning { warmth: 0.3, shades, ..plain };
    let entry = crate::domain::studio::Saved {
        name: "Dusk".into(),
        source: crate::domain::studio::area(),
        area: Some(skwd_palette::studio::Area { left: 0.1, top: 0.2, right: 0.6, bottom: 0.9 }),
        tuning,
    };
    assert_eq!(saved_entry(&saved_value(&entry)), Some(entry));
}

#[test]
fn undo_returns_the_wallpaper() {
    let mut app = test_app();
    let mut dusk = static_wall("dusk.png", 1);
    dusk["path"] = json!(ART);
    let mut moss = static_wall("moss.png", 2);
    moss["path"] = json!("/wp/moss.png");
    seed(&mut app, &[dusk, moss]);
    app.daemon.current_wallpaper_art = Some("/wp/moss.png".into());
    let _ = update(&mut app, Message::KeyColourStudio);
    let id = studio_request(&app).expect("pending");
    respond(&mut app, id, snapshot());
    let _ = update(&mut app, Message::Studio(StudioMsg::Apply));
    let _ = update(
        &mut app,
        Message::Studio(StudioMsg::Step(crate::frontend::colour_studio::Row::Warmth, 1, false)),
    );
    let _ = update(&mut app, Message::Studio(StudioMsg::Apply));
    drain_calls(&app);
    let _ = update(&mut app, Message::Studio(StudioMsg::Undo));
    let calls = drain_calls(&app);
    assert!(
        calls
            .iter()
            .any(|(method, params)| method == "wall.apply" && params["path"] == "/wp/moss.png")
    );
}

#[test]
fn area_keeps_its_image() {
    let studio = crate::domain::studio::Studio::new(
        crate::domain::studio::area(),
        crate::domain::studio::Mode::Dark,
        skwd_palette::studio::Tuning::default(),
    );
    let area = skwd_palette::studio::Area { left: 0.1, top: 0.1, right: 0.6, bottom: 0.7 };
    let writes = crate::app::update::colour_studio::config_writes(
        &studio,
        Some((area, "/t/dusk.webp".into())),
    );
    assert!(
        writes
            .iter()
            .any(|(key, value)| *key == theme::STUDIO_AREA_IMAGE && value == "/t/dusk.webp")
    );
}

#[test]
fn undo_skips_the_filter() {
    let mut app = test_app();
    let mut dusk = static_wall("dusk.png", 1);
    dusk["path"] = json!(ART);
    let mut moss = static_wall("moss.png", 2);
    moss["path"] = json!("/wp/moss.png");
    seed(&mut app, &[dusk, moss]);
    app.daemon.current_wallpaper_art = Some("/wp/moss.png".into());
    let items = app.library_session.library.catalog().items.clone();
    app.library_session.filtered.retain(|&index| items[index as usize].path == ART);
    let _ = update(&mut app, Message::KeyColourStudio);
    let id = studio_request(&app).expect("pending");
    respond(&mut app, id, snapshot());
    let _ = update(&mut app, Message::Studio(StudioMsg::Apply));
    drain_calls(&app);
    let _ = update(&mut app, Message::Studio(StudioMsg::Undo));
    let calls = drain_calls(&app);
    assert!(
        calls
            .iter()
            .any(|(method, params)| method == "wall.apply" && params["path"] == "/wp/moss.png")
    );
}

#[test]
fn foreign_area_uses_tonal() {
    let mut app = test_app();
    let mut dusk = static_wall("dusk.png", 1);
    dusk["path"] = json!(ART);
    seed(&mut app, &[dusk]);
    app.daemon.current_wallpaper_art = Some(ART.into());
    app.config.set_key(theme::ENGINE, json!(wall_proto::STUDIO_ENGINE));
    app.config.set_key(theme::STUDIO_METHOD, json!(skwd_palette::studio::AREA_KEY));
    app.config.set_key(theme::STUDIO_AREA, json!([0.1, 0.1, 0.6, 0.7]));
    app.config.set_key(theme::STUDIO_AREA_IMAGE, json!("/wp/other.png"));
    let _ = update(&mut app, Message::KeyColourStudio);
    let id = studio_request(&app).expect("pending");
    let mut reply = snapshot();
    let mut subject = reply["methods"][0].clone();
    subject["key"] = json!("subject");
    reply["methods"].as_array_mut().expect("methods").insert(0, subject);
    respond(&mut app, id, reply);
    let studio = app.panels.colour_studio.as_ref().expect("studio");
    assert!(studio.saved_area().is_none());
    assert_eq!(studio.model.source, crate::domain::studio::Source::Method("tonal".into()));
}

#[test]
fn themes_skip_the_scan() {
    let mut app = test_app();
    let mut dusk = static_wall("dusk.png", 1);
    dusk["path"] = json!(ART);
    seed(&mut app, &[dusk]);
    app.daemon.current_wallpaper_art = Some(ART.into());
    app.config.set_key(theme::STUDIO_METHOD, json!("theme:nord"));
    let _ = update(&mut app, Message::KeyColourStudio);
    let _ = update(&mut app, Message::Studio(StudioMsg::Apply));
    assert_eq!(app.config.str_path(theme::ENGINE), wall_proto::STUDIO_ENGINE);
}

#[test]
fn undo_keeps_unknown_wallpaper() {
    let mut app = test_app();
    let mut dusk = static_wall("dusk.png", 1);
    dusk["path"] = json!(ART);
    seed(&mut app, &[dusk]);
    app.daemon.current_wallpaper_art = Some("/elsewhere/gone.png".into());
    let _ = update(&mut app, Message::KeyColourStudio);
    let id = studio_request(&app).expect("pending");
    respond(&mut app, id, snapshot());
    let _ = update(&mut app, Message::Studio(StudioMsg::Apply));
    drain_calls(&app);
    let _ = update(&mut app, Message::Studio(StudioMsg::Undo));
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, _)| method == wall_proto::rpc::WALL_RETHEME));
    assert!(!calls.iter().any(|(method, _)| method == "wall.apply"));
    assert!(app.runtime_state.toast.is_some());
}

#[test]
fn own_area_is_kept() {
    let mut app = test_app();
    let mut dusk = static_wall("dusk.png", 1);
    dusk["path"] = json!(ART);
    seed(&mut app, &[dusk]);
    app.daemon.current_wallpaper_art = Some(ART.into());
    app.config.set_key(theme::ENGINE, json!(wall_proto::STUDIO_ENGINE));
    app.config.set_key(theme::STUDIO_METHOD, json!(skwd_palette::studio::AREA_KEY));
    app.config.set_key(theme::STUDIO_AREA, json!([0.1, 0.1, 0.6, 0.7]));
    app.config.set_key(theme::STUDIO_AREA_IMAGE, json!(ART));
    let _ = update(&mut app, Message::KeyColourStudio);
    let id = studio_request(&app).expect("pending");
    respond(&mut app, id, snapshot());
    let studio = app.panels.colour_studio.as_ref().expect("studio");
    assert!(studio.saved_area().is_some());
}

#[test]
fn old_time_mode_cleared() {
    let mut app = test_app();
    let mut dusk = static_wall("dusk.png", 1);
    dusk["path"] = json!(ART);
    seed(&mut app, &[dusk]);
    app.daemon.current_wallpaper_art = Some(ART.into());
    app.config.set_key(theme::STUDIO_MODE, json!(STALE_MODE));
    let _ = update(&mut app, Message::KeyColourStudio);
    let id = studio_request(&app).expect("pending");
    respond(&mut app, id, snapshot());
    let _ = update(&mut app, Message::Studio(StudioMsg::Apply));
    assert_eq!(app.config.str_path(theme::STUDIO_MODE), "");
}

fn schemes_request(app: &App) -> Option<u64> {
    app.daemon
        .pending
        .iter()
        .find_map(|(id, kind)| matches!(kind, Pending::Studio { schemes: true, .. }).then_some(*id))
}

#[test]
fn schemes_wait_for_need() {
    let mut app = opened(true);
    let id = studio_request(&app).expect("pending");
    respond(&mut app, id, snapshot());
    assert!(schemes_request(&app).is_none());
    let matugen = crate::domain::studio::Group::Matugen;
    let _ = update(&mut app, Message::Studio(StudioMsg::OpenGroup(matugen)));
    assert!(schemes_request(&app).is_some());
}

#[test]
fn material_waits_for_schemes() {
    let mut app = test_app();
    let mut dusk = static_wall("dusk.png", 1);
    dusk["path"] = json!(ART);
    seed(&mut app, &[dusk]);
    app.daemon.current_wallpaper_art = Some(ART.into());
    app.config.set_key(theme::STUDIO_METHOD, json!("material:content"));
    let _ = update(&mut app, Message::KeyColourStudio);
    let id = studio_request(&app).expect("pending");
    respond(&mut app, id, snapshot());
    let studio = app.panels.colour_studio.as_ref().expect("studio");
    assert_eq!(studio.model.source, crate::domain::studio::Source::Material("content".into()));
}

#[test]
fn apply_waits_for_analysis() {
    let mut app = test_app();
    let mut dusk = static_wall("dusk.png", 1);
    dusk["path"] = json!(ART);
    seed(&mut app, &[dusk]);
    app.daemon.current_wallpaper_art = Some(ART.into());
    app.config.set_key(theme::ENGINE, json!(wall_proto::STUDIO_ENGINE));
    app.config.set_key(theme::STUDIO_METHOD, json!(skwd_palette::studio::AREA_KEY));
    app.config.set_key(theme::STUDIO_AREA, json!([0.1, 0.1, 0.6, 0.7]));
    app.config.set_key(theme::STUDIO_AREA_IMAGE, json!(ART));
    let _ = update(&mut app, Message::KeyColourStudio);
    stripes(&mut app);
    assert!(app.panels.colour_studio.as_ref().is_some_and(|studio| studio.palette().is_some()));
    let _ = update(&mut app, Message::Studio(StudioMsg::Apply));
    assert!(app.panels.studio_restore.is_none());
}

#[test]
fn late_scan_keeps_schemes() {
    let mut app = test_app();
    let mut dusk = static_wall("dusk.png", 1);
    dusk["path"] = json!(ART);
    seed(&mut app, &[dusk]);
    app.daemon.current_wallpaper_art = Some(ART.into());
    app.config.set_key(theme::STUDIO_METHOD, json!("material:content"));
    let _ = update(&mut app, Message::KeyColourStudio);
    let id = studio_request(&app).expect("pending");
    respond(&mut app, id, snapshot());
    let mut rich = snapshot();
    rich["materials"] = rich["methods"].clone();
    rich["materials"][0]["key"] = json!("content");
    let id = schemes_request(&app).expect("schemes");
    respond(&mut app, id, rich);
    let studio = app.panels.colour_studio.as_ref().expect("studio");
    let key = studio.wallpaper.key.clone();
    let mut late = studio.model.snapshot.clone().expect("snapshot");
    late.materials.clear();
    crate::app::update::colour_studio::loaded(&mut app, &key, None, false, late);
    let studio = app.panels.colour_studio.as_ref().expect("studio");
    assert!(studio.palette().is_some());
    assert_eq!(studio.schemes, crate::frontend::colour_studio::Schemes::Read);
}

#[test]
fn idle_time_wake_updates() {
    use crate::domain::studio::{Mode, ModeChoice};

    let mut app = loaded(true);
    let _ = update(&mut app, Message::Studio(StudioMsg::Mode(ModeChoice::ByTime)));
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 600);
    assert!(!app.animating());
    let effective = crate::infrastructure::theme::studio_time().0;
    let stale = if effective == Mode::Dark { Mode::Light } else { Mode::Dark };
    app.panels.colour_studio.as_mut().expect("studio").model.mode = stale;
    now += Duration::from_millis(16);
    frame(&mut app, now, 1280.0, 720.0);
    let studio = app.panels.colour_studio.as_ref().expect("studio");
    assert_eq!(studio.model.mode, effective);
    assert_eq!(studio.model.mode_choice, ModeChoice::ByTime);
    assert_eq!(app.runtime_state.last_tick, Some(now));
}
