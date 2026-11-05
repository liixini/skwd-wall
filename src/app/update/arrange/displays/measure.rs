const PICKER_AREA_UNAVAILABLE: &str = "picker area unavailable";
const INVALID_DISPLAY_PLACEMENT: &str = "invalid display placement";
use iced::Rectangle;
use serde_json::json;
use skwd_config::keys::selector;
use skwd_config::picker_layout::{Component, LayoutProfile, Reference};

use super::super::placement::{position_keys, present};
use crate::app::App;
use crate::app::view::layout::LayoutContext;
use crate::frontend::arrange::{DisplayGeometry, PERCENT, Target};

pub(super) fn measure(
    app: &App,
    output: &str,
    profile: &LayoutProfile,
) -> Result<(DisplayGeometry, bool), String> {
    let config = app.config.project_layout(output, profile)?;
    let viewport = (profile.viewport.width, profile.viewport.height);
    let context = LayoutContext::projected(&config, viewport);
    let screen = Rectangle { width: viewport.0, height: viewport.1, ..Rectangle::default() };
    let picker_bounds = config.picker_frame().map(|frame| super::super::area_rect(frame, viewport));
    let bar_bounds = crate::app::view::bar_bounds(app, &context);
    let search_bounds = crate::app::view::tag_cloud_bounds(app, &context);
    let arrange_bounds = present(&config, Target::Arrange)
        .then(|| crate::app::view::arrange_bounds(app, &config, viewport));
    let swatch_bounds =
        present(&config, Target::Swatch).then(|| crate::app::swatch_bounds(app, &config, viewport));
    let mut compatible = [
        (Target::Picker, picker_bounds),
        (Target::Bar, Some(bar_bounds)),
        (Target::Search, Some(search_bounds)),
        (Target::Arrange, arrange_bounds),
        (Target::Swatch, swatch_bounds),
    ]
    .into_iter()
    .all(|(target, rect)| {
        rect.is_none_or(|rect| fits(requested(&config, target, rect, viewport), viewport))
    });
    let mut resolve = |component, rect, picker: Option<Rectangle>| {
        let requested = if let Some(position) = profile.positions.get(&component) {
            let reference = match position.reference {
                Reference::Screen => Some(screen),
                Reference::Picker => picker,
            };
            if let Some(reference) = reference {
                let base = super::super::placement::aligned(reference, rect, position.alignment);
                Rectangle { x: base.x + position.offset.x, y: base.y + position.offset.y, ..rect }
            } else {
                compatible = false;
                rect
            }
        } else {
            rect
        };
        compatible &= fits(requested, viewport);
        fit(requested, viewport, false)
    };
    let picker = picker_bounds.map(|rect| resolve(Component::Picker, rect, None));
    let bar = resolve(Component::Bar, bar_bounds, picker);
    let search = resolve(Component::Search, search_bounds, picker);
    let arrange = arrange_bounds.map(|rect| resolve(Component::Arrange, rect, picker));
    let swatch = swatch_bounds.map(|rect| resolve(Component::Swatch, rect, picker));
    Ok((DisplayGeometry { viewport, picker, bar, search, arrange, swatch }, compatible))
}

fn requested(
    config: &crate::infrastructure::config::Config,
    target: Target,
    rect: Rectangle,
    viewport: (f32, f32),
) -> Rectangle {
    let number = |path, fallback| {
        skwd_config::get(config.root(), path)
            .and_then(|value| value.as_f64().or_else(|| value.as_str()?.parse::<f64>().ok()))
            .map_or(fallback, |value| value as f32)
    };
    let (enabled, x, y) = position_keys(target);
    let mut requested = rect;
    if config.flag_default_config(enabled)
        && config.position(super::super::placement::component(target)).is_none()
    {
        requested.x = number(x, rect.x / viewport.0 * PERCENT) / PERCENT * viewport.0;
        requested.y = number(y, rect.y / viewport.1 * PERCENT) / PERCENT * viewport.1;
    }
    if target == Target::Picker {
        requested.width =
            number(selector::FRAME_WIDTH, rect.width / viewport.0 * PERCENT) / PERCENT * viewport.0;
        requested.height = number(selector::FRAME_HEIGHT, rect.height / viewport.1 * PERCENT)
            / PERCENT
            * viewport.1;
    }
    requested
}

fn fits(rect: Rectangle, viewport: (f32, f32)) -> bool {
    rect.x >= 0.0
        && rect.y >= 0.0
        && rect.x + rect.width <= viewport.0
        && rect.y + rect.height <= viewport.1
        && rect.width > 0.0
        && rect.height > 0.0
}

pub(super) fn fit(rect: Rectangle, viewport: (f32, f32), resize: bool) -> Rectangle {
    let width = if resize { rect.width.min(viewport.0) } else { rect.width };
    let height = if resize { rect.height.min(viewport.1) } else { rect.height };
    Rectangle {
        x: rect.x.clamp(0.0, (viewport.0 - width).max(0.0)),
        y: rect.y.clamp(0.0, (viewport.1 - height).max(0.0)),
        width,
        height,
    }
}

pub(super) fn edit(
    profile: &mut LayoutProfile,
    geometry: &DisplayGeometry,
    target: Target,
    rect: Rectangle,
) -> Result<(), String> {
    if ![rect.x, rect.y, rect.width, rect.height].into_iter().all(f32::is_finite)
        || rect.width <= 0.0
        || rect.height <= 0.0
    {
        return Err(INVALID_DISPLAY_PLACEMENT.to_owned());
    }
    let component = super::super::placement::component(target);
    if let Some(position) = profile.positions.get_mut(&component) {
        let reference = match position.reference {
            Reference::Screen => Rectangle {
                width: geometry.viewport.0,
                height: geometry.viewport.1,
                ..Rectangle::default()
            },
            Reference::Picker => {
                geometry.picker.ok_or_else(|| PICKER_AREA_UNAVAILABLE.to_owned())?
            }
        };
        let base = super::super::placement::aligned(reference, rect, position.alignment);
        position.offset.x = rect.x - base.x;
        position.offset.y = rect.y - base.y;
    }
    let (enabled, x, y) = position_keys(target);
    profile.set(enabled, Some(json!(true)));
    profile.set(x, Some(json!(rect.x / geometry.viewport.0 * PERCENT)));
    profile.set(y, Some(json!(rect.y / geometry.viewport.1 * PERCENT)));
    if target == Target::Picker {
        profile.set(selector::FRAME_WIDTH, Some(json!(rect.width / geometry.viewport.0 * PERCENT)));
        profile
            .set(selector::FRAME_HEIGHT, Some(json!(rect.height / geometry.viewport.1 * PERCENT)));
    }
    Ok(())
}
