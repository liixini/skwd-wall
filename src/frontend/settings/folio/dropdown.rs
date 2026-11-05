#[cfg(test)]
mod tests;

use std::fmt;

use iced::widget::canvas::{self, Frame, Stroke};
use iced::widget::{container, overlay, pick_list, stack};
use iced::{Background, Border, Element, Length, Rectangle, mouse};

use crate::app::Message;
use crate::frontend::components::with_alpha;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{UI_FONT, legible_type_scale};

use super::super::SettingsMsg;
use super::cut_field;

const TEXT_SIZE: f32 = 12.0;
const VERTICAL_PADDING: f32 = 14.0;
const HORIZONTAL_PADDING: f32 = 12.0;
const MENU_HEIGHT: f32 = 224.0;
const PLACEHOLDER_ALPHA: f32 = 0.56;
const FRAME_ALPHA: f32 = 0.96;
const OUTLINE_ALPHA: f32 = 0.48;

struct DropdownFrame {
    palette: Palette,
    fade: f32,
    scale: f32,
    focused: bool,
}

impl canvas::Program<Message> for DropdownFrame {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        frame.fill(
            &cut_field::fill(bounds.size(), self.scale),
            with_alpha(self.palette.background, FRAME_ALPHA * self.fade),
        );
        let emphasized = self.focused || cursor.is_over(bounds);
        frame.stroke(
            &cut_field::outline(bounds.size(), self.scale),
            Stroke::default()
                .with_color(with_alpha(
                    if emphasized { self.palette.primary } else { self.palette.outline },
                    if emphasized { self.fade } else { OUTLINE_ALPHA * self.fade },
                ))
                .with_width(cut_field::BORDER_WIDTH * self.scale),
        );
        vec![frame.into_geometry()]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Choice {
    key: String,
    label: String,
}

impl fmt::Display for Choice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.label, formatter)
    }
}

pub(super) struct DropdownSpec<'a> {
    pub path: String,
    pub options: Vec<(String, String)>,
    pub current: String,
    pub scale: f32,
    pub fade: f32,
    pub palette: &'a Palette,
    pub focused: bool,
}

fn selection(choices: &[Choice], current: &str) -> Option<Choice> {
    choices.iter().find(|choice| choice.key == current).cloned()
}

fn picked(path: &str, choice: Choice) -> Message {
    Message::Settings(SettingsMsg::Pick(path.to_owned(), choice.key))
}

pub(super) fn dropdown(spec: DropdownSpec<'_>) -> Element<'static, Message> {
    let DropdownSpec { path, options, current, scale, fade, palette, focused } = spec;
    let palette = *palette;
    let choices: Vec<_> = options.into_iter().map(|(key, label)| Choice { key, label }).collect();
    let selected = selection(&choices, &current);
    let text = with_alpha(palette.surface_text, palette.surface_text.a * fade);
    let accent = with_alpha(palette.primary, palette.primary.a * fade);
    let frame: Element<'static, Message> =
        iced::widget::canvas(DropdownFrame { palette, fade, scale, focused })
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    let pick = pick_list(choices, selected, move |choice| picked(&path, choice))
        .menu_height(Length::Fixed(MENU_HEIGHT * scale))
        .font(UI_FONT)
        .text_size(TEXT_SIZE * legible_type_scale(scale))
        .padding([VERTICAL_PADDING * scale, HORIZONTAL_PADDING * scale])
        .width(Length::Fill)
        .style(move |_, _status| pick_list::Style {
            text_color: text,
            placeholder_color: with_alpha(text, PLACEHOLDER_ALPHA * fade),
            handle_color: accent,
            background: Background::Color(iced::Color::TRANSPARENT),
            border: Border::default(),
        })
        .menu_style(move |theme| overlay::menu::Style {
            background: Background::Color(palette.background),
            border: Border {
                color: palette.outline,
                width: cut_field::BORDER_WIDTH,
                radius: 0.0.into(),
            },
            text_color: palette.surface_text,
            selected_text_color: palette.primary_text,
            selected_background: Background::Color(palette.primary),
            ..overlay::menu::default(theme)
        });
    container(stack![frame, pick])
        .width(Length::Fixed(cut_field::WIDTH * scale))
        .height(Length::Fixed(cut_field::HEIGHT * scale))
        .clip(true)
        .into()
}
