use iced::widget::{column, container, scrollable, text};
use iced::{Alignment, Element, Length};

use crate::app::Message;
use crate::frontend::theme::Palette;
use crate::frontend::ui::row;
use crate::i18n::{tr, tr_args};

use super::model::{CardPicker, CardPickerMsg};
use super::widgets::{field, hint_line, row_chip};

pub fn card_picker_view<'a>(
    picker: &'a CardPicker,
    viewport: (f32, f32),
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    let mut column = column![
        row![
            text(tr("playlists-card-add-title"))
                .size(13.0 * scale)
                .color(palette.surface_text)
                .width(Length::Fill),
            crate::frontend::ui::pl_chip(
                "×",
                false,
                Message::CardPicker(CardPickerMsg::Close),
                scale,
                palette,
            ),
        ]
        .align_y(Alignment::Center),
        text(&picker.name)
            .size(12.0 * scale)
            .color(crate::frontend::ui::with_alpha(palette.surface_text, 0.6)),
    ]
    .spacing(5.0 * scale);
    let mut curated = 0;
    for playlist in &picker.lists {
        if playlist.kind.is_smart() {
            continue;
        }
        curated += 1;
        let id = playlist.id;
        let member = picker.member_ids.contains(&id);
        let display = if playlist.name.is_empty() {
            tr("playlists-unnamed").to_owned()
        } else {
            playlist.name.clone()
        };
        let display = tr_args!("playlists-picker-entry", name => display, id => id.to_string());
        let label = if member { format!("\u{2713} {display}") } else { display };
        column = column.push(row_chip(
            label,
            member,
            Message::CardPicker(CardPickerMsg::Toggle(id)),
            scale,
            palette,
        ));
    }
    let hint = if curated == 0 { tr("playlists-card-none") } else { tr("playlists-card-new") };
    column = column.push(hint_line(hint, scale, palette));
    column = column.push(
        row![
            field(
                &picker.new_buf,
                tr("playlists-new-name"),
                |text| Message::CardPicker(CardPickerMsg::NewInput(text)),
                Message::CardPicker(CardPickerMsg::NewSubmit),
                Length::Fill,
                scale,
                palette
            ),
            crate::frontend::ui::pl_chip(
                "＋",
                true,
                Message::CardPicker(CardPickerMsg::NewSubmit),
                scale,
                palette
            ),
        ]
        .spacing(6.0 * scale)
        .align_y(Alignment::Center),
    );
    let panel_width = (viewport.0 * 0.32).clamp(260.0, 380.0);
    let list_height = (viewport.1 * 0.55).clamp(180.0, 420.0);
    let panel = container(
        scrollable(column)
            .direction(crate::frontend::ui::thin_vbar())
            .style(crate::frontend::ui::scroll_style(palette.primary)),
    )
    .width(Length::Fixed(panel_width))
    .max_height(list_height)
    .padding(14.0 * scale)
    .style(move |_theme| crate::frontend::ui::panel_style(palette));
    let scrim = container(iced::widget::mouse_area(panel).on_press(Message::Noop))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(move |_theme| crate::frontend::ui::scrim_style(0.55));
    iced::widget::mouse_area(scrim).on_press(Message::CardPicker(CardPickerMsg::Close)).into()
}
