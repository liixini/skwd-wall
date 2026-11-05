use iced::widget::canvas::{self, Action, Event, Frame};
use iced::{Rectangle, mouse};

use super::model::{ArrangeMsg, Pointer};
use super::state::{Arrange, Hit, Press};
use crate::domain::strand::CORNERS;
use crate::frontend::components::{Ink, Pen, Sheet, Unclipped};
use crate::frontend::theme::Palette;
use crate::frontend::ui::{EditBar, draw_edit_bar};

pub struct ArrangeCanvas<'a> {
    pub arrange: &'a Arrange,
}

pub struct EditBarCanvas<'a> {
    pub bar: EditBar,
    pub palette: &'a Palette,
}

fn emit(pointer: Pointer) -> Action<ArrangeMsg> {
    Action::publish(ArrangeMsg::Pointer(pointer)).and_capture()
}

impl canvas::Program<ArrangeMsg> for ArrangeCanvas<'_> {
    type State = ();

    fn update(
        &self,
        _state: &mut (),
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<ArrangeMsg>> {
        let at = cursor.position_in(bounds).map(|at| [at.x, at.y]);
        let arrange = self.arrange;
        let moved = matches!(event, Event::Mouse(mouse::Event::CursorMoved { .. }));
        let same = moved
            && arrange.press.is_none()
            && at.is_some_and(|at| arrange.hit(at) == arrange.hover.map(|(hit, _)| hit));
        if arrange.previewing() {
            let back = at.is_some_and(|at| arrange.back_rect().contains(at));
            if !back && arrange.press.is_none() {
                let at = at.filter(|_| moved && !same)?;
                return Some(Action::publish(ArrangeMsg::Pointer(Pointer::Move(at))));
            }
        }
        if same {
            return Some(Action::capture());
        }
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                Some(emit(Pointer::Press(at?)))
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => Some(emit(Pointer::Move(at?))),
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                Some(emit(at.map_or(Pointer::Leave, Pointer::Release)))
            }
            Event::Mouse(mouse::Event::CursorLeft) => Some(emit(Pointer::Leave)),
            Event::Mouse(mouse::Event::WheelScrolled { .. }) if !arrange.previewing() => {
                Some(Action::capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let area = Rectangle::with_size(bounds.size());
        let thread = self.arrange.thread.draw(renderer, bounds.size(), |frame: &mut Frame| {
            let mut sheet = Unclipped::new(frame, area);
            let mut pen = Pen::new(&mut sheet, iced::Vector::ZERO, 1.0, Ink::Shapes);
            super::draw::thread(&mut pen, self.arrange);
        });
        let mut sheet = Sheet::new(renderer, area);
        let mut pen = Pen::new(&mut sheet, iced::Vector::ZERO, 1.0, Ink::Shapes);
        super::draw::chrome(&mut pen, self.arrange);
        let mut layers = vec![thread];
        layers.extend(sheet.finish());
        layers
    }

    fn mouse_interaction(
        &self,
        _state: &(),
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        let arrange = self.arrange;
        match &arrange.press {
            Some(
                Press::Object { moved: true, .. }
                | Press::Button { moved: true, .. }
                | Press::Bead { pulling: true, .. },
            ) => return mouse::Interaction::Grabbing,
            Some(Press::Gem { corner, .. }) => return resizing(*corner),
            _ => {}
        }
        let Some(at) = cursor.position_in(bounds) else {
            return mouse::Interaction::default();
        };
        match arrange.hit([at.x, at.y]) {
            Some(Hit::Gem(corner)) => resizing(corner),
            Some(Hit::Object(_) | Hit::Button(_)) => mouse::Interaction::Grab,
            Some(Hit::Item(crate::domain::strand::Item::Bead(kind, style))) => {
                if arrange.pullable((kind, style)) {
                    mouse::Interaction::Grab
                } else {
                    mouse::Interaction::Pointer
                }
            }
            Some(_) => mouse::Interaction::Pointer,
            None => mouse::Interaction::default(),
        }
    }
}

impl canvas::Program<ArrangeMsg> for EditBarCanvas<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        draw_edit_bar(&mut frame, &self.bar, self.palette);
        vec![frame.into_geometry()]
    }
}

fn resizing(corner: usize) -> mouse::Interaction {
    let [x, y] = CORNERS[corner];
    if x * y > 0.0 {
        mouse::Interaction::ResizingDiagonallyDown
    } else {
        mouse::Interaction::ResizingDiagonallyUp
    }
}
