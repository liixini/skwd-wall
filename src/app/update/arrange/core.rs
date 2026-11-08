use iced::{Rectangle, Task};
use serde_json::json;

use super::{board, displays, effects, placement, preview};
use crate::app::state::arrange::{ArrangeSession, Closing};
use crate::app::state::edit_history::{EditHistory, EditSnapshot};
use crate::app::{App, Message};
use crate::contracts::settings::{SHOW_SORT_PREFIX, SHOW_TYPE_PREFIX};
use crate::domain::strand::{Chord, Command, Key as Stroke, command};
use crate::frontend::arrange::{self, Arrange, ArrangeMsg, Target};
use crate::frontend::scene::layout::Mode;
use skwd_config::keys::{filter_bar, selector};

pub(super) const CSV_SEPARATOR: &str = ",";
pub(super) const COLOR_GROUP: &str = "colors";
const TYPE_PREFIX: &str = "type:";
const SORT_PREFIX: &str = "sort:";
pub(super) const COLOR_PREFIX: &str = "color:";
const FRAME_FIELDS: usize = 4;
const STEP_START: &str = "arrange-step-start";
pub(super) const STEP_MOVE: &str = "arrange-step-move";
pub(super) const STEP_SIZE: &str = "arrange-step-size";
pub(super) const STEP_DENSITY: &str = "arrange-step-density";
pub(super) const STEP_BUTTON: &str = "arrange-step-button";
pub(super) const STEP_ORDER: &str = "arrange-step-order";
pub(super) const STEP_STYLE: &str = "arrange-step-style";
pub(super) const STEP_RESET: &str = "arrange-step-reset";
const DEFAULT_FRAME: [f32; 4] = [0.1, 0.2, 0.8, 0.6];
pub(super) const SAVE_ERROR: &str = "arrange-save-error";

pub(in crate::app) fn snapshot(app: &App) -> EditSnapshot {
    EditSnapshot {
        values: app
            .config
            .document()
            .as_object()
            .expect("configuration object")
            .iter()
            .map(|(path, value)| (path.clone(), Some(value.clone())))
            .collect(),
    }
}

pub(super) fn document(snapshot: &EditSnapshot) -> serde_json::Value {
    serde_json::Value::Object(
        snapshot
            .values
            .iter()
            .filter_map(|(path, value)| value.clone().map(|value| (path.clone(), value)))
            .collect(),
    )
}

pub(in crate::app) fn record(app: &mut App, label: &str) {
    if app.panels.arrange.is_none() {
        return;
    }
    super::super::settings_policy::normalize_picker_frame(app);
    let snapshot = snapshot(app);
    if let Some(session) = app.panels.arrange.as_mut()
        && session.history.record(snapshot, label.to_owned())
    {
        session.editor.recorded();
    }
}

pub(super) fn restore(app: &mut App, snapshot: &EditSnapshot) {
    app.config.replace_document(document(snapshot));
    app.sync_picker_preview();
    app.apply_layout();
    app.chrome.bar.cache.clear();
    app.invalidate_settings();
    app.retick();
}

fn begin(app: &mut App) {
    if app.panels.arrange.is_some() {
        return;
    }
    super::super::settings::commit_settings_input_edit(app);
    placement::refresh_output(app);
    let history = EditHistory::new(snapshot(app), STEP_START.to_owned());
    let bar_visible = app.chrome.filter_bar_visible;
    let _ = app.config.begin_transient();
    app.panels.settings.open = false;
    app.chrome.filter_bar_visible = true;
    app.chrome.bar.menu = None;
    app.scene.close_flip();
    let (width, height) = app.scene.viewport;
    let editor = Arrange::new([width, height], board::board(app, None), board::tones(app));
    app.panels.arrange = Some(ArrangeSession {
        editor,
        history,
        display_copy: None,
        outputs: crate::app::state::arrange::displays::OutputRefresh::default(),
        output_reviews: displays::initial_reviews(app),
        disconnected_outputs: Vec::new(),
        preview_config: None,
        conflict: None,
        bar_visible,
        gesture: None,
        shown: false,
        closing: None,
    });
    app.retick();
}

pub(super) fn dispatch(app: &mut App, message: ArrangeMsg) -> Task<Message> {
    if matches!(message, ArrangeMsg::Begin) {
        begin(app);
        return Task::none();
    }
    if app.panels.arrange.as_ref().is_none_or(|session| session.closing.is_some()) {
        return Task::none();
    }
    match message {
        ArrangeMsg::Begin => {}
        ArrangeMsg::Pointer(pointer) => {
            let effects = app.panels.arrange.as_mut().unwrap().editor.update(pointer);
            if effects.is_empty() {
                app.retick();
                return Task::none();
            }
            effects::apply(app, effects);
        }
        ArrangeMsg::Preview => {
            if preview::active(app) {
                preview::leave(app);
            } else {
                preview::enter(app);
            }
            let active = preview::active(app);
            app.panels.arrange.as_mut().unwrap().editor.preview_to(active);
        }
        ArrangeMsg::OpenDisplays
        | ArrangeMsg::CloseDisplays
        | ArrangeMsg::ReviewDisplay(_)
        | ArrangeMsg::FitDisplay
        | ArrangeMsg::AllowDisconnected(_)
        | ArrangeMsg::ApplyDisplay
        | ArrangeMsg::SkipDisplay
        | ArrangeMsg::ForgetDisplay(_) => displays::update(app, message),
        ArrangeMsg::Undo | ArrangeMsg::Redo => {
            effects::settle(app);
            let history = &mut app.panels.arrange.as_mut().unwrap().history;
            let snapshot =
                if message == ArrangeMsg::Undo { history.undo() } else { history.redo() }.cloned();
            if let Some(snapshot) = snapshot {
                restore(app, &snapshot);
                let _ = displays::refresh(app);
            }
        }
        ArrangeMsg::ConfirmDisconnected => {
            displays::confirm_disconnected(app);
            return dispatch(app, ArrangeMsg::Done);
        }
        ArrangeMsg::Cancel => cancel(app),
        ArrangeMsg::Done | ArrangeMsg::ResolveConflict(_) => save(app, &message),
        ArrangeMsg::Escape => return escape(app),
    }
    sync(app);
    app.retick();
    Task::none()
}

fn cancel(app: &mut App) {
    effects::settle(app);
    if preview::active(app) {
        preview::leave(app);
    }
    let session = app.panels.arrange.as_mut().unwrap();
    if !session.editor.reverting() {
        let _ = session.editor.command(Command::Cancel);
    }
    session.closing = Some(Closing::Cancelled);
    session.display_copy = None;
    session.conflict = None;
    session.disconnected_outputs.clear();
    let original = session.history.original().clone();
    restore(app, &original);
}

fn save(app: &mut App, message: &ArrangeMsg) {
    if displays::pending(app) {
        displays::update(app, ArrangeMsg::CloseDisplays);
        return;
    }
    if !matches!(message, ArrangeMsg::ResolveConflict(_)) && !displays::validate_save(app) {
        return;
    }
    effects::settle(app);
    let original = document(app.panels.arrange.as_ref().unwrap().history.original());
    let result = if let ArrangeMsg::ResolveConflict(keep) = message {
        let Some(reviewed) = app.panels.arrange.as_ref().unwrap().conflict.as_ref() else {
            return;
        };
        app.config.resolve_layout(&original, reviewed, *keep)
    } else {
        app.config.finish_layout(&original, true)
    };
    let session = app.panels.arrange.as_mut().unwrap();
    match result {
        Ok(()) => {
            session.conflict = None;
            session.closing = Some(Closing::Saved);
            session.editor.leave();
        }
        Err(crate::infrastructure::config::LayoutSaveError::Conflict(reviewed)) => {
            session.conflict = Some(reviewed);
        }
        Err(crate::infrastructure::config::LayoutSaveError::Failure(error)) => {
            app.show_toast(crate::i18n::tr_args!(SAVE_ERROR, error => error));
        }
    }
}

pub(super) fn sync(app: &mut App) {
    if app.panels.arrange.is_none() {
        return;
    }
    let known = app.panels.arrange.as_ref().map(|session| session.editor.board.shelves.clone());
    let board = board::board(app, known.as_deref());
    let tones = board::tones(app);
    app.panels.arrange.as_mut().unwrap().editor.sync(board, tones);
}

pub(in crate::app) fn tick(app: &mut App, dt: f32) {
    let Some(session) = app.panels.arrange.as_mut() else { return };
    session.editor.tick(dt);
    let shown = session.editor.shown(Target::Picker).map(|rect| Rectangle {
        x: rect.x,
        y: rect.y,
        width: rect.w,
        height: rect.h,
    });
    let finished = session.editor.finished();
    app.scene.show_frame(shown);
    if finished {
        close(app);
    }
}

pub(in crate::app) fn animating(app: &App) -> bool {
    app.panels.arrange.as_ref().is_some_and(|session| session.editor.animating())
}

fn close(app: &mut App) {
    let Some(session) = app.panels.arrange.take() else { return };
    app.chrome.bar.reorder = None;
    app.scene.show_frame(None);
    if session.closing != Some(Closing::Saved) {
        let original = document(session.history.original());
        let _ = app.config.finish_layout(&original, false);
    }
    app.sync_picker_preview();
    app.apply_layout();
    app.invalidate_settings();
    app.chrome.filter_bar_visible = session.bar_visible;
    app.chrome.bar.cache.clear();
    app.panels.settings.open = false;
    app.retick();
}

fn escape(app: &mut App) -> Task<Message> {
    key(
        app,
        &iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
        iced::keyboard::Modifiers::default(),
    )
}

fn chord(key: &iced::keyboard::Key, modifiers: iced::keyboard::Modifiers) -> Option<Chord> {
    use iced::keyboard::{Key, key::Named};
    let key = match key {
        Key::Named(Named::Escape) => Stroke::Escape,
        Key::Named(Named::Tab) => Stroke::Tab,
        Key::Named(Named::ArrowLeft) => Stroke::Left,
        Key::Named(Named::ArrowRight) => Stroke::Right,
        Key::Named(Named::ArrowUp) => Stroke::Up,
        Key::Named(Named::ArrowDown) => Stroke::Down,
        Key::Named(Named::PageUp) => Stroke::PageUp,
        Key::Named(Named::PageDown) => Stroke::PageDown,
        Key::Named(Named::Delete) => Stroke::Delete,
        Key::Named(Named::Backspace) => Stroke::Backspace,
        Key::Character(text) => Stroke::Char(text.chars().next()?),
        _ => return None,
    };
    Some(Chord { key, control: modifiers.control() || modifiers.logo(), shift: modifiers.shift() })
}

pub(in crate::app) fn key(
    app: &mut App,
    key: &iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
) -> Task<Message> {
    let Some(session) = app.panels.arrange.as_mut() else { return Task::none() };
    if session.closing.is_some() {
        return Task::none();
    }
    let named = |name| matches!(key, iced::keyboard::Key::Named(found) if *found == name);
    if preview::active(app) {
        return if named(iced::keyboard::key::Named::Escape) {
            dispatch(app, ArrangeMsg::Preview)
        } else {
            Task::none()
        };
    }
    let Some(session) = app.panels.arrange.as_mut() else { return Task::none() };
    let enter = named(iced::keyboard::key::Named::Enter);
    if enter && modifiers.control() {
        return if session.editor.hits_ready() {
            dispatch(app, ArrangeMsg::Done)
        } else {
            Task::none()
        };
    }
    let selected = session.editor.selected().is_some();
    let Some(found) = chord(key, modifiers).and_then(|chord| command(chord, selected)) else {
        return Task::none();
    };
    if found == Command::Cancel && session.editor.entering() {
        session.closing = Some(Closing::Cancelled);
        close(app);
        return Task::none();
    }
    let effects = session.editor.command(found);
    effects::apply(app, effects);
    sync(app);
    app.retick();
    Task::none()
}

pub(super) fn changed(app: &mut App, label: &str) {
    app.apply_layout();
    record(app, label);
    app.chrome.bar.cache.clear();
    app.invalidate_settings();
}

pub(in crate::app) fn spacing_key(mode: Mode) -> &'static str {
    match mode {
        Mode::Depth => selector::DEPTH_SPACING_PX,
        Mode::Grid => selector::GRID_GAP_X,
        Mode::Collection => selector::COLLECTION_SPACING,
        Mode::Hex => selector::HEX_GAP_X,
        Mode::Sandy => selector::SANDY_SPACING,
        _ => selector::SLICE_SPACING,
    }
}

pub(in crate::app) fn picker_rect(app: &App) -> Rectangle {
    app.scene
        .configured_frame_bounds()
        .unwrap_or_else(|| area_rect(DEFAULT_FRAME, app.scene.viewport))
}

pub(super) fn search_rect(app: &App) -> Rectangle {
    crate::app::view::tag_cloud_target(app, app.scene.viewport.1)
}

pub(in crate::app) fn area_rect(area: [f32; FRAME_FIELDS], viewport: (f32, f32)) -> Rectangle {
    let [x, y, width, height] = area;
    Rectangle {
        x: x * viewport.0,
        y: y * viewport.1,
        width: width * viewport.0,
        height: height * viewport.1,
    }
}

fn freeze_bar(app: &mut App) {
    if app.config.filter_bar_position().is_some() {
        return;
    }
    let viewport = app.scene.viewport;
    if viewport.0 <= 0.0 || viewport.1 <= 0.0 {
        return;
    }
    let rect = crate::app::view::filter_bar_target(app, viewport.0, viewport.1).1;
    app.config.set_key(filter_bar::POSITION_X, json!(rect.x / viewport.0 * arrange::PERCENT));
    app.config.set_key(filter_bar::POSITION_Y, json!(rect.y / viewport.1 * arrange::PERCENT));
    app.config.set_key(filter_bar::POSITION_ENABLED, json!(true));
}

pub(super) fn set_frame(app: &mut App, rect: Rectangle) {
    freeze_bar(app);
    freeze_search(app);
    placement::update_offset(app, Target::Picker, rect);
    let viewport = app.scene.viewport;
    if viewport.0 <= 0.0 || viewport.1 <= 0.0 {
        return;
    }
    if let Err(error) = app.config.set_keys(&[
        (selector::FRAME_X, json!(rect.x / viewport.0 * arrange::PERCENT)),
        (selector::FRAME_Y, json!(rect.y / viewport.1 * arrange::PERCENT)),
        (selector::FRAME_WIDTH, json!(rect.width / viewport.0 * arrange::PERCENT)),
        (selector::FRAME_HEIGHT, json!(rect.height / viewport.1 * arrange::PERCENT)),
        (selector::FRAME_ENABLED, json!(true)),
    ]) {
        app.show_toast(crate::i18n::tr_args!(SAVE_ERROR, error => error));
        return;
    }
    app.scene.set_picker_frame(placement::frame(app));
}

fn freeze_search(app: &mut App) {
    if app.config.tag_cloud_position().is_some() {
        return;
    }
    let rect = search_rect(app);
    apply_rect(app, Target::Search, rect);
}

pub(super) fn apply_rect(app: &mut App, selected: Target, rect: Rectangle) {
    if selected != Target::Picker {
        placement::update_offset(app, selected, rect);
    }
    match selected {
        Target::Picker => set_frame(app, rect),
        Target::Bar | Target::Search | Target::Arrange | Target::Swatch => {
            let viewport = app.scene.viewport;
            if viewport.0 > 0.0 && viewport.1 > 0.0 {
                let (enabled, x, y) = placement::position_keys(selected);
                app.config.set_key(x, json!(rect.x / viewport.0 * arrange::PERCENT));
                app.config.set_key(y, json!(rect.y / viewport.1 * arrange::PERCENT));
                app.config.set_key(enabled, json!(true));
            }
        }
    }
    app.scene.touch();
    app.chrome.bar.cache.clear();
}

pub(in crate::app) fn visibility_key(id: &str) -> Option<String> {
    use crate::frontend::ui::edit_id;
    if let Some(kind) = id.strip_prefix(TYPE_PREFIX) {
        return Some(format!("{SHOW_TYPE_PREFIX}{kind}"));
    }
    if let Some(sort) = id.strip_prefix(SORT_PREFIX) {
        return Some(format!("{SHOW_SORT_PREFIX}{sort}"));
    }
    if id.starts_with(COLOR_PREFIX) {
        return Some(filter_bar::SHOW_COLORS.to_owned());
    }
    let path = match id {
        edit_id::FOLDER => Some(filter_bar::SHOW_FOLDER),
        edit_id::FAVOURITES => Some(filter_bar::SHOW_FAVOURITES),
        edit_id::RANDOM => Some(filter_bar::SHOW_RANDOM),
        edit_id::ORIENT => Some(filter_bar::SHOW_ORIENT),
        edit_id::RESOLUTION => Some(filter_bar::SHOW_RESOLUTION),
        edit_id::THEME => Some(filter_bar::SHOW_THEME),
        edit_id::TAGCLOUD => Some(filter_bar::SHOW_TAG_CLOUD),
        edit_id::DOWNLOAD => Some(filter_bar::SHOW_DOWNLOAD),
        edit_id::PLAYLISTS => Some(filter_bar::SHOW_PLAYLISTS),
        edit_id::COLOUR_STUDIO => Some(filter_bar::SHOW_COLOUR_STUDIO),
        edit_id::WIDGETS => Some(filter_bar::SHOW_WIDGETS),
        edit_id::ARRANGE => Some(filter_bar::SHOW_ARRANGE),
        edit_id::SWATCH => Some(filter_bar::SHOW_SWATCH),
        _ => None,
    };
    path.map(str::to_owned)
}
