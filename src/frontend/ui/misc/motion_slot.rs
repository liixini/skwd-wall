#[cfg(test)]
mod tests;

use iced::advanced::{
    Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, overlay, renderer, widget,
};
use iced::{Element, Event, Length, Rectangle, Size, Theme, Vector};

pub struct MotionSlot<'a, Message> {
    current: Element<'a, Message>,
    previous: Option<Element<'a, Message>>,
    progress: f32,
    travel_limit: Option<f32>,
    identity: Option<u64>,
}

impl<Message> MotionSlot<'_, Message> {
    pub fn identity(mut self, identity: u64) -> Self {
        self.identity = Some(identity);
        self
    }

    fn ready(&self) -> bool {
        self.progress >= 1.0
    }

    fn travel(&self, height: f32) -> (Vector, Vector) {
        let progress = self.progress.clamp(0.0, 1.0);
        let distance = self.travel_limit.map_or(height, |limit| height.min(limit.max(0.0)));
        (Vector::new(0.0, distance * (1.0 - progress)), Vector::new(0.0, -distance * progress))
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for MotionSlot<'_, Message> {
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<Option<u64>>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(self.identity)
    }

    fn children(&self) -> Vec<widget::Tree> {
        let mut children = vec![widget::Tree::new(self.current.as_widget())];
        if let Some(previous) = &self.previous {
            children.push(widget::Tree::new(previous.as_widget()));
        }
        children
    }

    fn diff(&self, tree: &mut widget::Tree) {
        let identity = tree.state.downcast_mut::<Option<u64>>();
        if *identity != self.identity {
            *identity = self.identity;
            tree.children = self.children();
            return;
        }
        if tree.children.is_empty() {
            tree.children.push(widget::Tree::new(self.current.as_widget()));
        } else {
            tree.children[0].diff(self.current.as_widget());
        }
        if let Some(previous) = &self.previous {
            if tree.children.len() == 1 {
                tree.children.push(widget::Tree::new(previous.as_widget()));
            } else {
                tree.children[1].diff(previous.as_widget());
            }
        } else {
            tree.children.truncate(1);
        }
    }

    fn size(&self) -> Size<Length> {
        if self.identity.is_some() {
            self.current.as_widget().size()
        } else {
            Size::new(Length::Fill, Length::Shrink)
        }
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let child_limits = limits.loose();
        let current =
            self.current.as_widget_mut().layout(&mut tree.children[0], renderer, &child_limits);
        let previous = self.previous.as_mut().map(|previous| {
            previous.as_widget_mut().layout(&mut tree.children[1], renderer, &child_limits)
        });
        let old_height = previous.as_ref().map_or(current.size().height, |node| node.size().height);
        let progress = self.progress.clamp(0.0, 1.0);
        let height = old_height + (current.size().height - old_height) * progress;
        let width =
            current.size().width.max(previous.as_ref().map_or(0.0, |node| node.size().width));
        let sizing = self.size();
        let size = limits.resolve(sizing.width, sizing.height, Size::new(width, height));
        let mut children = vec![current];
        if let Some(previous) = previous {
            children.push(previous);
        }
        layout::Node::with_children(size, children)
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
        let Some(clip) = layout.bounds().intersection(viewport) else {
            return;
        };
        let (incoming, outgoing) = self.travel(layout.bounds().height);
        renderer.with_layer(clip, |renderer| {
            if !self.ready()
                && let (Some(previous), Some(previous_layout)) =
                    (&self.previous, layout.children().nth(1))
            {
                renderer.with_translation(outgoing, |renderer| {
                    previous.as_widget().draw(
                        &tree.children[1],
                        renderer,
                        theme,
                        style,
                        previous_layout,
                        mouse::Cursor::Unavailable,
                        &(clip - outgoing),
                    );
                });
            }
            renderer.with_translation(incoming, |renderer| {
                self.current.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    layout.children().next().unwrap(),
                    if self.ready() { cursor } else { mouse::Cursor::Unavailable },
                    &(clip - incoming),
                );
            });
        });
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        if self.ready() {
            self.current.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        }
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
        if self.ready() {
            self.current.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout.children().next().unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        } else if cursor.is_over(layout.bounds()) && matches!(event, Event::Mouse(_)) {
            shell.capture_event();
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
        if self.ready() {
            self.current.as_widget().mouse_interaction(
                &tree.children[0],
                layout.children().next().unwrap(),
                cursor,
                viewport,
                renderer,
            )
        } else {
            mouse::Interaction::None
        }
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut widget::Tree,
        layout: Layout<'a>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        offset: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, iced::Renderer>> {
        self.ready()
            .then(|| {
                self.current.as_widget_mut().overlay(
                    &mut tree.children[0],
                    layout.children().next().unwrap(),
                    renderer,
                    viewport,
                    offset,
                )
            })
            .flatten()
    }
}

pub fn motion_slot<'a, Message: 'a>(
    current: Element<'a, Message>,
    previous: Option<Element<'a, Message>>,
    progress: f32,
    travel_limit: Option<f32>,
) -> MotionSlot<'a, Message> {
    MotionSlot { current, previous, progress, travel_limit, identity: None }
}

impl<'a, Message: 'a> From<MotionSlot<'a, Message>> for Element<'a, Message> {
    fn from(slot: MotionSlot<'a, Message>) -> Self {
        Self::new(slot)
    }
}
