use iced::Task;
use serde_json::json;

use crate::app::{App, Message};
use skwd_config::corners::{self, CornerStyle, Corners, TOP_LEFT};

use crate::app::state::SideWrite;
use crate::contracts::settings::{DEFAULT_CORNER, PREMADES, keys};
use crate::frontend::settings::{
    ActionId, LAYOUT_NAME_KEY, LayoutMsg, LayoutPending, NudgeSpec, PREMADE_KEY, STRIP_STOPS,
    SettingsFocus, SettingsMsg, StripEntry, strip_entries, strip_key, workbench_input_id,
};

const FIRST_NAME: &str = "settings-layout-first";
const COPY_NAME: &str = "settings-layout-copy";
const NEXT_NAME: &str = "settings-layout-next";

pub(super) fn seed(app: &mut App) {
    app.config.seed_saved_layouts(crate::i18n::tr(FIRST_NAME));
}

fn refresh(app: &mut App) -> Task<Message> {
    app.apply_layout();
    app.apply_motion_speeds();
    app.scene.set_preview_config(
        app.config.video_preview_enabled(),
        app.config.video_preview_delay_ms(),
        app.config.video_preview_fps(),
    );
    let effect = app.config.str_path(keys::selector::FLIP_EFFECT);
    super::settings::settings_pick_side_effects(app, keys::selector::FLIP_EFFECT, &effect);
    app.init_settings_inputs();
    close_vanished_section(app);
    app.invalidate_settings();
    app.retick();
    Task::none()
}

pub(super) fn close_vanished_section(app: &mut App) {
    let place = super::settings::settings_place(app, &app.panels.settings.tab);
    let open = app.panels.settings.open_section;
    if open.is_some_and(|id| !place.sections.iter().any(|section| section.id == id)) {
        app.panels.settings.close_sections();
    }
    app.panels.settings.section =
        app.panels.settings.section.min(place.sections.len().saturating_sub(1));
}

fn run_pending(app: &mut App, pending: LayoutPending) -> Task<Message> {
    let ui = &mut app.panels.settings.layouts;
    ui.confirm = None;
    ui.armed = false;
    ui.premade = None;
    app.panels.settings.pop = None;
    app.panels.settings.focused_choice = None;
    let outcome = match pending {
        LayoutPending::Switch(at) => app.config.switch_layout(at),
        LayoutPending::Delete => app.config.delete_layout(),
        LayoutPending::Premade(at) => PREMADES.get(at).map_or(Ok(()), |premade| {
            app.config.apply_premade(premade)?;
            app.panels.settings.layouts.premade = Some(at);
            Ok(())
        }),
    };
    app.panels.settings.layouts.rejected = outcome.is_err();
    refresh(app)
}

fn guard(app: &mut App, pending: LayoutPending) -> Task<Message> {
    if !app.config.saved_layouts_view().dirty() {
        return run_pending(app, pending);
    }
    app.panels.settings.layouts.confirm = Some(pending);
    app.panels.settings.pop = Some(strip_key());
    app.panels.settings.focused_choice = None;
    app.retick();
    Task::none()
}

fn add_named(app: &mut App, name: &str) -> Task<Message> {
    let unique = app.config.unique_layout(name);
    app.config.add_layout(&unique);
    app.panels.settings.layouts.premade = None;
    app.panels.settings.pop = None;
    app.invalidate_settings();
    app.retick();
    Task::none()
}

fn current_name(app: &App) -> String {
    let view = app.config.saved_layouts_view();
    view.current().unwrap_or_else(|| crate::i18n::tr(FIRST_NAME)).to_string()
}

fn has_current(app: &App) -> bool {
    app.config.saved_layouts_view().current().is_some()
}

fn settle(app: &mut App) -> Task<Message> {
    app.invalidate_settings();
    app.retick();
    Task::none()
}

fn rename(app: &mut App, msg: LayoutMsg) -> Task<Message> {
    match msg {
        LayoutMsg::RenameInput(raw) => {
            app.panels.settings.inputs.insert(LAYOUT_NAME_KEY.to_string(), raw);
            app.retick();
            Task::none()
        }
        LayoutMsg::RenameCommit => {
            let draft = app.panels.settings.inputs.remove(LAYOUT_NAME_KEY).unwrap_or_default();
            app.config.rename_layout(&draft);
            app.panels.settings.layouts.renaming = false;
            settle(app)
        }
        _ => {
            let name = current_name(app);
            app.panels.settings.inputs.insert(LAYOUT_NAME_KEY.to_string(), name);
            app.panels.settings.layouts.renaming = true;
            app.panels.settings.pop = None;
            app.retick();
            iced::widget::operation::focus(workbench_input_id(LAYOUT_NAME_KEY))
        }
    }
}

pub(super) fn update(app: &mut App, msg: LayoutMsg) -> Task<Message> {
    if !matches!(msg, LayoutMsg::RenameInput(_)) {
        super::settings::commit_settings_input_edit(app);
    }
    app.panels.settings.layouts.rejected = false;
    match msg {
        LayoutMsg::Menu => {
            let key = strip_key();
            let opening = app.panels.settings.pop != Some(key);
            app.panels.settings.pop = opening.then_some(key);
            app.panels.settings.layouts.confirm = None;
            app.panels.settings.layouts.armed = false;
            app.panels.settings.focused_choice = None;
            settle(app)
        }
        LayoutMsg::Pick(at) => {
            if Some(at) == app.config.saved_layouts_view().active {
                app.panels.settings.pop = None;
                return settle(app);
            }
            guard(app, LayoutPending::Switch(at))
        }
        LayoutMsg::Premade(at) => guard(app, LayoutPending::Premade(at)),
        LayoutMsg::Save if has_current(app) => {
            app.config.save_layout();
            app.panels.settings.layouts.premade = None;
            settle(app)
        }
        LayoutMsg::Revert if has_current(app) => {
            let outcome = app.config.revert_layout();
            app.panels.settings.layouts.rejected = outcome.is_err();
            app.panels.settings.layouts.premade = None;
            refresh(app)
        }
        LayoutMsg::SaveNew => {
            let premade = app.panels.settings.layouts.premade.and_then(|at| PREMADES.get(at));
            let current = current_name(app);
            let name = premade.map_or_else(
                || crate::i18n::tr_args!(NEXT_NAME, name => current.as_str()),
                |premade| crate::i18n::tr(premade.name).to_string(),
            );
            add_named(app, &name)
        }
        LayoutMsg::Duplicate if has_current(app) => {
            let current = current_name(app);
            let name = crate::i18n::tr_args!(COPY_NAME, name => current.as_str());
            add_named(app, &name)
        }
        LayoutMsg::Rename if has_current(app) => rename(app, msg),
        LayoutMsg::RenameInput(_) | LayoutMsg::RenameCommit => rename(app, msg),
        LayoutMsg::Delete if has_current(app) => {
            if app.config.saved_layouts_view().names.len() < 2 {
                return Task::none();
            }
            if !app.panels.settings.layouts.armed {
                app.panels.settings.layouts.armed = true;
                return settle(app);
            }
            guard(app, LayoutPending::Delete)
        }
        LayoutMsg::ConfirmSave => {
            let Some(pending) = app.panels.settings.layouts.confirm else {
                return Task::none();
            };
            app.config.save_layout();
            run_pending(app, pending)
        }
        LayoutMsg::ConfirmDiscard => {
            let Some(pending) = app.panels.settings.layouts.confirm else {
                return Task::none();
            };
            run_pending(app, pending)
        }
        LayoutMsg::ConfirmCancel => {
            app.panels.settings.layouts.confirm = None;
            settle(app)
        }
        LayoutMsg::Save
        | LayoutMsg::Revert
        | LayoutMsg::Duplicate
        | LayoutMsg::Rename
        | LayoutMsg::Delete => Task::none(),
    }
}

pub(super) fn cancel_rename(app: &mut App) -> bool {
    if !app.panels.settings.layouts.renaming {
        return false;
    }
    app.panels.settings.layouts.renaming = false;
    app.panels.settings.inputs.remove(LAYOUT_NAME_KEY);
    app.retick();
    true
}

pub(super) fn pick(app: &mut App, path: &str, value: &str) -> Option<Task<Message>> {
    if path != PREMADE_KEY {
        return None;
    }
    app.panels.settings.pop = None;
    let at = value.parse().ok()?;
    Some(guard(app, LayoutPending::Premade(at)))
}

pub(super) fn run(app: &mut App, id: &ActionId) -> Task<Message> {
    match id {
        ActionId::UsePremade(at) => guard(app, LayoutPending::Premade(*at)),
        ActionId::Nudge(spec, dx, dy) => nudge(app, spec, *dx, *dy),
        ActionId::CentreNudge(spec) => {
            for path in [spec.x, spec.y] {
                super::settings_policy::save_value(app, path, &json!(0));
            }
            app.apply_layout();
            app.init_settings_inputs();
            app.invalidate_settings();
            Task::none()
        }
        _ => Task::none(),
    }
}

fn nudge(app: &mut App, spec: &NudgeSpec, dx: i8, dy: i8) -> Task<Message> {
    let step = if app.input.mods.shift { spec.fine } else { spec.coarse };
    let mut tasks = Vec::new();
    for (path, delta) in [(spec.x, dx), (spec.y, dy)] {
        if delta != 0 {
            let change = SettingsMsg::NumberStep(path.to_string(), f64::from(delta) * step);
            tasks.push(super::settings::update(app, change));
        }
    }
    Task::batch(tasks)
}

pub(super) fn sync_corners(app: &mut App, path: &str, value: &serde_json::Value) {
    let Some(keys) = corners::owner(path) else { return };
    let current = app.config.corners(keys);
    if path == keys.same {
        match value.as_bool() {
            Some(true) if !app.config.corners_all_stored(keys) => {
                side_write(app, keys.same, keys.all, json!(current.values[TOP_LEFT]));
            }
            Some(false) if keys.values.iter().all(|key| app.config.stored(key).is_none()) => {
                for key in keys.values {
                    side_write(app, keys.same, key, json!(current.all));
                }
            }
            _ => {}
        }
    } else if path == keys.style
        && current.style == CornerStyle::Straight
        && value.as_str() != Some(corners::STRAIGHT)
    {
        let shown = Corners { style: CornerStyle::Round, ..current }.sizes();
        if shown.iter().all(|size| *size == 0.0) {
            side_write(app, keys.style, keys.all, json!(DEFAULT_CORNER));
            side_write(app, keys.style, keys.same, json!(true));
        }
    }
}

fn side_write(app: &mut App, cause: &'static str, key: &'static str, value: serde_json::Value) {
    let stored = app.config.stored(key);
    let writes = &mut app.panels.settings.side_writes;
    let prior = match writes.iter().position(|write| write.key == key) {
        Some(at) => {
            let earlier = writes.remove(at);
            if earlier.wrote == stored { earlier.prior } else { stored }
        }
        None => stored,
    };
    app.config.set_key(key, value);
    let wrote = app.config.stored(key);
    app.panels.settings.side_writes.push(SideWrite { cause, key, prior, wrote });
}

pub(super) fn put_back(app: &mut App, owned: impl Fn(&SideWrite) -> bool) -> bool {
    let writes = std::mem::take(&mut app.panels.settings.side_writes);
    let (back, kept): (Vec<_>, Vec<_>) = writes.into_iter().partition(|write| owned(write));
    app.panels.settings.side_writes = kept;
    let mut restored = false;
    for write in back.into_iter().rev() {
        if app.config.stored(write.key) != write.wrote {
            continue;
        }
        match write.prior {
            Some(value) => app.config.set_key(write.key, value),
            None => app.config.remove_key(write.key),
        }
        restored = true;
    }
    if restored {
        app.config.persist();
        app.apply_layout();
    }
    restored
}

fn entry_messages(app: &App) -> Vec<LayoutMsg> {
    let view = app.config.saved_layouts_view();
    strip_entries(&view, app.panels.settings.layouts)
        .into_iter()
        .filter_map(|entry| match entry {
            StripEntry::Item { message, .. } => Some(message),
            _ => None,
        })
        .map(|message| message.unwrap_or(LayoutMsg::ConfirmCancel))
        .collect()
}

pub(super) fn strip_stops(app: &App) -> usize {
    if app.config.saved_layouts_view().dirty() { STRIP_STOPS } else { 1 }
}

pub(super) fn strip_arrow(app: &mut App, forwards: bool) -> Task<Message> {
    let stops = strip_stops(app);
    let settings = &mut app.panels.settings;
    settings.focused_control = if forwards {
        (settings.focused_control + 1).min(stops - 1)
    } else {
        settings.focused_control.saturating_sub(1)
    };
    app.retick();
    Task::none()
}

pub(super) fn strip_choice(app: &mut App, forwards: bool) -> Task<Message> {
    let count = entry_messages(app).len();
    if count == 0 || app.panels.settings.pop != Some(strip_key()) {
        return Task::none();
    }
    let settings = &mut app.panels.settings;
    let current = settings.focused_choice.unwrap_or_default();
    settings.focused_choice =
        Some(if forwards { (current + 1).min(count - 1) } else { current.saturating_sub(1) });
    app.retick();
    Task::none()
}

pub(super) fn strip_activate(app: &mut App) -> Task<Message> {
    let settings = &app.panels.settings;
    match settings.focused_control {
        0 if settings.pop == Some(strip_key()) => {
            let Some(message) = settings
                .focused_choice
                .and_then(|choice| entry_messages(app).into_iter().nth(choice))
            else {
                app.panels.settings.focused_choice = Some(0);
                app.retick();
                return Task::none();
            };
            let task = update(app, message);
            if app.panels.settings.pop == Some(strip_key()) {
                app.panels.settings.focused_choice = Some(0);
            }
            task
        }
        0 => {
            let task = update(app, LayoutMsg::Menu);
            app.panels.settings.focused_choice = Some(0);
            task
        }
        1 => update(app, LayoutMsg::Save),
        _ => update(app, LayoutMsg::Revert),
    }
}

pub(super) fn strip_focusable(app: &App) -> bool {
    app.panels.settings.tab == crate::frontend::settings::place::LAYOUT
}

pub(super) fn focus_strip(app: &mut App) {
    app.panels.settings.focus = SettingsFocus::Strip;
    app.panels.settings.focused_control = 0;
    app.panels.settings.focused_choice = None;
}
