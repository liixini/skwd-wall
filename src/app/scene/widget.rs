use std::sync::Arc;

use iced::widget::shader::{self, Action};
use iced::{Event, Rectangle, mouse, window};

use crate::app::Message;
use crate::contracts::preview::{BufPool, UploadQueue};
use crate::frontend::scene::RenderSnapshot;
use crate::frontend::scene::layout::Mode;
use crate::infrastructure::runtime::FrameClock;
use crate::rendering::scene::BrowserScenePrimitive;
use crate::rendering::scene::ScenePrimitive;

pub struct SceneProgram {
    pub render: Arc<RenderSnapshot>,
    pub uploads: UploadQueue,
    pub pool: Arc<BufPool>,
    pub frame_clock: FrameClock,
    pub pointer_enabled: bool,
    pub fade: f32,
}

pub struct BrowserSceneProgram {
    pub render: Arc<RenderSnapshot>,
    pub uploads: UploadQueue,
    pub pool: Arc<BufPool>,
}

pub fn shimmer_eligible(
    mode: u32,
    scene_mode: Mode,
    thumb: bool,
    failed: bool,
    near_failed: bool,
) -> bool {
    mode == 0 && scene_mode == Mode::Grid && thumb && !failed && !near_failed
}

#[derive(Default)]
pub struct WidgetState {
    last_pointer: Option<(f32, f32)>,
    pointer_enabled: bool,
}

pub(super) fn pointer_motion_changed(last: &mut Option<(f32, f32)>, x: f32, y: f32) -> bool {
    let changed =
        last.is_none_or(|(last_x, last_y)| (x - last_x).abs() > 2.0 || (y - last_y).abs() > 2.0);
    if changed {
        *last = Some((x, y));
    }
    changed
}

pub(super) fn sync_pointer_gate(
    last: &mut Option<(f32, f32)>,
    was_enabled: &mut bool,
    enabled: bool,
) {
    if *was_enabled != enabled {
        *was_enabled = enabled;
        *last = None;
    }
}

pub(super) fn wheel_amount(delta: &mouse::ScrollDelta) -> f32 {
    match delta {
        mouse::ScrollDelta::Lines { y, x } => {
            if *y == 0.0 {
                *x
            } else {
                *y
            }
        }
        mouse::ScrollDelta::Pixels { y, x } => {
            if *y == 0.0 {
                *x / 60.0
            } else {
                *y / 60.0
            }
        }
    }
}

impl shader::Program<Message> for SceneProgram {
    type State = WidgetState;
    type Primitive = ScenePrimitive;

    fn update(
        &self,
        state: &mut WidgetState,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        sync_pointer_gate(
            &mut state.last_pointer,
            &mut state.pointer_enabled,
            self.pointer_enabled,
        );
        match event {
            Event::Window(window::Event::RedrawRequested(_)) => {
                self.frame_clock.acknowledge_redraw();
                None
            }
            Event::Window(window::Event::Resized(size)) => {
                Some(Action::publish(Message::Viewport(size.width, size.height)))
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if !self.pointer_enabled {
                    return None;
                }
                let pos = cursor.position_in(bounds)?;
                pointer_motion_changed(&mut state.last_pointer, pos.x, pos.y)
                    .then(|| Action::publish(Message::MouseMoved(pos.x, pos.y)))
            }
            Event::Mouse(mouse::Event::CursorLeft) => {
                state.last_pointer = None;
                Some(Action::publish(Message::MouseLeft))
            }
            Event::Mouse(mouse::Event::ButtonPressed(btn)) => {
                let button = crate::app::input::mouse_button(*btn)?;
                let pos = cursor.position_in(bounds)?;
                Some(Action::publish(Message::Click(pos.x, pos.y, button)).and_capture())
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let normed = crate::frontend::components::norm_wheel(delta);
                if std::env::var_os("SKWD_WALL_SCROLL_DBG").is_some() {
                    log::info!(
                        "wheel delta={delta:?} scale={} normed={normed:?}",
                        crate::frontend::components::surface_scale()
                    );
                }
                let amount = wheel_amount(&normed?);
                if amount == 0.0 {
                    return None;
                }
                Some(Action::publish(Message::Wheel(amount)).and_capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &WidgetState,
        _cursor: mouse::Cursor,
        _bounds: Rectangle,
    ) -> ScenePrimitive {
        ScenePrimitive {
            render: self.render.renderer.clone(),
            uploads: self.uploads.clone(),
            pool: self.pool.clone(),
            fade: self.fade,
        }
    }
}

impl<Message> shader::Program<Message> for BrowserSceneProgram {
    type State = ();
    type Primitive = BrowserScenePrimitive;

    fn draw(
        &self,
        _state: &(),
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> BrowserScenePrimitive {
        BrowserScenePrimitive {
            render: self.render.renderer.clone(),
            uploads: self.uploads.clone(),
            pool: self.pool.clone(),
            origin: [bounds.x, bounds.y],
        }
    }
}
