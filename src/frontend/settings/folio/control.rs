use std::collections::HashMap;
use std::time::{Duration, Instant};

use iced::widget::canvas::{self, Frame, Path, Stroke};
use iced::widget::{button, column, container, stack, text, text_input};
use iced::{Alignment, Border, Color, Element, Event, Length, Point, Rectangle, mouse, window};

use crate::app::Message;
use crate::frontend::animation::{MotionProfile, MotionTier};
use crate::frontend::components::with_alpha;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{
    folio_diagonal_edges, folio_diagonal_wipe, label, legible_type_scale, mirror_x, row,
};
use crate::i18n::tr;

use super::super::{ActionId, Control, PRESET_NAME_KEY, SettingsMsg};

const BUTTON_FRAME: Duration = Duration::from_millis(16);
const BUTTON_HEIGHT: f32 = 30.0;
const BUTTON_MIN_WIDTH: f32 = 92.0;

pub(super) fn button_width(label: &str, scale: f32) -> f32 {
    crate::frontend::ui::folio_action_width_in(label, scale, BUTTON_MIN_WIDTH, f32::INFINITY)
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
        let hovered = self.enabled && cursor.is_over(bounds);
        frame.fill(
            &Path::rectangle(Point::ORIGIN, bounds.size()),
            if hovered {
                with_alpha(self.palette.surface_variant, 0.78 * self.fade)
            } else if !self.enabled {
                with_alpha(self.palette.surface_container, 0.42 * self.fade)
            } else {
                with_alpha(self.palette.surface_container, 0.92 * self.fade)
            },
        );

        let mix = if self.enabled {
            crate::frontend::animation::smoothstep(state.mix.clamp(0.0, 1.0))
        } else {
            0.0
        };
        if mix > 0.0 {
            let fill = if self.destructive {
                with_alpha(self.palette.tertiary, 0.96 * self.fade)
            } else {
                with_alpha(self.palette.primary, 0.96 * self.fade)
            };
            frame.fill(&folio_diagonal_wipe(bounds.width, bounds.height, mix), fill);
            if mix < 1.0 {
                let (top, bottom) = folio_diagonal_edges(bounds.width, bounds.height, mix);
                let top = mirror_x(top, 0.0, bounds.width);
                let bottom = mirror_x(bottom, 0.0, bounds.width);
                frame.stroke(
                    &Path::line(Point::new(top, 0.0), Point::new(bottom, bounds.height)),
                    Stroke::default()
                        .with_color(with_alpha(self.palette.tertiary, 0.9 * self.fade))
                        .with_width(2.0),
                );
            }
        }
        vec![frame.into_geometry()]
    }
}

fn overlay_style(
    active: bool,
    destructive: bool,
    focused: bool,
    fade: f32,
    palette: &Palette,
    status: button::Status,
) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered);
    let disabled = matches!(status, button::Status::Disabled);
    button::Style {
        background: Some(Color::TRANSPARENT.into()),
        text_color: if disabled {
            with_alpha(palette.surface_text, 0.34 * fade)
        } else if active {
            if destructive {
                with_alpha(palette.background, fade)
            } else {
                with_alpha(palette.primary_text, fade)
            }
        } else if destructive {
            with_alpha(palette.tertiary, fade)
        } else {
            with_alpha(palette.surface_text, 0.9 * fade)
        },
        border: Border {
            color: if disabled {
                with_alpha(palette.outline, 0.18 * fade)
            } else if destructive {
                with_alpha(palette.tertiary, if hovered || focused { fade } else { 0.62 * fade })
            } else {
                with_alpha(
                    if focused || hovered { palette.primary } else { palette.outline },
                    if focused {
                        fade
                    } else if hovered {
                        0.76 * fade
                    } else {
                        0.4 * fade
                    },
                )
            },
            width: if focused && !disabled { 2.0 } else { 1.0 },
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub(super) fn fixed_button<'a>(
    label: String,
    active: bool,
    destructive: bool,
    message: Option<Message>,
    focused: bool,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    width: f32,
    motion: MotionProfile,
) -> Element<'a, Message> {
    let enabled = message.is_some();
    let backdrop: Element<'a, Message> = iced::widget::canvas(ButtonFill {
        active,
        destructive,
        enabled,
        palette: *palette,
        fade,
        motion,
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into();
    let label = container(
        text(label).font(crate::frontend::ui::UI_FONT).size(11.0 * legible_type_scale(scale)),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center);
    let hit_target = button(label).width(Length::Fill).height(Length::Fill).padding(0.0).style(
        move |_theme, status| overlay_style(active, destructive, focused, fade, palette, status),
    );
    let hit_target =
        if let Some(message) = message { hit_target.on_press(message) } else { hit_target };
    container(stack![backdrop, hit_target])
        .width(Length::Fixed(width))
        .height(Length::Fixed(BUTTON_HEIGHT * scale.max(1.0)))
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
    fixed_button(
        label,
        active,
        destructive,
        Some(message),
        focused,
        scale,
        palette,
        fade,
        width,
        motion,
    )
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
            let item = fixed_button(
                label,
                active,
                false,
                message,
                enabled && keyboard_focused && focused_choice == Some(index),
                scale,
                palette,
                fade,
                width,
                motion,
            );
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
    pub(super) folds: &'a std::collections::HashSet<String>,
}

pub(super) fn widget<'a>(
    control: Control,
    values: &'a HashMap<String, String>,
    has_selected_preset: bool,
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
    match control {
        Control::Display(control) => control.view(scale, palette),
        Control::AppTheme { app, .. } => {
            label(app.name, crate::frontend::ui::TYPE_SMALL, scale, palette.surface_text).into()
        }
        Control::Toggle { path, value } => fixed_button(
            if value {
                tr("settings-control-enabled").into()
            } else {
                tr("settings-control-disabled").into()
            },
            value,
            false,
            Some(Message::Settings(SettingsMsg::Toggle(path, !value))),
            keyboard_focused,
            scale,
            palette,
            fade,
            button_width(tr("settings-control-enabled"), scale)
                .max(button_width(tr("settings-control-disabled"), scale)),
            motion,
        ),
        Control::ToggleAction { id, value } => fixed_button(
            if value {
                tr("settings-app-themes-on").into()
            } else {
                tr("settings-app-themes-off").into()
            },
            value,
            false,
            Some(Message::Settings(SettingsMsg::Run(id))),
            keyboard_focused,
            scale,
            palette,
            fade,
            button_width(tr("settings-app-themes-on"), scale)
                .max(button_width(tr("settings-app-themes-off"), scale)),
            motion,
        ),
        Control::Number { key, unit, .. } => {
            let value = values.get(&key).map_or("", String::as_str);
            let automatic = key == crate::contracts::settings::keys::transition::FPS;
            let value = if automatic && value.parse::<f64>() == Ok(0.0) { "" } else { value };
            let placeholder = if automatic { tr("settings-transitions-rate-auto") } else { "0" };
            let message_key = key.clone();
            row![
                text_input(placeholder, value)
                    .id(super::super::workbench_input_id(&key))
                    .font(crate::frontend::ui::UI_FONT)
                    .on_input(move |raw| Message::Settings(SettingsMsg::Input(
                        message_key.clone(),
                        raw,
                    )))
                    .on_submit(Message::Settings(SettingsMsg::Commit))
                    .size(16.0 * legible_type_scale(scale))
                    .padding([7.0 * scale, 10.0 * scale])
                    .width(Length::Fill)
                    .style(move |_theme, _status| {
                        crate::frontend::ui::workbench_input_style(
                            with_alpha(palette.surface_text, fade),
                            with_alpha(palette.primary, fade),
                        )
                    }),
                label(
                    unit,
                    crate::frontend::ui::TYPE_SMALL,
                    scale,
                    with_alpha(palette.surface_text, 0.56 * fade)
                ),
            ]
            .spacing(8.0 * scale)
            .align_y(Alignment::End)
            .into()
        }
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
            let enabled = key != PRESET_NAME_KEY || has_selected_preset;
            let hint =
                if enabled { placeholder } else { tr("settings-selector-preset-rename-hint") };
            let mut input = text_input(hint, value)
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
            if enabled {
                if key == PRESET_NAME_KEY {
                    input = input.on_input(Message::PresetNameInput);
                } else {
                    let message_key = key.clone();
                    input = input
                        .on_input(move |raw| {
                            Message::Settings(SettingsMsg::Input(message_key.clone(), raw))
                        })
                        .on_submit(Message::Settings(SettingsMsg::Commit));
                }
            }
            input.into()
        }
        Control::Resolution { key, width_placeholder, height_placeholder, .. } => {
            let value = values.get(&key).map_or("", String::as_str);
            let maximum = key.ends_with(".to");
            if maximum && value.is_empty() {
                return row![
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
                .into();
            }
            let (width, height) = value.split_once(['x', 'X', '×']).unwrap_or(("", ""));
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
        Control::Dropdown { path, options, current, palettes } => choice_buttons(
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
        ),
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
                let shown = matches!(current.as_str(), "center" | "fit");
                let reveal = reveals
                    .and_then(|maps| maps.bars.get(&format!("background:{path}")))
                    .map_or(if shown { 1.0 } else { 0.0 }, |value| value.x);
                let picker = reveals
                    .and_then(|maps| maps.bars.get(&background.picker_id()))
                    .map_or(0.0, |value| value.x);
                column![choices, background.view(reveal, picker, scale, palette)].into()
            } else {
                choices
            }
        }
        Control::MotionWeights { weights } => {
            let mut group = row![].spacing(8.0 * scale);
            for (label, key, reset) in weights {
                let value = values.get(&key).map_or("", String::as_str);
                let message_key = key.clone();
                group = group.push(
                    column![
                        crate::frontend::ui::label(
                            label,
                            10.0,
                            scale,
                            with_alpha(palette.surface_text, 0.62 * fade),
                        ),
                        row![
                            text_input("0", value)
                                .id(super::super::workbench_input_id(&key))
                                .font(crate::frontend::ui::UI_FONT)
                                .on_input(move |raw| Message::Settings(SettingsMsg::Input(
                                    message_key.clone(),
                                    raw,
                                )))
                                .on_submit(Message::Settings(SettingsMsg::Commit))
                                .size(13.0 * legible_type_scale(scale))
                                .padding([6.0 * scale, 8.0 * scale])
                                .style(move |_theme, _status| {
                                    crate::frontend::ui::workbench_input_style(
                                        with_alpha(palette.surface_text, fade),
                                        with_alpha(palette.primary, fade),
                                    )
                                }),
                            fixed_button(
                                "↺".into(),
                                false,
                                false,
                                Some(Message::Settings(SettingsMsg::Run(reset))),
                                false,
                                scale * 0.85,
                                palette,
                                fade,
                                34.0 * scale,
                                motion,
                            ),
                        ]
                        .spacing(5.0 * scale),
                    ]
                    .spacing(5.0 * scale)
                    .width(Length::FillPortion(1)),
                );
            }
            group.into()
        }
        Control::ActionBtn { id, label } => {
            let confirming = armed == Some(&id);
            let confirmation = crate::i18n::tr_args!("settings-control-confirm", label => &label);
            let width = button_width(&confirmation, scale).max(button_width(&label, scale));
            fixed_button(
                if confirming { confirmation } else { label },
                confirming,
                confirming,
                Some(Message::Settings(SettingsMsg::Run(id))),
                keyboard_focused,
                scale,
                palette,
                fade,
                width,
                motion,
            )
        }
        Control::ActionChips { items } => wrapped_options(
            items
                .into_iter()
                .enumerate()
                .map(|(index, (id, label))| {
                    let width = button_width(&label, scale);
                    (
                        width,
                        fixed_button(
                            label,
                            false,
                            false,
                            Some(Message::Settings(SettingsMsg::Run(id))),
                            keyboard_focused && focused_choice == Some(index),
                            scale,
                            palette,
                            fade,
                            width,
                            motion,
                        ),
                    )
                })
                .collect(),
            available_width,
            scale,
        ),
        Control::Presets { mode, items } => {
            let mut options = vec![(
                button_width(tr("settings-selector-preset-save-current"), scale),
                option_button(
                    tr("settings-selector-preset-save-current").into(),
                    false,
                    false,
                    Message::SavePreset,
                    keyboard_focused && focused_choice == Some(0),
                    scale,
                    palette,
                    fade,
                    motion,
                ),
            )];
            for (index, (name, active)) in items.into_iter().enumerate() {
                let apply_index = 1 + index * 2;
                let delete_index = apply_index + 1;
                let width = button_width(&name, scale);
                options.push((
                    width,
                    fixed_button(
                        name.clone(),
                        active,
                        false,
                        Some(Message::ApplyPreset(mode.clone(), name.clone())),
                        keyboard_focused && focused_choice == Some(apply_index),
                        scale,
                        palette,
                        fade,
                        width,
                        motion,
                    ),
                ));
                options.push((
                    34.0 * scale,
                    fixed_button(
                        "×".into(),
                        false,
                        true,
                        Some(Message::DeletePreset(mode.clone(), name)),
                        keyboard_focused && focused_choice == Some(delete_index),
                        scale,
                        palette,
                        fade,
                        34.0 * scale,
                        motion,
                    ),
                ));
            }
            wrapped_options(options, available_width, scale)
        }
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
    }
}
