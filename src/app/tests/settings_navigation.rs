use super::*;
use crate::frontend::settings::{Control, SectionId, SettingsFocus, SettingsKey, SettingsMsg};

fn place(app: &App) -> crate::frontend::settings::Place {
    app.place_source().place(&app.panels.settings.tab)
}

fn section_id(app: &App, index: usize) -> SectionId {
    place(app).sections[index].id
}

fn open_title(app: &App) -> Option<String> {
    let open = app.panels.settings.open_section?;
    place(app).sections.into_iter().find(|section| section.id == open).map(|section| section.title)
}

#[test]
fn switching_collapses_sections() {
    let mut app = test_app();
    app.panels.settings.open = true;
    app.panels.settings.entrance.snap(1.0);
    let mut now = Instant::now();
    let _ = update(&mut app, Message::Settings(SettingsMsg::ToggleSection(3)));
    tick_frames(&mut app, &mut now, 40);
    assert_eq!(app.panels.settings.open_section, Some(section_id(&app, 3)));
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("library".into())));
    tick_frames(&mut app, &mut now, 40);
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("layout".into())));
    tick_frames(&mut app, &mut now, 40);
    assert_eq!(app.panels.settings.open_section, None);
    assert!(app.panels.settings.previous.is_none());
}

#[test]
fn toggle_waits_for_close() {
    let mut app = test_app();
    app.panels.settings.open = true;
    app.panels.settings.entrance.snap(1.0);
    let mut now = Instant::now();
    let _ = update(&mut app, Message::Settings(SettingsMsg::ToggleSection(0)));
    tick_frames(&mut app, &mut now, 30);
    let _ = update(&mut app, Message::Settings(SettingsMsg::ToggleSection(2)));
    assert_eq!(app.panels.settings.open_section, None);
    tick_frames(&mut app, &mut now, 10);
    assert_eq!(app.panels.settings.open_section, None, "still closing");
    tick_frames(&mut app, &mut now, 20);
    assert_eq!(app.panels.settings.open_section, Some(section_id(&app, 2)));
    tick_frames(&mut app, &mut now, 30);
    assert!(!app.panels.settings.sections_animating());
}

#[test]
fn rapid_navigation_keeps_latest() {
    let mut app = test_app();
    app.panels.settings.open = true;
    app.panels.settings.entrance.snap(1.0);
    let mut now = Instant::now();
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("library".into())));
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("theme".into())));
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("playback".into())));
    assert_eq!(app.panels.settings.tab, "library");
    tick_frames(&mut app, &mut now, 80);
    assert_eq!(app.panels.settings.tab, "wallpapers");
    assert!(app.panels.settings.previous.is_none());
    assert!(app.panels.settings.pending.is_none());
}

#[test]
fn transition_blocks_activation() {
    let mut app = test_app();
    app.panels.settings.open = true;
    app.panels.settings.entrance.snap(1.0);
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("system".into())));
    app.panels.settings.focus = SettingsFocus::Controls;
    let before = app.config.root().clone();
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Activate)));
    assert_eq!(app.config.root(), &before);
    assert!(app.panels.settings.previous.is_some());
}

#[test]
fn fastest_profile_settles() {
    let mut app = test_app();
    app.panels.settings.open = true;
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("system".into())));
    assert!(app.panels.settings.previous.is_some());
    app.panels
        .settings
        .set_motion_profile(crate::frontend::animation::MotionProfile::new(35.0, 35.0, 35.0));
    assert!(app.panels.settings.previous.is_none());
    assert!(app.panels.settings.section_anim.settled());
    assert!(app.panels.settings.tab_anim.settled());
}

#[test]
fn instant_motion_drains_navigation() {
    let mut app = test_app();
    app.panels.settings.open = true;
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("library".into())));
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("colours".into())));
    assert!(app.panels.settings.pending.is_some());
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Input(
            skwd_config::keys::motion::FAST_MS.into(),
            "35".into(),
        )),
    );
    assert_eq!(app.panels.settings.tab, "colours");
    assert!(app.panels.settings.pending.is_none());
    assert!(app.panels.settings.previous.is_none());
}

#[test]
fn closing_discards_pending() {
    let mut app = test_app();
    app.panels.settings.open = true;
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("library".into())));
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("colours".into())));
    assert!(app.panels.settings.pending.is_some());
    app.close_settings();
    assert!(app.panels.settings.previous.is_none());
    assert!(app.panels.settings.pending.is_none());
}

#[test]
fn search_opens_section() {
    let mut app = test_app();
    app.panels.settings.open = true;
    app.panels.settings.entrance.snap(1.0);
    let mut now = Instant::now();
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::OpenSearchResult("system".into(), 0, 2)));
    tick_frames(&mut app, &mut now, 40);
    assert_eq!(app.panels.settings.tab, "system");
    assert_eq!(app.panels.settings.open_section, Some(section_id(&app, 0)));
    assert_eq!(app.panels.settings.focus, SettingsFocus::Controls);
    assert_eq!(app.panels.settings.focused_control, 2);
}

#[test]
fn arrows_walk_sections() {
    let mut app = test_app();
    app.panels.settings.open = true;
    app.panels.settings.focus = SettingsFocus::Sections;
    let count = place(&app).sections.len();
    let mut now = Instant::now();
    for expected in 1..count {
        let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Down)));
        tick_frames(&mut app, &mut now, 2);
        assert_eq!(app.panels.settings.section, expected);
    }
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Down)));
    assert_eq!(app.panels.settings.section, count - 1);
    assert_eq!(app.panels.settings.open_section, None, "arrows only move focus");
}

#[test]
fn right_arrow_opens_section() {
    let mut app = test_app();
    app.panels.settings.open = true;
    app.panels.settings.focus = SettingsFocus::Sections;
    app.panels.settings.section = 2;
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Next)));
    assert_eq!(app.panels.settings.open_section, Some(section_id(&app, 2)));
    assert_eq!(app.panels.settings.focus, SettingsFocus::Controls);
}

#[test]
fn number_steps_commit() {
    let mut app = test_app();
    let key = skwd_config::keys::general::UI_SCALE;
    app.config.save_key(key, json!(1.23));
    app.init_settings_inputs();
    let _ = update(&mut app, Message::Settings(SettingsMsg::NumberStep(key.into(), 0.01)));
    assert!((app.config.num_path(key) - 1.24).abs() < 1e-9);
    assert_eq!(app.panels.settings.inputs.get(key).unwrap(), "1.24");
    assert!(app.panels.settings.input_edit.is_none());
    let disk: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&app.config.config_path).unwrap()).unwrap();
    assert_eq!(disk["general"]["uiScale"], json!(1.24));
}

#[test]
fn legacy_routes_reach_controls() {
    let mut app = test_app();
    let mut now = Instant::now();
    for (route, tab, title) in [
        ("filter", "bar", "Filter bar"),
        ("search", "bar", "Search"),
        ("position", "layout", "Placement"),
        ("motion", "layout", "Opening and motion"),
        ("performance", "system", "Laptop & power"),
        ("language", "system", "Language"),
    ] {
        let _ = update(
            &mut app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(format!("tab {route}"))),
        );
        tick_frames(&mut app, &mut now, 80);
        assert_eq!(app.panels.settings.tab, tab);
        let focused = place(&app).sections[app.panels.settings.section].title.clone();
        assert_eq!(focused, title, "{route}");
    }
}

#[test]
fn studio_returns_to_layout() {
    let mut app = test_app();
    app.panels.settings.open = true;
    let mut now = Instant::now();
    let _ = update(&mut app, Message::Settings(SettingsMsg::ToggleSection(0)));
    tick_frames(&mut app, &mut now, 40);
    let _ = update(&mut app, Message::Settings(SettingsMsg::EnterLayoutStudio));
    assert!(app.panels.settings.layout_studio);
    let _ = update(&mut app, Message::Settings(SettingsMsg::LeaveLayoutStudio));
    assert!(!app.panels.settings.layout_studio);
    assert_eq!(open_title(&app).as_deref(), Some("Style"));
    let _ = update(&mut app, Message::Settings(SettingsMsg::EnterLayoutStudio));
    let _ = update(&mut app, Message::ToggleSettings);
    assert!(!app.panels.settings.open);
    assert!(!app.panels.settings.layout_studio);
}

#[test]
fn process_text_commits_directly() {
    let mut app = test_app();
    let path = skwd_config::keys::playback::PROCESSES;
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("playback".into())));
    app.config.save_key(path, json!("steam, obs"));
    app.init_settings_inputs();
    assert_eq!(app.panels.settings.inputs.get(path).unwrap(), "steam, obs");
    for raw in ["1234", "steam, obs", ""] {
        let _ = update(&mut app, Message::Settings(SettingsMsg::Input(path.into(), raw.into())));
        let _ = update(&mut app, Message::Settings(SettingsMsg::Commit));
        assert_eq!(app.config.str_path(path), raw);
    }
    assert!(!app.panels.settings.process_picker_open);
}

#[test]
fn pointer_edit_can_cancel() {
    let mut app = test_app();
    app.panels.settings.open = true;
    let path = skwd_config::keys::playback::PROCESSES;
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("playback".into())));
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 40);
    app.config.save_key(path, json!("steam"));
    app.init_settings_inputs();
    let _ = update(&mut app, Message::Settings(SettingsMsg::Input(path.into(), "obs".into())));
    assert_eq!(app.config.str_path(path), "obs");
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Cancel)));
    assert_eq!(app.config.str_path(path), "steam");
    assert!(app.panels.settings.open);
}

#[test]
fn token_edit_commits() {
    let mut app = test_app();
    app.panels.settings.open = true;
    let key = skwd_config::keys::general::UI_SCALE;
    app.init_settings_inputs();
    let token =
        crate::frontend::settings::token_key(key, 0, crate::frontend::settings::TokenSide::Body);
    let _ = update(&mut app, Message::Settings(SettingsMsg::EditToken(token, key.into())));
    assert_eq!(app.panels.settings.edit, Some(token));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Input(key.into(), "1.5".into())));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Commit));
    assert_eq!(app.panels.settings.edit, None);
    assert!((app.config.num_path(key) - 1.5).abs() < 1e-9);
}

#[test]
fn outside_click_closes_menu() {
    let mut app = test_app();
    app.panels.settings.open = true;
    let _ = update(&mut app, Message::Settings(SettingsMsg::OpenPop(7)));
    assert_eq!(app.panels.settings.pop, Some(7));
    let _ = update(&mut app, Message::Settings(SettingsMsg::OpenPop(7)));
    assert_eq!(app.panels.settings.pop, None, "second press closes");
    let _ = update(&mut app, Message::Settings(SettingsMsg::OpenPop(7)));
    let _ = update(&mut app, Message::Settings(SettingsMsg::ClosePop));
    assert_eq!(app.panels.settings.pop, None);
    let _ = update(&mut app, Message::Settings(SettingsMsg::OpenPop(7)));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Cancel)));
    assert_eq!(app.panels.settings.pop, None);
    assert!(app.panels.settings.open, "escape closes menu first");
}

#[test]
fn pick_closes_menu() {
    let mut app = test_app();
    app.panels.settings.open = true;
    let _ = update(&mut app, Message::Settings(SettingsMsg::OpenPop(7)));
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Pick(
            skwd_config::keys::selector::DISPLAY_MODE.into(),
            "hex".into(),
        )),
    );
    assert_eq!(app.panels.settings.pop, None);
}

#[test]
fn hand_opens_geometry() {
    let mut app = test_app();
    app.panels.settings.open = true;
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Pick(
            skwd_config::keys::selector::DISPLAY_MODE.into(),
            "hand".into(),
        )),
    );
    assert_eq!(open_title(&app).as_deref(), Some("Shape"));
    assert_eq!(app.panels.settings.previous.as_ref().unwrap().place.key, "layout");
}

#[test]
fn state_reports_visible_sections() {
    let mut app = test_app();
    app.panels.settings.open = true;
    app.panels.settings.entrance.snap(1.0);
    let count = place(&app).sections.len();
    let state: serde_json::Value =
        serde_json::from_str(&crate::app::update::ui_state_json(&app)).unwrap();
    assert_eq!(state["settings"]["sections"].as_array().unwrap().len(), count);
    assert_eq!(state["settings"]["sections"][0], json!("Style"));
    assert_eq!(state["settings"]["transitioning"], json!(false));
    assert_eq!(state["settings"]["open_section"], json!(null));
    let _ = update(&mut app, Message::Settings(SettingsMsg::ToggleSection(1)));
    let state: serde_json::Value =
        serde_json::from_str(&crate::app::update::ui_state_json(&app)).unwrap();
    assert_eq!(state["settings"]["open_section"], json!(1));
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("library".into())));
    let state: serde_json::Value =
        serde_json::from_str(&crate::app::update::ui_state_json(&app)).unwrap();
    assert_eq!(state["settings"]["transitioning"], json!(true));
}

#[test]
fn emblem_finishes_last_stroke() {
    let mut app = test_app();
    app.panels.settings.open = true;
    app.panels.settings.entrance.snap(1.0);
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("library".into())));
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 27);
    assert!(app.panels.settings.section_anim.settled());
    assert!(!app.panels.settings.emblem_anim.settled());
    tick_frames(&mut app, &mut now, 10);
    assert!(app.panels.settings.emblem_anim.settled());
}

fn focus_row(app: &mut App, title: &str, matches: impl Fn(&Control) -> bool) -> Control {
    let section = open_section(app, title);
    let (row, control) = section
        .rows()
        .enumerate()
        .find(|(_, row)| matches(&row.control))
        .map(|(at, row)| (at, row.control.clone()))
        .expect("row");
    app.panels.settings.focus = SettingsFocus::Controls;
    app.panels.settings.focused_control = row;
    control
}

fn body_token(identity: &str) -> u64 {
    crate::frontend::settings::token_key(identity, 0, crate::frontend::settings::TokenSide::Body)
}

fn head_token(identity: &str) -> u64 {
    crate::frontend::settings::token_key(
        identity,
        0,
        crate::frontend::settings::TokenSide::Sentence,
    )
}

#[test]
fn arrows_leave_numbers() {
    let mut app = test_app();
    app.panels.settings.open = true;
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("layout".into())));
    tick_frames(&mut app, &mut Instant::now(), 60);
    let key = skwd_config::keys::general::UI_SCALE;
    let before = app.config.num_path(key);
    focus_row(&mut app, "Opening and motion", |control| matches!(control, Control::Number { .. }));
    let row = app.panels.settings.focused_control;
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Up)));
    assert_eq!(app.panels.settings.focused_control, row - 1);
    assert_eq!(app.config.num_path(key), before, "arrows never step");
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Previous)));
    assert_eq!(app.panels.settings.focus, SettingsFocus::Sections);
}

#[test]
fn enter_engages_number() {
    let mut app = test_app();
    app.panels.settings.open = true;
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("layout".into())));
    tick_frames(&mut app, &mut Instant::now(), 60);
    focus_row(&mut app, "Opening and motion", |control| matches!(control, Control::Number { .. }));
    let key = skwd_config::keys::general::UI_SCALE;
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Activate)));
    assert_eq!(app.panels.settings.edit, Some(head_token(key)));
    assert!(app.panels.settings.input_edit.is_some());
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Cancel)));
    assert_eq!(app.panels.settings.edit, None);
    assert!(app.panels.settings.input_edit.is_none());
}

#[test]
fn keyboard_menu_picks() {
    let mut app = test_app();
    app.panels.settings.open = true;
    let control = focus_row(&mut app, "Opening and motion", |control| {
        matches!(control, Control::Dropdown { .. })
    });
    let Control::Dropdown { path, options, current, .. } = control else { unreachable!() };
    let at = options.iter().position(|(value, _)| *value == current).unwrap();
    let _ = update(&mut app, Message::Settings(SettingsMsg::OpenPop(head_token(&path))));
    assert_eq!(app.panels.settings.focused_choice, Some(at));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Down)));
    let next = (at + 1) % options.len();
    assert_eq!(app.panels.settings.focused_choice, Some(next));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Activate)));
    assert_eq!(app.config.str_path(&path), options[next].0);
    assert_eq!(app.panels.settings.pop, None);
    let _ = update(&mut app, Message::Settings(SettingsMsg::OpenPop(head_token(&path))));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Cancel)));
    assert_eq!((app.panels.settings.pop, app.panels.settings.focused_choice), (None, None));
    assert!(app.panels.settings.open);
}

#[test]
fn number_step_clamps() {
    let mut app = test_app();
    let key = skwd_config::keys::general::UI_SCALE;
    app.init_settings_inputs();
    let _ = update(&mut app, Message::Settings(SettingsMsg::NumberStep(key.into(), 1.0e6)));
    assert!(app.config.num_path(key) < 1.0e6);
}

#[test]
fn outside_range_steps_inward() {
    let mut app = test_app();
    let key = skwd_config::keys::selector::STEAM_ROWS;
    let step = |app: &mut App, step: f64| {
        let _ = update(app, Message::Settings(SettingsMsg::NumberStep(key.into(), step)));
        app.config.num_path(key)
    };
    app.config.save_key(key, json!(40));
    app.init_settings_inputs();
    assert_eq!(step(&mut app, 1.0), 40.0, "up stays put");
    assert_eq!(step(&mut app, -1.0), 8.0, "down lands on max");
}

#[test]
fn staged_drag_commits_once() {
    let mut app = test_app();
    app.panels.settings.open = true;
    let key = skwd_config::keys::general::UI_SCALE;
    app.init_settings_inputs();
    for raw in ["1.10", "1.20", "1.30"] {
        let _ = update(&mut app, Message::Settings(SettingsMsg::Input(key.into(), raw.into())));
    }
    assert!(app.panels.settings.input_edit.is_some(), "staged during drag");
    let _ = update(&mut app, Message::Settings(SettingsMsg::Commit));
    assert!((app.config.num_path(key) - 1.3).abs() < 1e-9);
    assert!(app.panels.settings.input_edit.is_none());
}

#[test]
fn toggle_commits_edit() {
    let mut app = test_app();
    app.panels.settings.open = true;
    let key = skwd_config::keys::general::UI_SCALE;
    app.init_settings_inputs();
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::EditToken(body_token(key), key.into())));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Input(key.into(), "1.4".into())));
    let toggle = skwd_config::keys::general::CLOSE_ON_SELECTION;
    let flag = crate::contracts::settings::SettingsSource::flag(&app.config, toggle);
    let _ = update(&mut app, Message::Settings(SettingsMsg::Toggle(toggle.into(), !flag)));
    assert_eq!(app.panels.settings.edit, None);
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Cancel)));
    assert!((app.config.num_path(key) - 1.4).abs() < 1e-9, "typed value kept");
}

#[test]
fn queued_route_opens_section() {
    let mut app = test_app();
    app.panels.settings.open = true;
    app.panels.settings.entrance.snap(1.0);
    let mut now = Instant::now();
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("library".into())));
    app.panels.settings.pop = Some(7);
    let _ = update(&mut app, Message::SetSettingsTab("search".into()));
    assert!(app.panels.settings.pending.is_some());
    assert_eq!(app.panels.settings.pop, None);
    tick_frames(&mut app, &mut now, 80);
    assert_eq!(app.panels.settings.tab, "bar");
    assert_eq!(open_title(&app).as_deref(), Some("Search"));
}

#[test]
fn section_by_key() {
    let mut app = test_app();
    let mut now = Instant::now();
    let command =
        |text: &str| Message::Daemon(crate::infrastructure::runtime::Wake::Command(text.into()));
    let _ = update(&mut app, command("tab system"));
    tick_frames(&mut app, &mut now, 80);
    let _ = update(&mut app, command("section settings-performance-power-card"));
    assert_eq!(open_title(&app).as_deref(), Some("Laptop & power"));
    let state: serde_json::Value =
        serde_json::from_str(&crate::app::update::ui_state_json(&app)).unwrap();
    assert_eq!(state["settings"]["open_section_key"], json!("settings-performance-power-card"));
    assert_eq!(state["settings"]["tab"], json!("system"));
}

#[test]
fn keyboard_opens_sentence_menu() {
    use crate::frontend::settings::{TokenSide, token_key};
    let mut app = test_app();
    let mut now = Instant::now();
    let _ = update(
        &mut app,
        Message::Daemon(crate::infrastructure::runtime::Wake::Command("tab system".into())),
    );
    tick_frames(&mut app, &mut now, 80);
    let language = place(&app)
        .sections
        .iter()
        .position(|section| section.id.key == "settings-language-choice-label")
        .expect("language");
    app.panels.settings.section = language;
    app.panels.settings.focus = SettingsFocus::Sections;
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Next)));
    assert_eq!(app.panels.settings.open_section, None);
    assert_eq!(app.panels.settings.focus, SettingsFocus::Controls);
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Activate)));
    let sentence = token_key(skwd_config::keys::general::LANGUAGE, 0, TokenSide::Sentence);
    assert_eq!(app.panels.settings.pop, Some(sentence));
}

#[test]
fn keyboard_keeps_custom_value() {
    let mut app = test_app();
    let mut now = Instant::now();
    app.config.set_key(skwd_config::keys::performance::GPU_PREFERENCE, json!("turbo"));
    let _ = update(
        &mut app,
        Message::Daemon(crate::infrastructure::runtime::Wake::Command("tab system".into())),
    );
    tick_frames(&mut app, &mut now, 80);
    app.panels.settings.section = place(&app)
        .sections
        .iter()
        .position(|section| section.id.key == "settings-performance-gpu-label")
        .expect("gpu");
    app.panels.settings.focus = SettingsFocus::Sections;
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Next)));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Activate)));
    assert_eq!(app.panels.settings.focused_choice, Some(0));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Activate)));
    assert_eq!(app.panels.settings.pop, None);
    assert_eq!(app.config.str_path(skwd_config::keys::performance::GPU_PREFERENCE), "turbo");
}

#[test]
fn parked_rows_skipped() {
    let mut app = test_app();
    app.config.save_key(skwd_config::keys::features::WALLHAVEN, json!(false));
    app.panels.settings.open = true;
    app.panels.settings.entrance.snap(1.0);
    let mut now = Instant::now();
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("sources".into())));
    tick_frames(&mut app, &mut now, 80);
    let _ = update(&mut app, Message::Settings(SettingsMsg::ToggleSection(0)));
    tick_frames(&mut app, &mut now, 40);
    let wallhaven = place(&app).sections.remove(0);
    let _ = update(&mut app, Message::Settings(SettingsMsg::FocusControl(0)));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Down)));
    assert_eq!(app.panels.settings.focused_control, wallhaven.header_rows());
    assert!(!wallhaven.say.parked.is_empty());
}

#[test]
fn parked_result_focused() {
    let mut app = test_app();
    app.config.save_key(skwd_config::keys::semantic::ENABLED, json!(false));
    app.panels.settings.open = true;
    app.panels.settings.entrance.snap(1.0);
    let _ = update(&mut app, Message::Settings(SettingsMsg::SearchOpen));
    let _ = update(&mut app, Message::Settings(SettingsMsg::SearchInput("active model".into())));
    let result = app
        .panels
        .settings
        .search_results
        .iter()
        .find(|result| result.tab == "bar" && result.title == "Active model")
        .cloned()
        .expect("model result");
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::OpenSearchResult(result.tab, result.section, result.row)),
    );
    let bar = place(&app);
    let target = &bar.sections[result.section];
    let cause = target
        .rows()
        .position(|row| row.control.path() == Some(skwd_config::keys::semantic::ENABLED))
        .expect("master");
    assert_ne!(result.row, cause);
    assert_eq!(app.panels.settings.focused_control, result.row);
    assert_eq!(app.panels.settings.open_section, Some(target.id));
}

#[test]
fn checks_one_stop() {
    use crate::contracts::settings::SettingsSource;
    use skwd_config::keys::wallhaven;
    let mut app = test_app();
    app.panels.settings.open = true;
    app.panels.settings.entrance.snap(1.0);
    let mut now = Instant::now();
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("sources".into())));
    tick_frames(&mut app, &mut now, 80);
    let _ = update(&mut app, Message::Settings(SettingsMsg::ToggleSection(0)));
    tick_frames(&mut app, &mut now, 40);
    let wallhaven = place(&app).sections.remove(0);
    let at = |path: &str| wallhaven.rows().position(|row| row.control.path() == Some(path));
    let key = |app: &mut App, key: SettingsKey| {
        let _ = update(app, Message::Settings(SettingsMsg::Key(key)));
    };
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::FocusControl(at(wallhaven::DEFAULT_GENERAL).unwrap())),
    );
    key(&mut app, SettingsKey::Down);
    assert_eq!(Some(app.panels.settings.focused_control), at(wallhaven::DEFAULT_SFW));
    key(&mut app, SettingsKey::Up);
    assert_eq!(Some(app.panels.settings.focused_control), at(wallhaven::DEFAULT_GENERAL));
    let anime = app.config.flag(wallhaven::DEFAULT_ANIME);
    key(&mut app, SettingsKey::Activate);
    assert!(app.panels.settings.pop.is_some());
    assert_eq!(app.panels.settings.focused_choice, Some(0));
    key(&mut app, SettingsKey::Down);
    key(&mut app, SettingsKey::Activate);
    assert_eq!(app.config.flag(wallhaven::DEFAULT_ANIME), !anime);
    assert!(app.panels.settings.pop.is_some(), "menu stays open");
    key(&mut app, SettingsKey::Cancel);
    assert_eq!((app.panels.settings.pop, app.panels.settings.focused_choice), (None, None));
}
