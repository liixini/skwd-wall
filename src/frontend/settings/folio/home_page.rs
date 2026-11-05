use iced::widget::{button, column, container};
use iced::{Alignment, Color, Element, Length, Padding};

use crate::app::Message;
use crate::contracts::settings::SettingChange;
use crate::frontend::ui::calm::{self, Piece, SAY_LEADING, SentenceSpec};
use crate::frontend::ui::{label, legible_type_scale, row, rtl};
use crate::i18n::tr;

use super::super::{ActionId, Row, Section, SettingsFocus, SettingsMsg};
use super::sections::{self, BODY, Paint};

pub(super) struct LeadMetrics {
    pub gap: f32,
    field: f32,
    padding: f32,
    spacing: f32,
    glyph: f32,
    input: f32,
    shortcut: f32,
    shortcut_alpha: f32,
    placeholder_alpha: f32,
    ground_alpha: f32,
    edge_alpha: f32,
    ring: f32,
    edge_width: f32,
    status: f32,
}

pub(super) const LEAD: LeadMetrics = LeadMetrics {
    gap: 14.0,
    field: 44.0,
    padding: 14.0,
    spacing: 12.0,
    glyph: 17.0,
    input: 15.0,
    shortcut: 10.0,
    shortcut_alpha: 0.42,
    placeholder_alpha: 0.48,
    ground_alpha: 0.58,
    edge_alpha: 0.28,
    ring: 2.0,
    edge_width: 1.0,
    status: 12.5,
};

struct ShiftMetrics {
    size: f32,
    alpha: f32,
    spacing: f32,
    width: f32,
}

const SHIFT: ShiftMetrics = ShiftMetrics { size: 11.0, alpha: 0.56, spacing: 10.0, width: 84.0 };

const SEARCH_GLYPH: &str = "⌕";
const SHORTCUT: &str = "settings-nav-search-shortcut";
const ARROWS: [&str; 2] = [" → ", "\u{200f} ← \u{200f}"];

pub(super) fn lead<'a>(section: &Section, paint: Paint<'a>) -> Element<'a, Message> {
    let at_lead = paint.calm.section == 0;
    let lit = |at: usize| at_lead && paint.focus.lit(SettingsFocus::Controls, at);
    let rows: Vec<&Row> = section.rows().collect();
    let focused = section
        .say
        .tokens
        .iter()
        .find(|slot| slot.rows().any(lit))
        .and_then(|slot| sections::slot_identity(slot, &rows))
        .map(|identity| sections::token_key_for(&identity));
    let pieces = calm::split(&section.say.text, sections::say_tokens(section, &rows, paint))
        .into_iter()
        .map(|piece| match piece {
            Piece::Plain(text) => Piece::Faint(text),
            other => other,
        })
        .collect();
    let status = calm::sentence(SentenceSpec {
        pieces,
        size: LEAD.status,
        leading: SAY_LEADING,
        scale: paint.scale,
        palette: *paint.palette,
        plane: paint.surface(),
        focused,
        ring: focused.is_some(),
        popover: None,
        centred: false,
    });
    let ringed = lit(0) || (at_lead && paint.focus.keyboard_focus == SettingsFocus::Sections);
    let copy = rows.first().map_or("", |row| row.title.as_str());
    column![search_field(copy.to_string(), ringed, paint), status]
        .spacing(paint.size(LEAD.gap))
        .padding(Padding { bottom: paint.size(LEAD.gap), ..Padding::ZERO })
        .width(Length::Fill)
        .into()
}

fn search_field(copy: String, ringed: bool, paint: Paint<'_>) -> Element<'_, Message> {
    let palette = *paint.palette;
    let ground = paint.surface();
    let line = row![
        label(SEARCH_GLYPH, LEAD.glyph, paint.scale, palette.primary),
        container(label(
            copy,
            LEAD.input,
            paint.scale,
            calm::tint(ground, palette.surface_text, LEAD.placeholder_alpha),
        ))
        .width(Length::Fill),
        label(
            tr(SHORTCUT),
            LEAD.shortcut,
            paint.scale,
            calm::tint(ground, palette.surface_text, LEAD.shortcut_alpha),
        ),
    ]
    .spacing(paint.size(LEAD.spacing))
    .align_y(Alignment::Center)
    .height(Length::Fill);
    let fill = Color { a: LEAD.ground_alpha, ..Color::BLACK };
    let edge = calm::tint(ground, palette.surface_text, LEAD.edge_alpha);
    let ring = paint.size(LEAD.ring);
    button(line)
        .width(Length::Fill)
        .height(Length::Fixed(paint.size(LEAD.field)))
        .padding([0.0, paint.size(LEAD.padding)])
        .on_press(Message::Settings(SettingsMsg::SearchOpen))
        .style(move |_, _| {
            let (edge, width) =
                if ringed { (palette.primary, ring) } else { (edge, LEAD.edge_width) };
            calm::style::field(fill, edge, palette.surface_text, width)
        })
        .into()
}

pub(super) fn shift<'a>(
    change: &SettingChange,
    id: &ActionId,
    caption: &str,
    focused: bool,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let size = paint.size(SHIFT.size) * legible_type_scale(1.0);
    let pieces = vec![
        Piece::inked(
            format!("{}{}", change.was, ARROWS[usize::from(rtl())]),
            size,
            paint.muted(paint.open_plane(), SHIFT.alpha),
        ),
        Piece::inked(change.now.clone(), size, paint.palette.surface_text),
    ];
    let words = calm::sentence(calm::prose(pieces, BODY.note_leading));
    row![
        container(words).max_width(paint.size(SHIFT.width)),
        sections::action_button(id.clone(), caption.to_string(), focused, paint),
    ]
    .spacing(paint.size(SHIFT.spacing))
    .align_y(Alignment::Center)
    .into()
}
