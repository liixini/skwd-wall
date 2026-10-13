use iced::widget::canvas::{self, Action};
use iced::{Event, Rectangle, mouse};

use crate::frontend::theme::Palette;

use super::action::BarIntent;
use super::model::{BarModel, BarVisualStyle};
use super::view::draw_filter_bar;

pub struct FilterBar<'a> {
    pub model: BarModel,
    pub hover: Option<usize>,
    pub menu_open: bool,
    pub backend_menu_open: bool,
    pub backend: String,
    pub backend_options: Vec<(&'static str, &'static str)>,
    pub menu_hover: Option<usize>,
    pub folder_options: &'a [String],
    pub folder_hidden: &'a [bool],
    pub folder_roots: &'a [String],
    pub selected_folder: &'a str,
    pub menu_scroll: f32,
    pub theme_swatch: &'a [iced::Color],
    pub pal: &'a Palette,
    pub cache: &'a canvas::Cache,
    pub scale: f32,
    pub fade: f32,
    pub visual_style: BarVisualStyle,
}

#[derive(Default)]
pub struct BarState {
    pub(super) audio_accum: f32,
}

impl canvas::Program<BarIntent> for FilterBar<'_> {
    type State = BarState;

    fn update(
        &self,
        state: &mut BarState,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<BarIntent>> {
        if self.fade <= 0.01 {
            return None;
        }
        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                self.on_move(cursor.position_in(bounds)?)
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                self.on_click(cursor.position_in(bounds)?)
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                self.on_scroll(state, cursor.position_in(bounds)?, delta)
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &BarState,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        vec![draw_filter_bar(self, renderer, bounds)]
    }

    fn mouse_interaction(
        &self,
        _state: &BarState,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if self.fade <= 0.01 {
            return mouse::Interaction::default();
        }
        if let Some(position) = cursor.position_in(bounds) {
            let over = if self.active_menu().is_some() {
                self.menu_index_at(position.x, position.y).is_some()
            } else {
                self.hit(position.x, position.y).is_some()
            };
            if over {
                return mouse::Interaction::Pointer;
            }
        }
        mouse::Interaction::default()
    }
}
