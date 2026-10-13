use super::*;

#[test]
fn seed_fills_store() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0), wall("b.png", "static", 2, 0)]);
    assert_eq!(app.library_session.library.catalog().items.len(), 2);
    assert_eq!(app.library_session.filtered.len(), 2);
    assert!(app.preview_resources.atlas.is_some());
}

#[test]
fn set_folder_narrows() {
    let mut app = test_app();
    seed(
        &mut app,
        &[
            wall("root.png", "static", 1, 0),
            wall("nature/a.png", "static", 2, 0),
            wall("nature/deep/b.png", "static", 3, 0),
            wall("city/c.png", "static", 4, 0),
        ],
    );
    app.chrome.bar.menu = Some(crate::frontend::ui::MenuKind::Folders);
    app.library_session.playlist_filter =
        Some((7, String::from("pl"), std::collections::HashSet::new()));
    let _ = update(&mut app, Message::SetFolder(String::from("nature")));
    assert_eq!(app.library_session.filters.folder, "nature");
    assert_eq!(app.chrome.bar.menu, None);
    assert!(app.library_session.playlist_filter.is_none());
    assert_eq!(filtered_names(&app), ["nature/a.png", "nature/deep/b.png"]);
    let _ = update(&mut app, Message::SetFolder(String::new()));
    assert_eq!(app.library_session.filters.folder, "*");
    assert_eq!(app.library_session.filtered.len(), 4);
    let _ = update(&mut app, Message::SetFolder(String::from("*")));
    assert_eq!(app.library_session.filtered.len(), 4);
}

#[test]
fn sticky_filters() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0), wall("vids/v.mp4", "video", 2, 0)]);
    let _ = update(&mut app, Message::SetFolder(String::from("vids")));
    assert!(app.config.root().pointer("/filterBar/last").is_none());

    app.config.save_key("filterBar.sticky", json!(true));
    let _ = update(&mut app, Message::SetFolder(String::from("vids")));
    let _ = update(&mut app, Message::SetSort(String::from("date")));
    let _ = update(&mut app, Message::SetOrient(String::from("landscape")));
    let _ = update(&mut app, Message::SetResolution(String::from("1920x1080")));
    assert_eq!(
        app.config.root().pointer("/filterBar/last/folder").and_then(Value::as_str),
        Some("vids")
    );
    assert_eq!(
        app.config.root().pointer("/filterBar/last/sort").and_then(Value::as_str),
        Some("date")
    );
    assert_eq!(
        app.config.root().pointer("/filterBar/last/orient").and_then(Value::as_str),
        Some("landscape")
    );
    assert_eq!(
        app.config.root().pointer("/filterBar/last/resolution").and_then(Value::as_str),
        Some("1920x1080")
    );

    let restored = startup_filters(&app.config);
    assert_eq!(restored.folder, "vids");
    assert_eq!(restored.sort, "date");
    assert_eq!(restored.orient, "landscape");
    assert_eq!(restored.resolution, "1920x1080");

    app.config.save_key("filterBar.sticky", json!(false));
    app.config.save_key("filterBar.defaultFolder", json!("anime"));
    let plain = startup_filters(&app.config);
    assert_eq!(plain.folder, "anime");
    assert_eq!(plain.sort, "color");
}

#[test]
fn resolution_presets_use_ranges() {
    use crate::frontend::settings::{ActionId, SettingsMsg};

    let mut app = test_app();
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::AddResolutionPreset)));
    assert_eq!(app.config.array_len(skwd_config::keys::filter_bar::RESOLUTION_PRESETS), 1);

    let base = format!("{}.0", skwd_config::keys::filter_bar::RESOLUTION_PRESETS);
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Input(format!("{base}.label"), String::from("FHD"))),
    );
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Input(format!("{base}.from"), String::from("1920x1080"))),
    );
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Input(format!("{base}.to"), String::from("2559x1439"))),
    );
    assert_eq!(app.config.resolution_presets()[0].label, "FHD");

    let mut exact = wall("exact.png", "static", 1, 0);
    exact["width"] = json!(1920);
    exact["height"] = json!(1080);
    let mut near = wall("near.png", "static", 2, 0);
    near["width"] = json!(2560);
    near["height"] = json!(1440);
    let mut portrait = wall("portrait.png", "static", 3, 0);
    portrait["width"] = json!(1439);
    portrait["height"] = json!(2559);
    seed(&mut app, &[exact, near, portrait]);
    let _ = update(&mut app, Message::SetResolution(String::from("wide:1920x1080..2559x1439")));
    assert_eq!(filtered_names(&app), ["exact.png"]);

    let _ =
        update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::RemoveResolutionPreset(0))));
    assert!(app.config.resolution_presets().is_empty());
    assert!(app.library_session.filters.resolution.is_empty());
    assert_eq!(app.library_session.filtered.len(), 3);
}

#[test]
fn resolution_preset_bands_create_complete_ranges() {
    use crate::frontend::settings::{ActionId, SettingsMsg};

    let mut app = test_app();
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Run(ActionId::CreateResolutionPresetBand("4k"))),
    );
    let preset = &app.config.resolution_presets()[0];
    assert_eq!(preset.label, "4K");
    assert_eq!(preset.orientation, "wide");
    assert_eq!((preset.from_width, preset.from_height), (3840, 2160));
    assert_eq!((preset.to_width, preset.to_height), (Some(5119), Some(2879)));

    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Run(ActionId::CreateResolutionPresetBand("8k"))),
    );
    let preset = &app.config.resolution_presets()[1];
    assert_eq!(preset.label, "8K+");
    assert_eq!((preset.to_width, preset.to_height), (None, None));
}

#[test]
fn resolution_preset_dimensions_store_a_width_and_height_pair() {
    use crate::frontend::settings::SettingsMsg;

    let mut app = test_app();
    app.panels.settings.tab = String::from("filter");
    let base = format!("{}.0", skwd_config::keys::filter_bar::RESOLUTION_PRESETS);
    app.config.array_push(
        skwd_config::keys::filter_bar::RESOLUTION_PRESETS,
        json!({"label": "Custom", "orientation": "wide", "from": "1920x1080", "to": ""}),
    );
    app.init_settings_inputs();
    assert_eq!(app.panels.settings.inputs[&format!("{base}.from")], "1920x1080");
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::ResolutionInput(
            format!("{base}.from"),
            true,
            String::from("2560"),
        )),
    );
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::ResolutionInput(
            format!("{base}.from"),
            false,
            String::from("1440"),
        )),
    );
    assert_eq!(app.config.str_path(&format!("{base}.from")), "2560x1440");
}

#[test]
fn resolution_presets_reject_invalid_drafts_and_keep_the_saved_range() {
    use crate::frontend::settings::{ActionId, SettingsMsg};
    let mut app = test_app();
    app.panels.settings.tab = String::from("filter");
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Run(ActionId::CreateResolutionPresetBand("4k"))),
    );
    let base = "filterBar.resolutionPresets.0";
    let from = format!("{base}.from");
    let to = format!("{base}.to");
    for invalid in ["bad", "-1", "12.5", "9999999999"] {
        let _ = update(
            &mut app,
            Message::Settings(SettingsMsg::ResolutionInput(from.clone(), true, invalid.into())),
        );
        assert_eq!(app.panels.settings.inputs[&from], "3840x2160");
    }
    for invalid in ["", "0", "9000"] {
        let _ = update(
            &mut app,
            Message::Settings(SettingsMsg::ResolutionInput(from.clone(), true, invalid.into())),
        );
        assert_eq!(app.config.str_path(&from), "3840x2160");
    }
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::ResolutionInput(from.clone(), true, "4000".into())),
    );
    assert_eq!(app.config.str_path(&from), "4000x2160");
    let _ = update(&mut app, Message::Settings(SettingsMsg::ResolutionLimit(to.clone(), false)));
    assert_eq!(app.config.str_path(&to), "");
    let _ = update(&mut app, Message::Settings(SettingsMsg::ResolutionLimit(to.clone(), true)));
    assert_eq!(app.config.str_path(&to), "4000x2160");
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Input(format!("{base}.label"), "2160".into())),
    );
    let _ = update(&mut app, Message::Settings(SettingsMsg::Commit));
    let persisted: Value =
        serde_json::from_slice(&std::fs::read(&app.config.config_path).unwrap()).unwrap();
    assert_eq!(persisted["filterBar"]["resolutionPresets"][0]["label"], "2160");
    assert_eq!(persisted["filterBar"]["resolutionPresets"][0]["from"], "4000x2160");
}

#[test]
fn resolution_templates_filter_their_bounds_and_support_portrait() {
    use crate::frontend::settings::{ActionId, SettingsMsg};
    for (band, width, height, next) in [
        ("fhd", 1920, 1080, Some((2560, 1440))),
        ("qhd", 2560, 1440, Some((3840, 2160))),
        ("4k", 3840, 2160, Some((5120, 2880))),
        ("5k", 5120, 2880, Some((7680, 4320))),
        ("8k", 7680, 4320, None),
    ] {
        let mut app = test_app();
        app.panels.settings.tab = "filter".into();
        let _ = update(
            &mut app,
            Message::Settings(SettingsMsg::Run(ActionId::CreateResolutionPresetBand(band))),
        );
        let mut exact = wall("exact.png", "static", 1, 0);
        exact["width"] = json!(width);
        exact["height"] = json!(height);
        let mut above = wall("above.png", "static", 1, 0);
        let upper = next.unwrap_or((width * 2, height * 2));
        above["width"] = json!(upper.0);
        above["height"] = json!(upper.1);
        seed(&mut app, &[exact, above]);
        let key = app.config.resolution_presets()[0].key();
        let _ = update(&mut app, Message::SetResolution(key));
        assert_eq!(app.library_session.filtered.len(), if next.is_some() { 1 } else { 2 });
        let _ = update(
            &mut app,
            Message::Settings(SettingsMsg::Pick(
                "filterBar.resolutionPresets.0.orientation".into(),
                "tall".into(),
            )),
        );
        assert_eq!(
            app.panels.settings.inputs["filterBar.resolutionPresets.0.from"],
            format!("{height}x{width}")
        );
        assert!(app.library_session.filters.resolution.is_empty());
        let key = app.config.resolution_presets()[0].key();
        let _ = update(&mut app, Message::SetResolution(key));
        assert!(app.library_session.filtered.is_empty());
        let mut portrait = wall("portrait.png", "static", 1, 0);
        portrait["width"] = json!(height);
        portrait["height"] = json!(width);
        seed(&mut app, &[portrait]);
        assert_eq!(filtered_names(&app), ["portrait.png"]);
    }
}

#[test]
fn custom_resolution_opens_a_valid_editable_range_and_restores_cancelled_input() {
    use crate::frontend::settings::{ActionId, SettingsKey, SettingsMsg};
    let mut app = test_app();
    app.panels.settings.tab = "filter".into();
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Run(ActionId::CreateResolutionPresetBand("custom"))),
    );
    assert!(app.panels.settings.expanded_details.contains("filterBar.resolutionPresets.0.custom"));
    assert_eq!(app.config.resolution_presets().len(), 1);
    let from = "filterBar.resolutionPresets.0.from";
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::ResolutionInput(from.into(), true, "2560".into())),
    );
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Key(SettingsKey::FocusNext { backwards: false })),
    );
    assert!(app.panels.settings.resolution_height);
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::ResolutionInput(from.into(), false, "1440".into())),
    );
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Cancel)));
    assert_eq!(app.config.str_path(from), "1920x1080");
    assert_eq!(app.panels.settings.inputs[from], "1920x1080");
}

#[test]
fn shape_change_clears_incompatible_resolution() {
    let mut app = test_app();
    app.config.save_key(
        skwd_config::keys::filter_bar::RESOLUTION_PRESETS,
        json!([
            {
                "label": "FHD",
                "orientation": "wide",
                "from": "1920x1080",
                "to": "2559x1439"
            },
            {
                "label": "FHD TALL",
                "orientation": "tall",
                "from": "1080x1920",
                "to": "1439x2559"
            }
        ]),
    );
    let tall = String::from("tall:1080x1920..1439x2559");
    let _ = update(&mut app, Message::SetResolution(tall.clone()));
    let _ = update(&mut app, Message::SetOrient(String::from("portrait")));
    assert_eq!(app.library_session.filters.resolution, tall);

    let _ = update(&mut app, Message::SetOrient(String::from("landscape")));
    assert!(app.library_session.filters.resolution.is_empty());
}

#[test]
fn set_sort_reorders() {
    let mut app = test_app();
    seed(
        &mut app,
        &[
            wall("a.png", "static", 5, 10),
            wall("b.png", "static", 6, 30),
            wall("c.png", "static", 7, 20),
        ],
    );
    assert_eq!(filtered_names(&app), ["a.png", "b.png", "c.png"]);
    let _ = update(&mut app, Message::SetSort(String::from("date")));
    assert_eq!(filtered_names(&app), ["b.png", "c.png", "a.png"]);
}

#[test]
fn toggle_favourites() {
    let mut app = test_app();
    let mut fav = wall("fav.png", "static", 1, 0);
    fav["favourite"] = json!(1);
    seed(&mut app, &[fav, wall("plain.png", "static", 2, 0)]);
    let _ = update(&mut app, Message::ToggleFavourites);
    assert_eq!(filtered_names(&app), ["fav.png"]);
    let _ = update(&mut app, Message::ToggleFavourites);
    assert_eq!(app.library_session.filtered.len(), 2);
}

#[test]
fn color_filter_cycle() {
    let mut app = test_app();
    seed(&mut app, &[wall("r.png", "static", 0, 0), wall("g.png", "static", 5, 0)]);
    assert_eq!(app.library_session.filters.color, -1);
    let _ = update(&mut app, Message::SetColorFilter(i64::MAX));
    assert_eq!(app.library_session.filters.color, 0);
    assert_eq!(filtered_names(&app), ["r.png"]);
    let _ = update(&mut app, Message::SetColorFilter(i64::MIN));
    assert_eq!(app.library_session.filters.color, 99);
    let _ = update(&mut app, Message::SetColorFilter(5));
    assert_eq!(filtered_names(&app), ["g.png"]);
}

#[test]
fn mass_tag_apply() {
    let mut app = test_app();
    seed(
        &mut app,
        &[
            wall("a.png", "static", 1, 0),
            wall("b.png", "static", 2, 0),
            wall("c.png", "static", 3, 0),
        ],
    );
    app.tags.mode = true;
    app.tags.select.insert(0);
    app.tags.select.insert(2);
    app.tags.mass_tags = vec!["forest".into(), "night".into()];
    let _ = update(&mut app, Message::MassTagApply);
    assert_eq!(
        app.library_session.library.catalog().tags.get("a.png").map(Vec::as_slice),
        Some(&["forest".into(), "night".into()][..])
    );
    assert_eq!(
        app.library_session.library.catalog().tags.get("c.png").map(Vec::as_slice),
        Some(&["forest".into(), "night".into()][..])
    );
    assert!(!app.library_session.library.catalog().tags.contains_key("b.png"));
    assert!(app.tags.select.is_empty());
    let calls = drain_calls(&app);
    assert_eq!(calls.iter().filter(|(method, _)| method == "wall.update_tags").count(), 2);
}

#[test]
fn tag_mode_clears() {
    let mut app = test_app();
    let _ = update(&mut app, Message::ToggleTagMode);
    assert!(app.tags.mode);
    app.tags.select.insert(0);
    app.tags.mass_tags.push("x".into());
    let _ = update(&mut app, Message::ToggleTagMode);
    assert!(!app.tags.mode);
    assert!(app.tags.select.is_empty() && app.tags.mass_tags.is_empty());
}

#[test]
fn weather_browse_stays_full() {
    let mut app = test_app();
    seed(&mut app, &[wall("rain.png", "static", 1, 0), wall("sun.png", "static", 2, 0)]);
    app.config.set_key(skwd_config::keys::general::WEATHER_MATCH, json!(true));
    app.config.set_key(skwd_config::keys::weather::SOURCE, json!("outdoors"));
    app.config.set_key(skwd_config::keys::weather::ENABLED, json!(true));
    app.refilter();
    assert_eq!(filtered_names(&app), ["rain.png", "sun.png"]);

    let location_key = app.outdoor_location_key();
    let active = json!({
        "weather": ["rainy"], "enabled": true, "source": "outdoors",
        "status": "active", "location_key": location_key,
        "member_keys": ["rain.png"],
        "outdoors": {"generation": 2, "season": "autumn", "light": "dusk",
            "match_kind": "library", "location": "Stockholm", "stale": false,
            "outputs": [{"output": "DP-1", "key": "rain.png", "name": "Rain",
                "preview": "/walls/rain-thumb.png", "held": false}]}
    });
    app.on_result(Pending::Weather, &active);
    assert_eq!(filtered_names(&app), ["rain.png", "sun.png"]);
    let state: serde_json::Value = super::update::ui_state_json(&app).parse().unwrap();
    assert_eq!(state["outdoors"]["active"], true);
    assert_eq!(state["outdoors"]["context"]["outputs"][0]["key"], "rain.png");

    app.scene.set_current(1, 2);
    let mut old = active.clone();
    old["outdoors"]["generation"] = json!(1);
    old["member_keys"] = json!(["sun.png"]);
    app.on_result(Pending::Weather, &old);
    assert_eq!(app.scene.current, 1);

    let mut unavailable = active.clone();
    unavailable["outdoors"]["generation"] = json!(3);
    unavailable["outdoors"]["stale"] = json!(true);
    unavailable["status"] = json!("unavailable");
    unavailable["member_keys"] = json!([]);
    app.on_result(Pending::Weather, &unavailable);
    assert_eq!(filtered_names(&app), ["rain.png", "sun.png"]);
    assert_eq!(app.scene.current, 1);

    let _ = update(
        &mut app,
        Message::Settings(crate::frontend::settings::SettingsMsg::Input(
            skwd_config::keys::schedule::LATITUDE.into(),
            "60.0".into(),
        )),
    );
    assert!(app.daemon.weather.is_none());
    assert_eq!(filtered_names(&app), ["rain.png", "sun.png"]);
    assert_eq!(app.scene.current, 1);
    app.on_result(Pending::Weather, &active);
    assert!(app.daemon.weather.is_none());
}

#[test]
fn weather_playlist_stays_manual() {
    let mut app = test_app();
    seed(&mut app, &[wall("rain.png", "static", 1, 0), wall("sun.png", "static", 2, 0)]);
    app.config.set_key(skwd_config::keys::general::WEATHER_MATCH, json!(true));
    app.on_result(
        Pending::Weather,
        &json!({
            "weather": ["rainy"], "status": "active", "category": "rainy",
            "playlist_id": 7, "playlist_name": "Rain", "member_keys": ["rain.png"]
        }),
    );
    assert_eq!(filtered_names(&app), ["rain.png", "sun.png"]);
    app.library_session.playlist_filter = Some((8, "Manual".into(), ["sun.png".into()].into()));
    app.refilter();
    assert_eq!(filtered_names(&app), ["sun.png"]);
    let _ = update(&mut app, Message::Pl(crate::frontend::playlists::PlMsg::Browse(None)));
    assert_eq!(filtered_names(&app), ["rain.png", "sun.png"]);

    edit_catalog(&mut app, |catalog| {
        catalog.tags.insert("sun.png".into(), vec!["bright".into()]);
    });
    app.library_session.filters.tags.push("bright".into());
    app.refilter();
    assert_eq!(filtered_names(&app), ["sun.png"]);
}

#[test]
fn weather_stale_requests() {
    use crate::frontend::settings::SettingsMsg;

    let mut app = test_app();
    app.daemon.pending.insert(10, Pending::Weather);
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Input(
            skwd_config::keys::weather::SEMANTIC_MAX_RESULTS.into(),
            "24".into(),
        )),
    );
    assert!(!app.daemon.pending.contains_key(&10));

    app.daemon.pending.insert(11, Pending::Weather);
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Input(
            skwd_config::keys::schedule::LATITUDE.into(),
            "59".into(),
        )),
    );
    assert!(!app.daemon.pending.contains_key(&11));
}

#[test]
fn semantic_disable_stale() {
    use crate::frontend::settings::SettingsMsg;

    let mut app = test_app();
    seed(&mut app, &[wall("rain.png", "static", 1, 0), wall("sun.png", "static", 2, 0)]);
    app.config.set_key(skwd_config::keys::general::WEATHER_MATCH, json!(true));
    app.config.set_key(skwd_config::keys::weather::SOURCE, json!("semantic"));
    app.config.set_key(skwd_config::keys::weather::SEMANTIC_RAINY, json!("rainy streets"));
    let active = json!({
        "weather": ["rainy"], "source": "semantic", "status": "active", "category": "rainy",
        "query": "rainy streets", "member_keys": ["rain.png"]
    });
    app.on_result(Pending::Weather, &active);
    assert_eq!(filtered_names(&app), ["rain.png", "sun.png"]);
    drain_calls(&app);

    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Toggle(skwd_config::keys::semantic::ENABLED.into(), false)),
    );
    assert_eq!(filtered_names(&app), ["rain.png", "sun.png"]);
    assert!(drain_calls(&app).iter().any(|(method, _)| method == wall_proto::rpc::WALL_WEATHER));
    app.on_result(Pending::Weather, &active);
    assert!(app.daemon.weather.is_none());
}

#[test]
fn initial_error_keeps_wallpaper() {
    let mut app = test_app();
    seed(&mut app, &[wall("rain.png", "static", 1, 0)]);
    app.config.set_key(skwd_config::keys::weather::SOURCE, json!("outdoors"));
    app.config.set_key(skwd_config::keys::weather::ENABLED, json!(true));
    app.rpc_error(Pending::Weather, String::from("offline"));

    let status = app.daemon.weather.as_ref().unwrap();
    assert_eq!(status.status, crate::contracts::daemon::WeatherState::Unavailable);
    assert!(!status.outdoors.as_ref().unwrap().stale);
    assert_eq!(status.outdoors.as_ref().unwrap().outputs.len(), 0);
    let state: serde_json::Value = super::update::ui_state_json(&app).parse().unwrap();
    assert_eq!(state["outdoors"]["status"], "unavailable");
}

#[test]
fn mass_tag_suggestions() {
    let mut app = test_app();
    edit_catalog(&mut app, |catalog| {
        catalog.tag_vocab =
            ["forest", "forestry", "field", "fog"].into_iter().map(String::from).collect();
        catalog.tag_counts =
            [("fog", 3usize), ("forest", 5)].into_iter().map(|(t, n)| (t.into(), n)).collect();
    });
    app.tags.mass_tags = vec!["forestry".into()];
    app.tags.mass_input = String::from("fo");
    assert_eq!(
        super::mass_tag_suggestions(&app),
        vec![(String::from("fog"), 3), (String::from("forest"), 5)]
    );
}

#[test]
fn playlist_toggle_all() {
    let mut app = test_app();
    app.panels.playlists = Some(crate::frontend::playlists::Playlists {
        outputs: vec!["DP-1".into(), "DP-2".into()],
        assign: Vec::new(),
        ..Default::default()
    });
    let _ = update(&mut app, Message::Pl(crate::frontend::playlists::PlMsg::ToggleAll(5)));
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, params)| method == "playlist.assign"
        && params["output"] == "*"
        && params["id"] == 5));

    app.panels.playlists = Some(crate::frontend::playlists::Playlists {
        outputs: vec!["DP-1".into(), "DP-2".into()],
        assign: vec![("*".into(), 5)],
        ..Default::default()
    });
    let _ = update(&mut app, Message::Pl(crate::frontend::playlists::PlMsg::ToggleAll(5)));
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, params)| method == "playlist.assign"
        && params["output"] == "*"
        && params["id"] == 0));
}

#[test]
fn playlist_toggle_output() {
    let mut app = test_app();
    app.panels.playlists = Some(crate::frontend::playlists::Playlists {
        outputs: vec!["DP-1".into(), "DP-2".into(), "HDMI-1".into()],
        assign: vec![("*".into(), 5)],
        ..Default::default()
    });
    let _ = update(
        &mut app,
        Message::Pl(crate::frontend::playlists::PlMsg::ToggleOutput("DP-1".into(), 5)),
    );
    let calls = drain_calls(&app);
    let assigns: Vec<(String, i64)> = calls
        .iter()
        .filter(|(method, _)| method == "playlist.assign")
        .map(|(_, params)| {
            (params["output"].as_str().unwrap().to_string(), params["id"].as_i64().unwrap())
        })
        .collect();
    assert!(assigns.contains(&("DP-2".into(), 5)));
    assert!(assigns.contains(&("HDMI-1".into(), 5)));
    assert!(assigns.contains(&("*".into(), 0)));
    assert!(!assigns.iter().any(|(output, _)| output == "DP-1"));
}

#[test]
fn playlist_member_controls() {
    let mut app = test_app();
    app.panels.playlists = Some(crate::frontend::playlists::Playlists::default());
    let key = String::from("static:field-test.png");

    let _ = update(
        &mut app,
        Message::Pl(crate::frontend::playlists::PlMsg::MoveMember(31, key.clone(), -1)),
    );
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, params)| {
        method == "playlist.move"
            && params["id"] == 31
            && params["key"] == key
            && params["delta"] == -1
    }));
    assert!(calls.iter().any(|(method, _)| method == "playlist.members"));

    let _ = update(
        &mut app,
        Message::Pl(crate::frontend::playlists::PlMsg::RemoveMember(31, key.clone())),
    );
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, params)| {
        method == "playlist.remove" && params["id"] == 31 && params["key"] == key
    }));
    assert!(calls.iter().any(|(method, _)| method == "playlist.members"));
    assert!(calls.iter().any(|(method, _)| method == "playlist.list"));
}

#[test]
fn tag_submit_once() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0), wall("b.png", "static", 2, 0)]);
    app.scene.set_flipped_for_test(Some(0));
    app.tags.editing = true;
    super::begin_card_tag_edit(&mut app, &[]);
    app.tags.input = String::from("  Sunset ");
    let gen_before = app.library_session.library.tag_revision();
    let _ = update(&mut app, Message::Tag(TagMsg::Submit));
    assert_eq!(
        app.library_session.library.catalog().tags.get("a.png").map(Vec::as_slice),
        Some(&["sunset".into()][..])
    );
    assert_ne!(app.library_session.library.tag_revision(), gen_before);
    let calls = drain_calls(&app);
    let upd = calls.iter().find(|(method, _)| method == "wall.update_tags").expect("daemon call");
    assert_eq!(upd.1["key"], "a.png");
    assert_eq!(upd.1["tags"], "sunset");
    app.tags.input = String::from("sunset");
    let _ = update(&mut app, Message::Tag(TagMsg::Submit));
    assert_eq!(app.library_session.library.catalog().tags["a.png"].len(), 1);
    assert!(drain_calls(&app).iter().all(|(method, _)| method != "wall.update_tags"));
}
