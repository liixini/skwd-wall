use iced::{Rectangle, Size};
use skwd_config::keys::{filter_bar, selector};
use skwd_config::picker_layout::{Alignment, Component, Offset, Position, Reference};

use crate::app::App;
use crate::frontend::arrange::{Anchor, Target};
use crate::infrastructure::config::Config;

use super::{area_rect, picker_rect, search_rect};

pub(super) fn refresh_output(app: &mut App) {
    let output = app
        .runtime_state
        .overlay
        .and_then(crate::shell::picker_output)
        .or_else(|| app.runtime_state.picker_output.clone())
        .or_else(|| app.config.layout_output().map(str::to_owned));
    app.runtime_state.picker_output.clone_from(&output);
    select(app, output);
}

fn monitor(app: &App, connector: &str) -> String {
    app.daemon
        .output_statuses
        .iter()
        .find(|status| status.name == connector && status.is_connected())
        .map_or_else(|| connector.to_owned(), |status| status.monitor().to_owned())
}

pub(in crate::app) fn select(app: &mut App, connector: Option<String>) {
    let key = connector.as_deref().map(|connector| monitor(app, connector));
    let (width, height) = app.scene.viewport;
    app.config.select_layout_monitor(
        key,
        connector,
        skwd_config::picker_layout::Size { width, height },
    );
}

pub(in crate::app) fn reselect(app: &mut App) -> bool {
    let Some(connector) = app.runtime_state.picker_output.clone() else { return false };
    if app.panels.arrange.is_some() || app.config.layout_output() == Some(&monitor(app, &connector))
    {
        return false;
    }
    select(app, Some(connector));
    true
}

pub(in crate::app) fn component(target: Target) -> Component {
    match target {
        Target::Picker => Component::Picker,
        Target::Bar => Component::Bar,
        Target::Search => Component::Search,
        Target::Arrange => Component::Arrange,
        Target::Swatch => Component::Swatch,
    }
}

pub(in crate::app) fn position_keys(target: Target) -> (&'static str, &'static str, &'static str) {
    match target {
        Target::Picker => (selector::FRAME_ENABLED, selector::FRAME_X, selector::FRAME_Y),
        Target::Bar => {
            (filter_bar::POSITION_ENABLED, filter_bar::POSITION_X, filter_bar::POSITION_Y)
        }
        Target::Search => (
            selector::TAG_CLOUD_POSITION_ENABLED,
            selector::TAG_CLOUD_POSITION_X,
            selector::TAG_CLOUD_POSITION_Y,
        ),
        Target::Arrange => (
            filter_bar::ARRANGE_POSITION_ENABLED,
            filter_bar::ARRANGE_POSITION_X,
            filter_bar::ARRANGE_POSITION_Y,
        ),
        Target::Swatch => (
            filter_bar::SWATCH_POSITION_ENABLED,
            filter_bar::SWATCH_POSITION_X,
            filter_bar::SWATCH_POSITION_Y,
        ),
    }
}

pub(in crate::app) fn present(config: &Config, target: Target) -> bool {
    match target {
        Target::Picker | Target::Bar | Target::Search => true,
        Target::Arrange => config.flag_default_true(filter_bar::SHOW_ARRANGE),
        Target::Swatch => crate::app::swatch_on(config),
    }
}

pub(in crate::app) fn floated(
    position: Option<(f32, f32)>,
    corner: (f32, f32),
    size: Size,
    viewport: (f32, f32),
) -> Rectangle {
    let (x, y) = position
        .map_or((corner.0, corner.1 - size.height), |(x, y)| (x * viewport.0, y * viewport.1));
    Rectangle {
        x: x.clamp(0.0, (viewport.0 - size.width).max(0.0)),
        y: y.clamp(0.0, (viewport.1 - size.height).max(0.0)),
        width: size.width,
        height: size.height,
    }
}

fn reference_rect(app: &App, reference: Reference) -> Rectangle {
    match reference {
        Reference::Screen => Rectangle {
            width: app.scene.viewport.0,
            height: app.scene.viewport.1,
            ..Rectangle::default()
        },
        Reference::Picker => picker_rect(app),
    }
}

pub(super) fn aligned(reference: Rectangle, rect: Rectangle, alignment: Alignment) -> Rectangle {
    let [x, y] = alignment.factors();
    Rectangle {
        x: reference.x + (reference.width - rect.width) * x,
        y: reference.y + (reference.height - rect.height) * y,
        ..rect
    }
}

pub(in crate::app) fn resolve(app: &App, target: Target, rect: Rectangle) -> Rectangle {
    let Some(position) = app.config.position(component(target)) else { return rect };
    if target == Target::Picker && position.reference == Reference::Picker {
        return rect;
    }
    let base = aligned(reference_rect(app, position.reference), rect, position.alignment);
    Rectangle {
        x: (base.x + position.offset.x).clamp(0.0, (app.scene.viewport.0 - rect.width).max(0.0)),
        y: (base.y + position.offset.y).clamp(0.0, (app.scene.viewport.1 - rect.height).max(0.0)),
        ..rect
    }
}

pub(in crate::app) fn frame(app: &App) -> Option<[f32; 4]> {
    let frame = app.config.picker_frame()?;
    let (width, height) = app.scene.viewport;
    if width <= 0.0 || height <= 0.0 {
        return Some(frame);
    }
    let rect = resolve(app, Target::Picker, area_rect(frame, (width, height)));
    Some([rect.x / width, rect.y / height, rect.width / width, rect.height / height])
}

pub(in crate::app) fn update_offset(app: &mut App, target: Target, rect: Rectangle) {
    let Some(mut position) = app.config.position(component(target)) else { return };
    let base = aligned(reference_rect(app, position.reference), rect, position.alignment);
    position.offset = Offset { x: rect.x - base.x, y: rect.y - base.y };
    if let Err(error) = app.config.set_position(component(target), Some(position)) {
        app.show_toast(crate::i18n::tr_args!(super::SAVE_ERROR, error => error));
    }
}

pub(in crate::app) fn rect(app: &App, target: Target) -> Rectangle {
    match target {
        Target::Picker => picker_rect(app),
        Target::Bar => {
            crate::app::view::filter_bar_target(app, app.scene.viewport.0, app.scene.viewport.1).1
        }
        Target::Search => search_rect(app),
        Target::Arrange => resolve(
            app,
            target,
            crate::app::view::arrange_bounds(app, &app.config, app.scene.viewport),
        ),
        Target::Swatch => {
            let bounds = crate::app::swatch_bounds(app, &app.config, app.scene.viewport);
            resolve(app, target, bounds)
        }
    }
}

pub(in crate::app) fn drawn(app: &App, target: Target) -> Rectangle {
    let shown = app.panels.arrange.as_ref().and_then(|session| session.editor.shown(target));
    shown.map_or_else(
        || rect(app, target),
        |shown| Rectangle { x: shown.x, y: shown.y, width: shown.w, height: shown.h },
    )
}

pub(super) fn anchor(app: &mut App, target: Target, anchor: Anchor) {
    if app.config.layout_output().is_none() {
        return;
    }
    let (reference, cell) = match anchor {
        Anchor::Screen(cell) => (Reference::Screen, cell),
        Anchor::Picker(cell) => (Reference::Picker, cell),
    };
    let rect = rect(app, target);
    let alignment = ALIGNMENTS[cell[1]][cell[0]];
    let base = aligned(reference_rect(app, reference), rect, alignment);
    let position = Position {
        reference,
        alignment,
        offset: Offset { x: rect.x - base.x, y: rect.y - base.y },
    };
    if let Err(error) = app.config.set_position(component(target), Some(position)) {
        app.show_toast(crate::i18n::tr_args!(super::SAVE_ERROR, error => error));
    }
}

const ALIGNMENTS: [[Alignment; 3]; 3] = [
    [Alignment::TopLeft, Alignment::Top, Alignment::TopRight],
    [Alignment::Left, Alignment::Center, Alignment::Right],
    [Alignment::BottomLeft, Alignment::Bottom, Alignment::BottomRight],
];
