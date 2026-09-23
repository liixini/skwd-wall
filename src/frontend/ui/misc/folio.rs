use std::time::{Duration, Instant};

use iced::widget::canvas::{self, Frame, Path, Stroke};
use iced::widget::{button, column, container, stack, text};
use iced::{
    Alignment, Background, Border, Color, Element, Event, Length, Point, Rectangle, mouse, window,
};

use crate::frontend::animation::{MotionProfile, MotionTier};
use crate::frontend::theme::Palette;
use crate::frontend::ui::{UI_FONT, logical_padding, mirror_x, row, with_alpha};

use super::typography::legible_type_scale;

const BUTTON_FRAME: Duration = Duration::from_millis(16);
const BUTTON_HEIGHT: f32 = 30.0;

pub const FOLIO_INDEX_WIDTH: f32 = 318.0;
pub const FOLIO_SCRIM_ALPHA: f32 = 0.68;
pub const FOLIO_INDEX_BG_ALPHA: f32 = 0.9;
pub const FOLIO_INDEX_OUTLINE_ALPHA: f32 = 0.46;

const FOLIO_ACTION_MIN_WIDTH: f32 = 52.0;
const FOLIO_ACTION_MAX_WIDTH: f32 = 190.0;

pub fn folio_action_width_in(action_label: &str, scale: f32, min: f32, max: f32) -> f32 {
    (super::typography::text_width(action_label, 10.0, false) + 28.0).clamp(min, max) * scale
}

pub fn folio_action_width(action_label: &str, scale: f32) -> f32 {
    folio_action_width_in(action_label, scale, FOLIO_ACTION_MIN_WIDTH, FOLIO_ACTION_MAX_WIDTH)
}

pub fn folio_scrim_style(ease: f32) -> container::Style {
    super::style::scrim_style(FOLIO_SCRIM_ALPHA * ease)
}

pub fn folio_sheet_panel_style(palette: &Palette, ease: f32) -> container::Style {
    let mut style = super::style::box_style(
        with_alpha(palette.surface, 0.99),
        with_alpha(palette.outline, 0.58),
    );
    style.shadow = iced::Shadow {
        color: with_alpha(Color::BLACK, 0.52 * ease),
        offset: iced::Vector::new(0.0, 16.0),
        blur_radius: 48.0,
    };
    style
}

pub fn folio_index_shell<'a, Message: Clone + 'a>(
    title: impl text::IntoFragment<'a>,
    note: impl text::IntoFragment<'a>,
    body: Vec<Element<'a, Message>>,
    spacing: f32,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    folio_index_shell_tinted(
        title,
        note,
        body,
        spacing,
        (FOLIO_INDEX_BG_ALPHA, FOLIO_INDEX_OUTLINE_ALPHA),
        scale,
        palette,
    )
}

pub fn folio_index_shell_tinted<'a, Message: Clone + 'a>(
    title: impl text::IntoFragment<'a>,
    note: impl text::IntoFragment<'a>,
    body: Vec<Element<'a, Message>>,
    spacing: f32,
    alphas: (f32, f32),
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    use super::typography::label;
    let heading = column![
        label(title, 20.0, scale, palette.surface_text),
        label(note, super::typography::TYPE_SMALL, scale, with_alpha(palette.surface_text, 0.56))
            .line_height(iced::widget::text::LineHeight::Relative(1.4)),
    ]
    .spacing(7.0 * scale);
    let mut content = column![heading].spacing(spacing * scale);
    for element in body {
        content = content.push(element);
    }
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(logical_padding(25.0 * scale, 20.0 * scale, 20.0 * scale, 22.0 * scale))
        .style(move |_| {
            super::style::box_style(
                with_alpha(palette.background, alphas.0),
                with_alpha(palette.outline, alphas.1),
            )
        })
        .into()
}

pub fn folio_stack_bar<'a, Message: Clone + 'a>(
    content: Element<'a, Message>,
    active: bool,
    expanded: bool,
    reveal: f32,
    on_activate: Message,
    trailing: Option<Element<'a, Message>>,
    options: Element<'a, Message>,
    bar_height: f32,
    options_height: f32,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    use super::typography::label;

    let disclosure = container(label(
        if expanded { "▴" } else { "▾" },
        9.0,
        scale,
        if expanded { palette.primary } else { with_alpha(palette.surface_text, 0.58) },
    ))
    .width(Length::Fixed(20.0 * scale))
    .height(Length::Fixed(32.0 * scale))
    .center(Length::Fill);
    let select = button(row![content, disclosure].spacing(6.0 * scale).align_y(Alignment::Center))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([9.0 * scale, 10.0 * scale])
        .on_press(on_activate)
        .style(move |_theme, status| {
            super::style::folio_embedded_button_style(
                palette.surface_text,
                !active,
                palette,
                status,
            )
        });
    let marker = container(text("")).width(Length::Fixed(2.0 * scale)).height(Length::Fill).style(
        move |_| {
            super::style::bg_style(Background::Color(if active {
                palette.primary
            } else {
                Color::TRANSPARENT
            }))
        },
    );
    let mut bar_content = row![marker, select].spacing(0.0).align_y(Alignment::Center);
    if let Some(trailing) = trailing {
        bar_content = bar_content.push(trailing);
    }
    let bar = container(bar_content)
        .width(Length::Fill)
        .height(Length::Fixed(bar_height * scale))
        .style(move |_| {
            super::style::box_style(
                if active {
                    with_alpha(palette.primary, 0.18)
                } else {
                    with_alpha(palette.surface_container, 0.22)
                },
                with_alpha(if active { palette.primary } else { palette.outline }, 0.3),
            )
        });
    let mut layers = column![bar].spacing(0.0);
    let reveal = crate::frontend::animation::smoothstep(reveal);
    if reveal > 0.0 {
        layers = layers.push(
            container(options)
                .width(Length::Fill)
                .height(Length::Fixed(options_height * scale * reveal))
                .clip(true)
                .padding(logical_padding(7.0 * scale, 30.0 * scale, 8.0 * scale, 12.0 * scale))
                .style(move |_| {
                    super::style::box_style(
                        with_alpha(palette.surface_container, 0.62),
                        with_alpha(if active { palette.primary } else { palette.outline }, 0.34),
                    )
                }),
        );
    }
    container(layers).width(Length::Fill).into()
}

fn folio_trailing_bar<'a, Message: Clone + 'a>(
    content: Element<'a, Message>,
    focused: bool,
    on_focus: Message,
    action: Element<'a, Message>,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    let marker = container(text("")).width(Length::Fixed(2.0 * scale)).height(Length::Fill).style(
        move |_| {
            super::style::bg_style(Background::Color(if focused {
                with_alpha(palette.primary, fade)
            } else {
                Color::TRANSPARENT
            }))
        },
    );
    let copy = iced::widget::mouse_area(
        container(content)
            .width(Length::Fill)
            .align_y(iced::alignment::Vertical::Center)
            .padding(logical_padding(4.0 * scale, 8.0 * scale, 4.0 * scale, 10.0 * scale))
            .clip(true),
    )
    .on_press(on_focus);
    let action = container(action).padding(logical_padding(0.0, 6.0 * scale, 0.0, 0.0));
    container(
        row![iced::widget::Space::new().height(60.0 * scale), marker, copy, action]
            .height(Length::Shrink)
            .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .style(move |_| {
        super::style::box_style(
            with_alpha(palette.surface_container, 0.22 * fade),
            with_alpha(if focused { palette.primary } else { palette.outline }, 0.3 * fade),
        )
    })
    .into()
}

pub fn folio_action_bar<'a, Message: Clone + 'a>(
    content: Element<'a, Message>,
    focused: bool,
    on_focus: Message,
    action: Element<'a, Message>,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    folio_trailing_bar(content, focused, on_focus, action, scale, palette, fade)
}

pub fn folio_inline_bar<'a, Message: Clone + 'a>(
    content: Element<'a, Message>,
    focused: bool,
    on_focus: Message,
    editor: Element<'a, Message>,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    folio_trailing_bar(content, focused, on_focus, editor, scale, palette, fade)
}

pub fn folio_field<'a, Message: Clone + 'a>(
    number: impl text::IntoFragment<'a>,
    title: impl text::IntoFragment<'a>,
    description: impl text::IntoFragment<'a>,
    body: Element<'a, Message>,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    use super::typography::label;
    container(
        column![
            super::layout::folio_horizontal_rule(with_alpha(palette.outline, 0.5)),
            row![
                label(number, 9.0, scale, palette.primary),
                label(title, 13.0, scale, palette.surface_text),
            ]
            .spacing(8.0 * scale)
            .align_y(Alignment::Center),
            label(
                description,
                super::typography::TYPE_SMALL,
                scale,
                with_alpha(palette.surface_text, 0.48)
            )
            .line_height(iced::widget::text::LineHeight::Relative(1.35)),
            body,
        ]
        .spacing(9.0 * scale),
    )
    .width(Length::Fill)
    .padding(iced::Padding { top: 0.0, right: 8.0 * scale, bottom: 6.0 * scale, left: 8.0 * scale })
    .into()
}

pub fn folio_ghost_field<'a, Message: Clone + 'a>(
    value: &'a str,
    placeholder: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
    on_submit: Message,
    width: Length,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    iced::widget::text_input(placeholder, value)
        .on_input(on_input)
        .on_submit(on_submit)
        .padding(8.0 * scale)
        .size(11.0 * legible_type_scale(scale))
        .font(UI_FONT)
        .width(width)
        .style(move |_theme, _status| super::style::ghost_input_style(palette))
        .into()
}

pub fn folio_details<'a, Message: Clone + 'a>(
    title: String,
    summary: String,
    description: String,
    expanded: bool,
    focused: bool,
    on_toggle: Message,
    body: Element<'a, Message>,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    use super::typography::label;
    let header =
        button(
            row![
                label(
                    if expanded { "▾" } else { "▸" },
                    11.0,
                    scale,
                    with_alpha(palette.primary, 0.94),
                ),
                label(title, 13.5, scale, palette.surface_text),
                container(text("")).width(Length::Fill),
                label(
                    summary,
                    super::typography::TYPE_SMALL,
                    scale,
                    with_alpha(palette.surface_text, 0.58)
                ),
            ]
            .spacing(9.0 * scale)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([10.0 * scale, 10.0 * scale])
        .on_press(on_toggle)
        .style(move |_theme, status| {
            super::style::folio_line_button_style(expanded, focused, palette, 1.0, status)
        });
    let mut details = column![header].spacing(0.0);
    if expanded {
        let content = column![
            label(
                description,
                super::typography::TYPE_SMALL,
                scale,
                with_alpha(palette.surface_text, 0.54)
            )
            .line_height(iced::widget::text::LineHeight::Relative(1.35)),
            super::layout::folio_horizontal_rule(with_alpha(palette.outline, 0.42)),
            body,
        ]
        .spacing(12.0 * scale);
        details = details.push(container(content).width(Length::Fill).padding(12.0 * scale).style(
            move |_| {
                super::style::box_style(
                    with_alpha(palette.surface_container, 0.72),
                    with_alpha(palette.outline, 0.46),
                )
            },
        ));
    }
    details.into()
}

pub fn folio_action_wrap<'a, Message: Clone + 'a>(
    actions: Vec<(String, bool, Message)>,
    available: f32,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    let sized =
        actions.into_iter().map(|action| (folio_action_width(&action.0, scale), action)).collect();
    let rows = super::layout::wrap_rows(sized, available.max(80.0), 6.0 * scale);
    let mut content = column![].spacing(6.0 * scale);
    for line_actions in rows {
        let mut line = row![].spacing(6.0 * scale).align_y(Alignment::Center);
        for (action_label, active, message) in line_actions {
            let width = folio_action_width(&action_label, scale);
            line = line.push(folio_action(
                action_label,
                active,
                Some(message),
                Length::Fixed(width),
                scale * 0.9,
                palette,
            ));
        }
        content = content.push(line);
    }
    content.into()
}

#[derive(Debug, Default)]
struct FolioButtonState {
    mix: f32,
    target: bool,
    initialized: bool,
    last_frame: Option<Instant>,
}

impl FolioButtonState {
    fn advance(&mut self, active: bool, now: Instant) -> bool {
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
            self.mix = MotionProfile::default().approach(self.mix, target, dt, MotionTier::Fast);
        }
        if (self.mix - target).abs() < 0.002 {
            self.mix = target;
        }
        self.mix != target
    }
}

pub(crate) fn folio_diagonal_edges(width: f32, height: f32, mix: f32) -> (f32, f32) {
    let mix = mix.clamp(0.0, 1.0);
    if mix <= 0.0 {
        return (0.0, 0.0);
    }
    if mix >= 1.0 {
        return (width, width);
    }
    let slant = height * 0.72;
    let sweep = mix.mul_add(width + slant * 2.0, -slant);
    ((sweep - slant).clamp(0.0, width), (sweep + slant).clamp(0.0, width))
}

pub(crate) fn folio_diagonal_wipe(width: f32, height: f32, mix: f32) -> Path {
    let (top, bottom) = folio_diagonal_edges(width, height, mix);
    let origin = mirror_x(0.0, 0.0, width);
    Path::new(|builder| {
        builder.move_to(Point::new(origin, 0.0));
        builder.line_to(Point::new(mirror_x(top, 0.0, width), 0.0));
        builder.line_to(Point::new(mirror_x(bottom, 0.0, width), height));
        builder.line_to(Point::new(origin, height));
        builder.close();
    })
}

struct FolioButtonFill {
    active: bool,
    destructive: bool,
    enabled: bool,
    palette: Palette,
}

impl<Message> canvas::Program<Message> for FolioButtonFill {
    type State = FolioButtonState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        match event {
            Event::Window(window::Event::RedrawRequested(now)) => state
                .advance(self.active && self.enabled, *now)
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
                with_alpha(self.palette.surface_variant, 0.78)
            } else {
                with_alpha(self.palette.surface_container, if self.enabled { 0.92 } else { 0.42 })
            },
        );
        let mix = crate::frontend::animation::smoothstep(state.mix.clamp(0.0, 1.0));
        if self.enabled && mix > 0.0 {
            let fill = if self.destructive { self.palette.tertiary } else { self.palette.primary };
            frame.fill(
                &folio_diagonal_wipe(bounds.width, bounds.height, mix),
                with_alpha(fill, 0.96),
            );
            if mix < 1.0 {
                let (top, bottom) = folio_diagonal_edges(bounds.width, bounds.height, mix);
                frame.stroke(
                    &Path::line(
                        Point::new(mirror_x(top, 0.0, bounds.width), 0.0),
                        Point::new(mirror_x(bottom, 0.0, bounds.width), bounds.height),
                    ),
                    Stroke::default()
                        .with_color(with_alpha(self.palette.tertiary, 0.9))
                        .with_width(2.0),
                );
            }
        }
        vec![frame.into_geometry()]
    }
}

fn animated_action<'a, Message: Clone + 'a>(
    label: impl text::IntoFragment<'a>,
    active: bool,
    destructive: bool,
    message: Option<Message>,
    width: Length,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    let enabled = message.is_some();
    let width = if matches!(width, Length::Shrink) {
        Length::Fixed(BUTTON_HEIGHT * scale.max(1.0))
    } else {
        width
    };
    let backdrop: Element<'a, Message> =
        iced::widget::canvas(FolioButtonFill { active, destructive, enabled, palette: *palette })
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    let label = container(text(label).font(UI_FONT).size(11.0 * legible_type_scale(scale)))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center);
    let control = button(label).width(Length::Fill).height(Length::Fill).padding(0.0).style(
        move |_theme, status| {
            let disabled = matches!(status, button::Status::Disabled);
            let hovered = matches!(status, button::Status::Hovered);
            let accent = if destructive { palette.tertiary } else { palette.primary };
            button::Style {
                background: Some(Color::TRANSPARENT.into()),
                text_color: if disabled {
                    with_alpha(palette.surface_text, 0.3)
                } else if active {
                    if destructive { palette.background } else { palette.primary_text }
                } else if destructive {
                    palette.tertiary
                } else {
                    palette.surface_text
                },
                border: Border {
                    color: if disabled {
                        with_alpha(palette.outline, 0.18)
                    } else if destructive || hovered || active {
                        with_alpha(accent, 0.86)
                    } else {
                        with_alpha(palette.outline, 0.4)
                    },
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        },
    );
    let control = match message {
        Some(message) => control.on_press(message),
        None => control,
    };
    container(stack![backdrop, control])
        .width(width)
        .height(Length::Fixed(BUTTON_HEIGHT * scale.max(1.0)))
        .clip(true)
        .into()
}

pub fn folio_action<'a, Message: Clone + 'a>(
    label: impl text::IntoFragment<'a>,
    active: bool,
    message: Option<Message>,
    width: Length,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    animated_action(label, active, false, message, width, scale, palette)
}

pub fn folio_destructive_action<'a, Message: Clone + 'a>(
    label: impl text::IntoFragment<'a>,
    active: bool,
    message: Option<Message>,
    width: Length,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    animated_action(label, active, true, message, width, scale, palette)
}

pub fn folio_masthead<'a, Message: Clone + 'a>(
    crumb: String,
    close_message: Message,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    let close = folio_action("×", false, Some(close_message), Length::Shrink, scale, palette);
    container(
        row![
            text("skwd-wall")
                .font(UI_FONT)
                .size(14.0 * legible_type_scale(scale))
                .color(palette.surface_text),
            container(text("")).width(Length::Fill),
            text(crumb)
                .font(UI_FONT)
                .size(10.0 * legible_type_scale(scale))
                .color(with_alpha(palette.surface_text, 0.54)),
            close,
        ]
        .spacing(13.0 * scale)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(54.0 * scale.max(1.0)))
    .padding([10.0 * scale, 20.0 * scale])
    .style(move |_| {
        crate::frontend::ui::box_style(
            with_alpha(palette.background, 0.94),
            with_alpha(palette.outline, 0.5),
        )
    })
    .into()
}

fn folio_blueprint<'a, Message: 'a>(palette: &Palette) -> Element<'a, Message> {
    iced::widget::canvas(FolioBlueprint { palette: *palette })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub fn folio_sheet<'a, Message: Clone + 'a>(
    masthead: Element<'a, Message>,
    index: Element<'a, Message>,
    reading: Element<'a, Message>,
    dismiss: Message,
    viewport: (f32, f32),
    scale: f32,
    ease: f32,
    index_width: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    let (panel_width, panel_height) = crate::frontend::ui::folio_sheet_dims(viewport, scale);
    let page = stack![
        container(text(""))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| crate::frontend::ui::bg_style(Background::Color(palette.surface))),
        folio_blueprint(palette),
        reading,
    ];
    let body = row![
        container(index).width(Length::Fixed(index_width * scale.max(0.9))),
        container(page).width(Length::Fill),
    ]
    .spacing(0.0)
    .height(Length::Fill);
    let panel = container(column![masthead, body])
        .width(Length::Fixed(panel_width))
        .height(Length::Fixed(panel_height))
        .clip(true)
        .style(move |_| folio_sheet_panel_style(palette, ease));
    let backdrop = crate::frontend::ui::inert_backdrop(
        container(text(""))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| folio_scrim_style(ease)),
        dismiss,
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

struct FolioBlueprint {
    palette: Palette,
}

impl<Message> canvas::Program<Message> for FolioBlueprint {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let line = with_alpha(self.palette.outline, 0.07);
        let accent = with_alpha(self.palette.primary, 0.1);
        let centre =
            Point::new(mirror_x(bounds.width * 0.76, 0.0, bounds.width), bounds.height * 0.48);
        for radius in [92.0, 184.0, 276.0] {
            frame.stroke(
                &Path::circle(centre, radius),
                Stroke::default().with_color(line).with_width(1.0),
            );
        }
        frame.stroke(
            &Path::line(Point::new(centre.x, 0.0), Point::new(centre.x, bounds.height)),
            Stroke::default().with_color(line).with_width(1.0),
        );
        frame.stroke(
            &Path::line(Point::new(0.0, centre.y), Point::new(bounds.width, centre.y)),
            Stroke::default().with_color(line).with_width(1.0),
        );
        let marker = Path::new(|builder| {
            builder.move_to(Point::new(centre.x, centre.y - 7.0));
            builder.line_to(Point::new(centre.x + 7.0, centre.y));
            builder.line_to(Point::new(centre.x, centre.y + 7.0));
            builder.line_to(Point::new(centre.x - 7.0, centre.y));
            builder.close();
        });
        frame.stroke(&marker, Stroke::default().with_color(accent).with_width(1.0));
        vec![frame.into_geometry()]
    }
}
