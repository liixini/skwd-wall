use iced::widget::canvas::{self, Action, Event};
use iced::{Rectangle, mouse};

use super::draw::{self, Part};
use super::gesture::StrandMsg;
use super::state::{Hit, Press, Strand};
use crate::app::Message;
use crate::domain::strand::{CORNERS, Item};
use crate::frontend::components::{Ink, Pen, Sheet};

pub struct StrandCanvas<'a> {
    pub strand: &'a Strand,
    pub part: (Part, Ink),
    pub top: bool,
}

fn emit(message: StrandMsg) -> Action<Message> {
    Action::publish(Message::Strand(message)).and_capture()
}

impl canvas::Program<Message> for StrandCanvas<'_> {
    type State = ();

    fn update(
        &self,
        _state: &mut (),
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        if !self.top {
            return None;
        }
        let at = cursor.position_in(bounds).map(|at| [at.x, at.y]);
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                Some(emit(StrandMsg::Press(at?)))
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => Some(emit(StrandMsg::Move(at?))),
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                Some(emit(at.map_or(StrandMsg::Leave, StrandMsg::Release)))
            }
            Event::Mouse(mouse::Event::CursorLeft) => Some(emit(StrandMsg::Leave)),
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
        let mut sheet = Sheet::new(renderer, Rectangle::with_size(bounds.size()));
        let (part, ink) = self.part;
        let mut pen = Pen::new(&mut sheet, iced::Vector::ZERO, 1.0, ink);
        draw::draw(&mut pen, self.strand, part);
        sheet.finish()
    }

    fn mouse_interaction(
        &self,
        _state: &(),
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if !self.top {
            return mouse::Interaction::default();
        }
        match &self.strand.press {
            Some(Press::Widget { .. } | Press::Bead { pulling: true, .. }) => {
                return mouse::Interaction::Grabbing;
            }
            Some(Press::Gem { corner, .. }) => return resizing(*corner),
            _ => {}
        }
        let Some(at) = cursor.position_in(bounds) else {
            return mouse::Interaction::default();
        };
        match self.strand.hit([at.x, at.y]) {
            Some(Hit::Gem(corner)) => resizing(corner),
            Some(Hit::Widget(_)) => mouse::Interaction::Grab,
            Some(Hit::Item(Item::Bead(kind, style))) => {
                let face = crate::domain::strand::KINDS[kind].styles()[style];
                if self.strand.placed(face) {
                    mouse::Interaction::Pointer
                } else {
                    mouse::Interaction::Grab
                }
            }
            Some(_) => mouse::Interaction::Pointer,
            None => mouse::Interaction::default(),
        }
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
