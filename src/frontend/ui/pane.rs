use iced::widget::canvas::{self, Action};
use iced::{Border, Color, Element, Event, Rectangle, mouse};

use super::with_alpha;

pub const BAR_WIDTH: f32 = 5.0;
const BAR_MARGIN: f32 = 2.0;
const OUTLINE_RAIL_ALPHA: f32 = 0.72;

fn bar() -> iced::widget::scrollable::Scrollbar {
    iced::widget::scrollable::Scrollbar::new()
        .width(BAR_WIDTH)
        .scroller_width(BAR_WIDTH)
        .margin(BAR_MARGIN)
}

pub fn thin_vbar() -> iced::widget::scrollable::Direction {
    use iced::widget::scrollable::Direction;
    Direction::Vertical(bar())
}

pub fn spaced_vbar(spacing: f32) -> iced::widget::scrollable::Direction {
    use iced::widget::scrollable::Direction;
    Direction::Vertical(bar().spacing(spacing))
}

pub fn thin_hbar() -> iced::widget::scrollable::Direction {
    use iced::widget::scrollable::Direction;
    Direction::Horizontal(bar())
}

pub fn scroll_style(
    primary: Color,
) -> impl Fn(&iced::Theme, iced::widget::scrollable::Status) -> iced::widget::scrollable::Style {
    use iced::widget::scrollable;
    move |theme, status| {
        let base = scrollable::default(theme, status);
        let hot = matches!(
            status,
            scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. }
        );
        let scol = with_alpha(primary, if hot { 0.85 } else { 0.45 });
        let sb = Border { radius: 2.0.into(), ..Border::default() };
        let mk = || scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller { background: scol.into(), border: sb },
        };
        scrollable::Style { vertical_rail: mk(), horizontal_rail: mk(), ..base }
    }
}

pub fn outline_scroll(
    outline: Color,
) -> impl Fn(&iced::Theme, iced::widget::scrollable::Status) -> iced::widget::scrollable::Style {
    scroll_style(with_alpha(outline, OUTLINE_RAIL_ALPHA))
}

pub const PANE_WHEEL_STEP: f32 = 90.0;

pub fn pane_id(key: &'static str) -> iced::widget::Id {
    iced::widget::Id::new(key)
}

pub struct PaneWheel<Message> {
    pub key: &'static str,
    pub enabled: bool,
    pub on_wheel: fn(&'static str, f32) -> Message,
}

impl<Message: Clone> canvas::Program<Message> for PaneWheel<Message> {
    type State = ();

    fn update(
        &self,
        _state: &mut (),
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        if !self.enabled {
            return None;
        }
        if let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event
            && cursor.position_in(bounds).is_some()
        {
            let dy = match crate::frontend::components::norm_wheel(delta)? {
                mouse::ScrollDelta::Lines { y, .. } => y,
                mouse::ScrollDelta::Pixels { y, .. } => y / 60.0,
            };
            if dy != 0.0 {
                return Some(Action::publish((self.on_wheel)(self.key, dy)).and_capture());
            }
        }
        None
    }

    fn draw(
        &self,
        _state: &(),
        _renderer: &iced::Renderer,
        _theme: &iced::Theme,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        Vec::new()
    }
}

pub fn smooth_pane<'a, Message: Clone + 'a>(
    key: &'static str,
    sc: iced::widget::Scrollable<'a, Message>,
    w: iced::Length,
    h: iced::Length,
    on_wheel: fn(&'static str, f32) -> Message,
    on_scroll: fn(&'static str, f32, f32) -> Message,
) -> Element<'a, Message> {
    smooth_pane_wheel(key, sc, w, h, true, on_wheel, on_scroll)
}

pub fn smooth_pane_wheel<'a, Message: Clone + 'a>(
    key: &'static str,
    sc: iced::widget::Scrollable<'a, Message>,
    w: iced::Length,
    h: iced::Length,
    wheel: bool,
    on_wheel: fn(&'static str, f32) -> Message,
    on_scroll: fn(&'static str, f32, f32) -> Message,
) -> Element<'a, Message> {
    iced::widget::stack![
        sc.width(w).height(h).id(pane_id(key)).on_scroll(move |vp| {
            let max = (vp.content_bounds().height - vp.bounds().height).max(0.0);
            on_scroll(key, vp.absolute_offset().y, max)
        }),
        iced::widget::canvas(PaneWheel { key, enabled: wheel, on_wheel })
            .width(iced::Length::Fill)
            .height(iced::Length::Fill),
    ]
    .width(w)
    .height(h)
    .into()
}

mod tests;
