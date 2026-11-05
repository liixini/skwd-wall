#[cfg(test)]
mod tests;

use iced::widget::{button, container, text_input};
use iced::{Border, Color, Shadow, Vector};

use crate::frontend::theme::Palette;
use crate::frontend::ui::box_style;

use super::tint::{SELECTED_TINT, tint};

const HOVER_TINT: f32 = 0.62;
const SELECTION_TINT: f32 = 0.35;
const POPOVER_RULE: f32 = 0.58;
const SHADOW_ALPHA: f32 = 0.5;
const SHADOW_DROP: f32 = 10.0;
const SHADOW_BLUR: f32 = 28.0;

pub fn item(ground: Color, palette: &Palette, lit: bool, status: button::Status) -> button::Style {
    let background = if lit {
        Some(tint(ground, palette.primary, SELECTED_TINT))
    } else if matches!(status, button::Status::Hovered) {
        Some(tint(ground, palette.surface_variant, HOVER_TINT))
    } else {
        None
    };
    button::Style {
        background: background.map(Into::into),
        text_color: palette.surface_text,
        ..button::Style::default()
    }
}

pub fn popover(ground: Color, palette: &Palette) -> container::Style {
    let mut style = box_style(ground, tint(ground, palette.outline, POPOVER_RULE));
    style.shadow = Shadow {
        color: Color { a: SHADOW_ALPHA, ..Color::BLACK },
        offset: Vector::new(0.0, SHADOW_DROP),
        blur_radius: SHADOW_BLUR,
    };
    style
}

pub fn editor(plane: Color, palette: &Palette) -> text_input::Style {
    text_input::Style {
        background: plane.into(),
        border: Border { color: palette.primary, width: 0.0, radius: 0.0.into() },
        icon: palette.primary,
        placeholder: palette.primary,
        value: palette.primary,
        selection: tint(plane, palette.primary, SELECTION_TINT),
    }
}

pub fn bare(ink: Color, ring: Color, width: Option<f32>) -> button::Style {
    button::Style {
        text_color: ink,
        border: Border { color: ring, width: width.unwrap_or_default(), ..Border::default() },
        ..button::Style::default()
    }
}

pub fn field(fill: Color, edge: Color, ink: Color, width: f32) -> button::Style {
    button::Style {
        background: Some(fill.into()),
        text_color: ink,
        border: Border { color: edge, width, ..Border::default() },
        ..button::Style::default()
    }
}
