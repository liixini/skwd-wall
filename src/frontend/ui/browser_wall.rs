use iced::widget::canvas::{self, Action};
use iced::{Event, Rectangle, mouse};

use crate::frontend::browser::{BrowserIntent, BrowserMsg};

#[derive(Debug, Clone, Copy)]
pub enum BrowserWallInput {
    Leave,
    Pointer(f32, f32),
    Click(f32, f32),
    Wheel(f32),
}

pub struct BrowserWallInputLayer;

impl canvas::Program<BrowserIntent> for BrowserWallInputLayer {
    type State = ();

    fn update(
        &self,
        _state: &mut (),
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<BrowserIntent>> {
        let Some(position) = cursor.position_in(bounds) else {
            return matches!(event, Event::Mouse(mouse::Event::CursorMoved { .. })).then(|| {
                Action::publish(BrowserIntent::Update(BrowserMsg::WallInput(
                    BrowserWallInput::Leave,
                )))
            });
        };
        let input = match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                BrowserWallInput::Pointer(position.x, position.y)
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                BrowserWallInput::Click(position.x, position.y)
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let amount = match delta {
                    mouse::ScrollDelta::Lines { x, y } => {
                        if *y == 0.0 {
                            *x
                        } else {
                            *y
                        }
                    }
                    mouse::ScrollDelta::Pixels { x, y } => {
                        if *y == 0.0 {
                            *x / 60.0
                        } else {
                            *y / 60.0
                        }
                    }
                };
                BrowserWallInput::Wheel(amount)
            }
            _ => return None,
        };
        Some(Action::publish(BrowserIntent::Update(BrowserMsg::WallInput(input))).and_capture())
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
