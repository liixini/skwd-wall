use iced::widget::{container, text};
use iced::{Background, Border, Color, Element, Length, Padding};

use crate::contracts::presentation::HudSnapshot;
use crate::frontend::theme::Palette;

use super::super::with_alpha;
use super::style::bg_style;

#[cfg(test)]
pub const MIN_VIEWPORT: (f32, f32) = (1366.0, 768.0);
const FOLIO_SHEET_WIDTH: f32 = 1390.0;
const FOLIO_SHEET_HEIGHT: f32 = 870.0;
const FOLIO_SHEET_MARGIN: f32 = 22.0;
pub(crate) const FOLIO_RULE_THICKNESS: f32 = 1.0;
pub const FOLIO_RULE_ALPHA: f32 = 0.58;

pub fn folio_horizontal_rule<'a, Message: 'a>(color: Color) -> Element<'a, Message> {
    container(text(""))
        .width(Length::Fill)
        .height(Length::Fixed(FOLIO_RULE_THICKNESS))
        .style(move |_| bg_style(Background::Color(color)))
        .into()
}

pub fn folio_rule<'a, Message: 'a>(palette: &Palette) -> Element<'a, Message> {
    folio_horizontal_rule(with_alpha(palette.outline, FOLIO_RULE_ALPHA))
}

pub fn folio_sheet_dims(viewport: (f32, f32), scale: f32) -> (f32, f32) {
    let width = (FOLIO_SHEET_WIDTH * scale).min(viewport.0 - FOLIO_SHEET_MARGIN).max(220.0);
    let height = (FOLIO_SHEET_HEIGHT * scale).min(viewport.1 - FOLIO_SHEET_MARGIN).max(160.0);
    (width, height)
}

pub fn folio_scroll_padding(top: f32, side: f32, scale: f32) -> Padding {
    Padding {
        top: top * scale,
        right: side * scale,
        bottom: (top + 1.0) * scale,
        left: side * scale,
    }
}

pub fn wrap_rows<T>(items: Vec<(f32, T)>, available_width: f32, gap: f32) -> Vec<Vec<T>> {
    let mut rows: Vec<Vec<T>> = Vec::new();
    let mut current: Vec<T> = Vec::new();
    let mut x = 0.0f32;
    for (width, item) in items {
        if !current.is_empty() && x + width > available_width {
            rows.push(std::mem::take(&mut current));
            x = 0.0;
        }
        current.push(item);
        x += width + gap;
    }
    if !current.is_empty() {
        rows.push(current);
    }
    rows
}

pub fn hud<'a, Message: 'a>(
    snapshot: &HudSnapshot,
    pal: &Palette,
    scale: f32,
) -> Element<'a, Message> {
    let surface = pal.surface;
    let primary = pal.primary;
    container(
        text(format!("{}\n\n{}", snapshot.metrics, snapshot.daemon))
            .size(11.0 * scale)
            .color(with_alpha(pal.surface_text, 0.95))
            .font(iced::Font::MONOSPACE),
    )
    .padding(8.0 * scale)
    .style(move |_theme| container::Style {
        background: Some(Background::Color(with_alpha(surface, 0.85))),
        border: Border { color: with_alpha(primary, 0.4), width: 1.0, radius: 0.0.into() },
        ..container::Style::default()
    })
    .into()
}
