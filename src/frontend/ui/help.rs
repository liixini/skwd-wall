use iced::widget::{container, text};
use iced::{Alignment, Element};

use crate::frontend::theme::Palette;
use crate::i18n::tr;

use super::chrome::ChamferPanel;
use super::misc::{UI_FONT, scrim};
use super::row;
use super::with_alpha;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpIntent {
    Close,
    Capture,
}

fn binding_rows(map: &crate::domain::input::InputMap, mouse: bool) -> Vec<(String, String)> {
    crate::contracts::picker::KEY_BINDINGS
        .into_iter()
        .filter_map(|descriptor| {
            let shown: Vec<_> = map
                .triggers(descriptor.action)
                .iter()
                .filter(|trigger| trigger.is_mouse() == mouse)
                .cloned()
                .collect();
            (!shown.is_empty()).then(|| {
                (
                    crate::domain::input::binding_label(&shown),
                    tr(descriptor.title_key).to_lowercase(),
                )
            })
        })
        .collect()
}

pub fn help_rows(map: &crate::domain::input::InputMap) -> Vec<(String, String)> {
    let mut rows = binding_rows(map, false);
    rows.push((tr("help-key-esc").to_owned(), tr("help-close-back-quit").to_owned()));
    rows
}

pub fn help_overlay(
    map: &crate::domain::input::InputMap,
    pal: &Palette,
    scale: f32,
) -> Element<'static, HelpIntent> {
    use iced::Length;
    use iced::widget::{button, column, stack};
    let key_col = |items: Vec<(String, String)>, title: &'static str| {
        let mut col = column![text(title).font(UI_FONT).size(13.0 * scale).color(pal.tertiary)]
            .spacing(7.0 * scale);
        for (k, d) in items {
            col = col.push(
                row![
                    text(k)
                        .font(UI_FONT)
                        .size(12.0 * scale)
                        .color(pal.primary)
                        .width(Length::Fixed(92.0 * scale)),
                    text(d).size(12.0 * scale).color(with_alpha(pal.surface_text, 0.85)),
                ]
                .spacing(10.0 * scale)
                .align_y(Alignment::Center),
            );
        }
        col
    };
    let mut mouse = binding_rows(map, true);
    mouse.push((tr("help-mouse-wheel").to_owned(), tr("help-wheel").to_owned()));
    mouse.push((tr("help-mouse-hover").to_owned(), tr("help-hover").to_owned()));
    let body = row![key_col(help_rows(map), tr("help-keyboard")), key_col(mouse, tr("help-mouse"))]
        .spacing(30.0 * scale);
    let close_palette = *pal;
    let close = button(text("×").font(UI_FONT).size(16.0 * scale))
        .padding([3.0 * scale, 9.0 * scale])
        .on_press(HelpIntent::Close)
        .style(move |_theme, status| {
            crate::frontend::ui::folio_button_style(false, false, &close_palette, status)
        });
    let panel_w = 560.0 * scale;
    let content = container(
        column![
            row![
                text(tr("help-shortcuts"))
                    .font(UI_FONT)
                    .size(15.0 * scale)
                    .color(pal.primary)
                    .width(Length::Fill),
                close,
            ]
            .align_y(Alignment::Center),
            body
        ]
        .spacing(14.0 * scale),
    )
    .width(iced::Length::Fixed(panel_w))
    .padding(iced::Padding::from([16.0 * scale, 20.0 * scale]));
    let panel = stack![content].push_under(
        iced::widget::canvas(ChamferPanel { pal: *pal }).width(Length::Fill).height(Length::Fill),
    );
    let scrim_col = scrim(0.5);
    crate::frontend::ui::inert_backdrop(
        container(iced::widget::mouse_area(panel).on_press(HelpIntent::Capture))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .style(move |_| crate::frontend::ui::bg_style(scrim_col)),
        HelpIntent::Close,
    )
}
