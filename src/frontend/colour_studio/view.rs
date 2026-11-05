use iced::widget::{Stack, canvas};
use iced::{Element, Length};

use super::draw::{Chrome, PARTS};
use super::program::StudioCanvas;
use super::state::ColourStudio;
use crate::app::Message;
use crate::frontend::theme::Palette;

impl ColourStudio {
    pub fn view<'a>(&'a self, palette: &Palette, ui_scale: f32) -> Element<'a, Message> {
        let chrome = Chrome::new(palette);
        let last = PARTS.len() - 1;
        let layers = PARTS.iter().enumerate().map(|(index, part)| {
            let program =
                StudioCanvas { studio: self, chrome, ui_scale, part: *part, top: index == last };
            Element::from(canvas(program).width(Length::Fill).height(Length::Fill))
        });
        Stack::with_children(layers).width(Length::Fill).height(Length::Fill).into()
    }
}
