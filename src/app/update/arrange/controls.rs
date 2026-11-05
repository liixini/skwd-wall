use serde_json::json;
use skwd_config::corners::{CornerStyle, Corners};
use skwd_config::keys::{filter_bar, selector};
use skwd_config::picker_layout::{LayoutProfile, Layouts, Size};

use crate::app::App;
use crate::frontend::arrange::{LAYOUTS, Target};
use crate::frontend::scene::layout::Mode;

use super::{STEP_STYLE, changed, picker_rect, set_frame, spacing_key};
use crate::contracts::settings::DEFAULT_CORNER;

pub(super) const SPACING_STEPS: [f64; 3] = [0.0, 8.0, 20.0];
const LARGE_CORNER: f64 = 28.0;
pub(super) const CORNER_STEPS: [(CornerStyle, f64); 5] = [
    (CornerStyle::Straight, 0.0),
    (CornerStyle::Round, DEFAULT_CORNER),
    (CornerStyle::Round, LARGE_CORNER),
    (CornerStyle::Chamfered, DEFAULT_CORNER),
    (CornerStyle::Chamfered, LARGE_CORNER),
];

pub(super) fn step(steps: &[f64; 3], value: f64) -> usize {
    steps
        .iter()
        .enumerate()
        .min_by(|one, other| (one.1 - value).abs().total_cmp(&(other.1 - value).abs()))
        .map_or(0, |(index, _)| index)
}

pub(super) fn corner_step(corners: &Corners) -> usize {
    let size = corners.shared();
    CORNER_STEPS
        .iter()
        .enumerate()
        .filter(|(_, (kind, _))| *kind == corners.style)
        .min_by(|one, other| (one.1.1 - size).abs().total_cmp(&(other.1.1 - size).abs()))
        .map_or(0, |(index, _)| index)
}

pub(super) fn layout(app: &mut App, forward: bool) {
    let current = app.scene.mode.as_key();
    let index = LAYOUTS.iter().position(|mode| *mode == current).unwrap_or_default();
    let count = LAYOUTS.len();
    let next = if forward { (index + 1) % count } else { (index + count - 1) % count };
    let frame = picker_rect(app);
    set_frame(app, frame);
    app.config.set_key(selector::DISPLAY_MODE, json!(LAYOUTS[next]));
    changed(app, STEP_STYLE);
}

pub(super) fn spacing(app: &mut App) {
    let key = spacing_key(app.scene.mode);
    let next =
        SPACING_STEPS[(step(&SPACING_STEPS, app.config.num_path(key)) + 1) % SPACING_STEPS.len()];
    let frame = picker_rect(app);
    set_frame(app, frame);
    app.config.set_key(key, json!(next));
    if app.scene.mode == Mode::Grid {
        app.config.set_key(selector::GRID_GAP_Y, json!(next));
    }
    changed(app, STEP_STYLE);
}

pub(super) fn corners(app: &mut App) {
    let Some(keys) = app.scene.mode.corners() else { return };
    let current = app.config.corners(keys);
    let at = corner_step(&current);
    let below = current.shared() < CORNER_STEPS[at].1;
    let (style, next) = CORNER_STEPS[if below { at } else { (at + 1) % CORNER_STEPS.len() }];
    let frame = picker_rect(app);
    set_frame(app, frame);
    app.config.set_key(keys.style, json!(style.key()));
    app.config.set_key(keys.same, json!(true));
    app.config.set_key(keys.all, json!(next));
    changed(app, STEP_STYLE);
}

const BAR_PREFIX: &str = "filterBar.";
const BAR_OFFSETS: &[&str] = &[
    selector::GRID_FILTER_BAR_OFFSET_X,
    selector::GRID_FILTER_BAR_OFFSET_Y,
    selector::HEX_FILTER_BAR_OFFSET_X,
    selector::HEX_FILTER_BAR_OFFSET_Y,
    selector::SANDY_FILTER_BAR_OFFSET_X,
    selector::SANDY_FILTER_BAR_OFFSET_Y,
    selector::SLICE_FILTER_BAR_OFFSET_X,
    selector::SLICE_FILTER_BAR_OFFSET_Y,
];
const SEARCH_KEYS: &[&str] = &[
    selector::TAG_CLOUD_POSITION_ENABLED,
    selector::TAG_CLOUD_POSITION_X,
    selector::TAG_CLOUD_POSITION_Y,
    selector::TAG_CLOUD_OFFSET_X,
    selector::TAG_CLOUD_OFFSET_Y,
    selector::TAG_CLOUD_WIDTH,
    selector::TAG_CLOUD_HEIGHT,
    selector::TAG_CLOUD_ROWS,
    selector::GRID_SEARCH_PANEL_OFFSET_X,
    selector::GRID_SEARCH_PANEL_OFFSET_Y,
    selector::HEX_SEARCH_PANEL_OFFSET_X,
    selector::HEX_SEARCH_PANEL_OFFSET_Y,
    selector::SANDY_SEARCH_PANEL_OFFSET_X,
    selector::SANDY_SEARCH_PANEL_OFFSET_Y,
    selector::SLICE_SEARCH_PANEL_OFFSET_X,
    selector::SLICE_SEARCH_PANEL_OFFSET_Y,
];

const ARRANGE_KEYS: &[&str] = &[
    filter_bar::SHOW_ARRANGE,
    filter_bar::ARRANGE_POSITION_ENABLED,
    filter_bar::ARRANGE_POSITION_X,
    filter_bar::ARRANGE_POSITION_Y,
];
const SWATCH_KEYS: &[&str] = &[
    filter_bar::SHOW_SWATCH,
    filter_bar::SWATCH_POSITION_ENABLED,
    filter_bar::SWATCH_POSITION_X,
    filter_bar::SWATCH_POSITION_Y,
];

fn key_target(key: &str) -> Target {
    if ARRANGE_KEYS.contains(&key) {
        Target::Arrange
    } else if SWATCH_KEYS.contains(&key) {
        Target::Swatch
    } else if key.starts_with(BAR_PREFIX) || BAR_OFFSETS.contains(&key) {
        Target::Bar
    } else if SEARCH_KEYS.contains(&key) {
        Target::Search
    } else {
        Target::Picker
    }
}

fn restored(before: &LayoutProfile, target: Target) -> Vec<(String, Option<serde_json::Value>)> {
    let unset = skwd_config::picker_layout::optional_paths()
        .filter(|key| key_target(key) == target && !before.values.contains_key(*key))
        .map(|key| (key.to_owned(), None));
    before
        .values
        .iter()
        .filter(|(key, _)| key_target(key) == target)
        .map(|(key, value)| (key.clone(), value.clone()))
        .chain(unset)
        .collect()
}

pub(super) fn reset(app: &mut App, target: Target) {
    let session = app.panels.arrange.as_ref().unwrap();
    let mut original = super::document(session.history.original());
    let viewport = Size { width: app.scene.viewport.0, height: app.scene.viewport.1 };
    if let Some(output) = app.config.layout_output().map(str::to_owned) {
        let result = (|| {
            let layouts = Layouts::read(&original)?;
            let stored = layouts.outputs.get(&output);
            if let Some(stored) = stored {
                stored.apply(&mut original);
            }
            let before = LayoutProfile::capture(&original, viewport);
            let mut current = app.config.layout_profile(&output, viewport)?;
            for (key, value) in restored(&before, target) {
                current.set(&key, value);
            }
            let component = super::placement::component(target);
            if let Some(position) = stored.and_then(|stored| stored.positions.get(&component)) {
                current.positions.insert(component, *position);
            } else {
                current.positions.remove(&component);
            }
            app.config.set_layout_profile(&output, &current)
        })();
        if let Err(error) = result {
            app.show_toast(crate::i18n::tr_args!(super::SAVE_ERROR, error => error));
            return;
        }
    } else {
        for (key, value) in restored(&LayoutProfile::capture(&original, viewport), target) {
            if let Some(value) = value {
                app.config.set_key(&key, value);
            } else {
                app.config.remove_key(&key);
            }
        }
    }
    changed(app, super::STEP_RESET);
}
