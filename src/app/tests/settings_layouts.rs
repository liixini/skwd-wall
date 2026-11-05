use skwd_config::keys::selector as sel;

use crate::contracts::settings::DEFAULT_CORNER;

use super::*;
use crate::frontend::settings::{
    ActionId, LayoutMsg, LayoutPending, NudgeSpec, PREMADE_KEY, SettingsFocus, SettingsKey,
    SettingsMsg, strip_key,
};

static PAD: NudgeSpec =
    NudgeSpec { x: sel::TAG_CLOUD_OFFSET_X, y: sel::TAG_CLOUD_OFFSET_Y, coarse: 10.0, fine: 1.0 };

fn opened() -> App {
    let mut app = test_app();
    let _ = update(&mut app, Message::ToggleSettings);
    app
}

fn layout(app: &mut App, msg: LayoutMsg) {
    let _ = update(app, Message::Settings(SettingsMsg::Layout(msg)));
}

fn key(app: &mut App, key: SettingsKey) {
    let _ = update(app, Message::Settings(SettingsMsg::Key(key)));
}

fn run(app: &mut App, id: ActionId) {
    let _ = update(app, Message::Settings(SettingsMsg::Run(id)));
}

#[test]
fn opening_seeds_layout() {
    let app = opened();
    let view = app.config.saved_layouts_view();
    assert_eq!(view.names.len(), 1);
    assert!(!view.dirty());
}

#[test]
fn dirty_switch_asks() {
    let mut app = opened();
    layout(&mut app, LayoutMsg::SaveNew);
    app.config.set_key(sel::VISIBLE_COUNT, json!(9));
    layout(&mut app, LayoutMsg::Pick(0));
    assert_eq!(app.panels.settings.layouts.confirm, Some(LayoutPending::Switch(0)));
    assert_eq!(app.panels.settings.pop, Some(strip_key()));
    assert_eq!(app.config.saved_layouts_view().active, Some(1));
    layout(&mut app, LayoutMsg::ConfirmCancel);
    assert_eq!(app.panels.settings.layouts.confirm, None);
    layout(&mut app, LayoutMsg::Pick(0));
    layout(&mut app, LayoutMsg::ConfirmDiscard);
    assert_eq!(app.config.saved_layouts_view().active, Some(0));
    assert_eq!(app.panels.settings.pop, None);
}

#[test]
fn save_first_keeps_changes() {
    let mut app = opened();
    layout(&mut app, LayoutMsg::SaveNew);
    app.config.set_key(sel::VISIBLE_COUNT, json!(9));
    layout(&mut app, LayoutMsg::Pick(0));
    layout(&mut app, LayoutMsg::ConfirmSave);
    layout(&mut app, LayoutMsg::Pick(1));
    assert_eq!(app.config.num_path(sel::VISIBLE_COUNT), 9.0);
}

#[test]
fn premade_stays_unsaved() {
    let mut app = opened();
    let _ = update(&mut app, Message::Settings(SettingsMsg::Pick(PREMADE_KEY.into(), "1".into())));
    assert_eq!(app.config.str_path(sel::DISPLAY_MODE), "hex");
    assert_eq!(app.panels.settings.layouts.premade, Some(1));
    assert!(app.config.saved_layouts_view().dirty());
    layout(&mut app, LayoutMsg::SaveNew);
    let view = app.config.saved_layouts_view();
    assert_eq!(view.current(), Some(crate::i18n::tr("settings-layout-premade-compact")));
    assert!(!view.dirty());
}

#[test]
fn delete_needs_arming() {
    let mut app = opened();
    layout(&mut app, LayoutMsg::Duplicate);
    layout(&mut app, LayoutMsg::Delete);
    assert!(app.panels.settings.layouts.armed);
    assert_eq!(app.config.saved_layouts_view().names.len(), 2);
    layout(&mut app, LayoutMsg::Delete);
    assert_eq!(app.config.saved_layouts_view().names.len(), 1);
}

#[test]
fn rename_commits_cancels() {
    let mut app = opened();
    layout(&mut app, LayoutMsg::Rename);
    layout(&mut app, LayoutMsg::RenameInput("Desk".into()));
    layout(&mut app, LayoutMsg::RenameCommit);
    assert_eq!(app.config.saved_layouts_view().current(), Some("Desk"));
    layout(&mut app, LayoutMsg::Rename);
    layout(&mut app, LayoutMsg::RenameInput("Other".into()));
    key(&mut app, SettingsKey::Cancel);
    assert!(!app.panels.settings.layouts.renaming);
    assert!(app.panels.settings.open);
    assert_eq!(app.config.saved_layouts_view().current(), Some("Desk"));
}

#[test]
fn tab_reaches_strip() {
    let mut app = opened();
    app.panels.settings.focus = SettingsFocus::Index;
    key(&mut app, SettingsKey::FocusNext { backwards: false });
    assert_eq!(app.panels.settings.focus, SettingsFocus::Strip);
    key(&mut app, SettingsKey::Activate);
    assert_eq!(app.panels.settings.pop, Some(strip_key()));
    assert_eq!(app.panels.settings.focused_choice, Some(0));
    key(&mut app, SettingsKey::Down);
    assert_eq!(app.panels.settings.focused_choice, Some(1));
    key(&mut app, SettingsKey::Activate);
    assert_eq!(app.config.saved_layouts_view().names.len(), 2);
    assert_eq!(app.panels.settings.pop, None);
}

#[test]
fn enter_runs_strip_buttons() {
    let mut app = opened();
    app.config.set_key(sel::VISIBLE_COUNT, json!(9));
    app.panels.settings.focus = SettingsFocus::Strip;
    app.panels.settings.focused_control = 0;
    key(&mut app, SettingsKey::Next);
    assert_eq!(app.panels.settings.focused_control, 1);
    key(&mut app, SettingsKey::Activate);
    assert!(!app.config.saved_layouts_view().dirty());
    app.config.set_key(sel::VISIBLE_COUNT, json!(8));
    key(&mut app, SettingsKey::Next);
    key(&mut app, SettingsKey::Activate);
    assert_eq!(app.config.num_path(sel::VISIBLE_COUNT), 9.0);
}

#[test]
fn nudge_steps_shift_fine() {
    let mut app = opened();
    run(&mut app, ActionId::Nudge(&PAD, 1, 0));
    assert_eq!(app.config.num_path(PAD.x), 10.0);
    app.input.mods.shift = true;
    run(&mut app, ActionId::Nudge(&PAD, 0, -1));
    assert_eq!(app.config.num_path(PAD.y), -1.0);
    run(&mut app, ActionId::CentreNudge(&PAD));
    assert_eq!((app.config.num_path(PAD.x), app.config.num_path(PAD.y)), (0.0, 0.0));
}

fn setting(app: &mut App, msg: SettingsMsg) {
    drop(update(app, Message::Settings(msg)));
}

#[test]
fn shared_edits_only_all() {
    for keys in skwd_config::corners::ALL {
        let mut app = opened();
        setting(&mut app, SettingsMsg::Pick(keys.style.into(), "chamfered".into()));
        let before = app.config.corners(&keys).values;
        setting(&mut app, SettingsMsg::Input(keys.all.into(), "21".into()));
        let now = app.config.corners(&keys);
        assert_eq!((now.all, now.values), (21.0, before), "{}", keys.style);
        assert_eq!(now.sizes(), [21.0; 4], "{}", keys.style);
    }
}

#[test]
fn same_toggle_keeps_values() {
    for keys in skwd_config::corners::ALL {
        let mut app = opened();
        setting(&mut app, SettingsMsg::Pick(keys.style.into(), "chamfered".into()));
        setting(&mut app, SettingsMsg::Toggle(keys.same.into(), false));
        setting(&mut app, SettingsMsg::Input(keys.values[0].into(), "5".into()));
        setting(&mut app, SettingsMsg::Input(keys.values[2].into(), "9".into()));
        let each = app.config.corners(&keys);
        setting(&mut app, SettingsMsg::Toggle(keys.same.into(), true));
        assert_eq!(app.config.corners(&keys).sizes(), [each.all; 4], "{}", keys.style);
        setting(&mut app, SettingsMsg::Toggle(keys.same.into(), false));
        assert_eq!(app.config.corners(&keys).sizes(), each.values, "{}", keys.style);
    }
}

#[test]
fn same_keeps_stored_all() {
    for keys in skwd_config::corners::ALL {
        let mut app = opened();
        setting(&mut app, SettingsMsg::Pick(keys.style.into(), "chamfered".into()));
        setting(&mut app, SettingsMsg::Toggle(keys.same.into(), false));
        setting(&mut app, SettingsMsg::Input(keys.all.into(), "21".into()));
        setting(&mut app, SettingsMsg::Input(keys.values[0].into(), "5".into()));
        setting(&mut app, SettingsMsg::Toggle(keys.same.into(), true));
        assert_eq!(app.config.corners(&keys).sizes(), [21.0; 4], "{}", keys.style);
    }
}

#[test]
fn same_off_seeds_values() {
    let keys = &skwd_config::corners::HAND;
    let mut app = opened();
    setting(&mut app, SettingsMsg::Pick(keys.style.into(), "round".into()));
    setting(&mut app, SettingsMsg::Toggle(keys.same.into(), false));
    assert_eq!(app.config.corners(keys).sizes(), [DEFAULT_CORNER; 4]);
}

#[test]
fn same_off_keeps_stored() {
    let keys = &skwd_config::corners::HAND;
    let mut app = opened();
    setting(&mut app, SettingsMsg::Pick(keys.style.into(), "round".into()));
    app.config.set_key(keys.values[1], json!(7));
    setting(&mut app, SettingsMsg::Toggle(keys.same.into(), false));
    let now = app.config.corners(keys);
    assert_eq!(now.values[1], 7.0);
    assert_eq!(app.config.stored(keys.values[0]), None);
}

#[test]
fn leaving_straight_shows() {
    for keys in [&skwd_config::corners::HAND, &skwd_config::corners::DEPTH] {
        let mut app = opened();
        setting(&mut app, SettingsMsg::Pick(keys.style.into(), "chamfered".into()));
        let now = app.config.corners(keys);
        assert!(now.same, "{}", keys.style);
        assert_eq!(now.sizes(), [DEFAULT_CORNER; 4], "{}", keys.style);
    }
    let mut app = opened();
    let keys = &skwd_config::corners::SLICES;
    setting(&mut app, SettingsMsg::Pick(keys.style.into(), "round".into()));
    assert_eq!(app.config.corners(keys).sizes(), [16.0; 4]);
    assert!(!app.config.corners_all_stored(keys));
}

#[test]
fn leaving_straight_waits() {
    let keys = &skwd_config::corners::HAND;
    let mut app = opened();
    app.config.set_key(keys.values[0], json!(4));
    setting(&mut app, SettingsMsg::Pick(keys.style.into(), "round".into()));
    assert!(!app.config.corners_all_stored(keys));
    let mut app = opened();
    setting(&mut app, SettingsMsg::Toggle(keys.same.into(), false));
    app.config.set_key(keys.values[2], json!(9));
    setting(&mut app, SettingsMsg::Pick(keys.style.into(), "round".into()));
    assert!(!app.config.corners_all_stored(keys));
    assert_eq!(app.config.corners(keys).values[2], 9.0);
}

#[test]
fn premade_asks_when_dirty() {
    let mut app = opened();
    app.config.set_key(sel::VISIBLE_COUNT, json!(9));
    layout(&mut app, LayoutMsg::Premade(1));
    assert_eq!(app.panels.settings.layouts.confirm, Some(LayoutPending::Premade(1)));
    assert_ne!(app.config.str_path(sel::DISPLAY_MODE), "hex");
    layout(&mut app, LayoutMsg::ConfirmCancel);
    layout(&mut app, LayoutMsg::Revert);
    layout(&mut app, LayoutMsg::Premade(1));
    assert_eq!(app.panels.settings.layouts.confirm, None);
    assert_eq!(app.config.str_path(sel::DISPLAY_MODE), "hex");
}

#[test]
fn many_layouts_switch() {
    let mut app = opened();
    for number in 0..300 {
        app.config.add_layout(&format!("Layout {number}"));
    }
    layout(&mut app, LayoutMsg::Pick(260));
    assert_eq!(app.config.saved_layouts_view().active, Some(260));
    layout(&mut app, LayoutMsg::Premade(0));
    layout(&mut app, LayoutMsg::ConfirmDiscard);
    assert_eq!(app.panels.settings.layouts.premade, Some(0));
}

#[test]
fn rejected_batch_shows() {
    let mut app = opened();
    layout(&mut app, LayoutMsg::SaveNew);
    let mut entries = app.config.array_values(crate::contracts::settings::SAVED_LAYOUTS);
    entries[0]["values"][sel::FRAME_X] = json!(25);
    app.config.set_key(crate::contracts::settings::SAVED_LAYOUTS, json!(entries));
    app.config.set_key(skwd_config::picker_layout::ROOT, json!({"version": 2, "outputs": {}}));
    app.config.select_layout_output(
        Some(String::from("DP-1")),
        skwd_config::picker_layout::Size { width: 1280.0, height: 720.0 },
    );
    let before = app.config.document().clone();
    layout(&mut app, LayoutMsg::Pick(0));
    assert!(app.panels.settings.layouts.rejected);
    assert_eq!(app.config.document(), &before);
    assert_eq!(app.config.saved_layouts_view().active, Some(1));
    layout(&mut app, LayoutMsg::Menu);
    assert!(!app.panels.settings.layouts.rejected);
}

#[test]
fn corrupt_name_save_new() {
    let mut app = opened();
    app.config.set_key(crate::contracts::settings::ACTIVE_LAYOUT, json!("Gone"));
    let before = app.config.document().clone();
    for msg in [LayoutMsg::Save, LayoutMsg::Revert, LayoutMsg::Rename, LayoutMsg::Delete] {
        layout(&mut app, msg);
    }
    assert_eq!(app.config.document(), &before);
    assert!(!app.panels.settings.layouts.renaming);
    layout(&mut app, LayoutMsg::SaveNew);
    assert_eq!(app.config.saved_layouts_view().names.len(), 2);
    assert!(app.config.saved_layouts_view().current().is_some());
}
