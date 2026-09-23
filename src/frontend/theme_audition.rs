use iced::widget::{Column, Space, button, column, container, mouse_area, scrollable, text};
use iced::{Alignment, Color, Element, Length, Padding};

use crate::app::{Message, ThemeAuditionPreview};
use crate::frontend::theme::Palette;
use crate::frontend::ui::{THEME_BACKENDS, UI_FONT, legible_type_scale, row, with_alpha};
use crate::i18n::{tr, tr_args};

fn fill<'a>(color: Color, width: f32, height: f32) -> Element<'a, Message> {
    container(Space::new())
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .style(move |_| crate::frontend::ui::bg_style(color))
        .into()
}

fn specimen(palette: &Palette, scale: f32) -> Element<'static, Message> {
    let palette = *palette;
    let signal = row![
        fill(palette.primary, 38.0 * scale, 8.0 * scale),
        fill(palette.tertiary, 23.0 * scale, 8.0 * scale),
    ]
    .spacing(4.0 * scale);
    let selected = container(
        row![
            fill(palette.primary_text, 7.0 * scale, 7.0 * scale),
            fill(with_alpha(palette.primary_text, 0.72), 31.0 * scale, 4.0 * scale),
        ]
        .spacing(5.0 * scale)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding([5.0 * scale, 6.0 * scale])
    .style(move |_| crate::frontend::ui::bg_style(palette.primary));
    let rail = container(
        column![
            fill(palette.tertiary, 15.0 * scale, 15.0 * scale),
            selected,
            fill(with_alpha(palette.surface_text, 0.42), 43.0 * scale, 4.0 * scale),
            fill(with_alpha(palette.surface_text, 0.28), 34.0 * scale, 4.0 * scale),
        ]
        .spacing(7.0 * scale),
    )
    .width(Length::Fixed(65.0 * scale))
    .height(Length::Fill)
    .padding(8.0 * scale)
    .style(move |_| crate::frontend::ui::bg_style(palette.surface));
    let header = container(signal)
        .width(Length::Fill)
        .padding([6.0 * scale, 8.0 * scale])
        .style(move |_| crate::frontend::ui::bg_style(palette.surface_container));
    let message = container(
        column![
            fill(with_alpha(palette.surface_text, 0.9), 68.0 * scale, 4.0 * scale),
            fill(with_alpha(palette.surface_text, 0.54), 91.0 * scale, 4.0 * scale),
            fill(with_alpha(palette.surface_text, 0.38), 74.0 * scale, 4.0 * scale),
        ]
        .spacing(5.0 * scale),
    )
    .width(Length::Fill)
    .padding(8.0 * scale)
    .style(move |_| {
        crate::frontend::ui::box_style(palette.surface_variant, with_alpha(palette.outline, 0.58))
    });
    let content = container(column![header, message].spacing(7.0 * scale))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(7.0 * scale)
        .style(move |_| crate::frontend::ui::bg_style(palette.background));
    container(row![rail, content])
        .width(Length::Fill)
        .height(Length::Fixed(83.0 * scale))
        .style(move |_| {
            crate::frontend::ui::box_style(Color::TRANSPARENT, with_alpha(palette.outline, 0.72))
        })
        .into()
}

fn backend_label(backend: &str) -> &str {
    THEME_BACKENDS
        .iter()
        .find(|(candidate, _)| *candidate == backend)
        .map_or(backend, |(_, label_key)| tr(label_key))
}

fn preview_card(
    preview: &ThemeAuditionPreview,
    active: bool,
    width: f32,
    scale: f32,
) -> Element<'_, Message> {
    let palette = &preview.palette;
    let body = column![
        specimen(palette, scale),
        row![
            text(&preview.label)
                .font(UI_FONT)
                .size(11.0 * legible_type_scale(scale))
                .color(if active { palette.primary } else { palette.surface_text }),
            Space::new().width(Length::Fill),
            text(if active { tr("theme-audition-selected") } else { tr("theme-audition-preview") })
                .font(UI_FONT)
                .size(8.0 * legible_type_scale(scale))
                .color(with_alpha(
                    if active { palette.primary } else { palette.surface_text },
                    0.68
                )),
        ]
        .align_y(Alignment::Center),
    ]
    .spacing(8.0 * scale);
    let colors = *palette;
    button(body)
        .width(Length::Fixed(width))
        .padding(8.0 * scale)
        .on_press(Message::ThemeAuditionSelect {
            backend: preview.backend.clone(),
            key: preview.key.clone(),
            value: preview.value.clone(),
        })
        .style(move |_theme, status| {
            let hovered = matches!(status, button::Status::Hovered);
            crate::frontend::ui::flat_button_style(
                colors.surface,
                colors.surface_text,
                if active || hovered { colors.primary } else { colors.outline },
                if active { 2.0 } else { 1.0 },
                status,
            )
        })
        .into()
}

fn backend_picker<'a>(
    backends: &'a [String],
    inspected: &str,
    applied: &str,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    let mut lines = Column::new().spacing(6.0 * scale);
    for group in backends.chunks(6) {
        let mut line = row![].spacing(6.0 * scale);
        for backend in group {
            let is_inspected = backend == inspected;
            let is_applied = backend == applied;
            let label = if is_applied {
                tr_args!("theme-audition-backend-on", backend => backend_label(backend))
            } else {
                backend_label(backend).to_string()
            };
            let colors = *palette;
            let control = button(
                text(label)
                    .font(UI_FONT)
                    .size(9.0 * legible_type_scale(scale))
                    .color(if is_inspected { palette.primary_text } else { palette.surface_text }),
            )
            .padding([5.0 * scale, 9.0 * scale])
            .on_press(Message::ThemeAuditionBackend(backend.clone()))
            .style(move |_theme, status| {
                crate::frontend::ui::flat_button_style(
                    if is_inspected { colors.primary } else { colors.surface_variant },
                    if is_inspected { colors.primary_text } else { colors.surface_text },
                    if is_applied || is_inspected { colors.primary } else { colors.outline },
                    if is_inspected { 2.0 } else { 1.0 },
                    status,
                )
            });
            line = line.push(control);
        }
        lines = lines.push(line);
    }
    column![
        text(tr("theme-audition-backend"))
            .font(UI_FONT)
            .size(9.0 * legible_type_scale(scale))
            .color(with_alpha(palette.surface_text, 0.58)),
        lines,
    ]
    .spacing(6.0 * scale)
    .into()
}

pub fn view<'a>(
    previews: &'a [ThemeAuditionPreview],
    backends: &'a [String],
    inspected_backend: &str,
    applied_backend: &str,
    current_value: &str,
    loading: bool,
    error: Option<&'a str>,
    viewport: (f32, f32),
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    let compact = viewport.0 < 760.0 * scale;
    let columns: usize = if compact { 2 } else { 3 };
    let card_width = if compact { 174.0 * scale } else { 202.0 * scale };
    let rows = previews.len().div_ceil(columns);
    let content_height = rows as f32 * 123.0 * scale + rows.saturating_sub(1) as f32 * 12.0 * scale;
    let mut grid = Column::new().spacing(12.0 * scale);
    for group in previews.chunks(columns) {
        let mut line = row![].spacing(12.0 * scale);
        for preview in group {
            let active = preview.backend == applied_backend && preview.value == current_value;
            line = line.push(preview_card(preview, active, card_width, scale));
        }
        grid = grid.push(line);
    }
    let status: Element<'_, Message> = if loading && previews.is_empty() {
        text(tr("theme-audition-loading"))
            .font(UI_FONT)
            .size(12.0 * legible_type_scale(scale))
            .color(with_alpha(palette.surface_text, 0.66))
            .into()
    } else if let Some(error) = error {
        text(error)
            .font(UI_FONT)
            .size(12.0 * legible_type_scale(scale))
            .color(palette.tertiary)
            .into()
    } else {
        scrollable(grid)
            .height(Length::Fixed(
                content_height.min((viewport.1 - 280.0 * scale).clamp(220.0, 520.0 * scale)),
            ))
            .into()
    };
    let close = button(text("×").font(UI_FONT).size(18.0 * legible_type_scale(scale)))
        .padding([2.0 * scale, 9.0 * scale])
        .on_press(Message::CloseThemeAudition)
        .style(move |_theme, status| {
            crate::frontend::ui::folio_button_style(false, false, palette, status)
        });
    let heading = row![
        column![
            text(tr("theme-audition-title"))
                .font(UI_FONT)
                .size(15.0 * legible_type_scale(scale))
                .color(palette.surface_text),
            text(tr("theme-audition-subtitle"))
                .font(UI_FONT)
                .size(10.0 * legible_type_scale(scale))
                .color(with_alpha(palette.surface_text, 0.62)),
        ]
        .spacing(3.0 * scale),
        Space::new().width(Length::Fill),
        close,
    ]
    .align_y(Alignment::Center);
    let backends = backend_picker(backends, inspected_backend, applied_backend, scale, palette);
    let panel_width = columns as f32 * card_width
        + (columns.saturating_sub(1) as f32 * 12.0 * scale)
        + 36.0 * scale;
    let panel = container(column![heading, backends, status].spacing(14.0 * scale))
        .width(Length::Fixed(panel_width))
        .padding(Padding::from(18.0 * scale))
        .style(move |_| {
            crate::frontend::ui::box_style(
                with_alpha(palette.surface, 0.98),
                with_alpha(palette.primary, 0.48),
            )
        });
    mouse_area(
        container(iced::widget::opaque(panel))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_| crate::frontend::ui::bg_style(with_alpha(Color::BLACK, 0.58))),
    )
    .on_press(Message::CloseThemeAudition)
    .into()
}
