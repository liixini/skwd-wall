#[cfg(test)]
mod tests;

use std::f32::consts::{FRAC_PI_2, PI};
use std::time::Duration;

use iced::widget::canvas::{self, Frame, Text};
use iced::{Color, Element, Event, Point, Rectangle, Vector, alignment, mouse, window};

use crate::frontend::animation::{CSS_EASE, Timing};
use crate::frontend::components::lerp_color;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{
    FOLIO_ACCENT_ALPHA, FOLIO_FILL_ALPHA, FOLIO_OUTLINE_ALPHA, TWISTY_CLOSED, UI_FONT,
    cached_line_width, legible_type_scale, mirror_x, rtl,
};

use super::switch::{LABEL_SIZE, SWITCH, Switch, Track, backdrop, canvas_button, outline, ring};
use super::tint::{SELECTED_TINT, tint};

const FADE: Timing = Timing::new(Duration::from_millis(280), CSS_EASE);
const TRACK_COUNT: usize = 3;
pub(super) const TRACKS: [Track; TRACK_COUNT] = [
    Track { timing: SWITCH, hover: false },
    Track { timing: FADE, hover: false },
    Track { timing: FADE, hover: true },
];
const CHEVRON_SIZE: f32 = 9.0;
const GAP: f32 = 8.0;
const PADDING: f32 = 12.0;

pub struct OptionsSpec<'a, Message> {
    pub identity: u64,
    pub label: &'a str,
    pub open: bool,
    pub focused: bool,
    pub message: Message,
    pub instant: bool,
    pub scale: f32,
    pub palette: Palette,
    pub plane: Color,
}

struct Paint<'a> {
    identity: u64,
    label: &'a str,
    open: bool,
    focused: bool,
    instant: bool,
    scale: f32,
    palette: Palette,
    plane: Color,
}

impl Paint<'_> {
    fn sizes(&self) -> (f32, f32, f32) {
        let type_scale = legible_type_scale(self.scale);
        let label = LABEL_SIZE * type_scale;
        let chevron = CHEVRON_SIZE * type_scale;
        (
            cached_line_width(self.label, label, UI_FONT),
            cached_line_width(TWISTY_CLOSED, chevron, UI_FONT),
            GAP * self.scale,
        )
    }

    fn width(&self) -> f32 {
        let (label, chevron, gap) = self.sizes();
        label + gap + chevron + PADDING * self.scale * 2.0
    }
}

impl<Message> canvas::Program<Message> for Paint<'_> {
    type State = Switch<TRACK_COUNT>;

    fn update(
        &self,
        state: &mut Switch<TRACK_COUNT>,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        match event {
            Event::Window(window::Event::RedrawRequested(now)) => state
                .sync(self.identity, self.open, *now, self.instant, TRACKS)
                .then(canvas::Action::request_redraw),
            Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::CursorLeft) => {
                state.hover(bounds, cursor).then(canvas::Action::request_redraw)
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Switch<TRACK_COUNT>,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let size = bounds.size();
        let [turn, fill, border] = state.values(self.identity, self.open, TRACKS);
        let palette = self.palette;
        let ground = lerp_color(
            tint(self.plane, palette.surface_container, FOLIO_FILL_ALPHA),
            tint(self.plane, palette.primary, SELECTED_TINT),
            fill,
        );
        backdrop(&mut frame, size, ground);
        let (label, chevron, gap) = self.sizes();
        let start = (size.width - label - gap - chevron) / 2.0;
        let middle = size.height / 2.0;
        let type_scale = legible_type_scale(self.scale);
        frame.fill_text(Text {
            content: self.label.to_string(),
            position: Point::new(mirror_x(start, label, size.width), middle),
            color: palette.surface_text,
            size: (LABEL_SIZE * type_scale).into(),
            font: UI_FONT,
            align_y: alignment::Vertical::Center,
            ..Text::default()
        });
        let pivot = mirror_x(start + label + gap, chevron, size.width) + chevron / 2.0;
        let (rest, swing) = if rtl() { (PI, -FRAC_PI_2) } else { (0.0, FRAC_PI_2) };
        frame.with_save(|frame| {
            frame.translate(Vector::new(pivot, middle));
            frame.rotate(rest + swing * turn);
            frame.fill_text(Text {
                content: TWISTY_CLOSED.to_string(),
                position: Point::ORIGIN,
                color: palette.primary,
                size: (CHEVRON_SIZE * type_scale).into(),
                font: UI_FONT,
                align_x: alignment::Horizontal::Center.into(),
                align_y: alignment::Vertical::Center,
                ..Text::default()
            });
        });
        let edge = lerp_color(
            tint(ground, palette.outline, FOLIO_OUTLINE_ALPHA),
            tint(ground, palette.primary, FOLIO_ACCENT_ALPHA),
            border,
        );
        outline(&mut frame, size, edge);
        if self.focused {
            ring(&mut frame, size, palette.primary);
        }
        vec![frame.into_geometry()]
    }
}

pub fn options_button<'a, Message: Clone + 'a>(
    spec: OptionsSpec<'a, Message>,
) -> Element<'a, Message> {
    let OptionsSpec { identity, label, open, focused, message, instant, scale, palette, plane } =
        spec;
    let paint = Paint { identity, label, open, focused, instant, scale, palette, plane };
    let width = paint.width();
    canvas_button(paint, message, width, scale)
}
