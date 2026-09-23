use iced::widget::{container, stack, text};
use iced::{Background, Color, Element, Length};

use crate::app::Message;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{row, with_alpha};

use super::super::PreviewRenderer;
use super::super::state::Effects;

const INDEX_WIDTH: f32 = 292.0;

impl Effects {
    pub(super) fn studio_view<'a>(
        &'a self,
        viewport: (f32, f32),
        scale: f32,
        palette: &'a Palette,
        preview_renderer: &impl PreviewRenderer<Message, Output = Element<'static, Message>>,
    ) -> Element<'a, Message> {
        let ease = self.open_ease();
        let (panel_width, panel_height) = crate::frontend::ui::folio_sheet_dims(viewport, scale);
        let index = self.effect_index(scale, palette);
        let editor = self.editor_overlay(scale, palette);

        let preview = self.immersive_preview(palette, preview_renderer);
        let tonal_veil = container(text("")).width(Length::Fill).height(Length::Fill).style(|_| {
            crate::frontend::ui::bg_style(Background::Gradient(
                iced::gradient::Linear::new(std::f32::consts::PI)
                    .add_stop(0.0, with_alpha(Color::BLACK, 0.38))
                    .add_stop(0.34, with_alpha(Color::BLACK, 0.08))
                    .add_stop(0.66, with_alpha(Color::BLACK, 0.12))
                    .add_stop(1.0, with_alpha(Color::BLACK, 0.72))
                    .into(),
            ))
        });
        let wash = container(text("")).width(Length::Fill).height(Length::Fill).style(move |_| {
            crate::frontend::ui::bg_style(Background::Color(with_alpha(palette.background, 0.16)))
        });

        let body = row![
            container(index).width(Length::Fixed(INDEX_WIDTH * scale.max(0.9))),
            container(editor).width(Length::Fill),
        ]
        .spacing(0.0)
        .height(Length::Fill);
        let panel = container(stack![preview, wash, tonal_veil, body])
            .width(Length::Fixed(panel_width))
            .height(Length::Fixed(panel_height))
            .clip(true)
            .style(move |_| {
                let mut style = crate::frontend::ui::box_style(
                    with_alpha(palette.surface, 0.99),
                    with_alpha(palette.outline, 0.66),
                );
                style.shadow = iced::Shadow {
                    color: with_alpha(Color::BLACK, 0.58 * ease),
                    offset: iced::Vector::new(0.0, 18.0),
                    blur_radius: 54.0,
                };
                style
            });
        let backdrop = crate::frontend::ui::inert_backdrop(
            container(text(""))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_| crate::frontend::ui::folio_scrim_style(ease)),
            Message::ToggleEffects,
        );

        stack![
            backdrop,
            container(iced::widget::opaque(panel))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        ]
        .into()
    }
}
