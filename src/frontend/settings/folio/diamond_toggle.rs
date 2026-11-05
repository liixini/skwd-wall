#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};

use iced::widget::canvas::{self, Frame, Path, Stroke, Text};
use iced::widget::{button, container, stack, text};
use iced::{Color, Element, Event, Length, Point, Rectangle, mouse, window};

use crate::app::Message;
use crate::frontend::animation::{Bezier, MotionProfile};
use crate::frontend::components::with_alpha;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{UI_FONT, diamond, legible_type_scale};
use crate::i18n::tr;

use super::super::{ActionId, choreography::SETTINGS_MOTION};

const FRAME: Duration = Duration::from_millis(16);
const TRAVEL: Duration = Duration::from_millis(420);
const FILL: Duration = Duration::from_millis(260);
const LABEL_OUT: Duration = Duration::from_millis(60);
const LABEL_IN: Duration = Duration::from_millis(140);
const LABEL_DELAY: Duration = LABEL_OUT;
const TRAVEL_EASE: Bezier = Bezier(0.16, 1.0, 0.3, 1.0);
const FILL_EASE: Bezier = Bezier(0.0, 0.0, 0.58, 1.0);

struct Geometry {
    width: f32,
    height: f32,
    label_width: f32,
    label_size: f32,
    track_width: f32,
    track_height: f32,
    track_gap: f32,
    endpoint: f32,
    thumb: f32,
    thumb_inset: f32,
    outline_gap: f32,
    wire: f32,
}

const GEOMETRY: Geometry = Geometry {
    width: 110.0,
    height: 36.0,
    label_width: 26.0,
    label_size: 11.0,
    track_width: 70.0,
    track_height: 32.0,
    track_gap: 10.0,
    endpoint: 6.0,
    thumb: 8.0,
    thumb_inset: 7.0,
    outline_gap: 4.0,
    wire: 1.0,
};

struct Ink {
    wire: f32,
    endpoint: f32,
    focus_outline: f32,
    hover: f32,
}

const INK: Ink = Ink { wire: 0.65, endpoint: 0.65, focus_outline: 0.45, hover: 0.12 };

#[derive(Clone, Copy)]
struct Visual {
    travel: f32,
    fill: f32,
    off: f32,
    on: f32,
}

impl Visual {
    fn settled(value: bool) -> Self {
        Self {
            travel: f32::from(u8::from(value)),
            fill: f32::from(u8::from(value)),
            off: f32::from(u8::from(!value)),
            on: f32::from(u8::from(value)),
        }
    }
}

#[derive(Default)]
struct State {
    identity: Option<ToggleIdentity>,
    from: Option<Visual>,
    visual: Option<Visual>,
    target: bool,
    started: Option<Instant>,
}

impl State {
    fn visual(&self, value: bool) -> Visual {
        self.visual.unwrap_or_else(|| Visual::settled(value))
    }

    fn visible(&self, identity: &ToggleIdentity, value: bool) -> Visual {
        if self.identity.as_ref() == Some(identity) {
            self.visual(value)
        } else {
            Visual::settled(value)
        }
    }

    fn progress(now: Instant, start: Instant, duration: Duration) -> f32 {
        (now.saturating_duration_since(start).as_secs_f32() / duration.as_secs_f32())
            .clamp(0.0, 1.0)
    }

    fn sample(&self, now: Instant) -> Visual {
        let Some(start) = self.started else {
            return self.visual(self.target);
        };
        let from = self.from.unwrap_or_else(|| Visual::settled(!self.target));
        let to = Visual::settled(self.target);
        let travel = TRAVEL_EASE.at(Self::progress(now, start, TRAVEL));
        let fill = FILL_EASE.at(Self::progress(now, start, FILL));
        let outgoing = 1.0 - Self::progress(now, start, LABEL_OUT);
        let incoming = FILL_EASE.at(Self::progress(now, start + LABEL_DELAY, LABEL_IN));
        Visual {
            travel: from.travel + (to.travel - from.travel) * travel,
            fill: from.fill + (to.fill - from.fill) * fill,
            off: if self.target {
                from.off * outgoing
            } else {
                from.off + (1.0 - from.off) * incoming
            },
            on: if self.target { from.on + (1.0 - from.on) * incoming } else { from.on * outgoing },
        }
    }

    fn sync(
        &mut self,
        identity: &ToggleIdentity,
        value: bool,
        now: Instant,
        instant: bool,
    ) -> bool {
        if self.identity.as_ref() != Some(identity) || self.visual.is_none() || instant {
            self.identity = Some(identity.clone());
            self.visual = Some(Visual::settled(value));
            self.from = None;
            self.started = None;
            self.target = value;
            return false;
        }
        if self.target != value {
            self.from = Some(self.sample(now));
            self.started = Some(now);
            self.target = value;
        }
        self.visual = Some(self.sample(now));
        let moving = self.started.is_some_and(|start| now < start + TRAVEL);
        if !moving {
            self.visual = Some(Visual::settled(value));
            self.from = None;
            self.started = None;
        }
        moving
    }
}

#[derive(Clone, PartialEq)]
pub(super) enum ToggleIdentity {
    Path(String),
    Action(ActionId),
}

pub(super) struct ToggleSpec {
    pub identity: ToggleIdentity,
    pub value: bool,
    pub focused: bool,
    pub scale: f32,
    pub fade: f32,
    pub palette: Palette,
    pub message: Message,
    pub motion: MotionProfile,
}

struct TogglePaint {
    identity: ToggleIdentity,
    value: bool,
    focused: bool,
    scale: f32,
    fade: f32,
    palette: Palette,
    instant: bool,
    off: &'static str,
    on: &'static str,
}

fn blend(from: Color, to: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);
    Color {
        r: from.r + (to.r - from.r) * amount,
        g: from.g + (to.g - from.g) * amount,
        b: from.b + (to.b - from.b) * amount,
        a: from.a + (to.a - from.a) * amount,
    }
}

impl canvas::Program<Message> for TogglePaint {
    type State = State;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        match event {
            Event::Window(window::Event::RedrawRequested(now)) => state
                .sync(&self.identity, self.value, *now, self.instant)
                .then(|| canvas::Action::request_redraw_at(*now + FRAME)),
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let visual = state.visible(&self.identity, self.value);
        let scale = self.scale.max(1.0);
        let track_left = (GEOMETRY.label_width + GEOMETRY.track_gap) * scale;
        let track_top = (bounds.height - GEOMETRY.track_height * scale) / 2.0;
        let center_y = track_top + GEOMETRY.track_height * scale / 2.0;
        let endpoint_radius = GEOMETRY.endpoint * scale / 2.0;
        let thumb_radius = GEOMETRY.thumb * scale / 2.0;
        let start = track_left + GEOMETRY.thumb_inset * scale + thumb_radius;
        let end = track_left + (GEOMETRY.track_width - GEOMETRY.thumb_inset) * scale - thumb_radius;
        let center_x = start + (end - start) * visual.travel;
        let outline_radius = thumb_radius + GEOMETRY.outline_gap * scale;
        let wire = blend(self.palette.outline, self.palette.primary, visual.fill);
        let accent = blend(self.palette.outline, self.palette.primary, visual.fill);
        let fill = blend(self.palette.surface_container, self.palette.primary, visual.fill);
        let ink = self.fade;

        frame.stroke(
            &Path::line(Point::new(start, center_y), Point::new(end, center_y)),
            Stroke::default()
                .with_color(with_alpha(wire, ink * INK.wire))
                .with_width(GEOMETRY.wire * scale),
        );
        for x in [start, end] {
            let path = diamond(Point::new(x, center_y), endpoint_radius);
            frame.fill(&path, with_alpha(self.palette.surface_container, ink));
            frame.stroke(
                &path,
                Stroke::default()
                    .with_color(with_alpha(self.palette.outline, ink * INK.endpoint))
                    .with_width(GEOMETRY.wire * scale),
            );
        }
        let thumb = diamond(Point::new(center_x, center_y), thumb_radius);
        frame.fill(&thumb, with_alpha(fill, ink));
        frame.stroke(
            &thumb,
            Stroke::default().with_color(with_alpha(accent, ink)).with_width(GEOMETRY.wire * scale),
        );
        if visual.fill > 0.0 || self.focused {
            frame.stroke(
                &diamond(Point::new(center_x, center_y), outline_radius),
                Stroke::default()
                    .with_color(with_alpha(
                        self.palette.primary,
                        ink * visual.fill.max(if self.focused { INK.focus_outline } else { 0.0 }),
                    ))
                    .with_width(GEOMETRY.wire * scale),
            );
        }
        for (word, alpha) in [(self.off, visual.off), (self.on, visual.on)] {
            frame.fill_text(Text {
                content: word.to_string(),
                position: Point::new(0.0, center_y),
                color: with_alpha(self.palette.surface_text, ink * alpha),
                size: (GEOMETRY.label_size * legible_type_scale(scale)).into(),
                font: UI_FONT,
                align_y: iced::alignment::Vertical::Center,
                ..Text::default()
            });
        }
        vec![frame.into_geometry()]
    }
}

pub(super) fn diamond_toggle(spec: ToggleSpec) -> Element<'static, Message> {
    let ToggleSpec { identity, value, focused, scale, fade, palette, message, motion } = spec;
    let off = tr("settings-app-themes-off");
    let on = tr("settings-app-themes-on");
    let canvas: Element<'static, Message> = iced::widget::canvas(TogglePaint {
        identity,
        value,
        focused,
        scale,
        fade,
        palette,
        instant: SETTINGS_MOTION.reduced(motion),
        off,
        on,
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into();
    let label = if value { on } else { off };
    let hit = button(text(label).color(Color::TRANSPARENT))
        .padding(0.0)
        .width(Length::Fill)
        .height(Length::Fill)
        .on_press(message)
        .style(move |_theme, status| button::Style {
            background: if matches!(status, button::Status::Hovered) {
                Some(with_alpha(palette.hover, INK.hover * fade).into())
            } else {
                None
            },
            ..Default::default()
        });
    container(stack![canvas, hit])
        .width(Length::Fixed(GEOMETRY.width * scale.max(1.0)))
        .height(Length::Fixed(GEOMETRY.height * scale.max(1.0)))
        .clip(true)
        .into()
}
