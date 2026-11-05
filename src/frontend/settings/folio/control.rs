#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

use iced::widget::canvas::{self, Frame, Stroke};
use iced::widget::{button, column, container, stack, text, text_input};
use iced::{Alignment, Border, Color, Element, Event, Length, Rectangle, mouse, window};

use crate::app::Message;
use crate::domain::library::filter::resolution_parts;
use crate::frontend::animation::{MotionProfile, MotionTier};
use crate::frontend::components::with_alpha;
use crate::frontend::theme::Palette;
use crate::frontend::ui::calm;
use crate::frontend::ui::{label, legible_type_scale, motion_slot, row};
use crate::i18n::tr;

use super::super::{ActionId, Control, SettingsMsg};
use super::cut_field;
use super::diamond_toggle::{ToggleIdentity, ToggleSpec, diamond_toggle};
use super::dropdown::{DropdownSpec, dropdown};
use super::number_editor::{NumberSpec, number_editor};

pub(super) const CONFIRM_KEY: &str = "settings-control-confirm";
const BUTTON_FRAME: Duration = Duration::from_millis(16);
const COMPACT_BUTTON_HEIGHT: f32 = 30.0;
const BUTTON_TEXT_SIZE: f32 = 11.0;
const BUTTON_LINE_LEADING: f32 = 1.3;
const WRAP_SLACK: f32 = 0.9;
const BUTTON_TEXT_ALPHA: f32 = 0.9;
const BUTTON_FILL_ALPHA: f32 = 0.96;
const BUTTON_OUTLINE_ALPHA: f32 = 0.48;
const PROCESS_EDITOR_WIDTH: f32 = 210.0;
const PROCESS_PICK_WIDTH: f32 = 30.0;
const PROCESS_EDITOR_GAP: f32 = 7.0;
const PROCESS_TEXT_SIZE: f32 = 13.0;
const PROCESS_PICK_LABEL: &str = "…";

pub(super) fn label_width(label: &str, scale: f32) -> f32 {
    crate::frontend::ui::cached_line_width(
        label,
        BUTTON_TEXT_SIZE * legible_type_scale(scale),
        crate::frontend::ui::UI_FONT,
    )
}

pub(super) fn button_width(label: &str, scale: f32) -> f32 {
    let text = label_width(label, scale);
    (text + crate::frontend::ui::FOLIO_BUTTON_INSET * scale)
        .max(crate::frontend::ui::FOLIO_BUTTON_MIN_WIDTH * scale)
}

pub(super) fn fitted_button(label: &str, max: f32, scale: f32) -> (f32, ButtonHeight) {
    let natural = button_width(label, scale);
    if natural <= max {
        return (natural, ButtonHeight::Compact);
    }
    let size = BUTTON_TEXT_SIZE * legible_type_scale(scale);
    let room = (max - crate::frontend::ui::FOLIO_BUTTON_INSET * scale) * WRAP_SLACK;
    (max, ButtonHeight::Lines(wrapped_lines(label, room, size)))
}

fn wrapped_lines(label: &str, room: f32, size: f32) -> usize {
    let measure = |text: &str| {
        crate::frontend::ui::cached_line_width(text, size, crate::frontend::ui::UI_FONT)
    };
    let space = measure(" ");
    let mut lines = 1;
    let mut line = 0.0;
    for word in label.split_whitespace() {
        let width = measure(word);
        if line > 0.0 && line + space + width <= room {
            line += space + width;
            continue;
        }
        if line > 0.0 {
            lines += 1;
        }
        let breaks = (width / room).ceil().max(1.0) as usize - 1;
        lines += breaks;
        line = width - breaks as f32 * room;
    }
    lines
}

#[derive(Debug, Default)]
struct ButtonFillState {
    mix: f32,
    target: bool,
    initialized: bool,
    last_frame: Option<Instant>,
}

impl ButtonFillState {
    fn advance(&mut self, active: bool, now: Instant, motion: MotionProfile) -> bool {
        if !self.initialized {
            self.mix = if active { 1.0 } else { 0.0 };
            self.target = active;
            self.initialized = true;
            self.last_frame = Some(now);
            return false;
        }
        if self.target != active {
            self.target = active;
            self.last_frame = Some(now);
        }
        let dt = self
            .last_frame
            .replace(now)
            .map_or(0.0, |last| now.saturating_duration_since(last).as_secs_f32().min(0.1));
        let target = if self.target { 1.0 } else { 0.0 };
        if dt > 0.0 {
            self.mix = motion.approach(self.mix, target, dt, MotionTier::Fast);
        }
        if (self.mix - target).abs() < 0.002 {
            self.mix = target;
        }
        self.mix != target
    }
}

struct ButtonFill {
    active: bool,
    destructive: bool,
    enabled: bool,
    palette: Palette,
    fade: f32,
    motion: MotionProfile,
    focused: bool,
    scale: f32,
}

impl canvas::Program<Message> for ButtonFill {
    type State = ButtonFillState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        match event {
            Event::Window(window::Event::RedrawRequested(now)) => state
                .advance(self.active, *now, self.motion)
                .then(|| canvas::Action::request_redraw_at(*now + BUTTON_FRAME)),
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        frame.fill(
            &cut_field::fill(bounds.size(), self.scale),
            if self.enabled {
                with_alpha(self.palette.background, BUTTON_FILL_ALPHA * self.fade)
            } else {
                with_alpha(self.palette.surface_container, 0.42 * self.fade)
            },
        );

        let mix = if self.enabled {
            crate::frontend::animation::smoothstep(state.mix.clamp(0.0, 1.0))
        } else {
            0.0
        };
        if mix > 0.0 {
            let fill = if self.destructive {
                with_alpha(self.palette.error, BUTTON_FILL_ALPHA * self.fade)
            } else {
                with_alpha(self.palette.primary, BUTTON_FILL_ALPHA * self.fade)
            };
            frame.fill(&cut_field::wipe(bounds.size(), self.scale, mix), fill);
        }
        let emphasized = self.focused || cursor.is_over(bounds);
        let outline = if self.destructive {
            self.palette.error
        } else if emphasized {
            self.palette.primary
        } else {
            self.palette.outline
        };
        let alpha = if !self.enabled {
            0.18
        } else if emphasized {
            0.9
        } else {
            BUTTON_OUTLINE_ALPHA
        };
        frame.stroke(
            &cut_field::outline(bounds.size(), self.scale),
            Stroke::default()
                .with_color(with_alpha(outline, alpha * self.fade))
                .with_width(cut_field::BORDER_WIDTH * self.scale),
        );
        vec![frame.into_geometry()]
    }
}

fn overlay_style(
    active: bool,
    destructive: bool,
    fade: f32,
    palette: &Palette,
    status: button::Status,
) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered);
    let disabled = matches!(status, button::Status::Disabled);
    button::Style {
        background: None,
        text_color: if disabled {
            with_alpha(palette.surface_text, 0.34 * fade)
        } else if active {
            if destructive {
                with_alpha(palette.error_text, fade)
            } else {
                with_alpha(palette.primary_text, fade)
            }
        } else if hovered {
            with_alpha(palette.hover_text, BUTTON_TEXT_ALPHA * fade)
        } else if destructive {
            with_alpha(palette.error, fade)
        } else {
            with_alpha(palette.surface_text, BUTTON_TEXT_ALPHA * fade)
        },
        border: Border::default(),
        ..Default::default()
    }
}

pub(super) enum ButtonHeight {
    Compact,
    Action,
    Lines(usize),
}

impl ButtonHeight {
    fn pixels(&self, scale: f32) -> f32 {
        let fixed = scale.max(1.0);
        match self {
            Self::Compact => COMPACT_BUTTON_HEIGHT * fixed,
            Self::Action => cut_field::HEIGHT * fixed,
            Self::Lines(count) => {
                let line = BUTTON_TEXT_SIZE * legible_type_scale(scale) * BUTTON_LINE_LEADING;
                COMPACT_BUTTON_HEIGHT * fixed + count.saturating_sub(1) as f32 * line
            }
        }
    }

    fn wraps(&self) -> bool {
        matches!(self, Self::Lines(_))
    }
}

pub(super) struct ButtonSpec<'a> {
    pub label: String,
    pub active: bool,
    pub destructive: bool,
    pub message: Option<Message>,
    pub focused: bool,
    pub scale: f32,
    pub palette: &'a Palette,
    pub fade: f32,
    pub width: f32,
    pub height: ButtonHeight,
    pub motion: MotionProfile,
}

fn caption<'a>(label: String, wraps: bool, scale: f32) -> Element<'a, Message> {
    let size = BUTTON_TEXT_SIZE * legible_type_scale(scale);
    let label = if wraps {
        let inset = crate::frontend::ui::FOLIO_BUTTON_INSET * scale / 2.0;
        let words = calm::Piece::Inked { text: label.into(), size, ink: None };
        let spec = calm::prose(vec![words], calm::PLAIN_LEADING);
        container(calm::sentence(calm::SentenceSpec { centred: true, ..spec }))
            .padding([0.0, inset])
    } else {
        container(text(label).font(crate::frontend::ui::UI_FONT).size(size))
    };
    label
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .into()
}

pub(super) fn fixed_button(spec: ButtonSpec<'_>) -> Element<'_, Message> {
    let ButtonSpec {
        label,
        active,
        destructive,
        message,
        focused,
        scale,
        palette,
        fade,
        width,
        height,
        motion,
    } = spec;
    let enabled = message.is_some();
    let backdrop: Element<'_, Message> = iced::widget::canvas(ButtonFill {
        active,
        destructive,
        enabled,
        palette: *palette,
        fade,
        motion,
        focused,
        scale,
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into();
    let label = caption(label, height.wraps(), scale);
    let hit_target = button(label)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(0.0)
        .style(move |_theme, status| overlay_style(active, destructive, fade, palette, status));
    let hit_target =
        if let Some(message) = message { hit_target.on_press(message) } else { hit_target };
    container(stack![backdrop, hit_target])
        .width(Length::Fixed(width))
        .height(Length::Fixed(height.pixels(scale)))
        .clip(true)
        .into()
}

fn option_button(
    label: String,
    active: bool,
    destructive: bool,
    message: Message,
    focused: bool,
    scale: f32,
    palette: &Palette,
    fade: f32,
    motion: MotionProfile,
) -> Element<'_, Message> {
    let width = button_width(&label, scale);
    fixed_button(ButtonSpec {
        label,
        active,
        destructive,
        message: Some(message),
        focused,
        scale,
        palette,
        fade,
        width,
        height: ButtonHeight::Compact,
        motion,
    })
}

#[allow(clippy::too_many_arguments)]
fn choice_buttons<'a>(
    path: &str,
    options: Vec<(String, String)>,
    current: &str,
    disabled: &[String],
    palettes: &[(String, Vec<String>)],
    keyboard_focused: bool,
    focused_choice: Option<usize>,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    motion: MotionProfile,
) -> Element<'a, Message> {
    let items = options
        .into_iter()
        .enumerate()
        .map(|(index, (key, label))| {
            let width = button_width(&label, scale);
            let enabled = !disabled.contains(&key);
            let active = key == current;
            let colors =
                palettes.iter().rev().find(|(name, _)| name == &key).map(|(_, colors)| colors);
            let message =
                enabled.then(|| Message::Settings(SettingsMsg::Pick(path.to_owned(), key)));
            let item = fixed_button(ButtonSpec {
                label,
                active,
                destructive: false,
                message,
                focused: enabled && keyboard_focused && focused_choice == Some(index),
                scale,
                palette,
                fade,
                width,
                height: ButtonHeight::Compact,
                motion,
            });
            let item = if let Some(colors) = colors {
                let mut strip = row![].width(Length::Fixed(width));
                for color in colors {
                    let color =
                        crate::frontend::theme::parse_hex(color).unwrap_or(Color::TRANSPARENT);
                    strip = strip.push(
                        container(text(""))
                            .width(Length::Fill)
                            .height(Length::Fixed(10.0 * scale))
                            .style(move |_| {
                                crate::frontend::ui::bg_style(iced::Background::Color(color))
                            }),
                    );
                }
                column![item, strip].into()
            } else {
                item
            };
            (width, item)
        })
        .collect();
    wrapped_options(items, available_width, scale)
}

fn wrapped_options(
    items: Vec<(f32, Element<'_, Message>)>,
    available_width: f32,
    scale: f32,
) -> Element<'_, Message> {
    let lines = crate::frontend::ui::wrap_rows(items, available_width, 7.0 * scale);
    let mut group = column![].spacing(7.0 * scale);
    for line in lines {
        let mut line_row = row![].spacing(7.0 * scale).align_y(Alignment::Center);
        for item in line {
            line_row = line_row.push(item);
        }
        group = group.push(line_row);
    }
    group.into()
}

#[derive(Clone, Copy)]
pub(super) struct Reveals<'a> {
    pub(super) bars: &'a HashMap<String, crate::frontend::animation::Tween>,
}

fn control_key(control: &Control) -> Option<u64> {
    let identity = match control {
        Control::Number { key, .. }
        | Control::TextField { key, .. }
        | Control::Resolution { key, .. }
        | Control::KeyBinding { key, .. } => Some(key.as_str()),
        Control::Dropdown { path, .. } | Control::Chips { path, .. } => Some(path.as_str()),
        Control::ActionBtn { id: ActionId::ChooseRunningProcess, .. } => {
            Some(crate::contracts::settings::keys::playback::PROCESSES)
        }
        _ => None,
    }
    .map(str::to_owned)
    .or_else(|| control.compact_bar_id())?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::mem::discriminant(control).hash(&mut hasher);
    identity.hash(&mut hasher);
    Some(hasher.finish())
}

fn keyed_control(view: Element<'_, Message>, identity: Option<u64>) -> Element<'_, Message> {
    match identity {
        Some(identity) => motion_slot(view, None, 1.0, None).identity(identity).into(),
        None => view,
    }
}

pub(super) fn widget<'a>(
    control: Control,
    values: &'a HashMap<String, String>,
    keyboard_focused: bool,
    focused_choice: Option<usize>,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    armed: Option<&ActionId>,
    motion: MotionProfile,
    reveals: Option<Reveals<'_>>,
) -> Element<'a, Message> {
    let identity = control_key(&control);
    let view = match control {
        Control::Display(control) => control.view(scale, palette),
        Control::AppTheme { app, .. } => {
            label(app.name, crate::frontend::ui::TYPE_SMALL, scale, palette.surface_text).into()
        }
        Control::Colour { path, hex } => {
            let picker = reveals
                .and_then(|maps| {
                    maps.bars.get(&crate::frontend::settings::colour::picker_id(&path))
                })
                .map_or(0.0, |value| value.x);
            crate::frontend::settings::colour::view(&path, &hex, picker, scale, palette)
        }
        Control::Toggle { path, value } => diamond_toggle(ToggleSpec {
            identity: ToggleIdentity::Path(path.clone()),
            value,
            focused: keyboard_focused,
            scale,
            fade,
            palette: *palette,
            message: Message::Settings(SettingsMsg::Toggle(path, !value)),
            motion,
        }),
        Control::ToggleAction { id, value } => diamond_toggle(ToggleSpec {
            identity: ToggleIdentity::Action(id.clone()),
            value,
            focused: keyboard_focused,
            scale,
            fade,
            palette: *palette,
            message: Message::Settings(SettingsMsg::Run(id)),
            motion,
        }),
        Control::Number { key, path, unit } => number_editor(NumberSpec {
            raw: values.get(&key).map_or("", String::as_str),
            key,
            path,
            unit,
            scale,
            fade,
            palette,
        }),
        Control::KeyBinding { path, default, .. } => {
            let value = values.get(&path).map_or("", String::as_str);
            let triggers = crate::domain::input::parse_binding(value)
                .or_else(|| crate::domain::input::parse_binding(default))
                .unwrap_or_default();
            let pal = *palette;
            let mut slots = row![].spacing(6.0 * scale);
            for slot in 0..crate::domain::input::SLOTS {
                let shown = crate::domain::input::slot_triggers(&triggers, slot);
                let (display, alpha) = if shown.is_empty() {
                    (tr("settings-keybind-unbound").to_string(), 0.4)
                } else {
                    (crate::domain::input::binding_label(shown), 1.0)
                };
                let slot_focused = keyboard_focused && focused_choice == Some(slot);
                slots = slots.push(
                    button(
                        text(display)
                            .font(crate::frontend::ui::UI_FONT)
                            .size(13.0 * legible_type_scale(scale))
                            .color(with_alpha(palette.surface_text, alpha * fade)),
                    )
                    .padding([8.0 * scale, 10.0 * scale])
                    .width(Length::Fill)
                    .on_press(Message::Settings(SettingsMsg::KeybindCapture(path.clone(), slot)))
                    .style(move |_theme, status| {
                        crate::frontend::ui::folio_button_style(slot_focused, false, &pal, status)
                    }),
                );
            }
            slots.into()
        }
        Control::TextField { key, placeholder, .. } => {
            let value = values.get(&key).map_or("", String::as_str);
            let mut input = text_input(placeholder, value)
                .id(super::super::workbench_input_id(&key))
                .font(crate::frontend::ui::UI_FONT)
                .size(13.0 * legible_type_scale(scale))
                .padding([8.0 * scale, 10.0 * scale])
                .width(Length::Fill)
                .style(move |_theme, _status| {
                    crate::frontend::ui::workbench_input_style(
                        with_alpha(palette.surface_text, fade),
                        with_alpha(palette.primary, fade),
                    )
                });
            let message_key = key.clone();
            input = input
                .on_input(move |raw| {
                    Message::Settings(SettingsMsg::Input(message_key.clone(), raw))
                })
                .on_submit(Message::Settings(SettingsMsg::Commit));
            input.into()
        }
        Control::Resolution { key, width_placeholder, height_placeholder, .. } => {
            let value = values.get(&key).map_or("", String::as_str);
            let maximum = key.ends_with(".to");
            if maximum && value.is_empty() {
                return keyed_control(
                    row![
                        label(
                            tr("settings-filter-resolution-unlimited"),
                            13.0,
                            scale,
                            with_alpha(palette.surface_text, fade)
                        ),
                        option_button(
                            tr("settings-filter-resolution-set-limit").into(),
                            false,
                            false,
                            Message::Settings(SettingsMsg::ResolutionLimit(key, true)),
                            keyboard_focused,
                            scale,
                            palette,
                            fade,
                            motion,
                        ),
                    ]
                    .spacing(12.0 * scale)
                    .align_y(Alignment::Center)
                    .into(),
                    identity,
                );
            }
            let (width, height) = resolution_parts(value);
            let width_key = key.clone();
            let height_key = key.clone();
            let width_input = text_input(width_placeholder, width)
                .id(super::super::workbench_input_id(&format!("{key}.width")))
                .font(crate::frontend::ui::UI_FONT)
                .on_input(move |raw| {
                    Message::Settings(SettingsMsg::ResolutionInput(width_key.clone(), true, raw))
                })
                .on_submit(Message::Settings(SettingsMsg::Commit))
                .size(13.0 * legible_type_scale(scale))
                .padding([8.0 * scale, 10.0 * scale])
                .width(Length::Fill)
                .style(move |_theme, _status| {
                    crate::frontend::ui::workbench_input_style(
                        with_alpha(palette.surface_text, fade),
                        with_alpha(palette.primary, fade),
                    )
                });
            let height_input = text_input(height_placeholder, height)
                .id(super::super::workbench_input_id(&format!("{key}.height")))
                .font(crate::frontend::ui::UI_FONT)
                .on_input(move |raw| {
                    Message::Settings(SettingsMsg::ResolutionInput(height_key.clone(), false, raw))
                })
                .on_submit(Message::Settings(SettingsMsg::Commit))
                .size(13.0 * legible_type_scale(scale))
                .padding([8.0 * scale, 10.0 * scale])
                .width(Length::Fill)
                .style(move |_theme, _status| {
                    crate::frontend::ui::workbench_input_style(
                        with_alpha(palette.surface_text, fade),
                        with_alpha(palette.primary, fade),
                    )
                });
            let dimensions = row![
                column![
                    label(
                        width_placeholder,
                        11.0,
                        scale,
                        with_alpha(palette.surface_text, 0.68 * fade)
                    ),
                    width_input
                ]
                .spacing(4.0 * scale)
                .width(Length::Fill),
                label("×", 14.0, scale, with_alpha(palette.surface_text, 0.56 * fade)),
                column![
                    label(
                        height_placeholder,
                        11.0,
                        scale,
                        with_alpha(palette.surface_text, 0.68 * fade)
                    ),
                    height_input
                ]
                .spacing(4.0 * scale)
                .width(Length::Fill),
            ]
            .spacing(8.0 * scale)
            .align_y(Alignment::End);
            let mut editor = column![dimensions].spacing(8.0 * scale);
            if maximum {
                editor = editor.push(option_button(
                    tr("settings-filter-resolution-clear-limit").into(),
                    false,
                    false,
                    Message::Settings(SettingsMsg::ResolutionLimit(key.clone(), false)),
                    false,
                    scale,
                    palette,
                    fade,
                    motion,
                ));
            }
            let base = key.rsplit_once('.').map_or("", |(base, _)| base);
            let from = values.get(&format!("{base}.from")).map_or("", String::as_str);
            let to = values.get(&format!("{base}.to")).map_or("", String::as_str);
            if !crate::domain::library::filter::resolution_bounds_valid(from, to) {
                editor = editor.push(label(
                    tr("settings-filter-resolution-invalid"),
                    11.0,
                    scale,
                    with_alpha(palette.primary, fade),
                ));
            }
            editor.into()
        }
        Control::Dropdown { path, options, current, palettes } => {
            if palettes.is_empty() {
                dropdown(DropdownSpec {
                    path,
                    options,
                    current,
                    scale,
                    fade,
                    palette,
                    focused: keyboard_focused,
                })
            } else {
                choice_buttons(
                    &path,
                    options,
                    &current,
                    &[],
                    &palettes,
                    keyboard_focused,
                    focused_choice,
                    available_width,
                    scale,
                    palette,
                    fade,
                    motion,
                )
            }
        }
        Control::Chips { path, options, current, disabled, background } => {
            let choices = choice_buttons(
                &path,
                options,
                &current,
                &disabled,
                &[],
                keyboard_focused,
                focused_choice,
                available_width,
                scale,
                palette,
                fade,
                motion,
            );
            if let Some(background) = background {
                let shown = background.placement_visible;
                let reveal = reveals
                    .and_then(|maps| {
                        maps.bars
                            .get(&super::super::background::BackgroundControl::reveal_id(&path))
                    })
                    .map_or(if shown { 1.0 } else { 0.0 }, |value| value.x);
                let picker = reveals
                    .and_then(|maps| maps.bars.get(&background.picker_id()))
                    .map_or(0.0, |value| value.x);
                column![choices, background.view(reveal, picker, scale, palette)].into()
            } else {
                choices
            }
        }
        Control::ActionBtn { id, label } => {
            if id == ActionId::ChooseRunningProcess {
                let key = crate::contracts::settings::keys::playback::PROCESSES;
                let value = values.get(key).map_or("", String::as_str);
                let message_key = key.to_string();
                let input = text_input(tr("settings-playback-target-manual-placeholder"), value)
                    .id(super::super::workbench_input_id(key))
                    .font(crate::frontend::ui::UI_FONT)
                    .on_input(move |raw| {
                        Message::Settings(SettingsMsg::Input(message_key.clone(), raw))
                    })
                    .on_submit(Message::Settings(SettingsMsg::Commit))
                    .size(PROCESS_TEXT_SIZE * legible_type_scale(scale))
                    .padding([8.0 * scale, 10.0 * scale])
                    .width(Length::Fixed(
                        (PROCESS_EDITOR_WIDTH - PROCESS_PICK_WIDTH - PROCESS_EDITOR_GAP) * scale,
                    ))
                    .style(move |_theme, _status| {
                        crate::frontend::ui::workbench_input_style(
                            with_alpha(palette.surface_text, fade),
                            with_alpha(palette.primary, fade),
                        )
                    });
                let discover = crate::frontend::ui::folio_action(
                    PROCESS_PICK_LABEL,
                    false,
                    Some(Message::Settings(SettingsMsg::Run(id))),
                    Length::Fixed(PROCESS_PICK_WIDTH * scale),
                    scale,
                    palette,
                );
                let discover = iced::widget::tooltip(
                    discover,
                    text(label).font(crate::frontend::ui::UI_FONT),
                    iced::widget::tooltip::Position::Top,
                );
                return keyed_control(
                    row![input, discover]
                        .spacing(PROCESS_EDITOR_GAP * scale)
                        .width(Length::Fixed(PROCESS_EDITOR_WIDTH * scale))
                        .align_y(Alignment::Center)
                        .into(),
                    identity,
                );
            }
            let confirming = armed == Some(&id);
            let confirmation = crate::i18n::tr_args!(CONFIRM_KEY, label => &label);
            let width = (cut_field::WIDTH * scale)
                .max(button_width(&confirmation, scale))
                .max(button_width(&label, scale));
            fixed_button(ButtonSpec {
                label: if confirming { confirmation } else { label },
                active: confirming,
                destructive: confirming,
                message: Some(Message::Settings(SettingsMsg::Run(id))),
                focused: keyboard_focused,
                scale,
                palette,
                fade,
                width,
                height: ButtonHeight::Action,
                motion,
            })
        }
        Control::ActionChips { items } => wrapped_options(
            items
                .into_iter()
                .enumerate()
                .map(|(index, (id, label))| {
                    let width = button_width(&label, scale);
                    (
                        width,
                        fixed_button(ButtonSpec {
                            label,
                            active: false,
                            destructive: false,
                            message: Some(Message::Settings(SettingsMsg::Run(id))),
                            focused: keyboard_focused && focused_choice == Some(index),
                            scale,
                            palette,
                            fade,
                            width,
                            height: ButtonHeight::Compact,
                            motion,
                        }),
                    )
                })
                .collect(),
            available_width,
            scale,
        ),
        Control::Details { summary, .. } | Control::StackBar { summary, .. } => label(
            format!("▸ {summary}"),
            11.0,
            scale,
            with_alpha(palette.surface_text, 0.68 * fade),
        )
        .into(),
        Control::Segment | Control::Static => text("").into(),
        Control::Code { snippet } => label(
            snippet,
            crate::frontend::ui::TYPE_SMALL,
            scale,
            with_alpha(palette.surface_text, 0.68 * fade),
        )
        .into(),
        Control::Preview => label(
            tr("settings-control-live-backdrop"),
            12.0,
            scale,
            with_alpha(palette.primary, fade),
        )
        .into(),
    };
    keyed_control(view, identity)
}
