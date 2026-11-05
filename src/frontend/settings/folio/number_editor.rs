#[cfg(test)]
mod tests;

use iced::widget::{button, column, container, text, text_input};
use iced::{Alignment, Background, Border, Color, Element, Length};

use crate::app::Message;
use crate::contracts::settings::keys;
use crate::contracts::settings::schema::{self, DefaultValue, NumberBounds, SettingSpec};
use crate::frontend::components::with_alpha;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{UI_FONT, folio_slider, label, legible_type_scale, row};
use crate::i18n::tr;

use super::super::SettingsMsg;

const EDITOR_WIDTH: f32 = super::cut_field::WIDTH;
const CAPTION_HEIGHT: f32 = 24.0;
const CAPTION_WIDTH: f32 = 72.0;
const CAPTION_SIZE: f32 = 13.0;
const UNIT_ALPHA: f32 = 0.72;
const STEP_WIDTH: f32 = 19.0;
const STEP_HEIGHT: f32 = 25.0;
const STEP_SIZE: f32 = 17.0;
const TRACK_GAP: f32 = 7.0;
const FINE_STEP: f64 = 0.01;
const WHOLE_STEP: f64 = 1.0;
const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 3_600;
const MINUTES_KEY: &str = "settings-automation-minutes";
const HOURS_KEY: &str = "settings-automation-hours";
const FINE_SPAN: f64 = 1.0;
const STEP_TOLERANCE: f64 = 1e-9;
const DECIMAL_RADIX: f64 = 10.0;
const MAX_STEP_DECIMALS: usize = 15;
const ZERO_PLACEHOLDER: &str = "0";
const DECREMENT_LABEL: &str = "−";
const INCREMENT_LABEL: &str = "+";

struct ScrubRange {
    path: &'static str,
    min: f64,
    max: f64,
    step: f64,
}

const SCRUB_RANGES: [ScrubRange; 19] = [
    ScrubRange { path: keys::matugen::CONTRAST, min: -1.0, max: 1.0, step: 0.1 },
    ScrubRange { path: keys::selector::WALLHAVEN_COLUMNS, min: 1.0, max: 16.0, step: 1.0 },
    ScrubRange { path: keys::selector::WALLHAVEN_ROWS, min: 1.0, max: 8.0, step: 1.0 },
    ScrubRange { path: keys::selector::WALLHAVEN_THUMB_WIDTH, min: 120.0, max: 960.0, step: 10.0 },
    ScrubRange { path: keys::selector::WALLHAVEN_THUMB_HEIGHT, min: 68.0, max: 540.0, step: 1.0 },
    ScrubRange { path: keys::selector::STEAM_COLUMNS, min: 1.0, max: 16.0, step: 1.0 },
    ScrubRange { path: keys::selector::STEAM_ROWS, min: 1.0, max: 8.0, step: 1.0 },
    ScrubRange { path: keys::selector::STEAM_THUMB_WIDTH, min: 120.0, max: 960.0, step: 10.0 },
    ScrubRange { path: keys::selector::STEAM_THUMB_HEIGHT, min: 68.0, max: 540.0, step: 1.0 },
    ScrubRange { path: keys::sources::YOUTUBE_MAX_HEIGHT, min: 144.0, max: 4320.0, step: 72.0 },
    ScrubRange { path: keys::sources::YOUTUBE_MAX_MINUTES, min: 0.0, max: 600.0, step: 1.0 },
    ScrubRange { path: keys::selector::CORNER_TL, min: 0.0, max: 64.0, step: 1.0 },
    ScrubRange { path: keys::selector::CORNER_TR, min: 0.0, max: 64.0, step: 1.0 },
    ScrubRange { path: keys::selector::CORNER_BR, min: 0.0, max: 64.0, step: 1.0 },
    ScrubRange { path: keys::selector::CORNER_BL, min: 0.0, max: 64.0, step: 1.0 },
    ScrubRange { path: keys::filter_bar::OFFSET_X, min: -2000.0, max: 2000.0, step: 1.0 },
    ScrubRange { path: keys::filter_bar::OFFSET_Y, min: -2000.0, max: 2000.0, step: 1.0 },
    ScrubRange { path: keys::selector::TAG_CLOUD_OFFSET_X, min: -2000.0, max: 2000.0, step: 1.0 },
    ScrubRange { path: keys::selector::TAG_CLOUD_OFFSET_Y, min: -2000.0, max: 2000.0, step: 1.0 },
];

pub(super) struct NumberSpec<'a> {
    pub key: String,
    pub path: String,
    pub unit: &'static str,
    pub raw: &'a str,
    pub scale: f32,
    pub fade: f32,
    pub palette: &'a Palette,
}

fn finite_bounds(spec: &SettingSpec) -> Option<NumberBounds> {
    spec.number_bounds.filter(|bounds| {
        bounds.min.is_finite() && bounds.max.is_finite() && bounds.max > bounds.min
    })
}

fn step_size(spec: &SettingSpec, bounds: NumberBounds) -> f64 {
    let defaults = match spec.default {
        DefaultValue::Number(value) => [value, value],
        DefaultValue::ResponsiveNumber { regular, compact } => [regular, compact],
        _ => [0.0, 0.0],
    };
    let decimals =
        defaults.into_iter().chain([bounds.min, bounds.max]).map(decimal_places).max().unwrap_or(0);
    if decimals == 0 && bounds.max - bounds.min > FINE_SPAN {
        return WHOLE_STEP;
    }
    FINE_STEP.min(DECIMAL_RADIX.powi(-(decimals as i32)))
}

fn decimal_places(value: f64) -> usize {
    let raw = value.to_string();
    raw.split_once('.').map_or(0, |(_, fraction)| fraction.len()).min(MAX_STEP_DECIMALS)
}

fn number_value(raw: &str, automatic: bool) -> Option<f64> {
    if automatic && raw.trim().is_empty() {
        return Some(0.0);
    }
    raw.trim().parse::<f64>().ok().filter(|value| value.is_finite())
}

fn can_step(value: f64, delta: f64, bounds: NumberBounds) -> bool {
    let next = value + delta;
    next >= bounds.min - STEP_TOLERANCE && next <= bounds.max + STEP_TOLERANCE
}

pub(crate) fn number_step_text(value: f64, step: f64, previous: Option<&str>) -> Option<String> {
    if !value.is_finite() || !step.is_finite() {
        return None;
    }
    let raw = previous.unwrap_or_default().trim();
    if raw.contains('e') || raw.contains('E') {
        return Some(value.to_string());
    }
    let previous_decimals = raw.split_once('.').map_or(0, |(_, fraction)| fraction.len());
    let step_decimals = decimal_places(step.abs());
    let decimals = previous_decimals.max(step_decimals).min(MAX_STEP_DECIMALS);
    Some(if decimals == 0 { value.to_string() } else { format!("{value:.decimals$}") })
}

fn slider_input(key: String, value: f64, step: f64, bounds: NumberBounds) -> Message {
    let raw = if value == bounds.min || value == bounds.max {
        value.to_string()
    } else {
        number_step_text(value, step, None).unwrap_or_default()
    };
    Message::Settings(SettingsMsg::Input(key, raw))
}

fn step_button(
    label: &'static str,
    message: Option<Message>,
    scale: f32,
    palette: &Palette,
    fade: f32,
) -> Element<'static, Message> {
    let palette = *palette;
    let button = button(text(label).font(UI_FONT).size(STEP_SIZE * legible_type_scale(scale)))
        .padding(0.0)
        .width(Length::Fixed(STEP_WIDTH * scale))
        .height(Length::Fixed(STEP_HEIGHT * scale))
        .style(move |_theme, status| {
            let mut style = crate::frontend::ui::folio_button_style(false, false, &palette, status);
            style.background = None;
            style.border = Border::default();
            style.text_color.a *= fade;
            style
        });
    match message {
        Some(message) => button.on_press(message).into(),
        None => button.into(),
    }
}

pub(crate) fn scrub_bounds(path: &str) -> (f64, f64, f64) {
    let scrubbed = || {
        SCRUB_RANGES
            .iter()
            .find(|range| range.path == path)
            .map(|range| (NumberBounds { min: range.min, max: range.max }, range.step))
    };
    schema::find(path)
        .and_then(|spec| finite_bounds(spec).map(|bounds| (bounds, step_size(spec, bounds))))
        .or_else(scrubbed)
        .map_or((f64::NEG_INFINITY, f64::INFINITY, WHOLE_STEP), |(bounds, step)| {
            (bounds.min, bounds.max, step)
        })
}

pub(super) fn scrubs_text(path: &str) -> bool {
    path == keys::matugen::CONTRAST
}

pub(crate) fn step_number(path: &str, current: f64, step: f64) -> Option<f64> {
    let (min, max, _) = scrub_bounds(path);
    let outward = (current > max && step > 0.0) || (current < min && step < 0.0);
    (!outward).then(|| (current + step).clamp(min, max))
}

pub(crate) fn duration_caption(raw: &str) -> Option<String> {
    let seconds = raw.trim().parse::<u64>().ok()?;
    let (unit, key) = [(SECONDS_PER_HOUR, HOURS_KEY), (SECONDS_PER_MINUTE, MINUTES_KEY)]
        .into_iter()
        .find(|(unit, _)| seconds > 0 && seconds % unit == 0)?;
    Some(crate::i18n::tr_args!(key, count => seconds / unit))
}

pub(crate) fn number_caption(key: &str, raw: &str, unit: &str) -> String {
    let automatic = key == crate::contracts::settings::keys::transition::FPS;
    if automatic && number_value(raw, true) == Some(0.0) {
        return tr("settings-transitions-rate-auto").to_string();
    }
    let raw = if raw.trim().is_empty() { ZERO_PLACEHOLDER } else { raw.trim() };
    if unit.is_empty() { raw.to_string() } else { format!("{raw} {unit}") }
}

pub(super) fn number_editor(spec: NumberSpec<'_>) -> Element<'_, Message> {
    let NumberSpec { key, path, unit, raw, scale, fade, palette } = spec;
    let automatic = key == crate::contracts::settings::keys::transition::FPS;
    let shown = if automatic && number_value(raw, true) == Some(0.0) { "" } else { raw };
    let placeholder =
        if automatic { tr("settings-transitions-rate-auto") } else { ZERO_PLACEHOLDER };
    let message_key = key.clone();
    let input = text_input(placeholder, shown)
        .id(super::super::workbench_input_id(&key))
        .font(UI_FONT)
        .on_input(move |raw| Message::Settings(SettingsMsg::Input(message_key.clone(), raw)))
        .on_submit(Message::Settings(SettingsMsg::Commit))
        .size(CAPTION_SIZE * legible_type_scale(scale))
        .padding(0.0)
        .width(Length::Fixed(CAPTION_WIDTH * scale))
        .style(move |_theme, status| {
            let mut style = crate::frontend::ui::workbench_input_style(
                with_alpha(palette.surface_text, fade),
                with_alpha(palette.primary, fade),
            );
            style.background = Background::Color(Color::TRANSPARENT);
            style.border.width =
                if matches!(status, text_input::Status::Focused { .. }) { 1.0 } else { 0.0 };
            style
        });
    let caption = row![
        input,
        label(
            unit,
            crate::frontend::ui::TYPE_SMALL,
            scale,
            with_alpha(palette.surface_text, UNIT_ALPHA * fade)
        ),
    ]
    .align_y(Alignment::Center);
    let mut editor = column![
        container(caption).height(Length::Fixed(CAPTION_HEIGHT * scale)).align_y(Alignment::Center)
    ]
    .align_x(Alignment::Center)
    .width(Length::Fill);
    let mut ink = *palette;
    ink.primary = with_alpha(ink.primary, ink.primary.a * fade);
    ink.outline = with_alpha(ink.outline, ink.outline.a * fade);
    ink.surface = with_alpha(ink.surface, ink.surface.a * fade);
    ink.surface_text = with_alpha(ink.surface_text, ink.surface_text.a * fade);
    ink.hover_text = with_alpha(ink.hover_text, ink.hover_text.a * fade);
    if let (Some(spec), Some(value)) = (schema::find(&path), number_value(raw, automatic))
        && let Some(bounds) = finite_bounds(spec)
    {
        let step = step_size(spec, bounds);
        let minus = can_step(value, -step, bounds)
            .then(|| Message::Settings(SettingsMsg::NumberStep(key.clone(), -step)));
        let plus = can_step(value, step, bounds)
            .then(|| Message::Settings(SettingsMsg::NumberStep(key.clone(), step)));
        let input_key = key.clone();
        let slider = folio_slider(
            bounds.min,
            bounds.max,
            value,
            step,
            move |value| slider_input(input_key.clone(), value, step, bounds),
            Message::Settings(SettingsMsg::Commit),
            &ink,
        );
        editor = editor.push(
            row![
                step_button(DECREMENT_LABEL, minus, scale, palette, fade),
                slider,
                step_button(INCREMENT_LABEL, plus, scale, palette, fade),
            ]
            .spacing(TRACK_GAP * scale)
            .align_y(Alignment::Center),
        );
    }
    editor.width(Length::Fixed(EDITOR_WIDTH * scale)).into()
}
