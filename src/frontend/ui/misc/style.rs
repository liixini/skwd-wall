use iced::widget::{button, container};
use iced::{Background, Border, Color};

use crate::frontend::theme::Palette;

use super::super::with_alpha;

pub fn scrim(alpha: f32) -> Color {
    Color { r: 0.0, g: 0.0, b: 0.0, a: alpha }
}

pub fn panel_style(pal: &Palette) -> container::Style {
    panel_style_a(pal, 0.98, 0.4)
}

pub fn panel_style_a(pal: &Palette, bg_a: f32, border_a: f32) -> container::Style {
    container::Style {
        background: Some(with_alpha(pal.surface, bg_a).into()),
        border: Border { radius: 0.0.into(), width: 1.0, color: with_alpha(pal.outline, border_a) },
        ..Default::default()
    }
}

const RAIL_FILL_ALPHA: f32 = 0.3;
const RAIL_OUTLINE_ALPHA: f32 = 0.34;

pub fn folio_rail_style(pal: &Palette) -> container::Style {
    box_style(
        with_alpha(pal.surface_variant, RAIL_FILL_ALPHA),
        with_alpha(pal.outline, RAIL_OUTLINE_ALPHA),
    )
}

pub fn box_style(fill: Color, border: Color) -> container::Style {
    container::Style {
        background: Some(fill.into()),
        border: Border { radius: 0.0.into(), width: 1.0, color: border },
        ..Default::default()
    }
}

pub fn bg_style(bg: impl Into<Background>) -> container::Style {
    container::Style { background: Some(bg.into()), ..Default::default() }
}

pub fn flat_button_style(
    background: Color,
    foreground: Color,
    border: Color,
    border_width: f32,
    status: button::Status,
) -> button::Style {
    let alpha = if matches!(status, button::Status::Hovered) { 0.85 } else { 1.0 };
    button::Style {
        background: Some(with_alpha(background, alpha).into()),
        text_color: foreground,
        border: Border { color: border, width: border_width, radius: 0.0.into() },
        ..Default::default()
    }
}

pub fn folio_button_style(
    active: bool,
    destructive: bool,
    pal: &Palette,
    status: button::Status,
) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered);
    let disabled = matches!(status, button::Status::Disabled);
    let accent = if destructive { pal.error } else { pal.primary };
    button::Style {
        background: if disabled {
            None
        } else if active {
            Some(with_alpha(accent, 0.92).into())
        } else if hovered {
            Some(with_alpha(pal.hover, 0.62).into())
        } else {
            Some(with_alpha(pal.surface_container, 0.92).into())
        },
        text_color: if disabled {
            with_alpha(pal.surface_text, 0.34)
        } else if active {
            if destructive { pal.error_text } else { pal.primary_text }
        } else if hovered {
            pal.hover_text
        } else if destructive {
            pal.error
        } else {
            pal.surface_text
        },
        border: Border {
            color: if disabled {
                with_alpha(pal.outline, 0.18)
            } else if destructive || hovered || active {
                with_alpha(accent, 0.9)
            } else {
                with_alpha(pal.outline, 0.4)
            },
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn folio_line_button_style(
    active: bool,
    focused: bool,
    pal: &Palette,
    fade: f32,
    status: button::Status,
) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered);
    button::Style {
        background: if active {
            Some(with_alpha(pal.primary, 0.18 * fade).into())
        } else if hovered {
            Some(with_alpha(pal.hover, 0.62 * fade).into())
        } else {
            None
        },
        text_color: with_alpha(
            if hovered && !active { pal.hover_text } else { pal.surface_text },
            fade,
        ),
        border: Border {
            color: if focused { with_alpha(pal.primary, fade) } else { Color::TRANSPARENT },
            width: if focused { 2.0 } else { 0.0 },
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn folio_embedded_button_style(
    foreground: Color,
    hover_fill: bool,
    pal: &Palette,
    status: button::Status,
) -> button::Style {
    button::Style {
        background: if hover_fill && matches!(status, button::Status::Hovered) {
            Some(with_alpha(pal.hover, 0.42).into())
        } else {
            None
        },
        text_color: if hover_fill && matches!(status, button::Status::Hovered) {
            pal.hover_text
        } else {
            foreground
        },
        ..Default::default()
    }
}

pub fn scrim_style(alpha: f32) -> container::Style {
    container::Style { background: Some(scrim(alpha).into()), ..Default::default() }
}

pub fn ghost_input_style(pal: &Palette) -> iced::widget::text_input::Style {
    iced::widget::text_input::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border { color: Color::TRANSPARENT, width: 0.0, radius: 0.0.into() },
        icon: pal.tertiary,
        placeholder: with_alpha(pal.surface_text, 0.42),
        value: pal.surface_text,
        selection: with_alpha(pal.primary, 0.35),
    }
}

pub fn field_input_style(pal: &Palette) -> iced::widget::text_input::Style {
    iced::widget::text_input::Style {
        background: with_alpha(pal.surface_container, 0.88).into(),
        border: Border { color: with_alpha(pal.outline, 0.48), width: 1.0, radius: 0.0.into() },
        icon: pal.surface_text,
        placeholder: with_alpha(pal.surface_text, 0.35),
        value: pal.surface_text,
        selection: with_alpha(pal.primary, 0.4),
    }
}

pub fn workbench_input_style(foreground: Color, accent: Color) -> iced::widget::text_input::Style {
    iced::widget::text_input::Style {
        background: with_alpha(Color::BLACK, 0.58).into(),
        border: Border { color: with_alpha(foreground, 0.28), width: 1.0, radius: 0.0.into() },
        icon: foreground,
        placeholder: with_alpha(foreground, 0.48),
        value: foreground,
        selection: with_alpha(accent, 0.45),
    }
}
