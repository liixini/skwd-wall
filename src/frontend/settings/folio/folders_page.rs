use std::collections::HashMap;

use iced::widget::{button, column, container, text};
use iced::{Alignment, Element, Length};

use super::super::{
    ActionId, FolderLedger, FolderRow, SettingsMsg, TagTone, flash_key, workbench_input_id,
};
use super::page::{
    NAME_SIZE, Paint, ROW_SPACING, SECTION_GAP, action, input, note, rule, tag, text_color,
};
use crate::app::Message;
use crate::contracts::settings::FolderKind;
use crate::frontend::animation::Tween;
use crate::frontend::components::with_alpha;
use crate::frontend::ui::{TYPE_SMALL, label, row};
use crate::i18n::{tr, tr_args};

const ROW_PADDING: [f32; 2] = [9.0, 8.0];
const FLASH_ALPHA: f32 = 0.24;
const UNDO_PADDING: [f32; 2] = [8.0, 12.0];
const UNDO_FILL_ALPHA: f32 = 0.42;
const UNDO_BORDER_ALPHA: f32 = 0.5;

pub(super) struct FoldersView<'a> {
    pub(super) ledger: FolderLedger,
    pub(super) draft: &'a str,
    pub(super) undo: Option<&'a str>,
    pub(super) values: &'a HashMap<String, String>,
    pub(super) editing: Option<&'a str>,
    pub(super) bars: &'a HashMap<String, Tween>,
}

pub(super) fn folders_page<'a>(view: FoldersView<'a>, paint: Paint<'a>) -> Element<'a, Message> {
    let FoldersView { ledger, draft, undo, values, editing, bars } = view;
    let FolderLedger { kind, rows } = ledger;
    let scale = paint.scale;
    let count = rows.len();
    let mut ledger_rows = column![rule(paint)].spacing(0.0);
    if rows.is_empty() {
        let empty = match kind {
            FolderKind::Source => tr("settings-paths-source-empty"),
            FolderKind::Excluded => tr("settings-paths-excluded-empty"),
        };
        ledger_rows = ledger_rows
            .push(
                container(note(empty, paint))
                    .padding([ROW_PADDING[0] * scale, ROW_PADDING[1] * scale]),
            )
            .push(rule(paint));
    }
    for folder in rows {
        let flash = bars.get(&flash_key(&folder.key)).map_or(0.0, |tween| tween.x);
        let editing = editing == Some(folder.key.as_str());
        ledger_rows = ledger_rows.push(folder_row(folder, kind, values, editing, flash, paint));
        ledger_rows = ledger_rows.push(rule(paint));
    }
    let mut page = column![
        note(tr_args!("settings-paths-count", count => count), paint),
        ledger_rows,
        add_line(kind, draft, paint),
    ]
    .spacing(SECTION_GAP * scale);
    if let Some(path) = undo {
        page = page.push(undo_bar(kind, path, paint));
    }
    if kind == FolderKind::Excluded {
        page = page.push(note(tr("settings-paths-excluded-footer"), paint));
    }
    page.into()
}

fn folder_row<'a>(
    folder: FolderRow,
    kind: FolderKind,
    values: &'a HashMap<String, String>,
    editing: bool,
    flash: f32,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let Paint { scale, palette, fade, .. } = paint;
    let FolderRow { index, key, path, tag: folder_tag, note: folder_note } = folder;
    let path: Element<'a, Message> = if editing {
        let value = values.get(&key).map_or("", String::as_str);
        let message_key = key.clone();
        input(tr("settings-paths-path-placeholder"), value, paint)
            .id(workbench_input_id(&key))
            .on_input(move |raw| Message::Settings(SettingsMsg::Input(message_key.clone(), raw)))
            .on_submit(Message::Settings(SettingsMsg::Commit))
            .into()
    } else {
        let title =
            if path.trim().is_empty() { tr("settings-paths-empty-path").to_owned() } else { path };
        let mut text_block = column![label(title, NAME_SIZE, scale, text_color(paint))];
        if let Some(folder_note) = folder_note {
            text_block = text_block.push(label(
                folder_note,
                TYPE_SMALL,
                scale,
                with_alpha(palette.tertiary, fade),
            ));
        }
        button(text_block.spacing(ROW_SPACING * 0.5 * scale))
            .width(Length::Fill)
            .padding(0)
            .on_press(Message::Settings(SettingsMsg::EditFolder(key)))
            .style(button::text)
            .into()
    };
    let mut line = row![path].spacing(ROW_SPACING * 2.0 * scale).align_y(Alignment::Center);
    if let Some((tag_label, tone)) = folder_tag.filter(|_| !editing) {
        let color = match tone {
            TagTone::Quiet => palette.surface_text,
            TagTone::Info => palette.primary,
            TagTone::Warn => palette.tertiary,
        };
        line = line.push(tag(tag_label, color, paint));
    }
    for (id, text_label) in [
        (ActionId::ChooseFolder(kind, Some(index)), tr("settings-paths-folder-choose-action")),
        (ActionId::RemoveFolder(kind, index), tr("settings-paths-remove-action")),
    ] {
        line = line.push(action(text_label, Some(Message::Settings(SettingsMsg::Run(id))), paint));
    }
    container(line)
        .width(Length::Fill)
        .padding([ROW_PADDING[0] * scale, ROW_PADDING[1] * scale])
        .style(move |_| {
            crate::frontend::ui::bg_style(iced::Background::Color(with_alpha(
                palette.primary,
                FLASH_ALPHA * flash * fade,
            )))
        })
        .into()
}

fn add_line<'a>(kind: FolderKind, draft: &str, paint: Paint<'a>) -> Element<'a, Message> {
    let scale = paint.scale;
    let placeholder = match kind {
        FolderKind::Source => tr("settings-paths-source-draft"),
        FolderKind::Excluded => tr("settings-paths-excluded-draft"),
    };
    let add = Message::Settings(SettingsMsg::Run(ActionId::AddFolder(kind)));
    let field = input(placeholder, draft, paint)
        .on_input(move |raw| Message::Settings(SettingsMsg::FolderDraft(kind, raw)))
        .on_submit(add.clone());
    let choose = action(
        tr("settings-paths-folder-choose-action"),
        Some(Message::Settings(SettingsMsg::Run(ActionId::ChooseFolder(kind, None)))),
        paint,
    );
    let add =
        action(tr("settings-paths-add-action"), (!draft.trim().is_empty()).then_some(add), paint);
    row![field, choose, add].spacing(ROW_SPACING * scale).align_y(Alignment::Center).into()
}

fn undo_bar<'a>(kind: FolderKind, path: &str, paint: Paint<'a>) -> Element<'a, Message> {
    let Paint { scale, palette, fade, .. } = paint;
    let removed = match kind {
        FolderKind::Source => tr_args!("settings-paths-removed-source", path => path),
        FolderKind::Excluded => tr_args!("settings-paths-removed-excluded", path => path),
    };
    let undo = action(
        tr("settings-paths-undo-action"),
        Some(Message::Settings(SettingsMsg::Run(ActionId::UndoFolderRemove))),
        paint,
    );
    container(
        row![
            label(removed, TYPE_SMALL, scale, text_color(paint)),
            container(text("")).width(Length::Fill),
            undo
        ]
        .spacing(ROW_SPACING * 2.0 * scale)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding([UNDO_PADDING[0] * scale, UNDO_PADDING[1] * scale])
    .style(move |_| {
        crate::frontend::ui::box_style(
            with_alpha(iced::Color::BLACK, UNDO_FILL_ALPHA * fade),
            with_alpha(palette.outline, UNDO_BORDER_ALPHA * fade),
        )
    })
    .into()
}
