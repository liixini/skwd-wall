use iced::advanced::widget::{Operation, operation};
use iced::widget::{Space, column, scrollable};
use iced::{Element, Length, Rectangle, Size, Vector};

use super::super::{RevealSpec, reveal};
use super::page;
use crate::frontend::ui::UI_FONT;

const SCREEN: Size = Size::new(200.0, 300.0);
const SHUT: f32 = 100.0;
const FOOT: f32 = 10.0;

struct Bodies(Vec<f32>);

impl Operation for Bodies {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        id: Option<&iced::widget::Id>,
        bounds: Rectangle,
        _content: Rectangle,
        _translation: Vector,
        _state: &mut dyn operation::Scrollable,
    ) {
        if id.is_some() {
            self.0.push(bounds.height);
        }
    }
}

fn shown(body: f32, floor: f32) -> Vec<f32> {
    let inner = scrollable(Space::new().height(body)).id(iced::widget::Id::new("body"));
    let spec = RevealSpec { identity: 1, open: true, instant: true, floor };
    let rows: Vec<(Element<'_, ()>, bool)> =
        vec![(Space::new().height(SHUT).into(), false), (reveal(inner, spec), true)];
    let view = column![scrollable(page(rows, FOOT)).height(Length::Fill)];
    let mut bodies = Bodies(Vec::new());
    crate::frontend::headless::operate(SCREEN, UI_FONT, view.into(), &mut bodies);
    bodies.0
}

#[test]
#[ignore = "offscreen layout probe"]
fn open_row_takes_rest() {
    let room = SCREEN.height - SHUT - FOOT;
    assert_eq!(shown(120.0, 0.0), [120.0], "fits");
    assert_eq!(shown(400.0, 0.0), [room], "capped");
    assert_eq!(shown(400.0, 250.0), [250.0], "floor");
}
