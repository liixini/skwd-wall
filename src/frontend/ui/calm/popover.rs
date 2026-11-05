#[cfg(test)]
mod tests;

use iced::advanced::{Clipboard, Layout, Shell, layout, mouse, overlay, renderer, widget};
use iced::{Element, Event, Point, Rectangle, Size, Theme};

const INSET: f32 = 10.0;
const GAP: f32 = 6.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Below,
    BelowEnd,
    AboveEnd,
    Over,
}

pub struct Popover<'a, Message> {
    pub key: u64,
    pub side: Side,
    pub content: Element<'a, Message>,
}

pub(super) struct Anchored<'a, 'b, Message> {
    pub content: &'b mut Element<'a, Message>,
    pub tree: &'b mut widget::Tree,
    pub anchor: Rectangle,
    pub side: Side,
    pub rtl: bool,
}

pub(super) fn position(
    anchor: Rectangle,
    side: Side,
    rtl: bool,
    size: Size,
    screen: Size,
) -> Point {
    let end = anchor.x + anchor.width;
    let (x, y) = match side {
        Side::Below => (
            if rtl { end + INSET - size.width } else { anchor.x - INSET },
            anchor.y + anchor.height + GAP,
        ),
        Side::BelowEnd => {
            (if rtl { anchor.x } else { end - size.width }, anchor.y + anchor.height + GAP)
        }
        Side::AboveEnd => {
            (if rtl { anchor.x } else { end - size.width }, anchor.y - GAP - size.height)
        }
        Side::Over => (if rtl { end - size.width } else { anchor.x }, anchor.y),
    };
    Point::new(
        x.clamp(0.0, (screen.width - size.width).max(0.0)),
        y.clamp(0.0, (screen.height - size.height).max(0.0)),
    )
}

impl<Message> overlay::Overlay<Message, Theme, iced::Renderer> for Anchored<'_, '_, Message> {
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        let min = if self.side == Side::Over { self.anchor.size() } else { Size::ZERO };
        let node = self.content.as_widget_mut().layout(
            self.tree,
            renderer,
            &layout::Limits::new(min, bounds),
        );
        let at = position(self.anchor, self.side, self.rtl, node.size(), bounds);
        node.move_to(at)
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let bounds = layout.bounds();
        self.content.as_widget().draw(self.tree, renderer, theme, style, layout, cursor, &bounds);
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content.as_widget_mut().operate(self.tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let bounds = layout.bounds();
        self.content
            .as_widget_mut()
            .update(self.tree, event, layout, cursor, renderer, clipboard, shell, &bounds);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        self.content.as_widget().mouse_interaction(self.tree, layout, cursor, &bounds, renderer)
    }
}
