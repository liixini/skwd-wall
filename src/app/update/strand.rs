use std::sync::Arc;
use std::time::Instant;

use iced::Task;
use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use paper_control::{MAX_WIDGETS, Widget};
use paper_widgets::Lettering;
use serde_json::{Value, json};
use skwd_config::keys::{display, widgets};
use wall_proto::apply_fields::ALL_OUTPUTS;
use wall_proto::widgets::{Desk, Editing};

use crate::app::state::StrandEntries;
use crate::app::{App, Message, Pending};
use crate::domain::strand::{Chord, Command, Key as Stroke, Saved, command, merge};
use crate::frontend::strand::{Effect, Scenery, Stage, Strand, StrandMsg};
use crate::infrastructure::desktop::{self, Desktop, Request};
use crate::infrastructure::runtime::Wake;

const FAILED: &str = "strand-failed";
const DESKTOP_THREAD: &str = "strand-desktop";
const SHARED_DESKTOP: &str = "desktop backdrop is still shared";
const UNUSABLE_FONT: &str = "widget font is not usable";

pub(in crate::app) fn open(app: &mut App) -> Task<Message> {
    if let Some(strand) = app.panels.strand.as_mut() {
        strand.arranging();
        app.retick();
        return Task::none();
    }
    let (width, height) = app.scene.viewport;
    if width <= 0.0 || height <= 0.0 {
        return Task::none();
    }
    let name = app.runtime_state.picker_output.clone();
    let logical = name
        .as_deref()
        .and_then(|name| status(app, name))
        .map(|status| (status.width, status.logical_size()))
        .filter(|(_, (wide, high))| *wide > 0 && *high > 0);
    let output = logical.map_or([width, height], |(_, (wide, high))| [wide as f32, high as f32]);
    let scale = logical
        .map(|(physical, (wide, _))| physical as f32 / wide as f32)
        .filter(|scale| scale.is_finite() && *scale > 0.0)
        .unwrap_or_else(crate::frontend::components::surface_scale);
    let raw = skwd_config::arr_ref(app.config.root(), widgets::ITEMS).to_vec();
    app.panels.strand_entries = entries(raw, name.as_deref());
    let mut strand = Strand::new(output, scale, name);
    strand.room = app.panels.strand_entries.room;
    app.panels.strand = Some(strand);
    app.call_tracked(wall_proto::rpc::WIDGETS_SCENE, json!({}), Pending::WidgetScene);
    app.retick();
    Task::none()
}

fn status<'a>(app: &'a App, name: &str) -> Option<&'a crate::contracts::daemon::OutputStatus> {
    app.daemon.output_statuses.iter().find(|status| status.name == name)
}

pub(in crate::app) fn entries(raw: Vec<Value>, name: Option<&str>) -> StrandEntries {
    let mut slots = Vec::new();
    let mut shown = Vec::new();
    let valid: Vec<(usize, Widget)> = raw
        .iter()
        .enumerate()
        .filter_map(|(position, entry)| {
            let widget = serde_json::from_value::<Widget>(entry.clone()).ok()?;
            widget.validate().is_ok().then_some((position, widget))
        })
        .collect();
    let total = valid.len();
    for (position, widget) in valid.into_iter().take(MAX_WIDGETS) {
        if name.map_or(widget.output.is_none(), |name| widget.shown_on(name)) {
            slots.push(position);
            shown.push(widget);
        }
    }
    let room = MAX_WIDGETS.saturating_sub(total - shown.len());
    StrandEntries { raw, slots, widgets: shown, room }
}

pub(in crate::app) fn scene(app: &mut App, desk: Desk) {
    let Some(strand) = app.panels.strand.as_ref().filter(|strand| !strand.ready()) else {
        return;
    };
    let size = strand.output.map(|side| (side * strand.scale).round() as u32);
    let named = |wanted: &str| desk.outputs.iter().find(|output| output.name == wanted);
    let output = strand.name.as_deref().and_then(named).or_else(|| named(ALL_OUTPUTS));
    let fill = output
        .map_or_else(|| app.config.str_path(display::FILL_MODE), |output| output.fill.clone());
    let request = Request {
        wallpaper: output.map(|output| std::path::PathBuf::from(&output.path)),
        fill: desktop::fill_mode(&fill),
        background: output.map(|output| output.background).unwrap_or_default(),
        size,
        outlines: desk.scene.outlines.clone(),
        font: desk.scene.font.clone(),
    };
    if let Some(strand) = app.panels.strand.as_mut() {
        strand.scene = Some(desk.scene);
    }
    let sender = app.runtime_state.wake_tx.clone();
    let spawned = std::thread::Builder::new().name(DESKTOP_THREAD.to_owned()).spawn(move || {
        let loaded = desktop::load(&request);
        let _ = sender.unbounded_send(Wake::Desktop(Arc::new(loaded)));
    });
    if let Err(error) = spawned {
        failed(app, &error.to_string());
    }
}

pub(in crate::app) fn failed(app: &mut App, error: &str) {
    if app.panels.strand.is_some() {
        close(app);
        app.show_toast(crate::i18n::tr_args!(FAILED, error => error));
    }
}

pub(in crate::app) fn dismiss(app: &mut App) {
    if app.panels.strand.is_some() {
        close(app);
    }
}

pub(in crate::app) fn resume(app: &App) {
    if app.panels.strand.as_ref().is_some_and(Strand::ready) {
        edit(app, true);
    }
}

fn close(app: &mut App) {
    if app.panels.strand.take().is_some() {
        edit(app, false);
    }
    app.panels.strand_entries = StrandEntries::default();
    app.retick();
}

fn edit(app: &App, active: bool) {
    app.daemon.client.call(wall_proto::rpc::WIDGETS_EDIT, json!(Editing { active }));
}

pub(in crate::app) fn delivered(app: &mut App, desktop: Arc<Desktop>) {
    match Arc::try_unwrap(desktop) {
        Ok(desktop) => loaded(app, desktop),
        Err(_) => failed(app, SHARED_DESKTOP),
    }
}

fn loaded(app: &mut App, desktop: Desktop) {
    let Some(strand) = app.panels.strand.as_mut() else { return };
    let Some(scene) = strand.scene.take() else { return };
    let Some(lettering) = Lettering::open(desktop.font) else {
        failed(app, UNUSABLE_FONT);
        return;
    };
    let (moment, wait) = desktop::moment();
    let scenery = Scenery { palette: scene.palette, labels: scene.labels, lettering, moment };
    let stage =
        desktop.backdrop.map(|backdrop| Stage::new(backdrop.size, backdrop.rgba, backdrop.matte));
    let shown = std::mem::take(&mut app.panels.strand_entries.widgets);
    strand.adopt(shown, scenery, stage);
    strand.minute = Some(Instant::now() + wait);
    edit(app, true);
    app.retick();
}

pub(in crate::app) fn tick(app: &mut App, now: Instant, dt: f32) {
    let Some(strand) = app.panels.strand.as_mut() else { return };
    if strand.minute.is_some_and(|due| due <= now) {
        let (moment, wait) = desktop::moment();
        strand.set_moment(moment);
        strand.minute = Some(now + wait);
    }
    strand.tick(dt);
    if strand.finished() {
        close(app);
    }
}

pub(in crate::app) fn update(app: &mut App, message: StrandMsg) -> Task<Message> {
    let effect = app.panels.strand.as_mut().map_or(Effect::None, |strand| strand.update(message));
    finish(app, effect);
    Task::none()
}

fn finish(app: &mut App, effect: Effect) {
    if let Effect::Save(items) = effect {
        save(app, items);
    }
    app.retick();
}

pub(in crate::app) fn save(app: &mut App, items: Vec<Saved>) {
    let entries = &app.panels.strand_entries;
    let merged = merge(entries.raw.clone(), &entries.slots, items, |widget| json!(widget));
    super::settings_policy::save_value(app, widgets::ITEMS, &Value::Array(merged));
}

fn chord(key: &Key, modifiers: Modifiers) -> Option<Chord> {
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

pub(in crate::app) fn key(app: &mut App, key: &Key, modifiers: Modifiers) -> Task<Message> {
    let Some(strand) = app.panels.strand.as_mut() else { return Task::none() };
    let selected = strand.selection.id.is_some();
    let Some(found) = chord(key, modifiers).and_then(|chord| command(chord, selected)) else {
        return Task::none();
    };
    if found == Command::Cancel && strand.entering() {
        close(app);
        return Task::none();
    }
    let effect = strand.command(found);
    finish(app, effect);
    Task::none()
}

pub(in crate::app) fn escape(app: &mut App) -> Task<Message> {
    key(app, &Key::Named(Named::Escape), Modifiers::empty())
}
