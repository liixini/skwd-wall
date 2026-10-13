use iced::widget::{TextInput, container, text_input};
use iced::{Color, Element, Length};

use crate::app::Message;
use crate::frontend::animation::MotionProfile;
use crate::frontend::components::with_alpha;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{TYPE_SMALL, label, legible_type_scale};

pub(super) const SECTION_GAP: f32 = 14.0;
pub(super) const NAME_SIZE: f32 = 14.0;
pub(super) const ROW_SPACING: f32 = 6.0;
const NOTE_ALPHA: f32 = 0.62;
const NOTE_LINE_HEIGHT: f32 = 1.4;
pub(super) const TAG_SIZE: f32 = 10.0;
const TAG_PADDING: [f32; 2] = [3.0, 7.0];
const TAG_ALPHA: f32 = 0.7;
const INPUT_SIZE: f32 = 13.0;
const INPUT_PADDING: [f32; 2] = [8.0, 10.0];

#[derive(Clone, Copy)]
pub(super) struct Paint<'a> {
    pub(super) scale: f32,
    pub(super) palette: &'a Palette,
    pub(super) fade: f32,
    pub(super) motion: MotionProfile,
}

pub(super) fn tag<'a>(
    content: impl iced::widget::text::IntoFragment<'a>,
    color: Color,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let Paint { scale, fade, .. } = paint;
    let color = with_alpha(color, TAG_ALPHA * fade);
    container(label(content, TAG_SIZE, scale, color))
        .padding([TAG_PADDING[0] * scale, TAG_PADDING[1] * scale])
        .style(move |_| crate::frontend::ui::box_style(Color::TRANSPARENT, color))
        .into()
}

pub(super) fn input<'a>(
    placeholder: &str,
    value: &str,
    paint: Paint<'a>,
) -> TextInput<'a, Message> {
    let Paint { scale, palette, fade, .. } = paint;
    text_input(placeholder, value)
        .font(crate::frontend::ui::UI_FONT)
        .size(INPUT_SIZE * legible_type_scale(scale))
        .padding([INPUT_PADDING[0] * scale, INPUT_PADDING[1] * scale])
        .width(Length::Fill)
        .style(move |_theme, _status| {
            crate::frontend::ui::workbench_input_style(
                with_alpha(palette.surface_text, fade),
                with_alpha(palette.primary, fade),
            )
        })
}

pub(super) fn action<'a>(
    text: &str,
    message: Option<Message>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let Paint { scale, palette, fade, motion } = paint;
    super::control::fixed_button(
        text.to_owned(),
        false,
        false,
        message,
        false,
        scale,
        palette,
        fade,
        super::control::button_width(text, scale),
        motion,
    )
}

pub(super) fn rule(paint: Paint<'_>) -> Element<'_, Message> {
    crate::frontend::ui::folio_horizontal_rule(with_alpha(
        paint.palette.outline,
        crate::frontend::ui::FOLIO_RULE_ALPHA * paint.fade,
    ))
}

pub(super) fn note<'a>(
    content: impl iced::widget::text::IntoFragment<'a>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    label(content, TYPE_SMALL, paint.scale, muted(paint))
        .line_height(iced::widget::text::LineHeight::Relative(NOTE_LINE_HEIGHT))
        .into()
}

pub(super) fn text_color(paint: Paint<'_>) -> Color {
    with_alpha(paint.palette.surface_text, paint.fade)
}

pub(super) fn muted(paint: Paint<'_>) -> Color {
    with_alpha(paint.palette.surface_text, NOTE_ALPHA * paint.fade)
}
