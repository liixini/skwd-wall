use iced::widget::{Stack, canvas};
use iced::{Element, Length};

use super::model::ArrangeMsg;
use super::program::{ArrangeCanvas, EditBarCanvas};
use super::state::Arrange;
use crate::frontend::theme::Palette;
use crate::frontend::ui::EditBar;

impl Arrange {
    pub fn view<'a>(
        &'a self,
        edit_bar: Option<EditBar>,
        palette: &'a Palette,
    ) -> Element<'a, ArrangeMsg> {
        let mut layers = Stack::new().width(Length::Fill).height(Length::Fill);
        if let Some(bar) = edit_bar {
            let program = EditBarCanvas { bar, palette };
            layers = layers.push(canvas(program).width(Length::Fill).height(Length::Fill));
        }
        let program = ArrangeCanvas { arrange: self };
        layers.push(canvas(program).width(Length::Fill).height(Length::Fill)).into()
    }
}
