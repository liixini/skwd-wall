use iced::widget::{Stack, canvas};
use iced::{Element, Length};

use super::draw::PARTS;
use super::program::StrandCanvas;
use super::state::Strand;
use crate::app::Message;

impl Strand {
    pub fn view(&self) -> Element<'_, Message> {
        let last = PARTS.len() - 1;
        let layers = PARTS.iter().enumerate().map(|(index, part)| {
            let program = StrandCanvas { strand: self, part: *part, top: index == last };
            Element::from(canvas(program).width(Length::Fill).height(Length::Fill))
        });
        Stack::with_children(layers).width(Length::Fill).height(Length::Fill).into()
    }
}
