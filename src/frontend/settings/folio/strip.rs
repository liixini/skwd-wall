#[cfg(test)]
mod tests;

use iced::widget::{Column, column, container, text_input};
use iced::{Alignment, Element, Length};

use crate::app::Message;
use crate::contracts::settings::{PREMADES, SavedLayouts};
use crate::frontend::ui::calm::{self, Piece, Popover, SAY_LEADING, SentenceSpec, Side, Token};
use crate::frontend::ui::{UI_FONT, label, legible_type_scale, row};
use crate::i18n::{tr, tr_args};

use super::super::{LAYOUT_NAME_KEY, LayoutMsg, LayoutsUi, SettingsFocus, SettingsMsg};
use super::sections::{Item, Paint, menu_heading, menu_item, rule, token_key_for};

struct StripMetrics {
    size: f32,
    gap: f32,
    alpha: f32,
    dot_alpha: f32,
    menu_width: f32,
    intro: f32,
    intro_alpha: f32,
    intro_padding: [f32; 2],
    separator_inset: f32,
    menu_padding: f32,
    editor_min: f32,
    chip: f32,
    chip_line: f32,
    chip_rule: f32,
    chip_padding: f32,
}

const STRIP: StripMetrics = StripMetrics {
    size: 12.5,
    gap: 8.0,
    alpha: 0.72,
    dot_alpha: 0.4,
    menu_width: 250.0,
    intro: 10.0,
    intro_alpha: 0.5,
    intro_padding: [6.0, 10.0],
    separator_inset: 6.0,
    menu_padding: 4.0,
    editor_min: 96.0,
    chip: 9.0,
    chip_line: 14.0,
    chip_rule: 0.5,
    chip_padding: 5.0,
};

const STRIP_IDENTITY: &str = "layout.strip";
const LABEL: &str = "settings-layout-saved";
const FIRST: &str = "settings-layout-first";
const HOLDS: &str = "settings-layout-holds";
const YOURS: &str = "settings-layout-yours";
const SAVE_NEW: &str = "settings-layout-save-new";
const PREMADE_MENU: &str = "settings-layout-premade-menu";
const MANAGE: &str = "settings-layout-manage";
const RENAME: &str = "settings-layout-rename";
const DUPLICATE: &str = "settings-layout-duplicate";
const DELETE: &str = "settings-layout-delete";
const DELETE_LAST: &str = "settings-layout-delete-last";
const DELETE_CONFIRM: &str = "settings-layout-delete-confirm";
const KEEP: &str = "settings-layout-keep";
const SAVE_FIRST: &str = "settings-layout-save-first";
const DISCARD: &str = "settings-layout-discard";
const CANCEL: &str = "settings-layout-cancel";
const SAVED: &str = "settings-layout-state-saved";
const REJECTED: &str = "settings-layout-rejected";
const CHANGES: &str = "settings-layout-changes";
const SAVE: &str = "settings-layout-save";
const REVERT: &str = "settings-layout-revert";
const DOT: &str = "·";
const ADD_MARK: &str = "+";
const ACTION_MARK: &str = "◇";
const MIN_LAYOUTS: usize = 2;
const STRIP_SAVE: usize = 1;
const STRIP_REVERT: usize = 2;
pub const STRIP_STOPS: usize = STRIP_REVERT + 1;

#[derive(Debug, Clone, PartialEq)]
pub enum StripEntry {
    Heading(String),
    Intro(String),
    Separator,
    Item { caption: String, on: bool, mark: Option<&'static str>, message: Option<LayoutMsg> },
}

fn manage(caption: &str, message: LayoutMsg, live: bool) -> StripEntry {
    StripEntry::Item {
        caption: caption.to_string(),
        on: false,
        mark: Some(ACTION_MARK),
        message: live.then_some(message),
    }
}

fn action(caption: &str, message: LayoutMsg) -> StripEntry {
    manage(caption, message, true)
}

pub fn strip_key() -> u64 {
    token_key_for(STRIP_IDENTITY)
}

pub fn strip_entries(view: &SavedLayouts, ui: LayoutsUi) -> Vec<StripEntry> {
    let mut entries = vec![StripEntry::Intro(tr(HOLDS).to_string())];
    if ui.confirm.is_some() {
        entries.push(StripEntry::Intro(tr_args!(KEEP, count => view.changes)));
        entries.extend([
            StripEntry::Item {
                caption: tr(SAVE_FIRST).to_string(),
                on: true,
                mark: None,
                message: Some(LayoutMsg::ConfirmSave),
            },
            action(tr(DISCARD), LayoutMsg::ConfirmDiscard),
            action(tr(CANCEL), LayoutMsg::ConfirmCancel),
        ]);
        return entries;
    }
    entries.push(StripEntry::Heading(tr(YOURS).to_string()));
    entries.extend(view.names.iter().enumerate().map(|(at, name)| StripEntry::Item {
        caption: name.clone(),
        on: Some(at) == view.active,
        mark: None,
        message: Some(LayoutMsg::Pick(at)),
    }));
    entries.push(StripEntry::Item {
        caption: tr(SAVE_NEW).to_string(),
        on: false,
        mark: Some(ADD_MARK),
        message: Some(LayoutMsg::SaveNew),
    });
    entries.push(StripEntry::Separator);
    entries.push(StripEntry::Heading(tr(PREMADE_MENU).to_string()));
    entries.extend(
        PREMADES
            .iter()
            .enumerate()
            .map(|(at, premade)| action(tr(premade.name), LayoutMsg::Premade(at))),
    );
    entries.push(StripEntry::Separator);
    let name = view.current().unwrap_or_else(|| tr(FIRST));
    entries.push(StripEntry::Heading(tr_args!(MANAGE, name => name)));
    let live = view.current().is_some();
    entries.push(manage(tr(RENAME), LayoutMsg::Rename, live));
    entries.push(manage(tr(DUPLICATE), LayoutMsg::Duplicate, live));
    entries.push(if view.names.len() < MIN_LAYOUTS {
        manage(tr(DELETE_LAST), LayoutMsg::Delete, false)
    } else if ui.armed {
        manage(&tr_args!(DELETE_CONFIRM, name => name), LayoutMsg::Delete, live)
    } else {
        manage(tr(DELETE), LayoutMsg::Delete, live)
    });
    entries
}

fn menu<'a>(view: &SavedLayouts, ui: LayoutsUi, paint: Paint<'a>) -> Element<'a, Message> {
    let palette = *paint.palette;
    let ground = palette.surface;
    let choice =
        paint.focus.focused_choice.filter(|_| paint.focus.keyboard_focus == SettingsFocus::Strip);
    let mut list: Column<'a, Message> = column![];
    let mut item_at = 0;
    for entry in strip_entries(view, ui) {
        let cell: Element<'a, Message> = match entry {
            StripEntry::Intro(copy) => container(label(
                copy,
                STRIP.intro,
                paint.scale,
                calm::tint(ground, palette.surface_text, STRIP.intro_alpha),
            ))
            .padding(STRIP.intro_padding.map(|px| paint.size(px)))
            .into(),
            StripEntry::Heading(copy) => menu_heading(&copy.to_uppercase(), paint),
            StripEntry::Separator => container(rule(paint, super::sections::SURFACE.secs_rule))
                .padding([paint.size(STRIP.menu_padding), paint.size(STRIP.separator_inset)])
                .into(),
            StripEntry::Item { caption, on, mark, message } => {
                let enabled = message.is_some();
                let lit = choice == Some(item_at);
                item_at += 1;
                let ink = if enabled {
                    palette.surface_text
                } else {
                    calm::tint(ground, palette.surface_text, STRIP.intro_alpha)
                };
                let message =
                    message.map(|message| Message::Settings(SettingsMsg::Layout(message)));
                menu_item(
                    Item {
                        caption,
                        on,
                        lit: lit || (choice.is_none() && on),
                        ink,
                        message,
                        ..Item::default()
                    },
                    mark,
                    false,
                    paint,
                )
            }
        };
        list = list.push(cell);
    }
    container(list)
        .padding(paint.size(STRIP.menu_padding))
        .width(Length::Fixed(paint.size(STRIP.menu_width)))
        .style(move |_| calm::style::popover(ground, &palette))
        .into()
}

fn lit(index: usize, paint: Paint<'_>) -> bool {
    paint.focus.lit(SettingsFocus::Strip, index)
}

fn strip_button<'a>(
    key: &'static str,
    message: LayoutMsg,
    focused: bool,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let caption = tr(key).to_string();
    let width = super::control::button_width(&caption, paint.scale);
    let message = Some(Message::Settings(SettingsMsg::Layout(message)));
    super::control::fixed_button(paint.compact(caption, width, message, focused))
}

fn name_editor(paint: Paint<'_>) -> Element<'_, Message> {
    let palette = *paint.palette;
    let plane = paint.surface();
    let raw = paint.values.get(LAYOUT_NAME_KEY).map_or("", String::as_str);
    let measured = crate::frontend::ui::cached_line_width(raw, paint.size(STRIP.size), UI_FONT);
    text_input("", raw)
        .id(super::super::workbench_input_id(LAYOUT_NAME_KEY))
        .on_input(|raw| Message::Settings(SettingsMsg::Layout(LayoutMsg::RenameInput(raw))))
        .on_submit(Message::Settings(SettingsMsg::Layout(LayoutMsg::RenameCommit)))
        .font(UI_FONT)
        .size(paint.size(STRIP.size) * legible_type_scale(1.0))
        .padding(iced::Padding::ZERO)
        .width(Length::Fixed(paint.size(STRIP.editor_min).max(measured)))
        .style(move |_, _| calm::style::editor(plane, &palette))
        .into()
}

fn name_token<'a>(view: &SavedLayouts, ui: LayoutsUi, paint: Paint<'a>) -> Element<'a, Message> {
    let key = strip_key();
    let name = view.current().unwrap_or_else(|| tr(FIRST)).to_string();
    let press = Message::Settings(SettingsMsg::Layout(LayoutMsg::Menu));
    let popover = (paint.calm.pop == Some(key)).then(|| Popover {
        key,
        side: Side::Below,
        content: menu(view, ui, paint),
    });
    let focused = lit(0, paint).then_some(key);
    calm::sentence(SentenceSpec {
        pieces: vec![Piece::<Message>::token(key, name, Token::Menu, press)],
        size: STRIP.size,
        leading: SAY_LEADING,
        scale: paint.scale,
        palette: *paint.palette,
        plane: paint.surface(),
        focused,
        ring: focused.is_some(),
        popover,
        centred: false,
    })
}

fn changes_chip(count: usize, paint: Paint<'_>) -> Element<'_, Message> {
    let primary = paint.palette.primary;
    let edge = calm::tint(paint.surface(), primary, STRIP.chip_rule);
    container(
        label(tr_args!(CHANGES, count => count), STRIP.chip, paint.scale, primary)
            .line_height(iced::Pixels(paint.size(STRIP.chip_line))),
    )
    .padding([0.0, paint.size(STRIP.chip_padding)])
    .style(move |_| crate::frontend::ui::box_style(iced::Color::TRANSPARENT, edge))
    .into()
}

pub(super) fn strip<'a>(
    view: &SavedLayouts,
    ui: LayoutsUi,
    fade: f32,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let muted = paint.muted(paint.surface(), STRIP.alpha * fade);
    let name = if ui.renaming { name_editor(paint) } else { name_token(view, ui, paint) };
    let mut line = row![
        label(tr(LABEL), STRIP.size, paint.scale, muted),
        name,
        label(DOT, STRIP.size, paint.scale, paint.muted(paint.surface(), STRIP.dot_alpha * fade)),
    ]
    .spacing(paint.size(STRIP.gap))
    .align_y(Alignment::Center);
    if view.dirty() {
        line = line
            .push(strip_button(SAVE, LayoutMsg::Save, lit(STRIP_SAVE, paint), paint))
            .push(strip_button(REVERT, LayoutMsg::Revert, lit(STRIP_REVERT, paint), paint))
            .push(changes_chip(view.changes, paint));
    } else if !ui.rejected {
        line = line.push(label(tr(SAVED), STRIP.size, paint.scale, muted));
    }
    if ui.rejected {
        line = line.push(label(tr(REJECTED), STRIP.size, paint.scale, paint.palette.error));
    }
    line.width(Length::Fill).wrap().into()
}
