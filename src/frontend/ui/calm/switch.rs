#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};

use iced::widget::canvas::{self, Frame, Path, Stroke};
use iced::widget::{Space, button, container, stack};
use iced::{Color, Element, Length, Point, Rectangle, Size, mouse};

use crate::frontend::animation::{CALM, Timing, Transition};
use crate::frontend::ui::{FOLIO_BUTTON_HEIGHT, FOLIO_BUTTON_MIN_WIDTH};

pub(super) const SWITCH: Timing = Timing::new(Duration::from_millis(280), CALM);
pub(super) const LABEL_SIZE: f32 = 11.0;
const RULE: f32 = 1.0;
pub(super) const RING: f32 = 2.0;

#[derive(Clone, Copy)]
pub(super) struct Track {
    pub timing: Timing,
    pub hover: bool,
}

pub(super) struct Switch<const N: usize> {
    identity: Option<u64>,
    tracks: [Transition; N],
    hovered: bool,
}

impl<const N: usize> Default for Switch<N> {
    fn default() -> Self {
        Self { identity: None, tracks: [Transition::new(0.0); N], hovered: false }
    }
}

impl<const N: usize> Switch<N> {
    pub(super) fn values(&self, identity: u64, on: bool, tracks: [Track; N]) -> [f32; N] {
        if self.identity != Some(identity) {
            return tracks.map(|track| f32::from(u8::from(on || track.hover && self.hovered)));
        }
        self.tracks.map(|track| track.x)
    }

    pub(super) fn sync(
        &mut self,
        identity: u64,
        on: bool,
        now: Instant,
        instant: bool,
        tracks: [Track; N],
    ) -> bool {
        let lit = |track: &Track| on || track.hover && self.hovered;
        if self.identity != Some(identity) || instant {
            self.identity = Some(identity);
            self.tracks = tracks.map(|track| Transition::flag(lit(&track)));
            return false;
        }
        let mut moving = false;
        for (state, track) in self.tracks.iter_mut().zip(&tracks) {
            state.toward_flag(lit(track), now, track.timing);
            moving |= state.tick(now);
        }
        moving
    }

    pub(super) fn hover(&mut self, bounds: Rectangle, cursor: mouse::Cursor) -> bool {
        let hovered = cursor.is_over(bounds);
        let changed = hovered != self.hovered;
        self.hovered = hovered;
        changed
    }
}

pub(super) fn backdrop(frame: &mut Frame, size: Size, fill: Color) {
    frame.fill(&Path::rectangle(Point::ORIGIN, size), fill);
}

pub(super) fn outline(frame: &mut Frame, size: Size, color: Color) {
    let half = RULE / 2.0;
    frame.stroke(
        &Path::rectangle(Point::new(half, half), size - Size::new(RULE, RULE)),
        Stroke::default().with_color(color).with_width(RULE),
    );
}

pub(super) fn ring(frame: &mut Frame, size: Size, color: Color) {
    let half = RING / 2.0;
    frame.stroke(
        &Path::rectangle(Point::new(half, half), size - Size::new(RING, RING)),
        Stroke::default().with_color(color).with_width(RING),
    );
}

pub(super) fn inner(size: Size) -> Rectangle {
    Rectangle::new(Point::new(RULE, RULE), size - Size::new(RULE, RULE) * 2.0)
}

pub(super) fn canvas_button<'a, Message, Paint>(
    paint: Paint,
    message: Message,
    width: f32,
    scale: f32,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    Paint: canvas::Program<Message> + 'a,
{
    let hit = button(Space::new())
        .padding(0.0)
        .width(Length::Fill)
        .height(Length::Fill)
        .on_press(message)
        .style(|_theme, _status| button::Style::default());
    container(stack![hit, iced::widget::canvas(paint).width(Length::Fill).height(Length::Fill)])
        .width(Length::Fixed(width.max(FOLIO_BUTTON_MIN_WIDTH * scale)))
        .height(Length::Fixed(FOLIO_BUTTON_HEIGHT * scale.max(1.0)))
        .clip(true)
        .into()
}
