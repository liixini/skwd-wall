use iced::widget::{button, column, container};
use iced::{Element, Length};

use crate::app::Message;
use crate::contracts::settings::{SettingsSource, keys};
use crate::frontend::theme::Palette;
use crate::frontend::ui::{label, row};
use crate::i18n::tr;

use super::SettingsMsg;

const HEX_SIZE: f32 = 12.0;
const WHEEL_GAP: f32 = 14.0;

#[derive(Debug, Clone, Default)]
pub struct BackgroundControl {
    pub output: String,
    pub mode: String,
    pub color: String,
    pub inherited: bool,
    pub placement_visible: bool,
}

impl BackgroundControl {
    pub fn new(cfg: &dyn SettingsSource, output: &str) -> Self {
        let mode = cfg.text(&format!("{}.{output}", keys::display::BACKGROUND_MODES));
        let inherited = !output.is_empty() && !matches!(mode.as_str(), "color" | "blur");
        let mode = if output.is_empty() || inherited {
            cfg.text(keys::display::BACKGROUND_MODE)
        } else {
            mode
        };
        let color = if output.is_empty() || inherited {
            String::new()
        } else {
            cfg.text(&format!("{}.{output}", keys::display::BACKGROUND_COLORS))
        };
        let color = if color.is_empty() { cfg.text(keys::display::FILL_COLOR) } else { color };
        let color =
            super::colour::normalized(&color).unwrap_or_else(|| super::colour::BLACK.to_string());
        let placement = if output.is_empty() {
            cfg.text(keys::display::FILL_MODE)
        } else {
            let local = cfg.text(&format!("{}.{output}", keys::display::FILL_MODES));
            if local.is_empty() { cfg.text(keys::display::FILL_MODE) } else { local }
        };
        let placement_visible = matches!(placement.as_str(), "center" | "fit");
        Self { output: output.to_string(), mode, color, inherited, placement_visible }
    }

    pub fn button_width(scale: f32) -> f32 {
        let width = crate::frontend::ui::folio_action_width(tr("settings-background-color"), scale)
            + 28.0 * scale;
        width
            .max(92.0 * scale)
            .max(crate::frontend::ui::folio_action_width(tr("settings-background-blur"), scale))
            .max(crate::frontend::ui::folio_action_width(tr("settings-background-inherit"), scale))
    }

    pub fn choices_width(scale: f32) -> f32 {
        72.0 * scale + 3.0 * Self::button_width(scale) + 30.0 * scale
    }

    pub const REVEAL_PREFIX: &'static str = "background:";

    pub fn reveal_id(path: &str) -> String {
        format!("{}{path}", Self::REVEAL_PREFIX)
    }

    pub fn picker_id(&self) -> String {
        format!("background-picker:{}", self.output)
    }

    pub fn view<'a>(
        &self,
        reveal: f32,
        picker: f32,
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        let solid = self.mode != "blur";
        let width = Self::button_width(scale);
        let color = button(
            container(
                row![
                    super::colour::swatch(&self.color, super::colour::SWATCH_SIDE * scale, palette),
                    label(
                        tr("settings-background-color"),
                        10.0,
                        scale,
                        if solid { palette.primary_text } else { palette.surface_text }
                    ),
                ]
                .spacing(8.0 * scale)
                .align_y(iced::Alignment::Center),
            )
            .center(Length::Fill),
        )
        .width(Length::Fixed(width))
        .height(Length::Fixed(crate::frontend::ui::FOLIO_BUTTON_HEIGHT * scale.max(1.0)))
        .padding([0.0, 8.0 * scale])
        .on_press(Message::Settings(SettingsMsg::BackgroundPicker(self.output.clone())))
        .style(move |_, status| {
            crate::frontend::ui::folio_button_style(solid, false, palette, status)
        });
        let mut choices = row![
            label(tr("settings-background-label"), 9.0, scale, palette.surface_text)
                .width(Length::Fixed(72.0 * scale)),
            color,
            crate::frontend::ui::folio_action(
                tr("settings-background-blur"),
                self.mode == "blur",
                Some(Message::Settings(SettingsMsg::BackgroundMode(
                    self.output.clone(),
                    String::from("blur")
                ))),
                Length::Fixed(width),
                scale,
                palette
            ),
        ]
        .spacing(10.0 * scale)
        .align_y(iced::Alignment::Center);
        if !self.output.is_empty() {
            choices = choices.push(crate::frontend::ui::folio_action(
                tr("settings-background-inherit"),
                self.inherited,
                Some(Message::Settings(SettingsMsg::BackgroundMode(
                    self.output.clone(),
                    String::from("inherit"),
                ))),
                Length::Fixed(width),
                scale,
                palette,
            ));
        }
        let mut body = column![choices].spacing(8.0 * scale);
        if picker > 0.0 {
            let output = self.output.clone();
            let wheel = super::colour::wheel(
                &self.color,
                scale,
                move |hex| Message::Settings(SettingsMsg::BackgroundColor(output.clone(), hex)),
                Message::Settings(SettingsMsg::BackgroundCommit(self.output.clone())),
            );
            body = body.push(super::colour::reveal(
                row![wheel, label(self.color.clone(), HEX_SIZE, scale, palette.surface_text)]
                    .spacing(WHEEL_GAP * scale),
                picker,
                scale,
            ));
        }
        container(body)
            .width(Length::Fill)
            .height(Length::Fixed(
                (52.0 + 192.0 * picker) * scale * crate::frontend::animation::smoothstep(reveal),
            ))
            .clip(true)
            .padding(iced::Padding { top: 8.0 * scale, ..iced::Padding::ZERO })
            .into()
    }
}
