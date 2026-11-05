use iced::widget::{button, canvas, column, container, text};
use iced::{Color, Element, Length};

use crate::app::Message;
use crate::domain::theme::{hex_to_hsv, hsv_to_hex};
use crate::frontend::theme::Palette;
use crate::frontend::theme_designer::ThemeMsg;
use crate::frontend::ui::{label, row};

use super::SettingsMsg;

const PICKER_PREFIX: &str = "colour-picker:";
pub(crate) const BLACK: &str = "#000000";
const NEUTRAL: f32 = 0.5;
const OPEN: f32 = 0.5;
const WHEEL_SIDE: f32 = 176.0;
const WHEEL_HEIGHT: f32 = 184.0;
pub(crate) const SWATCH_SIDE: f32 = 20.0;
const SWATCH_GAP: f32 = 8.0;
const BUTTON_SIZE: f32 = 10.0;

pub fn normalized(text: &str) -> Option<String> {
    crate::contracts::settings::parse_rgb(text).map(|[r, g, b]| format!("#{r:02x}{g:02x}{b:02x}"))
}

pub fn picker_id(path: &str) -> String {
    format!("{PICKER_PREFIX}{path}")
}

pub fn swatch<'a>(hex: &str, side: f32, palette: &'a Palette) -> Element<'a, Message> {
    let fill = crate::frontend::theme::parse_hex(hex).unwrap_or(Color::BLACK);
    container(text(""))
        .width(Length::Fixed(side))
        .height(Length::Fixed(side))
        .style(move |_| crate::frontend::ui::box_style(fill, palette.outline))
        .into()
}

pub fn wheel<'a>(
    hex: &str,
    scale: f32,
    change: impl Fn(String) -> Message + 'a,
    commit: Message,
) -> Element<'a, Message> {
    let hsv = hex_to_hsv(hex).unwrap_or_default();
    let wheel: Element<'a, Message> =
        canvas(crate::frontend::theme_designer::wheel::ColorWheel { hsv })
            .width(Length::Fixed(WHEEL_SIDE * scale))
            .height(Length::Fixed(WHEEL_SIDE * scale))
            .into();
    wheel.map(move |message| match message {
        Message::Theme(ThemeMsg::Hue(hue)) => change(hsv_to_hex(
            hue,
            if hsv.1 == 0.0 { NEUTRAL } else { hsv.1 },
            if hsv.2 == 0.0 { NEUTRAL } else { hsv.2 },
        )),
        Message::Theme(ThemeMsg::SV(saturation, value)) => {
            change(hsv_to_hex(hsv.0, saturation, value))
        }
        Message::Theme(ThemeMsg::DragEnd) => commit.clone(),
        message => message,
    })
}

pub fn reveal<'a>(
    content: impl Into<Element<'a, Message>>,
    picker: f32,
    scale: f32,
) -> Element<'a, Message> {
    container(content)
        .height(Length::Fixed(
            WHEEL_HEIGHT * scale * crate::frontend::animation::smoothstep(picker),
        ))
        .clip(true)
        .into()
}

pub fn view<'a>(
    path: &str,
    hex: &str,
    picker: f32,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    let open = picker > OPEN;
    let toggle = button(
        row![
            swatch(hex, SWATCH_SIDE * scale, palette),
            label(
                hex.to_string(),
                BUTTON_SIZE,
                scale,
                if open { palette.primary_text } else { palette.surface_text }
            ),
        ]
        .spacing(SWATCH_GAP * scale)
        .align_y(iced::Alignment::Center),
    )
    .height(Length::Fixed(crate::frontend::ui::FOLIO_BUTTON_HEIGHT * scale.max(1.0)))
    .padding([0.0, SWATCH_GAP * scale])
    .on_press(Message::Settings(SettingsMsg::ColourPicker(path.to_string())))
    .style(move |_, status| crate::frontend::ui::folio_button_style(open, false, palette, status));
    let mut body = column![toggle].spacing(SWATCH_GAP * scale);
    if picker > 0.0 {
        let changed = path.to_string();
        let wheel = wheel(
            hex,
            scale,
            move |hex| Message::Settings(SettingsMsg::Colour(changed.clone(), hex)),
            Message::Settings(SettingsMsg::ColourCommit(path.to_string())),
        );
        body = body.push(reveal(wheel, picker, scale));
    }
    body.into()
}

#[cfg(test)]
#[path = "colour_tests.rs"]
mod tests;
