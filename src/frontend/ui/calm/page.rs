#[cfg(test)]
mod tests;

use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer, widget};
use iced::{Element, Event, Length, Rectangle, Size, Theme, Vector};

use super::reveal::short;

struct Page<'a, Message> {
    rows: Vec<Element<'a, Message>>,
    open: Vec<bool>,
    foot: f32,
}

fn bounded(width: f32, height: f32) -> layout::Limits {
    layout::Limits::with_compression(Size::ZERO, Size::new(width, height), Size::new(false, true))
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Page<'_, Message> {
    fn children(&self) -> Vec<widget::Tree> {
        self.rows.iter().map(widget::Tree::new).collect()
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&self.rows);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let width = limits.max().width;
        let mut nodes = vec![layout::Node::default(); self.rows.len()];
        let mut taken = 0.0;
        let rows = self.rows.iter_mut().zip(&mut tree.children).zip(&self.open);
        for (at, ((row, child), _)) in rows.enumerate().filter(|(_, (_, open))| !**open) {
            nodes[at] = row.as_widget_mut().layout(child, renderer, &bounded(width, f32::INFINITY));
            taken += nodes[at].size().height;
        }
        let room = (limits.min().height - self.foot - taken).max(0.0);
        let rows = self.rows.iter_mut().zip(&mut tree.children).zip(&self.open);
        for (at, ((row, child), _)) in rows.enumerate().filter(|(_, (_, open))| **open) {
            let row = row.as_widget_mut();
            nodes[at] = row.layout(child, renderer, &bounded(width, room));
            let missing = short(child);
            if missing > 0.0 {
                nodes[at] = row.layout(child, renderer, &bounded(width, room + missing));
            }
        }
        let mut height = 0.0;
        for node in &mut nodes {
            node.move_to_mut((0.0, height));
            height += node.size().height;
        }
        layout::Node::with_children(Size::new(width, height), nodes)
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            for ((row, child), layout) in
                self.rows.iter_mut().zip(&mut tree.children).zip(layout.children())
            {
                row.as_widget_mut().operate(child, layout, renderer, operation);
            }
        });
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        for ((row, child), layout) in
            self.rows.iter_mut().zip(&mut tree.children).zip(layout.children())
        {
            row.as_widget_mut()
                .update(child, event, layout, cursor, renderer, clipboard, shell, viewport);
        }
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.rows
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((row, child), layout)| {
                row.as_widget().mouse_interaction(child, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((row, child), layout) in self
            .rows
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .filter(|(_, layout)| layout.bounds().intersects(viewport))
        {
            row.as_widget().draw(child, renderer, theme, style, layout, cursor, viewport);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        overlay::from_children(&mut self.rows, tree, layout, renderer, viewport, translation)
    }
}

pub fn page<'a, Message: 'a>(
    rows: Vec<(Element<'a, Message>, bool)>,
    foot: f32,
) -> Element<'a, Message> {
    let (rows, open) = rows.into_iter().unzip();
    Element::new(Page { rows, open, foot })
}
