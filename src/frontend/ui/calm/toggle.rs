#[cfg(test)]
mod tests;

use std::time::Duration;

use iced::widget::canvas::{self, Frame};
use iced::{Color, Element, Event, Point, Rectangle, mouse, window};

use crate::frontend::animation::{CSS_EASE, Timing};
use crate::frontend::components::lerp_color;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{
    FOLIO_ACCENT_ALPHA, FOLIO_FILL_ALPHA, FOLIO_OUTLINE_ALPHA, UI_FONT, WIPE_EDGE_ALPHA,
    WIPE_FILL_ALPHA, folio_action_width_in, folio_wipe, legible_type_scale,
};

use super::switch::{
    LABEL_SIZE, SWITCH, Switch, Track, backdrop, canvas_button, inner, outline, ring,
};
use super::tint::tint;

const BORDER: Timing = Timing::new(Duration::from_millis(180), CSS_EASE);
pub(super) const TRACKS: [Track; 2] =
    [Track { timing: SWITCH, hover: false }, Track { timing: BORDER, hover: true }];

pub struct ToggleSpec<'a, Message> {
    pub identity: u64,
    pub on: bool,
    pub focused: bool,
    pub labels: [&'a str; 2],
    pub message: Message,
    pub instant: bool,
    pub scale: f32,
    pub palette: Palette,
    pub plane: Color,
}

struct Paint<'a> {
    identity: u64,
    on: bool,
    focused: bool,
    labels: [&'a str; 2],
    instant: bool,
    scale: f32,
    palette: Palette,
    plane: Color,
}

impl<Message> canvas::Program<Message> for Paint<'_> {
    type State = Switch<2>;

    fn update(
        &self,
        state: &mut Switch<2>,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        match event {
            Event::Window(window::Event::RedrawRequested(now)) => state
                .sync(self.identity, self.on, *now, self.instant, TRACKS)
                .then(canvas::Action::request_redraw),
            Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::CursorLeft) => {
                state.hover(bounds, cursor).then(canvas::Action::request_redraw)
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Switch<2>,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let size = bounds.size();
        let [wipe, border] = state.values(self.identity, self.on, TRACKS);
        let palette = self.palette;
        let fill = tint(self.plane, palette.surface_container, FOLIO_FILL_ALPHA);
        backdrop(&mut frame, size, fill);
        folio_wipe(
            &mut frame,
            size,
            wipe,
            tint(fill, palette.primary, WIPE_FILL_ALPHA),
            tint(fill, palette.tertiary, WIPE_EDGE_ALPHA),
        );
        let inner = inner(size);
        frame.with_clip(inner, |frame| {
            let words = self.labels.iter().zip([palette.surface_text, palette.primary_text]);
            for (idx, (word, color)) in words.enumerate() {
                let centre = Point::new(
                    inner.x + inner.width / 2.0,
                    inner.y + inner.height * (idx as f32 + 0.5 - wipe),
                );
                frame.fill_text(label(word, centre, color, self.scale));
            }
        });
        let edge = lerp_color(
            tint(fill, palette.outline, FOLIO_OUTLINE_ALPHA),
            tint(fill, palette.primary, FOLIO_ACCENT_ALPHA),
            border,
        );
        outline(&mut frame, size, edge);
        if self.focused {
            ring(&mut frame, size, palette.primary);
        }
        vec![frame.into_geometry()]
    }
}

pub fn wipe_toggle<'a, Message: Clone + 'a>(spec: ToggleSpec<'a, Message>) -> Element<'a, Message> {
    let ToggleSpec { identity, on, focused, labels, message, instant, scale, palette, plane } =
        spec;
    let width = labels
        .iter()
        .map(|word| folio_action_width_in(word, scale, 0.0, f32::INFINITY))
        .fold(0.0, f32::max);
    let paint = Paint { identity, on, focused, labels, instant, scale, palette, plane };
    canvas_button(paint, message, width, scale)
}

fn label(word: &str, centre: Point, color: Color, scale: f32) -> canvas::Text {
    crate::frontend::ui::mid_text(
        word,
        centre,
        color,
        LABEL_SIZE * legible_type_scale(scale),
        UI_FONT,
        iced::Alignment::Center,
    )
}
