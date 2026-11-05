use iced::widget::canvas::{self, Frame, Path, Stroke};
use iced::{Color, Point, Rectangle, Size, Theme, border, mouse};

use crate::app::Message;

const STROKE: f32 = 1.0;

pub(super) struct Outline {
    pub sizes: [f32; 4],
    pub cut: bool,
    pub edge: Color,
}

impl canvas::Program<Message> for Outline {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let inset = STROKE * 0.5;
        let rect = Rectangle::new(
            Point::new(inset, inset),
            Size::new(bounds.width - STROKE, bounds.height - STROKE),
        );
        let path = if self.cut {
            crate::frontend::components::chamfered(rect, self.sizes.map(|size| [size; 2]))
        } else {
            let [top_left, top_right, bottom_right, bottom_left] = self.sizes;
            let radius = border::Radius { top_left, top_right, bottom_right, bottom_left };
            Path::rounded_rectangle(rect.position(), rect.size(), radius)
        };
        frame.stroke(&path, Stroke::default().with_color(self.edge).with_width(STROKE));
        vec![frame.into_geometry()]
    }
}
