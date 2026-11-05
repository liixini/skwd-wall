use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer, widget};
use iced::keyboard::{Key, key::Named};
use iced::{Background, Border, Color, Element, Event, Rectangle, Size, Theme, Vector};

const FOCUS_RULE: f32 = 2.0;

#[derive(Default)]
struct State {
    focused: bool,
}

impl widget::operation::Focusable for State {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
    }
}

fn activates(event: &Event, focused: bool) -> bool {
    focused
        && matches!(
            event,
            Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key: Key::Named(Named::Enter | Named::Space),
                modifiers,
                repeat: false,
                ..
            }) if !modifiers.control() && !modifiers.alt() && !modifiers.logo()
        )
}

pub struct FocusAction<'a, Message> {
    content: Element<'a, Message>,
    message: Option<Message>,
    id: Option<widget::Id>,
    focus_color: Color,
    blur_on_pointer_outside: bool,
}

impl<'a, Message> FocusAction<'a, Message> {
    pub fn new(
        content: Element<'a, Message>,
        message: Option<Message>,
        focus_color: Color,
    ) -> Self {
        Self { content, message, id: None, focus_color, blur_on_pointer_outside: false }
    }

    pub fn id(mut self, id: widget::Id) -> Self {
        self.id = Some(id);
        self
    }

    pub fn blur_on_pointer_outside(mut self) -> Self {
        self.blur_on_pointer_outside = true;
        self
    }
}

fn pointer_outside(event: &Event, cursor: mouse::Cursor, bounds: Rectangle) -> bool {
    matches!(event, Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)))
        && !cursor.is_over(bounds)
}

fn blur_pointer_focus(state: &mut State, event: &Event, cursor: mouse::Cursor, bounds: Rectangle) {
    if pointer_outside(event, cursor, bounds) {
        state.focused = false;
    }
}

impl<Message: Clone> Widget<Message, Theme, iced::Renderer> for FocusAction<'_, Message> {
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn children(&self) -> Vec<widget::Tree> {
        vec![widget::Tree::new(self.content.as_widget())]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        if self.message.is_none() {
            tree.state.downcast_mut::<State>().focused = false;
        }
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<iced::Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let child = self.content.as_widget_mut().layout(&mut tree.children[0], renderer, limits);
        layout::Node::with_children(child.size(), vec![child])
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        if self.message.is_some() {
            operation.focusable(
                self.id.as_ref(),
                layout.bounds(),
                tree.state.downcast_mut::<State>(),
            );
        }
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
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
        if self.blur_on_pointer_outside {
            blur_pointer_focus(tree.state.downcast_mut::<State>(), event, cursor, layout.bounds());
        }
        if activates(event, tree.state.downcast_ref::<State>().focused)
            && let Some(message) = &self.message
        {
            shell.publish(message.clone());
            shell.capture_event();
            return;
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
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
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
        if tree.state.downcast_ref::<State>().focused {
            use iced::advanced::Renderer as _;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border: Border {
                        color: self.focus_color,
                        width: FOCUS_RULE,
                        ..Border::default()
                    },
                    ..renderer::Quad::default()
                },
                Background::Color(Color::TRANSPARENT),
            );
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
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut widget::Tree,
        layout: Layout<'a>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        offset: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            offset,
        )
    }
}

impl<'a, Message: Clone + 'a> From<FocusAction<'a, Message>> for Element<'a, Message> {
    fn from(action: FocusAction<'a, Message>) -> Self {
        Self::new(action)
    }
}

#[cfg(test)]
mod tests;
